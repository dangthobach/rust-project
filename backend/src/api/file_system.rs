use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::shared::Pagination;
use crate::domains::file_system::commands::*;
use crate::domains::file_system::queries::*;
use crate::domains::file_system::handlers::HandlerState;

/// Create File Request
#[derive(Debug, Deserialize)]
pub struct CreateFileRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
}

/// Create File Response
#[derive(Debug, Serialize)]
pub struct CreateFileResponse {
    pub file_id: Uuid,
}

/// Create Folder Request
#[derive(Debug, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<Uuid>,
}

/// Create Folder Response
#[derive(Debug, Serialize)]
pub struct CreateFolderResponse {
    pub folder_id: Uuid,
}

/// Move Item Request
#[derive(Debug, Deserialize)]
pub struct MoveItemRequest {
    pub new_parent_id: Option<Uuid>,
}

/// Rename Item Request
#[derive(Debug, Deserialize)]
pub struct RenameItemRequest {
    pub new_name: String,
}

/// Set Permissions Request
#[derive(Debug, Deserialize)]
pub struct SetPermissionsRequest {
    pub acl: Vec<crate::core::shared::AccessControlEntry>,
}

/// List Files Query Params
#[derive(Debug, Deserialize)]
pub struct ListFilesParams {
    pub parent_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// Search Files Query Params
#[derive(Debug, Deserialize)]
pub struct SearchFilesParams {
    pub q: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// Create File Handler
pub async fn create_file(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateFileRequest>,
) -> Result<Json<CreateFileResponse>, (StatusCode, String)> {
    let handler = state.create_file_handler();
    
    // Use user_id from auth context
    let cmd = CreateFileCommand {
        name: req.name,
        parent_id: req.parent_id,
        size: req.size,
        mime_type: req.mime_type,
        owner_id: user_id,
    };

    match cmd.validate() {
        Ok(_) => {}
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Validation error: {:?}", e))),
    }

    handler
        .handle(cmd)
        .await
        .map(|file_id| Json(CreateFileResponse { file_id }))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get File Handler
pub async fn get_file(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<FileView>, (StatusCode, String)> {
    let handler = state.get_file_handler();
    let query = GetFileQuery { 
        file_id,
        user_id,
    };

    handler
        .handle(query)
        .await
        .map(Json)
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, "File not found".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}

/// List Files Handler
pub async fn list_files(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<ListFilesParams>,
) -> Result<Json<Vec<FileView>>, (StatusCode, String)> {
    let handler = state.list_files_handler();
    let pagination = Pagination::new(params.page.unwrap_or(1), params.page_size.unwrap_or(20));
    let query = ListFilesQuery {
        parent_id: params.parent_id,
        user_id,
        pagination,
    };

    handler
        .handle(query)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Move File Handler
pub async fn move_file(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Path(file_id): Path<Uuid>,
    Json(req): Json<MoveItemRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let handler = state.move_file_handler();
    let cmd = MoveFileCommand {
        file_id,
        new_parent_id: req.new_parent_id,
        moved_by: user_id,
    };

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Delete File Handler
pub async fn delete_file(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Path(file_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let handler = state.delete_file_handler();
    let cmd = DeleteFileCommand {
        file_id,
        deleted_by: user_id,
    };

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Rename File Handler
pub async fn rename_file(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Path(file_id): Path<Uuid>,
    Json(req): Json<RenameItemRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let handler = state.rename_file_handler();
    let cmd = RenameFileCommand {
        file_id,
        new_name: req.new_name,
        renamed_by: user_id,
    };

    match cmd.validate() {
        Ok(_) => {}
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Validation error: {:?}", e))),
    }

    handler
        .handle(cmd)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (StatusCode::NOT_FOUND, "File not found".to_string())
            } else if e.to_string().contains("Permission denied") {
                (StatusCode::FORBIDDEN, "Permission denied".to_string())
            } else if e.to_string().contains("already exists") {
                (StatusCode::CONFLICT, "A file with this name already exists".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "An error occurred".to_string())
            }
        })
}

/// Create Folder Handler
pub async fn create_folder(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<CreateFolderRequest>,
) -> Result<Json<CreateFolderResponse>, (StatusCode, String)> {
    let handler = state.create_folder_handler();
    let cmd = CreateFolderCommand {
        name: req.name,
        parent_id: req.parent_id,
        owner_id: user_id,
    };

    match cmd.validate() {
        Ok(_) => {}
        Err(e) => return Err((StatusCode::BAD_REQUEST, format!("Validation error: {:?}", e))),
    }

    handler
        .handle(cmd)
        .await
        .map(|folder_id| Json(CreateFolderResponse { folder_id }))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Get Folder Tree Handler
pub async fn get_folder_tree(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Path(folder_id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<FolderTreeView>, (StatusCode, String)> {
    let handler = state.get_folder_tree_handler();
    let depth = params
        .get("depth")
        .and_then(|d| d.parse::<i32>().ok());

    let query = GetFolderTreeQuery { 
        folder_id, 
        depth,
        user_id,
    };

    handler
        .handle(query)
        .await
        .map(Json)
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                (StatusCode::NOT_FOUND, "Folder not found".to_string())
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            }
        })
}

/// Search Files Handler
pub async fn search_files(
    State(state): State<HandlerState>,
    Extension(user_id): Extension<Uuid>,
    Query(params): Query<SearchFilesParams>,
) -> Result<Json<Vec<FileView>>, (StatusCode, String)> {
    let handler = state.search_files_handler();
    if params.q.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Query cannot be empty".to_string()));
    }

    let pagination = Pagination::new(params.page.unwrap_or(1), params.page_size.unwrap_or(20));
    let query = SearchFilesQuery {
        query: params.q,
        user_id,
        pagination,
    };

    handler
        .handle(query)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

