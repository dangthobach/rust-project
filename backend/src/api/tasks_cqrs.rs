use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domains::tasks::handlers::{
    CompleteTaskHandler, CreateTaskHandler, DeleteTaskHandler, GetTaskHandler, ListTasksHandler,
    UpdateTaskHandler,
};
use crate::domains::tasks::{
    CompleteTaskCommand, CreateTaskCommand, DeleteTaskCommand, GetTaskQuery, ListTasksQuery,
    UpdateTaskCommand,
};
use crate::error::{AppError, AppResult};
use crate::models::Task;

#[derive(Debug, Deserialize)]
pub struct CreateTaskPayload {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub client_id: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskPayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub client_id: Option<String>,
    pub due_date: Option<String>,
}

pub async fn create_task(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskPayload>,
) -> AppResult<(StatusCode, Json<Task>)> {
    let command = CreateTaskCommand {
        title: payload.title,
        description: payload.description,
        status: payload.status,
        priority: payload.priority,
        assigned_to: payload.assigned_to,
        client_id: payload.client_id,
        due_date: payload.due_date,
        created_by: Some(actor_id.clone()),
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(CreateTaskHandler::new(state.pool.clone()));
    let task = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;
    let event = serde_json::json!({
        "event_type": "TaskCreated",
        "task_id": task.id,
        "status": task.status,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.task", &task.id.to_string(), &event.to_string())
        .await;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Query(params): Query<ListTasksQuery>,
) -> AppResult<Json<Vec<Task>>> {
    let handler = Arc::new(ListTasksHandler::new(state.pool.clone()));
    let tasks = state
        .query_bus
        .dispatch_with_handler(params, handler)
        .await?;
    Ok(Json(tasks))
}

pub async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Task>> {
    let query = GetTaskQuery { id: id.clone() };
    let handler = Arc::new(GetTaskHandler::new(state.pool.clone()));
    let task = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Task {} not found", id)))?;
    Ok(Json(task))
}

pub async fn update_task(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskPayload>,
) -> AppResult<Json<Task>> {
    let command = UpdateTaskCommand {
        id,
        title: payload.title,
        description: payload.description,
        status: payload.status,
        priority: payload.priority,
        assigned_to: payload.assigned_to,
        client_id: payload.client_id,
        due_date: payload.due_date,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(UpdateTaskHandler::new(state.pool.clone()));
    let task = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;
    let event = serde_json::json!({
        "event_type": "TaskUpdated",
        "task_id": task.id,
        "status": task.status,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.task", &task.id.to_string(), &event.to_string())
        .await;
    Ok(Json(task))
}

pub async fn delete_task(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let command = DeleteTaskCommand {
        id: id.clone(),
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(DeleteTaskHandler::new(state.pool.clone()));
    state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;
    let event = serde_json::json!({
        "event_type": "TaskDeleted",
        "task_id": id,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.task", event["task_id"].as_str().unwrap_or(""), &event.to_string())
        .await;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn complete_task(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Task>> {
    let command = CompleteTaskCommand {
        id,
        actor_id: Some(actor_id),
    };
    let handler = Arc::new(CompleteTaskHandler::new(state.pool.clone()));
    let task = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;
    let event = serde_json::json!({
        "event_type": "TaskCompleted",
        "task_id": task.id,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.task", &task.id.to_string(), &event.to_string())
        .await;
    Ok(Json(task))
}
