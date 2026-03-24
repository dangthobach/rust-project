use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::core::shared::{append_aggregate_history, Pagination};
use crate::domains::file_system::commands::{
    CreateFileCommand, CreateFolderCommand, DeleteFileCommand, MoveFileCommand, RenameFileCommand,
};
use crate::domains::file_system::handlers::state::HandlerState;
use crate::domains::file_system::queries::{
    GetFileQuery, GetFolderTreeQuery, ListFilesQuery, SearchFilesQuery,
};
use crate::domains::file_system::read_models::{FileView, FolderTreeView};
use crate::error::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct FsPaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFilePayload {
    pub name: String,
    pub parent_id: Option<String>,
    pub size: i64,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateFolderPayload {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MoveFilePayload {
    pub new_parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameFilePayload {
    pub new_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchFsQuery {
    pub query: String,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

fn parse_uuid(raw: &str, field: &str) -> AppResult<Uuid> {
    Uuid::parse_str(raw).map_err(|_| AppError::ValidationError(format!("{field} must be UUID")))
}

fn map_opt_uuid(raw: Option<String>, field: &str) -> AppResult<Option<Uuid>> {
    match raw {
        Some(v) => Ok(Some(parse_uuid(&v, field)?)),
        None => Ok(None),
    }
}

fn build_pagination(page: Option<i64>, page_size: Option<i64>) -> Pagination {
    Pagination::new(page.unwrap_or(1), page_size.unwrap_or(20))
}

fn fs_state(state: &AppState) -> HandlerState {
    HandlerState::new(state.pool.as_ref().clone(), state.event_bus.clone())
}

pub async fn create_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateFilePayload>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let actor_uuid = parse_uuid(&actor_id, "actor_id")?;
    let cmd = CreateFileCommand {
        name: payload.name,
        parent_id: map_opt_uuid(payload.parent_id, "parent_id")?,
        size: payload.size,
        mime_type: payload.mime_type,
        owner_id: actor_uuid,
    };
    let h = Arc::new(fs_state(&state).create_file_handler());
    let id = state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &id.to_string(),
        "CREATE",
        None,
        Some("active"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn create_folder(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateFolderPayload>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let actor_uuid = parse_uuid(&actor_id, "actor_id")?;
    let cmd = CreateFolderCommand {
        name: payload.name,
        parent_id: map_opt_uuid(payload.parent_id, "parent_id")?,
        owner_id: actor_uuid,
    };
    let h = Arc::new(fs_state(&state).create_folder_handler());
    let id = state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "folder",
        &id.to_string(),
        "CREATE",
        None,
        Some("active"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": id }))))
}

pub async fn move_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(payload): Json<MoveFilePayload>,
) -> AppResult<StatusCode> {
    let actor_uuid = parse_uuid(&actor_id, "actor_id")?;
    let cmd = MoveFileCommand {
        file_id: parse_uuid(&file_id, "file_id")?,
        new_parent_id: map_opt_uuid(payload.new_parent_id, "new_parent_id")?,
        moved_by: actor_uuid,
    };
    let h = Arc::new(fs_state(&state).move_file_handler());
    state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &file_id,
        "MOVE",
        Some("active"),
        Some("active"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn rename_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(payload): Json<RenameFilePayload>,
) -> AppResult<StatusCode> {
    let cmd = RenameFileCommand {
        file_id: parse_uuid(&file_id, "file_id")?,
        new_name: payload.new_name,
        renamed_by: parse_uuid(&actor_id, "actor_id")?,
    };
    let h = Arc::new(fs_state(&state).rename_file_handler());
    state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &file_id,
        "RENAME",
        Some("active"),
        Some("active"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<StatusCode> {
    let cmd = DeleteFileCommand {
        file_id: parse_uuid(&file_id, "file_id")?,
        deleted_by: parse_uuid(&actor_id, "actor_id")?,
    };
    let h = Arc::new(fs_state(&state).delete_file_handler());
    state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &file_id,
        "DELETE",
        Some("active"),
        Some("deleted"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<Json<FileView>> {
    let query = GetFileQuery {
        file_id: parse_uuid(&file_id, "file_id")?,
        user_id: parse_uuid(&actor_id, "actor_id")?,
    };
    let h = Arc::new(fs_state(&state).get_file_handler());
    let file = state.query_bus.dispatch_with_handler(query, h).await?;
    Ok(Json(file))
}

pub async fn list_files(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Query(p): Query<FsPaginationQuery>,
) -> AppResult<Json<Vec<FileView>>> {
    let query = ListFilesQuery {
        parent_id: None,
        user_id: parse_uuid(&actor_id, "actor_id")?,
        pagination: build_pagination(p.page, p.page_size),
    };
    let h = Arc::new(fs_state(&state).list_files_handler());
    let files = state.query_bus.dispatch_with_handler(query, h).await?;
    Ok(Json(files))
}

pub async fn search_files(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Query(q): Query<SearchFsQuery>,
) -> AppResult<Json<Vec<FileView>>> {
    if q.query.trim().is_empty() {
        return Err(AppError::ValidationError("query is required".to_string()));
    }
    let query = SearchFilesQuery {
        query: q.query,
        user_id: parse_uuid(&actor_id, "actor_id")?,
        pagination: build_pagination(q.page, q.page_size),
    };
    let h = Arc::new(fs_state(&state).search_files_handler());
    let files = state.query_bus.dispatch_with_handler(query, h).await?;
    Ok(Json(files))
}

pub async fn get_folder_tree(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
) -> AppResult<Json<FolderTreeView>> {
    let query = GetFolderTreeQuery {
        folder_id: parse_uuid(&folder_id, "folder_id")?,
        depth: Some(10),
        user_id: parse_uuid(&actor_id, "actor_id")?,
    };
    let h = Arc::new(fs_state(&state).get_folder_tree_handler());
    let tree = state.query_bus.dispatch_with_handler(query, h).await?;
    Ok(Json(tree))
}
