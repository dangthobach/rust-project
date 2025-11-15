use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::domains::users::{
    RegisterUserCommand, UpdateUserCommand, ChangePasswordCommand, DeleteUserCommand,
    GetUserQuery, GetUserByEmailQuery, GetUserByUsernameQuery, ListUsersQuery,
};
use crate::error::AppError;
use crate::models::User;
use crate::utils::password;

// ============ Command Handlers ============

pub struct RegisterUserHandler {
    pool: Arc<SqlitePool>,
}

impl RegisterUserHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for RegisterUserHandler {
    type Error = AppError;

    async fn handle(&self, command: RegisterUserCommand) -> Result<User, Self::Error> {
        // Check if user already exists
        let existing = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = ? OR username = ?"
        )
        .bind(&command.email)
        .bind(&command.username)
        .fetch_optional(&*self.pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("User already exists".to_string()));
        }

        // Hash password
        let password_hash = password::hash_password(&command.password)?;

        // Create user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, full_name, role)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&command.username)
        .bind(&command.email)
        .bind(&password_hash)
        .bind(&command.full_name)
        .bind(command.role.unwrap_or_else(|| "user".to_string()))
        .fetch_one(&*self.pool)
        .await?;

        Ok(user)
    }
}

pub struct UpdateUserHandler {
    pool: Arc<SqlitePool>,
}

impl UpdateUserHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUserHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateUserCommand) -> Result<User, Self::Error> {
        let mut query = String::from("UPDATE users SET ");
        let mut updates = Vec::new();
        let mut bind_count = 0;

        if command.email.is_some() {
            updates.push("email = ?");
            bind_count += 1;
        }
        if command.full_name.is_some() {
            updates.push("full_name = ?");
            bind_count += 1;
        }
        if command.role.is_some() {
            updates.push("role = ?");
            bind_count += 1;
        }

        if bind_count == 0 {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = {} RETURNING *", command.id));

        let mut q = sqlx::query_as::<_, User>(&query);
        
        if let Some(email) = command.email {
            q = q.bind(email);
        }
        if let Some(full_name) = command.full_name {
            q = q.bind(full_name);
        }
        if let Some(role) = command.role {
            q = q.bind(role);
        }

        let user = q.fetch_one(&*self.pool).await?;
        Ok(user)
    }
}

pub struct ChangePasswordHandler {
    pool: Arc<SqlitePool>,
}

impl ChangePasswordHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<ChangePasswordCommand> for ChangePasswordHandler {
    type Error = AppError;

    async fn handle(&self, command: ChangePasswordCommand) -> Result<(), Self::Error> {
        // Get current user
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(command.user_id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Verify old password
        if !password::verify_password(&command.old_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("Invalid old password".to_string()));
        }

        // Hash new password
        let new_password_hash = password::hash_password(&command.new_password)?;

        // Update password
        sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
            .bind(&new_password_hash)
            .bind(command.user_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }
}

pub struct DeleteUserHandler {
    pool: Arc<SqlitePool>,
}

impl DeleteUserHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUserHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteUserCommand) -> Result<(), Self::Error> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(command.id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("User not found".to_string()));
        }

        Ok(())
    }
}

// ============ Query Handlers ============

pub struct GetUserHandler {
    pool: Arc<SqlitePool>,
}

impl GetUserHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetUserQuery> for GetUserHandler {
    type Error = AppError;

    async fn handle(&self, query: GetUserQuery) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(query.id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(user)
    }
}

pub struct GetUserByEmailHandler {
    pool: Arc<SqlitePool>,
}

impl GetUserByEmailHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetUserByEmailQuery> for GetUserByEmailHandler {
    type Error = AppError;

    async fn handle(&self, query: GetUserByEmailQuery) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(query.email)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(user)
    }
}

pub struct GetUserByUsernameHandler {
    pool: Arc<SqlitePool>,
}

impl GetUserByUsernameHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetUserByUsernameQuery> for GetUserByUsernameHandler {
    type Error = AppError;

    async fn handle(&self, query: GetUserByUsernameQuery) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(query.username)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(user)
    }
}

pub struct ListUsersHandler {
    pool: Arc<SqlitePool>,
}

impl ListUsersHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListUsersQuery> for ListUsersHandler {
    type Error = AppError;

    async fn handle(&self, query: ListUsersQuery) -> Result<Vec<User>, Self::Error> {
        let mut sql = String::from("SELECT * FROM users");
        
        if let Some(role) = &query.role {
            sql.push_str(&format!(" WHERE role = '{}'", role));
        }
        
        sql.push_str(" ORDER BY created_at DESC");
        
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }
        
        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let users = sqlx::query_as::<_, User>(&sql)
            .fetch_all(&*self.pool)
            .await?;

        Ok(users)
    }
}
