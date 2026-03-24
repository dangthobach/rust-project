use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::authz::{invalidate_all_permission_cache, invalidate_permission_cache};
use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::domains::rbac::commands::*;
use crate::domains::rbac::queries::*;
use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RoleDto {
    pub id: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct PermissionDto {
    pub code: String,
    pub description: Option<String>,
    pub is_active: i64,
    pub created_at: String,
}

fn validate_slug(slug: &str) -> Result<String, AppError> {
    let s = slug.trim().to_ascii_lowercase();
    if s.is_empty() {
        return Err(AppError::ValidationError("slug is required".to_string()));
    }
    if !s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-') {
        return Err(AppError::ValidationError("slug contains invalid characters".to_string()));
    }
    Ok(s)
}

fn validate_permission_code(code: &str) -> Result<String, AppError> {
    let c = code.trim().to_ascii_lowercase();
    if c.is_empty() {
        return Err(AppError::ValidationError("permission code is required".to_string()));
    }
    if !c.chars().all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '.' || x == '_' || x == '-') {
        return Err(AppError::ValidationError("permission code contains invalid characters".to_string()));
    }
    Ok(c)
}

pub struct RbacCommandHandler { pool: Arc<SqlitePool> }
impl RbacCommandHandler { pub fn new(pool: Arc<SqlitePool>) -> Self { Self { pool } } }

pub struct RbacQueryHandler { pool: Arc<SqlitePool> }
impl RbacQueryHandler { pub fn new(pool: Arc<SqlitePool>) -> Self { Self { pool } } }

#[async_trait]
impl CommandHandler<CreateRoleCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: CreateRoleCommand) -> Result<RoleDto, Self::Error> {
        let slug = validate_slug(&c.slug)?;
        let role = sqlx::query_as::<_, RoleDto>("INSERT INTO roles (id, slug, description, is_active) VALUES (?1, ?2, ?3, 1) RETURNING *")
            .bind(Uuid::new_v4().to_string()).bind(slug).bind(c.description)
            .fetch_one(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(role)
    }
}

#[async_trait]
impl CommandHandler<UpdateRoleCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: UpdateRoleCommand) -> Result<RoleDto, Self::Error> {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM roles WHERE id = ?1").bind(&c.role_id).fetch_optional(&*self.pool).await?;
        if exists.is_none() { return Err(AppError::NotFound("Role not found".to_string())); }
        let slug = match c.slug { Some(s) => Some(validate_slug(&s)?), None => None };
        sqlx::query("UPDATE roles SET slug = COALESCE(?1, slug), description = COALESCE(?2, description), is_active = COALESCE(?3, is_active) WHERE id = ?4")
            .bind(slug).bind(c.description).bind(c.is_active.map(|v| if v {1} else {0})).bind(&c.role_id)
            .execute(&*self.pool).await?;
        let role = sqlx::query_as::<_, RoleDto>("SELECT * FROM roles WHERE id = ?1").bind(&c.role_id).fetch_one(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(role)
    }
}

#[async_trait]
impl CommandHandler<DeleteRoleCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: DeleteRoleCommand) -> Result<(), Self::Error> {
        let r = sqlx::query("DELETE FROM roles WHERE id = ?1").bind(&c.role_id).execute(&*self.pool).await?;
        if r.rows_affected() == 0 { return Err(AppError::NotFound("Role not found".to_string())); }
        invalidate_all_permission_cache();
        Ok(())
    }
}

#[async_trait]
impl CommandHandler<CreatePermissionCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: CreatePermissionCommand) -> Result<PermissionDto, Self::Error> {
        let code = validate_permission_code(&c.code)?;
        let p = sqlx::query_as::<_, PermissionDto>("INSERT INTO permissions (code, description, is_active) VALUES (?1, ?2, 1) RETURNING *")
            .bind(code).bind(c.description).fetch_one(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(p)
    }
}

#[async_trait]
impl CommandHandler<UpdatePermissionCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: UpdatePermissionCommand) -> Result<PermissionDto, Self::Error> {
        let code = validate_permission_code(&c.code)?;
        let exists: Option<String> = sqlx::query_scalar("SELECT code FROM permissions WHERE code = ?1").bind(&code).fetch_optional(&*self.pool).await?;
        if exists.is_none() { return Err(AppError::NotFound("Permission not found".to_string())); }
        sqlx::query("UPDATE permissions SET description = COALESCE(?1, description), is_active = COALESCE(?2, is_active) WHERE code = ?3")
            .bind(c.description).bind(c.is_active.map(|v| if v {1} else {0})).bind(&code)
            .execute(&*self.pool).await?;
        let p = sqlx::query_as::<_, PermissionDto>("SELECT * FROM permissions WHERE code = ?1").bind(&code).fetch_one(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(p)
    }
}

#[async_trait]
impl CommandHandler<DeletePermissionCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: DeletePermissionCommand) -> Result<(), Self::Error> {
        let code = validate_permission_code(&c.code)?;
        let r = sqlx::query("DELETE FROM permissions WHERE code = ?1").bind(&code).execute(&*self.pool).await?;
        if r.rows_affected() == 0 { return Err(AppError::NotFound("Permission not found".to_string())); }
        invalidate_all_permission_cache();
        Ok(())
    }
}

#[async_trait]
impl CommandHandler<AssignPermissionToRoleCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: AssignPermissionToRoleCommand) -> Result<(), Self::Error> {
        let code = validate_permission_code(&c.code)?;
        sqlx::query("INSERT OR IGNORE INTO role_permissions (role_id, permission_code) VALUES (?1, ?2)")
            .bind(&c.role_id).bind(&code).execute(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(())
    }
}

#[async_trait]
impl CommandHandler<RevokePermissionFromRoleCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: RevokePermissionFromRoleCommand) -> Result<(), Self::Error> {
        let code = validate_permission_code(&c.code)?;
        sqlx::query("DELETE FROM role_permissions WHERE role_id = ?1 AND permission_code = ?2")
            .bind(&c.role_id).bind(&code).execute(&*self.pool).await?;
        invalidate_all_permission_cache();
        Ok(())
    }
}

#[async_trait]
impl CommandHandler<AssignRoleToUserCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: AssignRoleToUserCommand) -> Result<(), Self::Error> {
        let user_uuid = Uuid::parse_str(&c.user_id).map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?;
        sqlx::query("INSERT OR IGNORE INTO user_roles (user_id, role_id) VALUES (?1, ?2)")
            .bind(&c.user_id).bind(&c.role_id).execute(&*self.pool).await?;
        invalidate_permission_cache(user_uuid);
        Ok(())
    }
}

#[async_trait]
impl CommandHandler<RevokeRoleFromUserCommand> for RbacCommandHandler {
    type Error = AppError;
    async fn handle(&self, c: RevokeRoleFromUserCommand) -> Result<(), Self::Error> {
        let user_uuid = Uuid::parse_str(&c.user_id).map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?;
        sqlx::query("DELETE FROM user_roles WHERE user_id = ?1 AND role_id = ?2")
            .bind(&c.user_id).bind(&c.role_id).execute(&*self.pool).await?;
        invalidate_permission_cache(user_uuid);
        Ok(())
    }
}

#[async_trait]
impl QueryHandler<ListRolesQuery> for RbacQueryHandler {
    type Error = AppError;
    async fn handle(&self, _q: ListRolesQuery) -> Result<Vec<RoleDto>, Self::Error> {
        Ok(sqlx::query_as::<_, RoleDto>("SELECT * FROM roles ORDER BY slug ASC").fetch_all(&*self.pool).await?)
    }
}

#[async_trait]
impl QueryHandler<ListPermissionsQuery> for RbacQueryHandler {
    type Error = AppError;
    async fn handle(&self, _q: ListPermissionsQuery) -> Result<Vec<PermissionDto>, Self::Error> {
        Ok(sqlx::query_as::<_, PermissionDto>("SELECT * FROM permissions ORDER BY code ASC").fetch_all(&*self.pool).await?)
    }
}
