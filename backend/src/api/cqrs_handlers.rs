use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::app_state::AppState;
use crate::domains::clients::{
    CreateClientCommand, UpdateClientCommand, DeleteClientCommand,
    GetClientQuery, ListClientsQuery, SearchClientsQuery,
    CreateClientHandler, UpdateClientHandler, DeleteClientHandler,
    GetClientHandler, ListClientsHandler, SearchClientsHandler,
};
use crate::error::AppResult;
use crate::models::Client;

// ============================================================================
// CLIENT CQRS HANDLERS
// ============================================================================

/// Create Client (POST /api/clients)
///
/// Uses CQRS pattern:
/// 1. Extract CreateClientCommand from request
/// 2. Validate command (automatic via Command trait)
/// 3. Dispatch via CommandBus to CreateClientHandler
/// 4. Handler executes DB write + publishes ClientCreatedEvent
/// 5. Return created client
pub async fn create_client(
    State(state): State<AppState>,
    Json(payload): Json<CreateClientCommand>,
) -> AppResult<Json<Client>> {
    tracing::debug!("Creating client via CQRS: {:?}", payload);

    // 1. Create handler with dependencies (pool + event_bus)
    let handler = Arc::new(CreateClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    // 2. Dispatch command via CommandBus (validates + executes)
    let client = state
        .command_bus
        .dispatch_with_handler(payload, handler)
        .await?;

    tracing::info!("Client created successfully: {}", client.id);

    Ok(Json(client))
}

/// Update Client (PATCH /api/clients/:id)
///
/// CQRS pattern:
/// - Merges path parameter ID into command
/// - Validates all updates
/// - Publishes ClientUpdatedEvent with change map
pub async fn update_client(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(mut payload): Json<UpdateClientCommand>,
) -> AppResult<Json<Client>> {
    tracing::debug!("Updating client {} via CQRS", id);

    // Inject ID from path into command
    payload.id = id;

    let handler = Arc::new(UpdateClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    let client = state
        .command_bus
        .dispatch_with_handler(payload, handler)
        .await?;

    tracing::info!("Client updated successfully: {}", client.id);

    Ok(Json(client))
}

/// Delete Client (DELETE /api/clients/:id)
///
/// CQRS pattern:
/// - Creates DeleteClientCommand from path param
/// - Publishes ClientDeletedEvent before deletion
/// - Returns 204 No Content on success
pub async fn delete_client(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<StatusCode> {
    tracing::debug!("Deleting client {} via CQRS", id);

    let command = DeleteClientCommand { id };

    let handler = Arc::new(DeleteClientHandler::new(
        state.pool.clone(),
        state.event_bus.clone(),
    ));

    state
        .command_bus
        .dispatch_with_handler(command, handler)
        .await?;

    tracing::info!("Client deleted successfully: {}", id);

    Ok(StatusCode::NO_CONTENT)
}

/// Get Client by ID (GET /api/clients/:id)
///
/// CQRS Query pattern:
/// - Read-only operation via QueryBus
/// - No events published
/// - Returns 404 if not found
pub async fn get_client(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<Json<Client>> {
    tracing::debug!("Getting client {} via CQRS", id);

    let query = GetClientQuery { id };

    let handler = Arc::new(GetClientHandler::new(state.pool.clone()));

    let client = state
        .query_bus
        .dispatch_with_handler(query, handler)
        .await?
        .ok_or_else(|| {
            crate::error::AppError::NotFound(format!("Client with id {} not found", id))
        })?;

    Ok(Json(client))
}

/// List Clients (GET /api/clients)
///
/// Query parameters:
/// - status: Filter by status (active, inactive, prospect, customer)
/// - limit: Max results (default: all)
/// - offset: Skip N records
///
/// CQRS Query pattern - read-only
pub async fn list_clients(
    State(state): State<AppState>,
    Query(params): Query<ListClientsQuery>,
) -> AppResult<Json<Vec<Client>>> {
    tracing::debug!("Listing clients via CQRS: {:?}", params);

    let handler = Arc::new(ListClientsHandler::new(state.pool.clone()));

    let clients = state
        .query_bus
        .dispatch_with_handler(params, handler)
        .await?;

    tracing::debug!("Retrieved {} clients", clients.len());

    Ok(Json(clients))
}

/// Search Clients (GET /api/clients/search)
///
/// Query parameters:
/// - search_term: Search in name, email, company
/// - limit: Max results (default: 50)
///
/// CQRS Query pattern - optimized for search
pub async fn search_clients(
    State(state): State<AppState>,
    Query(params): Query<SearchClientsQuery>,
) -> AppResult<Json<Vec<Client>>> {
    tracing::debug!("Searching clients via CQRS: {:?}", params);

    let handler = Arc::new(SearchClientsHandler::new(state.pool.clone()));

    let clients = state
        .query_bus
        .dispatch_with_handler(params, handler)
        .await?;

    tracing::debug!("Search returned {} clients", clients.len());

    Ok(Json(clients))
}

// ============================================================================
// HELPER STRUCTS (for query parameters that don't match domain queries)
// ============================================================================

// Note: If your query params differ from domain queries, define them here
// and map to domain queries inside the handler functions

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests will be added in Week 2 Day 4-5
    // These tests require:
    // 1. Test database setup
    // 2. AppState mock or test instance
    // 3. Redis mock (or TestContainers)

    #[test]
    fn test_placeholder() {
        // Placeholder to ensure module compiles
        assert!(true);
    }
}
