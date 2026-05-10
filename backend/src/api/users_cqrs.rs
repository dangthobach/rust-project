use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::authz::permissions as perm;
use crate::authz::AuthContext;
use crate::domains::users::{
    ChangePasswordCommand, DeleteUserCommand, GetUserQuery, ListUsersQuery, RegisterUserCommand,
    UpdateUserCommand,
};
use crate::domains::users::handlers::{
    ChangePasswordHandler, DeleteUserHandler, GetUserHandler, ListUsersHandler,
    RegisterUserHandler, UpdateUserHandler,
};
use crate::error::{AppError, AppResult};
use crate::models::User;

// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateUserPayload {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub email: String,
    pub full_name: String,
    pub password: String,
    /// Optional role slug; falls back to system_settings.default_role_slugs.
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserPayload {
    pub email: Option<String>,
    pub full_name: Option<String>,
    /// Must be the slug of an existing active role if provided.
    pub role: Option<String>,
    pub avatar_url: Option<String>,
}

// ── Self-service handlers (all authenticated users) ───────────────────────────

pub async fn get_current_user(
    Extension(user): Extension<User>,
) -> AppResult<Json<User>> {
    Ok(Json(user))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<User>> {
    let handler = Arc::new(GetUserHandler::new(state.pool.clone()));
    let user = state
        .query_bus
        .dispatch_with_handler(GetUserQuery { id: id.clone() }, handler)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", id)))?;
    Ok(Json(user))
}

pub async fn update_user_self(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserPayload>,
) -> AppResult<Json<User>> {
    if user_id != id {
        return Err(AppError::Forbidden("Cannot update other users".to_string()));
    }

    let command = UpdateUserCommand {
        id,
        email: None,
        full_name: payload.full_name,
        avatar_url: payload.avatar_url,
        role: None, // self-service cannot change own role
        actor_id: Some(user_id),
    };
    let handler = Arc::new(UpdateUserHandler::new(state.pool.clone()));
    Ok(Json(state.command_bus.dispatch_with_handler(command, handler).await?))
}

pub async fn change_password(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordPayload>,
) -> AppResult<Json<serde_json::Value>> {
    let command = ChangePasswordCommand {
        user_id: user_id.clone(),
        old_password: payload.current_password,
        new_password: payload.new_password,
        actor_id: Some(user_id),
    };
    let handler = Arc::new(ChangePasswordHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(command, handler).await?;
    Ok(Json(serde_json::json!({"message": "Password changed successfully"})))
}

// ── Admin handlers (require user.manage) ─────────────────────────────────────

pub async fn list_users_admin(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> AppResult<Json<Vec<User>>> {
    ctx.require(perm::USER_MANAGE)?;

    let handler = Arc::new(ListUsersHandler::new(state.pool.clone()));
    let users = state.query_bus.dispatch_with_handler(params, handler).await?;
    Ok(Json(users))
}

pub async fn create_user_admin(
    Extension(ctx): Extension<AuthContext>,
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> AppResult<(StatusCode, Json<User>)> {
    ctx.require(perm::USER_MANAGE)?;

    let command = RegisterUserCommand {
        email: payload.email,
        password: payload.password,
        full_name: payload.full_name,
        role: payload.role,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(RegisterUserHandler::new(state.pool.clone()));
    let user = state.command_bus.dispatch_with_handler(command, handler).await?;

    let _ = state
        .kafka_publisher
        .publish(
            "crm.domain.user",
            &user.id.to_string(),
            &serde_json::json!({
                "event_type": "UserCreated",
                "user_id": user.id,
                "occurred_at": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        )
        .await;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user_admin(
    Extension(ctx): Extension<AuthContext>,
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AdminUpdateUserPayload>,
) -> AppResult<Json<User>> {
    ctx.require(perm::USER_MANAGE)?;

    let command = UpdateUserCommand {
        id,
        email: payload.email,
        full_name: payload.full_name,
        avatar_url: payload.avatar_url,
        role: payload.role,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(UpdateUserHandler::new(state.pool.clone()));
    let user = state.command_bus.dispatch_with_handler(command, handler).await?;

    let _ = state
        .kafka_publisher
        .publish(
            "crm.domain.user",
            &user.id.to_string(),
            &serde_json::json!({
                "event_type": "UserUpdated",
                "user_id": user.id,
                "occurred_at": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        )
        .await;

    Ok(Json(user))
}

pub async fn delete_user_admin(
    Extension(ctx): Extension<AuthContext>,
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    ctx.require(perm::USER_MANAGE)?;

    let command = DeleteUserCommand { id: id.clone(), actor_id: Some(actor_id) };
    let handler = Arc::new(DeleteUserHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(command, handler).await?;

    let _ = state
        .kafka_publisher
        .publish(
            "crm.domain.user",
            &id,
            &serde_json::json!({
                "event_type": "UserDeleted",
                "user_id": id,
                "occurred_at": chrono::Utc::now().to_rfc3339()
            })
            .to_string(),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

// ── User-Role assignment endpoints ────────────────────────────────────────────

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct UserWithRolesDto {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub is_active: bool,
    pub created_at: String,
    pub roles: serde_json::Value, // JSON array: [{id, slug}]
}

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub(crate) struct UserWithRolesRow {
    id: String,
    email: String,
    full_name: String,
    is_active: bool,
    created_at: String,
    roles: serde_json::Value,
}

/// List users with their assigned roles (paginated).
/// GET /api/admin/users/with-roles?page=&limit=&search=
pub async fn list_users_with_roles(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<crate::core::shared::pagination::PagedSearchParams>,
) -> AppResult<Json<crate::core::shared::pagination::PaginatedResponse<UserWithRolesRow>>> {
    ctx.require(perm::USER_MANAGE)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM users u
           WHERE ($1 = '' OR u.email ILIKE '%' || $1 || '%' OR u.full_name ILIKE '%' || $1 || '%')"#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, UserWithRolesRow>(
        r#"
        SELECT
            u.id::text AS id,
            u.email,
            u.full_name,
            u.is_active,
            u.created_at::text AS created_at,
            COALESCE(
                json_agg(
                    json_build_object('id', r.id::text, 'slug', r.slug)
                ) FILTER (WHERE r.id IS NOT NULL),
                '[]'::json
            ) AS roles
        FROM users u
        LEFT JOIN user_roles ur ON ur.user_id = u.id
        LEFT JOIN roles r ON r.id = ur.role_id
        WHERE ($1 = '' OR u.email ILIKE '%' || $1 || '%' OR u.full_name ILIKE '%' || $1 || '%')
        GROUP BY u.id
        ORDER BY u.email ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(crate::core::shared::pagination::PaginatedResponse::new(items, total, pagination)))
}

#[derive(Debug, serde::Deserialize)]
pub struct AssignRolePayload {
    pub role: String, // role slug
}

/// Assign a single role to a user (replaces all existing roles).
/// PUT /api/admin/users/:id/role
pub async fn assign_user_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(payload): Json<AssignRolePayload>,
) -> AppResult<StatusCode> {
    ctx.require(perm::USER_MANAGE)?;

    let user_uuid = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user id".into()))?;

    let role = sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT id FROM roles WHERE slug = $1 AND is_active = true",
    )
    .bind(&payload.role)
    .fetch_optional(state.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Role '{}' not found", payload.role)))?;

    let mut tx = state.pool().begin().await?;
    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_uuid)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_uuid)
    .bind(role.0)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    crate::authz::invalidate_permission_cache(user_uuid);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Deserialize)]
pub struct BulkAssignRolePayload {
    pub user_ids: Vec<String>,
    pub role: String,
}

/// Bulk assign a role to multiple users (replaces each user's roles).
/// POST /api/admin/users/bulk-assign-role
pub async fn bulk_assign_user_role(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<BulkAssignRolePayload>,
) -> AppResult<StatusCode> {
    ctx.require(perm::USER_MANAGE)?;
    if payload.user_ids.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    let role = sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT id FROM roles WHERE slug = $1 AND is_active = true",
    )
    .bind(&payload.role)
    .fetch_optional(state.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Role '{}' not found", payload.role)))?;

    let user_uuids: Vec<uuid::Uuid> = payload
        .user_ids
        .iter()
        .filter_map(|s| uuid::Uuid::parse_str(s).ok())
        .collect();

    let mut tx = state.pool().begin().await?;
    sqlx::query("DELETE FROM user_roles WHERE user_id = ANY($1)")
        .bind(&user_uuids)
        .execute(&mut *tx)
        .await?;
    for uid in &user_uuids {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(uid)
        .bind(role.0)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    for uid in user_uuids {
        crate::authz::invalidate_permission_cache(uid);
    }
    Ok(StatusCode::NO_CONTENT)
}
