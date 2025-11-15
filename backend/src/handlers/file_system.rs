/// Anti-Corruption Layer for CQRS File System
///
/// This module provides thin Axum handlers that:
/// 1. Extract HTTP request data
/// 2. Convert to CQRS Commands/Queries
/// 3. Dispatch to CommandBus/QueryBus
/// 4. Convert domain results back to HTTP responses
///
/// CQRS layer remains pure, independent of web framework

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::config::Config;
use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::events::{EventBus, RedisEventBus};
use crate::core::shared::Pagination;
use crate::domains::file_system::commands::*;
use crate::domains::file_system::queries::*;
use crate::domains::file_system::handlers::{command_handlers::*, query_handlers::*};
use sqlx::SqlitePool;

// ============================================================================
// Request/Response DTOs (HTTP Layer)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct CreateFileResponse {
    pub file_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CreateFolderResponse {
    pub folder_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct MoveItemRequest {
    pub new_parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct RenameItemRequest {
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesParams {
    pub parent_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchFilesParams {
    pub query: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// ============================================================================
// Anti-Corruption Layer: Axum Handlers
// ============================================================================

/// Create File - Dispatches to CQRS CreateFileCommand
pub async fn create_file(
    State((pool, config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateFileRequest>,
) -> Result<Json<CreateFileResponse>, (StatusCode, String)> {
    let event_bus = build_event_bus(&config)?;
    let service = build_file_service(pool.clone());
    let handler = CreateFileHandler::new(pool, event_bus, service);

    let cmd = CreateFileCommand {
        name: req.name,
        parent_id: req.parent_id,
        size: req.size,
        mime_type: req.mime_type,
        owner_id: user_id,
    };

    cmd.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation: {:?}", e)))?;

    handler
        .handle(cmd)
        .await
        .map(|file_id| Json(CreateFileResponse { file_id }))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get File - Dispatches to CQRS GetFileQuery
pub async fn get_file(
    State((pool, _config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let handler = GetFileHandler::new(pool);

    let query = GetFileQuery {
        file_id: id,
        user_id,
    };

    handler
        .handle(query)
        .await
        .map(|file| Json(serde_json::to_value(file).unwrap()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// List Files - Dispatches to CQRS ListFilesQuery
pub async fn list_files(
    State((pool, _config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let handler = ListFilesHandler::new(pool);

    let query = ListFilesQuery {
        parent_id: params.parent_id,
        user_id,
        pagination: Pagination::new(
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(20),
        ),
    };

    handler
        .handle(query)
        .await
        .map(|files| Json(serde_json::to_value(files).unwrap()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Move File - Dispatches to CQRS MoveFileCommand
pub async fn move_file(
    State((pool, config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(req): Json<MoveItemRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let event_bus = build_event_bus(&config)?;
    let service = build_file_service(pool.clone());
    let handler = MoveFileHandler::new(pool, event_bus, service);

    let cmd = MoveFileCommand {
        file_id: id,
        new_parent_id: req.new_parent_id,
        moved_by: user_id,
    };

    cmd.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation: {:?}", e)))?;

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Delete File - Dispatches to CQRS DeleteFileCommand
pub async fn delete_file(
    State((pool, config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let event_bus = build_event_bus(&config)?;
    let service = build_file_service(pool.clone());
    let handler = DeleteFileHandler::new(pool, event_bus, service);

    let cmd = DeleteFileCommand {
        file_id: id,
        deleted_by: user_id,
    };

    cmd.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation: {:?}", e)))?;

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Rename File - Dispatches to CQRS RenameFileCommand
pub async fn rename_file(
    State((pool, config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
    Json(req): Json<RenameItemRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let event_bus = build_event_bus(&config)?;
    let service = build_file_service(pool.clone());
    let handler = RenameFileHandler::new(pool, event_bus, service);

    let cmd = RenameFileCommand {
        file_id: id,
        new_name: req.new_name,
        renamed_by: user_id,
    };

    cmd.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation: {:?}", e)))?;

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Create Folder - Dispatches to CQRS CreateFolderCommand
pub async fn create_folder(
    State((pool, config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<CreateFolderResponse>, (StatusCode, String)> {
    let event_bus = build_event_bus(&config)?;
    let service = build_file_service(pool.clone());
    let handler = CreateFolderHandler::new(pool, event_bus, service);

    let cmd = CreateFolderCommand {
        name: req.name,
        parent_id: req.parent_id,
        owner_id: user_id,
    };

    cmd.validate()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation: {:?}", e)))?;

    handler
        .handle(cmd)
        .await
        .map(|folder_id| Json(CreateFolderResponse { folder_id }))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get Folder Tree - Dispatches to CQRS GetFolderTreeQuery
pub async fn get_folder_tree(
    State((pool, _config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let handler = GetFolderTreeHandler::new(pool);

    let query = GetFolderTreeQuery {
        folder_id: id,
        depth: None, // Could be query param
        user_id,
    };

    handler
        .handle(query)
        .await
        .map(|tree| Json(serde_json::to_value(tree).unwrap()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Search Files - Dispatches to CQRS SearchFilesQuery
pub async fn search_files(
    State((pool, _config)): State<(SqlitePool, Config)>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<SearchFilesParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let handler = SearchFilesHandler::new(pool);

    let query = SearchFilesQuery {
        query: params.query,
        user_id,
        pagination: Pagination::new(
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(20),
        ),
    };

    handler
        .handle(query)
        .await
        .map(|files| Json(serde_json::to_value(files).unwrap()))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ============================================================================
// Helper Functions
// ============================================================================

use crate::core::events::PostgresEventStore;
use crate::core::infrastructure::PostgresRepository;
use crate::domains::file_system::aggregates::{File, Folder};
use crate::domains::file_system::services::FileSystemService;

/// Build Event Bus from config
fn build_event_bus(config: &Config) -> Result<Arc<dyn EventBus + Send + Sync>, (StatusCode, String)> {
    RedisEventBus::new(&config.redis_url, "file_system_events".to_string())
        .map(|bus| Arc::new(bus) as Arc<dyn EventBus + Send + Sync>)
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create Event Bus: {}", e),
        ))
}

/// Build FileSystemService with all dependencies
///
/// In production, these should be cached/pooled for reuse
fn build_file_service(pool: SqlitePool) -> Arc<FileSystemService> {
    // Build Event Store
    let event_store = PostgresEventStore::new(pool.clone());

    // Build Repositories
    let file_repo = Arc::new(PostgresRepository::new(
        pool.clone(),
        event_store.clone(),
        "file".to_string(),
    ));

    let folder_repo = Arc::new(PostgresRepository::new(
        pool.clone(),
        event_store,
        "folder".to_string(),
    ));

    // Build Service
    Arc::new(FileSystemService::new(pool, file_repo, folder_repo))
}
