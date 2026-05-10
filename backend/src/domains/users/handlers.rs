use async_trait::async_trait;
use sqlx::{QueryBuilder, Postgres, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::authz::load_system_settings;
use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::shared::append_aggregate_history;
use crate::domains::users::{
    RegisterUserCommand, UpdateUserCommand, ChangePasswordCommand, DeleteUserCommand,
    GetUserQuery, GetUserByEmailQuery, ListUsersQuery,
};
use crate::error::AppError;
use crate::models::User;
use crate::utils::password;

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Verify that a role slug exists and is active.
async fn ensure_role_exists(pool: &PgPool, slug: &str) -> Result<(), AppError> {
    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM roles WHERE slug = $1 AND is_active = TRUE LIMIT 1")
            .bind(slug)
            .fetch_optional(pool)
            .await?;
    if exists.is_none() {
        return Err(AppError::ValidationError(format!(
            "Role '{}' does not exist or is inactive",
            slug
        )));
    }
    Ok(())
}

/// Return a comma-separated list of role slugs assigned to a user (for audit).
async fn get_user_role_slugs(pool: &PgPool, user_id: Uuid) -> Result<String, AppError> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT r.slug FROM roles r
         JOIN user_roles ur ON ur.role_id = r.id
         WHERE ur.user_id = $1
         ORDER BY r.slug",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(s,)| s).collect::<Vec<_>>().join(","))
}

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
        let pool = &*self.pool;

        // ── Duplicate check ───────────────────────────────────────────────────
        let existing: Option<(i32,)> =
            sqlx::query_as("SELECT 1 FROM users WHERE email = $1 LIMIT 1")
                .bind(&command.email)
                .fetch_optional(pool)
                .await?;
        if existing.is_some() {
            return Err(AppError::Conflict("User already exists".to_string()));
        }

        // ── Determine role slugs ──────────────────────────────────────────────
        let settings = load_system_settings(pool).await?;
        let role_slugs: Vec<String> = if let Some(ref slug) = command.role {
            ensure_role_exists(pool, slug).await?;
            vec![slug.clone()]
        } else {
            settings.default_role_slugs.clone()
        };

        // ── Insert user (no role column — source of truth is user_roles) ─────
        let password_hash = password::hash_password(&command.password)?;
        let user_id = Uuid::new_v4();

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(&command.email)
        .bind(&password_hash)
        .bind(&command.full_name)
        .fetch_one(pool)
        .await?;

        // ── Assign RBAC roles ─────────────────────────────────────────────────
        for slug in &role_slugs {
            sqlx::query(
                r#"
                INSERT INTO user_roles (user_id, role_id)
                SELECT $1, r.id FROM roles r WHERE r.slug = $2 AND r.is_active = TRUE LIMIT 1
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(user_id)
            .bind(slug)
            .execute(pool)
            .await?;
        }

        // ── Assign default branch ─────────────────────────────────────────────
        let branch_uuid = Uuid::parse_str(&settings.default_branch_id).map_err(|_| {
            AppError::InternalServerError(format!(
                "system_settings.default_branch_id is not a valid UUID: {}",
                settings.default_branch_id
            ))
        })?;

        sqlx::query(
            "INSERT INTO user_branches (user_id, branch_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(branch_uuid)
        .execute(pool)
        .await?;

        // ── Audit ─────────────────────────────────────────────────────────────
        let roles_label = role_slugs.join(",");
        append_aggregate_history(
            &self.pool,
            "user",
            &user.id.to_string(),
            "CREATE",
            None,
            Some(&roles_label),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "email": user.email,
                "full_name": user.full_name,
                "roles": role_slugs
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

        if let Some(ref slug) = command.role {
            ensure_role_exists(&self.pool, slug).await?;
        }

        let user_uuid = Uuid::parse_str(&command.id)
            .map_err(|_| AppError::ValidationError("Invalid user ID".to_string()))?;

        let before = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_uuid)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let old_roles = get_user_role_slugs(&self.pool, user_uuid).await?;

        // ── Update profile fields (role lives in user_roles, not here) ────────
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
        separated.push("updated_at = NOW()");
        drop(separated);

        qb.push(" WHERE id = ").push_bind(user_uuid);
        qb.build().execute(&*self.pool).await?;

        // ── Replace role assignment in user_roles if provided ─────────────────
        if let Some(ref slug) = command.role {
            let mut tx = self.pool.begin().await?;
            sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
                .bind(user_uuid)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id)
                 SELECT $1, id FROM roles WHERE slug = $2 AND is_active = TRUE
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_uuid)
            .bind(slug)
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            crate::authz::invalidate_permission_cache(user_uuid);
        }

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_uuid)
            .fetch_one(&*self.pool)
            .await?;

        let new_roles = get_user_role_slugs(&self.pool, user_uuid).await?;

        append_aggregate_history(
            &self.pool,
            "user",
            &user.id.to_string(),
            "UPDATE",
            Some(&old_roles),
            Some(&new_roles),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "before": {
                    "email": before.email,
                    "full_name": before.full_name,
                    "avatar_url": before.avatar_url,
                    "roles": old_roles
                },
                "after": {
                    "email": user.email,
                    "full_name": user.full_name,
                    "avatar_url": user.avatar_url,
                    "roles": new_roles
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
        let uid = Uuid::parse_str(&command.user_id)
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(uid)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        if !password::verify_password(&command.old_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("Invalid old password".to_string()));
        }

        let new_password_hash = password::hash_password(&command.new_password)?;

        sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
            .bind(&new_password_hash)
            .bind(uid)
            .execute(&*self.pool)
            .await?;

        append_aggregate_history(
            &self.pool,
            "user",
            &uid.to_string(),
            "PASSWORD_CHANGE",
            None,
            None,
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({"password_changed": true})),
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
        let user_uuid = Uuid::parse_str(&command.id)
            .map_err(|_| AppError::ValidationError("Invalid user ID".to_string()))?;

        let existing = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_uuid)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Fetch roles before delete (user_roles cascade-deletes with the user row)
        let role_slugs = get_user_role_slugs(&*self.pool, user_uuid).await?;

        append_aggregate_history(
            &self.pool,
            "user",
            &command.id,
            "DELETE",
            Some(&role_slugs),
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
            .bind(user_uuid)
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
        let uid = Uuid::parse_str(&query.id)
            .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(uid)
                .fetch_optional(&*self.pool)
                .await?,
        )
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
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
                .bind(query.email)
                .fetch_optional(&*self.pool)
                .await?,
        )
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
        let limit = query.limit.unwrap_or(50).clamp(1, 200);
        let offset = query.offset.unwrap_or(0).max(0);

        let users = if let Some(role) = query.role {
            sqlx::query_as::<_, User>(
                r#"
                SELECT u.* FROM users u
                WHERE EXISTS (
                    SELECT 1 FROM user_roles ur
                    JOIN roles r ON r.id = ur.role_id
                    WHERE ur.user_id = u.id AND r.slug = $1
                )
                ORDER BY u.created_at DESC LIMIT $2 OFFSET $3
                "#,
            )
            .bind(role)
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?
        } else {
            sqlx::query_as::<_, User>(
                "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?
        };

        Ok(users)
    }
}
