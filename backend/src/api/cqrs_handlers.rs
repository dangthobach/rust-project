use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::app_state::AppState;
use crate::authz::data_scope::DataScope;
use crate::authz::AuthContext;
use crate::domains::clients::{
    CreateClientCommand, CreateClientHandler, DeleteClientCommand, DeleteClientHandler,
    GetClientHandler, GetClientQuery, ListClientsHandler, ListClientsParams, ListClientsQuery,
    SearchClientsHandler, SearchClientsQuery, UpdateClientCommand, UpdateClientHandler,
};
use crate::error::{AppError, AppResult};
use crate::models::Client;

// ============================================================================
// CLIENT CQRS HANDLERS
// ============================================================================

#[derive(Debug, Deserialize, Validate)]
pub struct CreateClientPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub phone: Option<String>,
    #[validate(length(max = 500))]
    pub address: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub notes: Option<String>,
    pub branch_id: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateClientPayload {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub phone: Option<String>,
    #[validate(length(max = 500))]
    pub address: Option<String>,
    pub company: Option<String>,
    pub position: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchClientsParams {
    pub search_term: String,
    pub limit: Option<i64>,
}

pub async fn create_client(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateClientPayload>,
) -> AppResult<Json<Client>> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let command = CreateClientCommand {
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        address: payload.address,
        company: payload.company,
        position: payload.position,
        status: payload.status,
        assigned_to: payload.assigned_to,
        notes: payload.notes,
        actor_id: Some(actor_id.clone()),
        branch_id: payload.branch_id,
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id.clone(),
    };

    let handler = Arc::new(CreateClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    let client = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "ClientCreated",
        "client_id": client.id,
        "status": client.status,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.client", &client.id.to_string(), &event.to_string())
        .await;

    Ok(Json(client))
}

pub async fn update_client(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateClientPayload>,
) -> AppResult<Json<Client>> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let command = UpdateClientCommand {
        id,
        name: payload.name,
        email: payload.email,
        phone: payload.phone,
        address: payload.address,
        company: payload.company,
        position: payload.position,
        status: payload.status,
        assigned_to: payload.assigned_to,
        notes: payload.notes,
        actor_id: Some(actor_id.clone()),
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id,
    };

    let handler = Arc::new(UpdateClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    let client = state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "ClientUpdated",
        "client_id": client.id,
        "status": client.status,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.client", &client.id.to_string(), &event.to_string())
        .await;

    Ok(Json(client))
}

pub async fn delete_client(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let command = DeleteClientCommand {
        id: id.clone(),
        actor_id: Some(actor_id.clone()),
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id,
    };

    let handler = Arc::new(DeleteClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    let event = serde_json::json!({
        "event_type": "ClientDeleted",
        "client_id": id,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.client", event["client_id"].as_str().unwrap_or(""), &event.to_string())
        .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_client(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<Client>> {
    let query = GetClientQuery {
        id: id.clone(),
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id,
    };

    let handler = Arc::new(GetClientHandler::new(state.pool.clone()));

    let client = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Client with id {} not found", id))
        })?;

    Ok(Json(client))
}

pub async fn list_clients(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ListClientsParams>,
) -> AppResult<Json<Vec<Client>>> {
    let query = ListClientsQuery {
        status: params.status.clone(),
        assigned_to: params.assigned_to.clone(),
        limit: params.limit,
        offset: params.offset,
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id,
    };

    let handler = Arc::new(ListClientsHandler::new(state.pool.clone()));

    let clients = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?;

    Ok(Json(clients))
}

pub async fn search_clients(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<SearchClientsParams>,
) -> AppResult<Json<Vec<Client>>> {
    let query = SearchClientsQuery {
        search_term: params.search_term.clone(),
        limit: params.limit,
        data_scope: DataScope::from_auth_context(&ctx),
        actor_user_id: actor_id,
    };

    let handler = Arc::new(SearchClientsHandler::new(state.pool.clone()));

    let clients = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?;

    Ok(Json(clients))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
