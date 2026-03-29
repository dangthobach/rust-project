use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{QueryBuilder, Postgres, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::shared::append_aggregate_history;
use crate::authz::data_scope::{self, can_read_task};
use crate::domains::tasks::{
    CompleteTaskCommand, CreateTaskCommand, DeleteTaskCommand, UpdateTaskCommand,
    GetTaskQuery, GetTasksByClientQuery, GetTasksByUserQuery, ListTasksQuery, ListTasksResult,
};
use crate::error::AppError;
use crate::models::Task;

// ============ Command Handlers ============

pub struct CreateTaskHandler {
    pool: Arc<PgPool>,
}

impl CreateTaskHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<CreateTaskCommand> for CreateTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: CreateTaskCommand) -> Result<Task, Self::Error> {
        let status = normalize_status(command.status.as_deref())?;
        let priority = normalize_priority(command.priority.as_deref())?;

        if let Some(assigned_to) = &command.assigned_to {
            validate_user_exists(&self.pool, assigned_to).await?;
        }
        if let Some(client_id) = &command.client_id {
            validate_client_exists(&self.pool, client_id).await?;
        }

        let branch_id = resolve_task_branch_id(&self.pool, command.client_id.as_ref()).await?;
        data_scope::ensure_branch_allowed(&command.data_scope, &branch_id)?;

        let task_id = Uuid::new_v4();
        let assigned_to = command
            .assigned_to
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok());
        let client_id = command
            .client_id
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok());
        let due_date: Option<DateTime<Utc>> = command
            .due_date
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        let created_by = command
            .created_by
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok());
        let branch_uuid = Uuid::parse_str(&branch_id).unwrap_or_else(|_| {
            Uuid::parse_str(data_scope::ROOT_BRANCH_ID).expect("root branch uuid")
        });

        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, title, description, status, priority, assigned_to, client_id, due_date, created_by, branch_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(task_id)
        .bind(&command.title)
        .bind(&command.description)
        .bind(&status)
        .bind(&priority)
        .bind(assigned_to)
        .bind(client_id)
        .bind(due_date)
        .bind(created_by)
        .bind(branch_uuid)
        .fetch_one(&*self.pool)
        .await?;

        append_aggregate_history(
            &self.pool,
            "task",
            &task.id.to_string(),
            "CREATE",
            None,
            Some(task.status.as_str()),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "title": task.title,
                "priority": task.priority,
                "assigned_to": task.assigned_to,
                "client_id": task.client_id
            })),
        )
        .await?;

        Ok(task)
    }
}

pub struct UpdateTaskHandler {
    pool: Arc<PgPool>,
}

impl UpdateTaskHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateTaskCommand> for UpdateTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateTaskCommand) -> Result<Task, Self::Error> {
        if command.title.is_none()
            && command.description.is_none()
            && command.status.is_none()
            && command.priority.is_none()
            && command.assigned_to.is_none()
            && command.client_id.is_none()
            && command.due_date.is_none()
        {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        let before = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if !can_read_task(
            &self.pool,
            &command.actor_user_id,
            &command.data_scope,
            &before,
        )
        .await?
        {
            return Err(AppError::NotFound("Task not found".to_string()));
        }

        if let Some(assigned_to) = &command.assigned_to {
            validate_user_exists(&self.pool, assigned_to).await?;
        }
        if let Some(client_id) = &command.client_id {
            validate_client_exists(&self.pool, client_id).await?;
        }

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE tasks SET ");
        let mut separated = qb.separated(", ");

        if let Some(title) = &command.title {
            separated.push("title = ").push_bind(title);
        }
        if let Some(description) = &command.description {
            separated.push("description = ").push_bind(description);
        }
        if let Some(status) = &command.status {
            separated
                .push("status = ")
                .push_bind(normalize_status(Some(status.as_str()))?);
            if status == "done" || status == "completed" {
                separated.push("completed_at = NOW()");
            }
        }
        if let Some(priority) = &command.priority {
            separated
                .push("priority = ")
                .push_bind(normalize_priority(Some(priority.as_str()))?);
        }
        if let Some(assigned_to) = &command.assigned_to {
            separated.push("assigned_to = ").push_bind(assigned_to);
        }
        if let Some(client_id) = &command.client_id {
            separated.push("client_id = ").push_bind(client_id);
            let bid = resolve_task_branch_id(&self.pool, Some(client_id)).await?;
            data_scope::ensure_branch_allowed(&command.data_scope, &bid)?;
            separated.push("branch_id = ").push_bind(bid);
        }
        if let Some(due_date) = &command.due_date {
            separated.push("due_date = ").push_bind(due_date);
        }
        separated.push("updated_at = NOW()");
        drop(separated);

        qb.push(" WHERE id = ").push_bind(&command.id);
        qb.build().execute(&*self.pool).await?;

        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(&command.id)
            .fetch_one(&*self.pool)
            .await?;

        append_aggregate_history(
            &self.pool,
            "task",
            &task.id.to_string(),
            "UPDATE",
            Some(before.status.as_str()),
            Some(task.status.as_str()),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "before": {
                    "title": before.title,
                    "status": before.status,
                    "priority": before.priority,
                    "assigned_to": before.assigned_to,
                    "client_id": before.client_id
                },
                "after": {
                    "title": task.title,
                    "status": task.status,
                    "priority": task.priority,
                    "assigned_to": task.assigned_to,
                    "client_id": task.client_id
                }
            })),
        )
        .await?;
        Ok(task)
    }
}

pub struct DeleteTaskHandler {
    pool: Arc<PgPool>,
}

impl DeleteTaskHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<DeleteTaskCommand> for DeleteTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteTaskCommand) -> Result<(), Self::Error> {
        let existing = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if !can_read_task(
            &self.pool,
            &command.actor_user_id,
            &command.data_scope,
            &existing,
        )
        .await?
        {
            return Err(AppError::NotFound("Task not found".to_string()));
        }

        append_aggregate_history(
            &self.pool,
            "task",
            &command.id,
            "DELETE",
            Some(existing.status.as_str()),
            None,
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "title": existing.title
            })),
        )
        .await?;

        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(&command.id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Task not found".to_string()));
        }

        Ok(())
    }
}

pub struct CompleteTaskHandler {
    pool: Arc<PgPool>,
}

impl CompleteTaskHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<CompleteTaskCommand> for CompleteTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: CompleteTaskCommand) -> Result<Task, Self::Error> {
        let before = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if !can_read_task(
            &self.pool,
            &command.actor_user_id,
            &command.data_scope,
            &before,
        )
        .await?
        {
            return Err(AppError::NotFound("Task not found".to_string()));
        }

        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET status = 'done', completed_at = NOW() WHERE id = $1 RETURNING *"
        )
        .bind(&command.id)
        .fetch_one(&*self.pool)
        .await?;

        append_aggregate_history(
            &self.pool,
            "task",
            &task.id.to_string(),
            "COMPLETE",
            Some(before.status.as_str()),
            Some(task.status.as_str()),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "title": task.title
            })),
        )
        .await?;

        Ok(task)
    }
}

// ============ Query Handlers ============

pub struct GetTaskHandler {
    pool: Arc<PgPool>,
}

impl GetTaskHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTaskQuery> for GetTaskHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTaskQuery) -> Result<Option<Task>, Self::Error> {
        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(&query.id)
            .fetch_optional(&*self.pool)
            .await?;

        let Some(t) = task else {
            return Ok(None);
        };
        if !can_read_task(
            &self.pool,
            &query.actor_user_id,
            &query.data_scope,
            &t,
        )
        .await?
        {
            return Ok(None);
        }
        Ok(Some(t))
    }
}

pub struct ListTasksHandler {
    pool: Arc<PgPool>,
}

impl ListTasksHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

/// Shared WHERE clause for list + count (must stay in sync).
fn push_list_tasks_filters<'a>(
    qb: &mut QueryBuilder<'a, Postgres>,
    query: &'a ListTasksQuery,
) -> Result<(), AppError> {
    if let Some(status) = &query.status {
        qb.push(" AND status = ")
            .push_bind(normalize_status(Some(status.as_str()))?);
    }
    if let Some(priority) = &query.priority {
        qb.push(" AND priority = ")
            .push_bind(normalize_priority(Some(priority.as_str()))?);
    }
    if let Some(assigned_to) = &query.assigned_to {
        qb.push(" AND assigned_to = ").push_bind(assigned_to);
    }
    if let Some(client_id) = &query.client_id {
        qb.push(" AND client_id = ").push_bind(client_id);
    }

    if query.due_today.unwrap_or(false) {
        qb.push(" AND due_date IS NOT NULL AND (due_date AT TIME ZONE 'UTC')::date = (CURRENT_TIMESTAMP AT TIME ZONE 'UTC')::date AND status != 'done'");
    }

    if query.overdue.unwrap_or(false) {
        qb.push(" AND due_date IS NOT NULL AND due_date < NOW() AND status != 'done'");
    }

    data_scope::push_task_scope_filter(qb, &query.data_scope, &query.actor_user_id);

    Ok(())
}

#[async_trait]
impl QueryHandler<ListTasksQuery> for ListTasksHandler {
    type Error = AppError;

    async fn handle(&self, query: ListTasksQuery) -> Result<ListTasksResult, Self::Error> {
        let mut count_qb =
            QueryBuilder::<Postgres>::new("SELECT COUNT(*)::bigint FROM tasks WHERE 1=1");
        push_list_tasks_filters(&mut count_qb, &query)?;
        let total: i64 = count_qb
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await?;

        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM tasks WHERE 1=1");
        push_list_tasks_filters(&mut qb, &query)?;
        qb.push(" ORDER BY created_at DESC");
        qb.push(" LIMIT ").push_bind(query.limit.unwrap_or(50).max(1));
        qb.push(" OFFSET ").push_bind(query.offset.unwrap_or(0).max(0));

        let tasks = qb
            .build_query_as::<Task>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(ListTasksResult { tasks, total })
    }
}

pub struct GetTasksByUserHandler {
    pool: Arc<PgPool>,
}

impl GetTasksByUserHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTasksByUserQuery> for GetTasksByUserHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTasksByUserQuery) -> Result<Vec<Task>, Self::Error> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM tasks WHERE assigned_to = ");
        qb.push_bind(&query.user_id);
        if let Some(status) = &query.status {
            qb.push(" AND status = ")
                .push_bind(normalize_status(Some(status.as_str()))?);
        }
        data_scope::push_task_scope_filter(
            &mut qb,
            &query.data_scope,
            &query.actor_user_id,
        );
        qb.push(" ORDER BY created_at DESC");
        let tasks = qb.build_query_as::<Task>().fetch_all(&*self.pool).await?;
        Ok(tasks)
    }
}

pub struct GetTasksByClientHandler {
    pool: Arc<PgPool>,
}

impl GetTasksByClientHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTasksByClientQuery> for GetTasksByClientHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTasksByClientQuery) -> Result<Vec<Task>, Self::Error> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM tasks WHERE client_id = ");
        qb.push_bind(&query.client_id);
        data_scope::push_task_scope_filter(
            &mut qb,
            &query.data_scope,
            &query.actor_user_id,
        );
        qb.push(" ORDER BY created_at DESC");
        let tasks = qb.build_query_as::<Task>().fetch_all(&*self.pool).await?;
        Ok(tasks)
    }
}

async fn resolve_task_branch_id(
    pool: &PgPool,
    client_id: Option<&String>,
) -> Result<String, AppError> {
    if let Some(cid) = client_id {
        let b: Option<String> = sqlx::query_scalar(
            "SELECT branch_id::text FROM clients WHERE id = $1",
        )
        .bind(
            Uuid::parse_str(cid)
                .map_err(|_| AppError::ValidationError("client_id must be UUID".to_string()))?,
        )
        .fetch_optional(pool)
        .await?;
        if let Some(bid) = b {
            if bid.is_empty() {
                return Ok(data_scope::ROOT_BRANCH_ID.to_string());
            }
            return Ok(bid);
        }
    }
    Ok(data_scope::ROOT_BRANCH_ID.to_string())
}

fn normalize_status(status: Option<&str>) -> Result<String, AppError> {
    let value = status.unwrap_or("todo").to_ascii_lowercase();
    match value.as_str() {
        "todo" | "in_progress" | "done" | "cancelled" => Ok(value),
        _ => Err(AppError::ValidationError("Invalid task status".to_string())),
    }
}

fn normalize_priority(priority: Option<&str>) -> Result<String, AppError> {
    let value = priority.unwrap_or("medium").to_ascii_lowercase();
    match value.as_str() {
        "low" | "medium" | "high" | "urgent" => Ok(value),
        _ => Err(AppError::ValidationError("Invalid task priority".to_string())),
    }
}

async fn validate_user_exists(pool: &PgPool, user_id: &str) -> Result<(), AppError> {
    Uuid::parse_str(user_id)
        .map_err(|_| AppError::ValidationError("assigned_to must be UUID".to_string()))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        return Err(AppError::ValidationError("Assigned user not found".to_string()));
    }
    Ok(())
}

async fn validate_client_exists(pool: &PgPool, client_id: &str) -> Result<(), AppError> {
    Uuid::parse_str(client_id)
        .map_err(|_| AppError::ValidationError("client_id must be UUID".to_string()))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        return Err(AppError::ValidationError("Client not found".to_string()));
    }
    Ok(())
}
