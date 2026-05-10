use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::authz::permissions as perm;
use crate::authz::AuthContext;
use crate::core::shared::pagination::{PagedSearchParams, PaginatedResponse};
use crate::domains::rbac::handlers::{PermissionDto, RbacCommandHandler, RbacQueryHandler, RoleDto};
use crate::domains::rbac::{
    AssignPermissionToRoleCommand, AssignRoleToUserCommand, CreatePermissionCommand,
    CreateRoleCommand, DeletePermissionCommand, DeleteRoleCommand, ListPermissionsQuery,
    ListRolesQuery, RevokePermissionFromRoleCommand, RevokeRoleFromUserCommand,
    UpdatePermissionCommand, UpdateRoleCommand,
};
use crate::error::{AppError, AppResult};

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
pub struct CreateRoleRequest {
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateRoleRequest {
    pub slug: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CreatePermissionRequest {
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdatePermissionRequest {
    pub description: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RoleWithAssignmentDto {
    pub id: String,
    pub slug: String,
    pub description: Option<String>,
    pub is_active: i64,
    pub created_at: String,
    pub assigned: bool,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct PermissionWithAssignmentDto {
    pub code: String,
    pub description: Option<String>,
    pub is_active: i64,
    pub created_at: String,
    pub assigned: bool,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct UserWithAssignmentDto {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub is_active: bool,
    pub status: String,
    pub created_at: String,
    pub assigned: bool,
}

// ── Role handlers ─────────────────────────────────────────────────────────────

/// List all roles — read-only, available to all authenticated users (e.g. for UI dropdowns).
pub async fn list_roles(State(state): State<AppState>) -> AppResult<Json<Vec<RoleDto>>> {
    let h = Arc::new(RbacQueryHandler::new(state.pool.clone()));
    Ok(Json(state.query_bus.dispatch_with_handler(ListRolesQuery, h).await?))
}

/// List roles with pagination + search (admin UI).
/// GET /api/admin/rbac/roles/paged?page=&limit=&search=
pub async fn list_roles_paged(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<RoleDto>>> {
    ctx.require(perm::ROLE_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM roles r
        WHERE ($1 = '' OR r.slug ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, RoleDto>(
        r#"
        SELECT r.id, r.slug, r.description, r.is_active, r.created_at
        FROM roles r
        WHERE ($1 = '' OR r.slug ILIKE '%' || $1 || '%')
        ORDER BY r.slug ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}

pub async fn create_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<(StatusCode, Json<RoleDto>)> {
    ctx.require(perm::ROLE_MANAGE)?;

    let cmd = CreateRoleCommand { slug: payload.slug, description: payload.description };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    let role = state.command_bus.dispatch_with_handler(cmd, h).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> AppResult<Json<RoleDto>> {
    ctx.require(perm::ROLE_MANAGE)?;

    let cmd = UpdateRoleCommand {
        role_id,
        slug: payload.slug,
        description: payload.description,
        is_active: payload.is_active,
    };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    Ok(Json(state.command_bus.dispatch_with_handler(cmd, h).await?))
}

pub async fn delete_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(DeleteRoleCommand { role_id }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Permission handlers ───────────────────────────────────────────────────────

/// List all permissions — read-only, available to all authenticated users.
pub async fn list_permissions(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<PermissionDto>>> {
    let h = Arc::new(RbacQueryHandler::new(state.pool.clone()));
    Ok(Json(state.query_bus.dispatch_with_handler(ListPermissionsQuery, h).await?))
}

/// List permissions with pagination + search (admin UI).
/// GET /api/admin/rbac/permissions/paged?page=&limit=&search=
pub async fn list_permissions_paged(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<PermissionDto>>> {
    ctx.require(perm::PERMISSION_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM permissions p
        WHERE ($1 = '' OR p.code ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, PermissionDto>(
        r#"
        SELECT p.code, p.description, p.is_active, p.created_at
        FROM permissions p
        WHERE ($1 = '' OR p.code ILIKE '%' || $1 || '%')
        ORDER BY p.code ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}

pub async fn create_permission(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreatePermissionRequest>,
) -> AppResult<(StatusCode, Json<PermissionDto>)> {
    ctx.require(perm::PERMISSION_MANAGE)?;

    let cmd = CreatePermissionCommand { code: payload.code, description: payload.description };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    let p = state.command_bus.dispatch_with_handler(cmd, h).await?;
    Ok((StatusCode::CREATED, Json(p)))
}

pub async fn update_permission(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(payload): Json<UpdatePermissionRequest>,
) -> AppResult<Json<PermissionDto>> {
    ctx.require(perm::PERMISSION_MANAGE)?;

    let cmd = UpdatePermissionCommand {
        code,
        description: payload.description,
        is_active: payload.is_active,
    };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    Ok(Json(state.command_bus.dispatch_with_handler(cmd, h).await?))
}

pub async fn delete_permission(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<StatusCode> {
    ctx.require(perm::PERMISSION_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(DeletePermissionCommand { code }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Role ↔ Permission assignment ─────────────────────────────────────────────

pub async fn assign_permission_to_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((role_id, code)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state
        .command_bus
        .dispatch_with_handler(AssignPermissionToRoleCommand { role_id, code }, h)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_permission_from_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((role_id, code)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state
        .command_bus
        .dispatch_with_handler(RevokePermissionFromRoleCommand { role_id, code }, h)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── User ↔ Role assignment ────────────────────────────────────────────────────

pub async fn assign_role_to_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state
        .command_bus
        .dispatch_with_handler(AssignRoleToUserCommand { user_id, role_id }, h)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_role_from_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state
        .command_bus
        .dispatch_with_handler(RevokeRoleFromUserCommand { user_id, role_id }, h)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Paged M2M listing (immediate-toggle UI support) ────────────────────────────

// ── Extra role endpoints ──────────────────────────────────────────────────────

/// Get a single role by id.
/// GET /api/admin/rbac/roles/:role_id
pub async fn get_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
) -> AppResult<Json<RoleDto>> {
    ctx.require(perm::ROLE_MANAGE)?;

    let role_uuid = uuid::Uuid::parse_str(&role_id)
        .map_err(|_| AppError::BadRequest("Invalid role id".into()))?;

    let row = sqlx::query_as::<_, RoleDto>(
        "SELECT id, slug, description, is_active, created_at FROM roles WHERE id = $1",
    )
    .bind(role_uuid)
    .fetch_optional(state.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Role {role_id} not found")))?;

    Ok(Json(row))
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkDeleteRolesRequest {
    pub ids: Vec<String>,
}

/// Bulk-delete roles by id list.
/// POST /api/admin/rbac/roles/bulk-delete
pub async fn bulk_delete_roles(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<BulkDeleteRolesRequest>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;
    if payload.ids.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    // Convert string UUIDs, ignore invalid ones
    let uuids: Vec<uuid::Uuid> = payload
        .ids
        .iter()
        .filter_map(|s| uuid::Uuid::parse_str(s).ok())
        .collect();

    sqlx::query("DELETE FROM roles WHERE id = ANY($1)")
        .bind(&uuids)
        .execute(state.pool())
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Permission matrix ─────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct MatrixRoleDto {
    pub id: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct MatrixPermDto {
    pub id: String,
    pub code: String,
    pub description: Option<String>,
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct MatrixAssignment {
    pub role_id: String,
    pub permission_id: String,
}

#[derive(Debug, serde::Serialize)]
pub struct PermissionMatrixResponse {
    pub roles: Vec<MatrixRoleDto>,
    pub permissions: Vec<MatrixPermDto>,
    pub assignments: Vec<MatrixAssignment>,
}

/// Full role × permission matrix.
/// GET /api/admin/rbac/matrix
pub async fn get_permission_matrix(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<PermissionMatrixResponse>> {
    ctx.require(perm::ROLE_MANAGE)?;

    let roles = sqlx::query_as::<_, MatrixRoleDto>(
        "SELECT id::text AS id, slug, description FROM roles WHERE is_active = true ORDER BY slug",
    )
    .fetch_all(state.pool())
    .await?;

    let permissions = sqlx::query_as::<_, MatrixPermDto>(
        "SELECT code AS id, code, description FROM permissions WHERE is_active = true ORDER BY code",
    )
    .fetch_all(state.pool())
    .await?;

    let assignments = sqlx::query_as::<_, MatrixAssignment>(
        r#"
        SELECT rp.role_id::text AS role_id, rp.permission_code AS permission_id
        FROM role_permissions rp
        JOIN roles r ON r.id = rp.role_id AND r.is_active = true
        JOIN permissions p ON p.code = rp.permission_code AND p.is_active = true
        "#,
    )
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PermissionMatrixResponse { roles, permissions, assignments }))
}

#[derive(Debug, serde::Deserialize)]
pub struct SetRolePermissionsRequest {
    pub permission_ids: Vec<String>,
}

/// Replace all permissions for a role (bulk set).
/// PUT /api/admin/rbac/roles/:role_id/permissions
pub async fn set_role_permissions(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
    Json(payload): Json<SetRolePermissionsRequest>,
) -> AppResult<StatusCode> {
    ctx.require(perm::ROLE_MANAGE)?;

    let role_uuid = uuid::Uuid::parse_str(&role_id)
        .map_err(|_| AppError::BadRequest("Invalid role id".into()))?;

    let mut tx = state.pool().begin().await?;

    // Delete all existing assignments for this role
    sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
        .bind(role_uuid)
        .execute(&mut *tx)
        .await?;

    // Insert new assignments (ignore unknown permission codes)
    for code in &payload.permission_ids {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_code)
            SELECT $1, $2 WHERE EXISTS (SELECT 1 FROM permissions WHERE code = $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(role_uuid)
        .bind(code)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Invalidate permission cache for all users holding this role
    if let Ok(user_ids) = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT user_id FROM user_roles WHERE role_id = $1",
    )
    .bind(role_uuid)
    .fetch_all(state.pool())
    .await
    {
        for uid in user_ids {
            crate::authz::invalidate_permission_cache(uid);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

/// List all roles + assignment status for a given user (admin UI).
/// GET /api/admin/rbac/users/:user_id/roles?page=&limit=&search=
pub async fn list_roles_for_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<RoleWithAssignmentDto>>> {
    ctx.require(perm::ROLE_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM roles r
        WHERE ($1 = '' OR r.slug ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, RoleWithAssignmentDto>(
        r#"
        SELECT
            r.id::text AS id,
            r.slug,
            r.description,
            CASE WHEN r.is_active THEN 1 ELSE 0 END AS is_active,
            r.created_at::text AS created_at,
            EXISTS(
                SELECT 1 FROM user_roles ur
                WHERE ur.user_id = $1::uuid AND ur.role_id = r.id
            ) AS assigned
        FROM roles r
        WHERE ($2 = '' OR r.slug ILIKE '%' || $2 || '%')
        ORDER BY assigned DESC, r.slug ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&user_id)
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}

/// List all permissions + assignment status for a given role (admin UI).
/// GET /api/admin/rbac/roles/:role_id/permissions?page=&limit=&search=
pub async fn list_permissions_for_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<PermissionWithAssignmentDto>>> {
    ctx.require(perm::ROLE_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM permissions p
        WHERE ($1 = '' OR p.code ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, PermissionWithAssignmentDto>(
        r#"
        SELECT
            p.code,
            p.description,
            CASE WHEN p.is_active THEN 1 ELSE 0 END AS is_active,
            p.created_at::text AS created_at,
            EXISTS(
                SELECT 1 FROM role_permissions rp
                WHERE rp.role_id = $1::uuid AND rp.permission_code = p.code
            ) AS assigned
        FROM permissions p
        WHERE ($2 = '' OR p.code ILIKE '%' || $2 || '%')
        ORDER BY assigned DESC, p.code ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&role_id)
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}

/// List all users + assignment status for a given role (admin UI).
/// GET /api/admin/rbac/roles/:role_id/users?page=&limit=&search=
pub async fn list_users_for_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<UserWithAssignmentDto>>> {
    ctx.require(perm::ROLE_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM users u
        WHERE ($1 = '' OR u.email ILIKE '%' || $1 || '%' OR u.full_name ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, UserWithAssignmentDto>(
        r#"
        SELECT
            u.id::text AS id,
            u.email,
            u.full_name,
            u.is_active,
            u.status,
            u.created_at::text AS created_at,
            EXISTS(
                SELECT 1 FROM user_roles ur
                WHERE ur.user_id = u.id AND ur.role_id = $1::uuid
            ) AS assigned
        FROM users u
        WHERE ($2 = '' OR u.email ILIKE '%' || $2 || '%' OR u.full_name ILIKE '%' || $2 || '%')
        ORDER BY assigned DESC, u.email ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&role_id)
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}
