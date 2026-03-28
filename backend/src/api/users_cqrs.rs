use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domains::users::{
    ChangePasswordCommand, DeleteUserCommand, GetUserQuery, ListUsersQuery, RegisterUserCommand,
    UpdateUserCommand,
};
use crate::domains::users::handlers::{
    ChangePasswordHandler, DeleteUserHandler, GetUserHandler, ListUsersHandler, RegisterUserHandler,
    UpdateUserHandler,
};
use crate::error::{AppError, AppResult};
use crate::models::User;

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
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserPayload {
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: Option<String>,
    pub avatar_url: Option<String>,
}

pub async fn get_current_user(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<User>> {
    let query = GetUserQuery { id: user_id };
    let handler = Arc::new(GetUserHandler::new(state.pool.clone()));

    let user = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<User>> {
    let query = GetUserQuery { id: id.clone() };
    let handler = Arc::new(GetUserHandler::new(state.pool.clone()));

    let user = state
        .query_bus
        .dispatch_with_handler(query, handler)
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
        role: None,
        actor_id: Some(user_id),
    };
    let handler = Arc::new(UpdateUserHandler::new(state.pool.clone()));

    let user = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    Ok(Json(user))
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

    state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

pub async fn list_users_admin(
    State(state): State<AppState>,
    Query(params): Query<ListUsersQuery>,
) -> AppResult<Json<Vec<User>>> {
    let handler = Arc::new(ListUsersHandler::new(state.pool.clone()));
    let users = state
        .query_bus
        .dispatch_with_handler(params, handler)
        .await?;
    Ok(Json(users))
}

pub async fn create_user_admin(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> AppResult<(StatusCode, Json<User>)> {
    let command = RegisterUserCommand {
        email: payload.email,
        password: payload.password,
        full_name: payload.full_name,
        role: payload.role,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(RegisterUserHandler::new(state.pool.clone()));

    let user = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "UserCreated",
        "user_id": user.id,
        "role": user.role,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.user", &user.id.to_string(), &event.to_string())
        .await;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user_admin(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<AdminUpdateUserPayload>,
) -> AppResult<Json<User>> {
    let command = UpdateUserCommand {
        id,
        email: payload.email,
        full_name: payload.full_name,
        avatar_url: payload.avatar_url,
        role: payload.role,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(UpdateUserHandler::new(state.pool.clone()));

    let user = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "UserUpdated",
        "user_id": user.id,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.user", &user.id.to_string(), &event.to_string())
        .await;

    Ok(Json(user))
}

pub async fn delete_user_admin(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let command = DeleteUserCommand {
        id: id.clone(),
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(DeleteUserHandler::new(state.pool.clone()));

    state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "UserDeleted",
        "user_id": id,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.user", &event["user_id"].as_str().unwrap_or(""), &event.to_string())
        .await;

    Ok(StatusCode::NO_CONTENT)
}
