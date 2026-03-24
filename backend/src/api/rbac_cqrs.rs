use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domains::rbac::handlers::{PermissionDto, RbacCommandHandler, RbacQueryHandler, RoleDto};
use crate::domains::rbac::{
    AssignPermissionToRoleCommand, AssignRoleToUserCommand, CreatePermissionCommand, CreateRoleCommand,
    DeletePermissionCommand, DeleteRoleCommand, ListPermissionsQuery, ListRolesQuery,
    RevokePermissionFromRoleCommand, RevokeRoleFromUserCommand, UpdatePermissionCommand,
    UpdateRoleCommand,
};
use crate::error::AppResult;

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

pub async fn list_roles(State(state): State<AppState>) -> AppResult<Json<Vec<RoleDto>>> {
    let h = Arc::new(RbacQueryHandler::new(state.pool.clone()));
    Ok(Json(state.query_bus.dispatch_with_handler(ListRolesQuery, h).await?))
}

pub async fn create_role(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateRoleRequest>,
) -> AppResult<(StatusCode, Json<RoleDto>)> {
    let cmd = CreateRoleCommand { slug: payload.slug, description: payload.description };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    let role = state.command_bus.dispatch_with_handler(cmd, h).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_role(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
    Json(payload): Json<UpdateRoleRequest>,
) -> AppResult<Json<RoleDto>> {
    let cmd = UpdateRoleCommand { role_id, slug: payload.slug, description: payload.description, is_active: payload.is_active };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    Ok(Json(state.command_bus.dispatch_with_handler(cmd, h).await?))
}

pub async fn delete_role(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path(role_id): Path<String>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(DeleteRoleCommand { role_id }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_permissions(State(state): State<AppState>) -> AppResult<Json<Vec<PermissionDto>>> {
    let h = Arc::new(RbacQueryHandler::new(state.pool.clone()));
    Ok(Json(state.query_bus.dispatch_with_handler(ListPermissionsQuery, h).await?))
}

pub async fn create_permission(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreatePermissionRequest>,
) -> AppResult<(StatusCode, Json<PermissionDto>)> {
    let cmd = CreatePermissionCommand { code: payload.code, description: payload.description };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    let p = state.command_bus.dispatch_with_handler(cmd, h).await?;
    Ok((StatusCode::CREATED, Json(p)))
}

pub async fn update_permission(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(payload): Json<UpdatePermissionRequest>,
) -> AppResult<Json<PermissionDto>> {
    let cmd = UpdatePermissionCommand { code, description: payload.description, is_active: payload.is_active };
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    Ok(Json(state.command_bus.dispatch_with_handler(cmd, h).await?))
}

pub async fn delete_permission(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(DeletePermissionCommand { code }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_permission_to_role(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path((role_id, code)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(AssignPermissionToRoleCommand { role_id, code }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_permission_from_role(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path((role_id, code)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(RevokePermissionFromRoleCommand { role_id, code }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_role_to_user(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(AssignRoleToUserCommand { user_id, role_id }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_role_from_user(
    Extension(_actor): Extension<String>,
    State(state): State<AppState>,
    Path((user_id, role_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    let h = Arc::new(RbacCommandHandler::new(state.pool.clone()));
    state.command_bus.dispatch_with_handler(RevokeRoleFromUserCommand { user_id, role_id }, h).await?;
    Ok(StatusCode::NO_CONTENT)
}
