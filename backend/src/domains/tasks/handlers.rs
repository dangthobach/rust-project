use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::core::cqrs::{CommandHandler, QueryHandler};
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
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (title, description, status, priority, assigned_to, client_id, due_date)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&command.title)
        .bind(&command.description)
        .bind(&command.status)
        .bind(&command.priority)
        .bind(&command.assigned_to)
        .bind(&command.client_id)
        .bind(&command.due_date)
        .fetch_one(&*self.pool)
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
        let mut query = String::from("UPDATE tasks SET ");
        let mut updates = Vec::new();
        let mut bind_count = 0;

        if command.title.is_some() {
            updates.push("title = ?");
            bind_count += 1;
        }
        if command.description.is_some() {
            updates.push("description = ?");
            bind_count += 1;
        }
        if command.status.is_some() {
            updates.push("status = ?");
            bind_count += 1;
        }
        if command.priority.is_some() {
            updates.push("priority = ?");
            bind_count += 1;
        }
        if command.assigned_to.is_some() {
            updates.push("assigned_to = ?");
            bind_count += 1;
        }
        if command.client_id.is_some() {
            updates.push("client_id = ?");
            bind_count += 1;
        }
        if command.due_date.is_some() {
            updates.push("due_date = ?");
            bind_count += 1;
        }

        if bind_count == 0 {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = {} RETURNING *", command.id));

        let mut q = sqlx::query_as::<_, Task>(&query);
        
        if let Some(title) = command.title {
            q = q.bind(title);
        }
        if let Some(description) = command.description {
            q = q.bind(description);
        }
        if let Some(status) = command.status {
            q = q.bind(status);
        }
        if let Some(priority) = command.priority {
            q = q.bind(priority);
        }
        if let Some(assigned_to) = command.assigned_to {
            q = q.bind(assigned_to);
        }
        if let Some(client_id) = command.client_id {
            q = q.bind(client_id);
        }
        if let Some(due_date) = command.due_date {
            q = q.bind(due_date);
        }

        let task = q.fetch_one(&*self.pool).await?;
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
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(command.id)
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
        let task = sqlx::query_as::<_, Task>(
            "UPDATE tasks SET status = 'completed' WHERE id = ? RETURNING *"
        )
        .bind(command.id)
        .fetch_one(&*self.pool)
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
            .bind(query.id)
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
        let mut sql = String::from("SELECT * FROM tasks WHERE 1=1");
        
        if let Some(status) = &query.status {
            sql.push_str(&format!(" AND status = '{}'", status));
        }
        
        if let Some(assigned_to) = query.assigned_to {
            sql.push_str(&format!(" AND assigned_to = {}", assigned_to));
        }
        
        if let Some(client_id) = query.client_id {
            sql.push_str(&format!(" AND client_id = {}", client_id));
        }
        
        sql.push_str(" ORDER BY created_at DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let tasks = sqlx::query_as::<_, Task>(&sql)
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
        let mut sql = String::from("SELECT * FROM tasks WHERE assigned_to = ?");
        
        if let Some(status) = &query.status {
            sql.push_str(&format!(" AND status = '{}'", status));
        }
        
        sql.push_str(" ORDER BY created_at DESC");

        let tasks = sqlx::query_as::<_, Task>(&sql)
            .bind(query.user_id)
            .fetch_all(&*self.pool)
            .await?;

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
        .bind(query.client_id)
        .fetch_all(&*self.pool)
        .await?;

        Ok(tasks)
    }
}
