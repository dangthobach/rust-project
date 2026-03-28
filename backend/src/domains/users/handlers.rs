use async_trait::async_trait;
use sqlx::{QueryBuilder, Postgres, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::shared::append_aggregate_history;
use crate::domains::users::{
    RegisterUserCommand, UpdateUserCommand, ChangePasswordCommand, DeleteUserCommand,
    GetUserQuery, GetUserByEmailQuery, ListUsersQuery,
};
use crate::error::AppError;
use crate::models::User;
use crate::utils::password;

// ============ Command Handlers ============

pub struct RegisterUserHandler {
    pool: Arc<PgPool>,
}

impl RegisterUserHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for RegisterUserHandler {
    type Error = AppError;

    async fn handle(&self, command: RegisterUserCommand) -> Result<User, Self::Error> {
        let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(&command.email)
            .fetch_optional(&*self.pool)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("User already exists".to_string()));
        }

        let password_hash = password::hash_password(&command.password)?;
        let user_id = Uuid::new_v4();
        let role = command.role.unwrap_or_else(|| "user".to_string());

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, role)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&command.email)
        .bind(&password_hash)
        .bind(&command.full_name)
        .bind(&role)
        .fetch_one(&*self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT $1, r.id FROM roles r WHERE LOWER(r.slug) = LOWER($2) LIMIT 1
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(&role)
        .execute(&*self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO user_branches (user_id, branch_id)
            VALUES ($1, '00000000-0000-0000-0000-0000000000b1'::uuid)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .execute(&*self.pool)
        .await?;

        let aggregate_id = user.id.to_string();
        append_aggregate_history(
            &self.pool,
            "user",
            &aggregate_id,
            "CREATE",
            None,
            Some(&role),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "email": user.email,
                "full_name": user.full_name,
                "role": user.role
            })),
        )
        .await?;

        Ok(user)
    }
}

pub struct UpdateUserHandler {
    pool: Arc<PgPool>,
}

impl UpdateUserHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateUserCommand> for UpdateUserHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateUserCommand) -> Result<User, Self::Error> {
        if command.email.is_none()
            && command.full_name.is_none()
            && command.avatar_url.is_none()
            && command.role.is_none()
        {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE users SET ");
        let mut separated = qb.separated(", ");

        if let Some(email) = &command.email {
            separated.push("email = ").push_bind(email);
        }
        if let Some(full_name) = &command.full_name {
            separated.push("full_name = ").push_bind(full_name);
        }
        if let Some(avatar_url) = &command.avatar_url {
            separated.push("avatar_url = ").push_bind(avatar_url);
        }
        if let Some(role) = &command.role {
            separated.push("role = ").push_bind(role);
        }
        separated.push("updated_at = NOW()");
        drop(separated);

        qb.push(" WHERE id = ").push_bind(&command.id);
        qb.build().execute(&*self.pool).await?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(&command.id)
            .fetch_one(&*self.pool)
            .await?;

        let aggregate_id = user.id.to_string();
        append_aggregate_history(
            &self.pool,
            "user",
            &aggregate_id,
            "UPDATE",
            Some(&before.role),
            Some(&user.role),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "before": {
                    "email": before.email,
                    "full_name": before.full_name,
                    "avatar_url": before.avatar_url,
                    "role": before.role
                },
                "after": {
                    "email": user.email,
                    "full_name": user.full_name,
                    "avatar_url": user.avatar_url,
                    "role": user.role
                }
            })),
        )
        .await?;

        Ok(user)
    }
}

pub struct ChangePasswordHandler {
    pool: Arc<PgPool>,
}

impl ChangePasswordHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<ChangePasswordCommand> for ChangePasswordHandler {
    type Error = AppError;

    async fn handle(&self, command: ChangePasswordCommand) -> Result<(), Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(&command.user_id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !password::verify_password(&command.old_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("Invalid old password".to_string()));
        }

        let new_password_hash = password::hash_password(&command.new_password)?;

        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(&new_password_hash)
            .bind(&command.user_id)
            .execute(&*self.pool)
            .await?;

        append_aggregate_history(
            &self.pool,
            "user",
            &command.user_id,
            "PASSWORD_CHANGE",
            None,
            None,
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "password_changed": true
            })),
        )
        .await?;

        Ok(())
    }
}

pub struct DeleteUserHandler {
    pool: Arc<PgPool>,
}

impl DeleteUserHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<DeleteUserCommand> for DeleteUserHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteUserCommand) -> Result<(), Self::Error> {
        let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        append_aggregate_history(
            &self.pool,
            "user",
            &command.id,
            "DELETE",
            Some(&existing.role),
            None,
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "email": existing.email,
                "full_name": existing.full_name
            })),
        )
        .await?;

        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(&command.id)
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
    pool: Arc<PgPool>,
}

impl GetUserHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetUserQuery> for GetUserHandler {
    type Error = AppError;

    async fn handle(&self, query: GetUserQuery) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(&query.id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(user)
    }
}

pub struct GetUserByEmailHandler {
    pool: Arc<PgPool>,
}

impl GetUserByEmailHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetUserByEmailQuery> for GetUserByEmailHandler {
    type Error = AppError;

    async fn handle(&self, query: GetUserByEmailQuery) -> Result<Option<User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(query.email)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(user)
    }
}

pub struct ListUsersHandler {
    pool: Arc<PgPool>,
}

impl ListUsersHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListUsersQuery> for ListUsersHandler {
    type Error = AppError;

    async fn handle(&self, query: ListUsersQuery) -> Result<Vec<User>, Self::Error> {
        let limit = query.limit.unwrap_or(50).max(1);
        let offset = query.offset.unwrap_or(0).max(0);

        let users = if let Some(role) = query.role {
            sqlx::query_as::<_, User>(
                "SELECT * FROM users WHERE role = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
            )
            .bind(role)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?
        } else {
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2")
                .bind(limit)
                .bind(offset)
                .fetch_all(&*self.pool)
                .await?
        };

        Ok(users)
    }
}
