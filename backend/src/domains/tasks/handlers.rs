use async_trait::async_trait;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::shared::append_aggregate_history;
use crate::domains::tasks::{
    CreateTaskCommand, UpdateTaskCommand, DeleteTaskCommand, CompleteTaskCommand,
    GetTaskQuery, ListTasksQuery, GetTasksByUserQuery, GetTasksByClientQuery,
};
use crate::error::AppError;
use crate::models::Task;

// ============ Command Handlers ============

pub struct CreateTaskHandler {
    pool: Arc<SqlitePool>,
}

impl CreateTaskHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
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

        let task_id = Uuid::new_v4().to_string();
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (id, title, description, status, priority, assigned_to, client_id, due_date, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&task_id)
        .bind(&command.title)
        .bind(&command.description)
        .bind(&status)
        .bind(&priority)
        .bind(&command.assigned_to)
        .bind(&command.client_id)
        .bind(&command.due_date)
        .bind(&command.created_by)
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
    pool: Arc<SqlitePool>,
}

impl UpdateTaskHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
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

        let before = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        if let Some(assigned_to) = &command.assigned_to {
            validate_user_exists(&self.pool, assigned_to).await?;
        }
        if let Some(client_id) = &command.client_id {
            validate_client_exists(&self.pool, client_id).await?;
        }

        let mut qb = QueryBuilder::<Sqlite>::new("UPDATE tasks SET ");
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
                separated.push("completed_at = datetime('now')");
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
        }
        if let Some(due_date) = &command.due_date {
            separated.push("due_date = ").push_bind(due_date);
        }
        separated.push("updated_at = datetime('now')");
        drop(separated);

        qb.push(" WHERE id = ").push_bind(&command.id);
        qb.build().execute(&*self.pool).await?;

        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
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
    pool: Arc<SqlitePool>,
}

impl DeleteTaskHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<DeleteTaskCommand> for DeleteTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteTaskCommand) -> Result<(), Self::Error> {
        let existing = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

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

        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
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
    pool: Arc<SqlitePool>,
}

impl CompleteTaskHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<CompleteTaskCommand> for CompleteTaskHandler {
    type Error = AppError;

    async fn handle(&self, command: CompleteTaskCommand) -> Result<Task, Self::Error> {
        let before = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET status = 'done', completed_at = datetime('now') WHERE id = ? RETURNING *"
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
    pool: Arc<SqlitePool>,
}

impl GetTaskHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTaskQuery> for GetTaskHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTaskQuery) -> Result<Option<Task>, Self::Error> {
        let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?")
            .bind(&query.id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(task)
    }
}

pub struct ListTasksHandler {
    pool: Arc<SqlitePool>,
}

impl ListTasksHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListTasksQuery> for ListTasksHandler {
    type Error = AppError;

    async fn handle(&self, query: ListTasksQuery) -> Result<Vec<Task>, Self::Error> {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM tasks");
        let mut has_where = false;

        if let Some(status) = &query.status {
            if !has_where {
                qb.push(" WHERE ");
                has_where = true;
            } else {
                qb.push(" AND ");
            }
            qb.push("status = ").push_bind(normalize_status(Some(status.as_str()))?);
        }
        if let Some(priority) = &query.priority {
            if !has_where {
                qb.push(" WHERE ");
                has_where = true;
            } else {
                qb.push(" AND ");
            }
            qb.push("priority = ").push_bind(normalize_priority(Some(priority.as_str()))?);
        }
        if let Some(assigned_to) = &query.assigned_to {
            if !has_where {
                qb.push(" WHERE ");
                has_where = true;
            } else {
                qb.push(" AND ");
            }
            qb.push("assigned_to = ").push_bind(assigned_to);
        }
        if let Some(client_id) = &query.client_id {
            if !has_where {
                qb.push(" WHERE ");
            } else {
                qb.push(" AND ");
            }
            qb.push("client_id = ").push_bind(client_id);
        }

        if query.due_today.unwrap_or(false) {
            if !has_where {
                qb.push(" WHERE ");
                has_where = true;
            } else {
                qb.push(" AND ");
            }
            // due today + not completed
            qb.push("due_date IS NOT NULL AND date(due_date) = date('now') AND status != 'done'");
        }

        if query.overdue.unwrap_or(false) {
            if !has_where {
                qb.push(" WHERE ");
                has_where = true;
            } else {
                qb.push(" AND ");
            }
            // overdue + not completed
            qb.push("due_date IS NOT NULL AND due_date < datetime('now') AND status != 'done'");
        }

        qb.push(" ORDER BY created_at DESC");
        qb.push(" LIMIT ").push_bind(query.limit.unwrap_or(50).max(1));
        qb.push(" OFFSET ").push_bind(query.offset.unwrap_or(0).max(0));

        let tasks = qb
            .build_query_as::<Task>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(tasks)
    }
}

pub struct GetTasksByUserHandler {
    pool: Arc<SqlitePool>,
}

impl GetTasksByUserHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTasksByUserQuery> for GetTasksByUserHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTasksByUserQuery) -> Result<Vec<Task>, Self::Error> {
        let tasks = if let Some(status) = &query.status {
            sqlx::query_as::<_, Task>(
                "SELECT * FROM tasks WHERE assigned_to = ? AND status = ? ORDER BY created_at DESC",
            )
            .bind(&query.user_id)
            .bind(normalize_status(Some(status.as_str()))?)
            .fetch_all(&*self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Task>(
                "SELECT * FROM tasks WHERE assigned_to = ? ORDER BY created_at DESC",
            )
            .bind(&query.user_id)
            .fetch_all(&*self.pool)
            .await?
        };

        Ok(tasks)
    }
}

pub struct GetTasksByClientHandler {
    pool: Arc<SqlitePool>,
}

impl GetTasksByClientHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetTasksByClientQuery> for GetTasksByClientHandler {
    type Error = AppError;

    async fn handle(&self, query: GetTasksByClientQuery) -> Result<Vec<Task>, Self::Error> {
        let tasks = sqlx::query_as::<_, Task>(
            "SELECT * FROM tasks WHERE client_id = ? ORDER BY created_at DESC"
        )
        .bind(&query.client_id)
        .fetch_all(&*self.pool)
        .await?;

        Ok(tasks)
    }
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

async fn validate_user_exists(pool: &SqlitePool, user_id: &str) -> Result<(), AppError> {
    Uuid::parse_str(user_id)
        .map_err(|_| AppError::ValidationError("assigned_to must be UUID".to_string()))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        return Err(AppError::ValidationError("Assigned user not found".to_string()));
    }
    Ok(())
}

async fn validate_client_exists(pool: &SqlitePool, client_id: &str) -> Result<(), AppError> {
    Uuid::parse_str(client_id)
        .map_err(|_| AppError::ValidationError("client_id must be UUID".to_string()))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE id = ?")
        .bind(client_id)
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        return Err(AppError::ValidationError("Client not found".to_string()));
    }
    Ok(())
}
