use axum::{
    body::Body,
    extract::Multipart,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    Extension,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::core::shared::{append_aggregate_history, Pagination};
use crate::domains::file_system::commands::{
    CreateFileCommand, CreateFolderCommand, DeleteFileCommand, MoveFileCommand, RenameFileCommand,
    SetFilePermissionsCommand,
};
use crate::domains::file_system::handlers::state::HandlerState;
use crate::domains::file_system::queries::{
    GetFileQuery, GetFolderTreeQuery, ListFilesQuery, SearchFilesQuery,
};
use crate::domains::file_system::read_models::{FileView, FolderTreeView};
use crate::error::{AppError, AppResult};
use crate::utils::file_validator;
use tokio_util::io::ReaderStream;
use tokio::fs;
use futures_util::TryStreamExt;

#[derive(Debug, Deserialize)]
pub struct FsPaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub tab: Option<String>, // recent|shared|starred
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFilePayload {
    pub name: String,
    pub parent_id: Option<String>,
    pub size: i64,
    pub mime_type: String,
}

#[derive(Debug, Deserialize)]
pub struct SetPermissionsPayload {
    pub acl: Vec<crate::core::shared::AccessControlEntry>,
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

#[derive(Debug, serde::Serialize)]
pub struct FsPaginationMeta {
    pub page: i64,
    pub page_size: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct FsListResponse<T> {
    pub data: Vec<T>,
    pub pagination: FsPaginationMeta,
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

fn normalize_tab(raw: Option<String>) -> String {
    match raw.as_deref().map(|s| s.trim().to_lowercase()).as_deref() {
        Some("shared") => "shared".to_string(),
        Some("starred") => "starred".to_string(),
        _ => "recent".to_string(),
    }
}

fn content_disposition_attachment(original_name: &str) -> HeaderValue {
    let safe: String = original_name
        .chars()
        .filter(|c| !matches!(c, '"' | '\\' | '\r' | '\n') && !c.is_control())
        .take(200)
        .collect();
    let safe = if safe.trim().is_empty() {
        "download".to_string()
    } else {
        safe
    };
    let s = format!("attachment; filename=\"{}\"", safe);
    HeaderValue::from_str(&s).unwrap_or_else(|_| HeaderValue::from_static("attachment"))
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

/// Upload binary + create fs file + create version (v1, current).
///
/// This endpoint is the recommended way for the frontend; it replaces legacy `/api/files/upload`.
pub async fn upload_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let actor_uuid = parse_uuid(&actor_id, "actor_id")?;
    let max = state.config().max_file_size;

    let mut original_name: Option<String> = None;
    let mut file_field_consumed = false;
    let mut buf: Vec<u8> = Vec::new();

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {e}")))? {
        let field_name = field.name().unwrap_or("").trim();
        if field_name.is_empty() {
            continue;
        }
        if field_name != "file" {
            return Err(AppError::BadRequest(format!(
                "Unexpected multipart field: {field_name}"
            )));
        }
        if file_field_consumed {
            return Err(AppError::BadRequest(
                "Multiple file fields are not allowed".to_string(),
            ));
        }
        file_field_consumed = true;

        original_name = field
            .file_name()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());

        loop {
            let chunk = field
                .chunk()
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to read upload chunk: {e}")))?;
            let Some(bytes) = chunk else { break };
            if buf.len().saturating_add(bytes.len()) > max {
                return Err(AppError::BadRequest(format!(
                    "File exceeds maximum size of {max} bytes"
                )));
            }
            buf.extend_from_slice(&bytes);
        }
    }

    if !file_field_consumed {
        return Err(AppError::BadRequest("No file provided".to_string()));
    }
    let original_name =
        original_name.ok_or_else(|| AppError::BadRequest("No file name provided".to_string()))?;

    let mime_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    let safe_filename = file_validator::validate_upload(&original_name, &buf, &mime_type, max)?;
    let file_size = buf.len() as i64;

    // 1) Persist binary object via object storage (same as legacy).
    let storage_file_id = Uuid::new_v4();
    let extension = std::path::Path::new(&safe_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let object_name = if !extension.is_empty() {
        format!("{}_{}.{}", storage_file_id, Uuid::new_v4().simple(), extension)
    } else {
        format!("{}_{}", storage_file_id, Uuid::new_v4().simple())
    };

    use chrono::Datelike;
    let now = chrono::Utc::now();
    let tenant_id = state.config().default_tenant_id.clone();
    let object_key = format!(
        "{}/{}/{:04}/{:02}/{:02}/{}",
        tenant_id,
        actor_uuid,
        now.year(),
        now.month(),
        now.day(),
        object_name
    );

    let file_path = state
        .object_storage
        .put_object(&object_key, &buf, &mime_type)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Object storage put failed: {e}")))?;

    // 2) Insert into legacy `files` table (source of truth for storage uri).
    // Use a transaction to ensure atomicity with version records.
    let mut tx = state.pool().begin().await?;
    let _legacy = sqlx::query(
        r#"
        INSERT INTO files (id, name, original_name, file_path, file_type, file_size, uploaded_by, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        "#,
    )
    .bind(storage_file_id)
    .bind(&object_name)
    .bind(&safe_filename)
    .bind(&file_path)
    .bind(&mime_type)
    .bind(file_size)
    .bind(actor_uuid)
    .execute(&mut *tx)
    .await?;

    // 3) Create fs file aggregate (metadata) and synchronously upsert read model.
    // Keep fs id independent from storage id to allow versions to point to different binaries.
    let cmd = CreateFileCommand {
        name: safe_filename.clone(),
        parent_id: None,
        size: file_size,
        mime_type: mime_type.clone(),
        owner_id: actor_uuid,
    };
    let h = Arc::new(fs_state(&state).create_file_handler());
    let fs_file_id = state.command_bus.dispatch_with_handler(cmd, h).await?;

    // Link to storage and create version v1 current.
    let version_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO fs_file_versions (id, file_id, storage_file_id, version_no, is_current, note, created_by)
        VALUES ($1, $2, $3, 1, TRUE, NULL, $4)
        "#,
    )
    .bind(version_id)
    .bind(fs_file_id)
    .bind(storage_file_id)
    .bind(actor_uuid)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE file_views
        SET storage_file_id = $1, current_version_id = $2, updated_at = NOW(), updated_by = $3
        WHERE id = $4
        "#,
    )
    .bind(storage_file_id)
    .bind(version_id)
    .bind(actor_uuid)
    .bind(fs_file_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &fs_file_id.to_string(),
        "UPLOAD",
        None,
        Some("active"),
        Some(&actor_id),
        None,
        Some(serde_json::json!({ "storage_file_id": storage_file_id })),
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": fs_file_id,
            "storage_file_id": storage_file_id,
            "current_version_id": version_id
        })),
    ))
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
) -> AppResult<Json<FsListResponse<FileView>>> {
    let tab = normalize_tab(p.tab);
    let parent_id = map_opt_uuid(p.parent_id, "parent_id")?;
    let query = ListFilesQuery {
        parent_id,
        user_id: parse_uuid(&actor_id, "actor_id")?,
        pagination: build_pagination(p.page, p.page_size),
    };
    let h = Arc::new(fs_state(&state).list_files_handler());
    let mut files = state.query_bus.dispatch_with_handler(query, h).await?;

    // Apply tab filtering at API level to avoid breaking existing query handler signature.
    // Later we can push this into the query handler for performance.
    match tab.as_str() {
        "starred" => {
            let uid = parse_uuid(&actor_id, "actor_id")?;
            let ids: Vec<Uuid> = sqlx::query_scalar(
                "SELECT file_id FROM fs_file_stars WHERE user_id = $1",
            )
            .bind(uid)
            .fetch_all(state.pool())
            .await
            .unwrap_or_default();
            files.retain(|f| ids.contains(&f.id));
        }
        "shared" => {
            let uid = parse_uuid(&actor_id, "actor_id")?;
            // Shared = not owner and has any ACL entry for user/everyone
            let ids: Vec<Uuid> = sqlx::query_scalar(
                r#"
                SELECT DISTINCT fp.file_id
                FROM file_permissions fp
                JOIN file_views fv ON fv.id = fp.file_id
                WHERE fv.owner_id <> $1
                  AND (
                    (fp.subject_type = 'user' AND fp.subject_id = $1)
                    OR fp.subject_type = 'everyone'
                  )
                "#,
            )
            .bind(uid)
            .fetch_all(state.pool())
            .await
            .unwrap_or_default();
            files.retain(|f| ids.contains(&f.id));
        }
        _ => {}
    }

    // Compute pagination meta (best-effort; count based on filtered list if tab applied).
    let page = p.page.unwrap_or(1);
    let page_size = p.page_size.unwrap_or(20);
    let total = files.len() as i64;
    let total_pages = (total + page_size - 1) / page_size;
    let has_prev = page > 1;
    let has_next = page < total_pages;
    let meta = FsPaginationMeta {
        page,
        page_size,
        total,
        total_pages,
        has_next,
        has_prev,
    };
    Ok(Json(FsListResponse { data: files, pagination: meta }))
}

pub async fn search_files(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Query(q): Query<SearchFsQuery>,
) -> AppResult<Json<FsListResponse<FileView>>> {
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
    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(20);
    let total = files.len() as i64;
    let total_pages = (total + page_size - 1) / page_size;
    let has_prev = page > 1;
    let has_next = page < total_pages;
    Ok(Json(FsListResponse {
        data: files,
        pagination: FsPaginationMeta {
            page,
            page_size,
            total,
            total_pages,
            has_next,
            has_prev,
        },
    }))
}

pub async fn get_folder_tree(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
) -> AppResult<Json<FolderTreeView>> {
    let cache_key = format!("folder_tree:{folder_id}");
    
    // Try to get from cache first
    if let Some(redis_client) = state.redis_client.clone() {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            use redis::AsyncCommands;
            let cached: redis::RedisResult<String> = conn.get(&cache_key).await;
            if let Ok(json_str) = cached {
                if let Ok(tree) = serde_json::from_str::<FolderTreeView>(&json_str) {
                    tracing::debug!("Folder tree cache hit for {}", folder_id);
                    return Ok(Json(tree));
                }
            }
        }
    }

    let query = GetFolderTreeQuery {
        folder_id: parse_uuid(&folder_id, "folder_id")?,
        depth: Some(10),
        user_id: parse_uuid(&actor_id, "actor_id")?,
    };
    let h = Arc::new(fs_state(&state).get_folder_tree_handler());
    let tree = state.query_bus.dispatch_with_handler(query, h).await?;
    
    // Cache the result
    if let Some(redis_client) = state.redis_client.clone() {
        if let Ok(mut conn) = redis_client.get_multiplexed_async_connection().await {
            use redis::AsyncCommands;
            if let Ok(json_str) = serde_json::to_string(&tree) {
                // Cache for 5 minutes
                let _: redis::RedisResult<()> = conn.set_ex(&cache_key, json_str, 300).await;
            }
        }
    }
    
    Ok(Json(tree))
}

pub async fn get_permissions(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;

    // Owner can always read permissions; otherwise require admin permission.
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT owner_id FROM file_views WHERE id = $1")
        .bind(file_uuid)
        .fetch_optional(state.pool())
        .await?;
    let is_owner = owner.map(|o| o == user_uuid).unwrap_or(false);
    if !is_owner {
        let svc = fs_state(&state).service;
        let ok = svc
            .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Admin)
            .await
            .unwrap_or(false);
        if !ok {
            return Err(AppError::NotFound("File not found".to_string()));
        }
    }

    let rows: Vec<(String, Option<Uuid>, String, bool)> = sqlx::query_as(
        r#"
        SELECT subject_type, subject_id, permission, inherited
        FROM file_permissions
        WHERE file_id = $1
        ORDER BY subject_type, subject_id NULLS FIRST, permission
        "#,
    )
    .bind(file_uuid)
    .fetch_all(state.pool())
    .await
    .unwrap_or_default();

    Ok(Json(serde_json::json!({
        "file_id": file_uuid,
        "owner_id": owner,
        "entries": rows.into_iter().map(|(subject_type, subject_id, permission, inherited)| serde_json::json!({
            "subject_type": subject_type,
            "subject_id": subject_id,
            "permission": permission,
            "inherited": inherited
        })).collect::<Vec<_>>()
    })))
}

pub async fn set_permissions(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(payload): Json<SetPermissionsPayload>,
) -> AppResult<StatusCode> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;

    // Require admin permission (or owner).
    let owner: Option<Uuid> = sqlx::query_scalar("SELECT owner_id FROM file_views WHERE id = $1")
        .bind(file_uuid)
        .fetch_optional(state.pool())
        .await?;
    let is_owner = owner.map(|o| o == user_uuid).unwrap_or(false);
    if !is_owner {
        let svc = fs_state(&state).service;
        let ok = svc
            .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Admin)
            .await
            .unwrap_or(false);
        if !ok {
            return Err(AppError::NotFound("File not found".to_string()));
        }
    }

    let cmd = SetFilePermissionsCommand {
        file_id: file_uuid,
        acl: payload.acl,
    };
    let h = Arc::new(fs_state(&state).set_file_permissions_handler());
    state.command_bus.dispatch_with_handler(cmd, h).await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &file_id,
        "PERMISSIONS",
        Some("active"),
        Some("active"),
        Some(&actor_id),
        None,
        None,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_versions(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Read)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    let rows: Vec<(Uuid, i32, bool, Option<String>, chrono::DateTime<chrono::Utc>, Option<Uuid>, Uuid)> =
        sqlx::query_as(
            r#"
            SELECT v.id, v.version_no, v.is_current, v.note, v.created_at, v.created_by, v.storage_file_id
            FROM fs_file_versions v
            WHERE v.file_id = $1
            ORDER BY v.version_no DESC
            "#,
        )
        .bind(file_uuid)
        .fetch_all(state.pool())
        .await
        .unwrap_or_default();

    Ok(Json(
        rows.into_iter()
            .map(|(id, version_no, is_current, note, created_at, created_by, storage_file_id)| {
                serde_json::json!({
                    "id": id,
                    "version_no": version_no,
                    "is_current": is_current,
                    "note": note,
                    "created_at": created_at,
                    "created_by": created_by,
                    "storage_file_id": storage_file_id
                })
            })
            .collect(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct RollbackPayload {
    pub version_id: String,
    pub note: Option<String>,
}

pub async fn rollback_version(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(payload): Json<RollbackPayload>,
) -> AppResult<(StatusCode, Json<serde_json::Value>)> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let version_uuid = parse_uuid(&payload.version_id, "version_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Write)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    // Load target version + current version.
    let target: Option<(Uuid, i32, Uuid)> = sqlx::query_as(
        "SELECT id, version_no, storage_file_id FROM fs_file_versions WHERE id = $1 AND file_id = $2",
    )
    .bind(version_uuid)
    .bind(file_uuid)
    .fetch_optional(state.pool())
    .await?;
    let Some((_, target_no, target_storage)) = target else {
        return Err(AppError::NotFound("Version not found".to_string()));
    };

    let mut tx = state.pool().begin().await?;

    // Unset current, then create new head version referencing same storage_file_id.
    sqlx::query("UPDATE fs_file_versions SET is_current = FALSE WHERE file_id = $1 AND is_current = TRUE")
        .bind(file_uuid)
        .execute(&mut *tx)
        .await?;

    let next_no: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version_no), 0) + 1 FROM fs_file_versions WHERE file_id = $1")
        .bind(file_uuid)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or(target_no + 1);

    let new_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO fs_file_versions (id, file_id, storage_file_id, version_no, is_current, note, created_by)
        VALUES ($1, $2, $3, $4, TRUE, $5, $6)
        "#,
    )
    .bind(new_id)
    .bind(file_uuid)
    .bind(target_storage)
    .bind(next_no)
    .bind(payload.note.clone())
    .bind(user_uuid)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE file_views
        SET storage_file_id = $1, current_version_id = $2, updated_at = NOW(), updated_by = $3
        WHERE id = $4
        "#,
    )
    .bind(target_storage)
    .bind(new_id)
    .bind(user_uuid)
    .bind(file_uuid)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    append_aggregate_history(
        &state.pool,
        "file",
        &file_id,
        "ROLLBACK",
        Some("active"),
        Some("active"),
        Some(&actor_id),
        None,
        Some(serde_json::json!({
            "to_version_id": version_uuid,
            "to_version_no": target_no,
            "new_head_version_id": new_id,
            "new_head_version_no": next_no
        })),
    )
    .await?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": new_id }))))
}

pub async fn star_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<StatusCode> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Read)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }
    let _ = sqlx::query(
        "INSERT INTO fs_file_stars (file_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(file_uuid)
    .bind(user_uuid)
    .execute(state.pool())
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn unstar_file(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<StatusCode> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let _ = sqlx::query("DELETE FROM fs_file_stars WHERE file_id = $1 AND user_id = $2")
        .bind(file_uuid)
        .bind(user_uuid)
        .execute(state.pool())
        .await;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<i64>,
}

pub async fn get_activity(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Query(q): Query<ActivityQuery>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Read)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    let rows: Vec<(
        Uuid,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        chrono::DateTime<chrono::Utc>,
    )> =
        sqlx::query_as(
            r#"
            SELECT id, aggregate_type, aggregate_id, action, old_status, new_status, actor_id, comment, metadata, created_at
            FROM aggregate_history
            WHERE aggregate_type = 'file' AND aggregate_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(file_uuid.to_string())
        .bind(limit)
        .fetch_all(state.pool())
        .await
        .unwrap_or_default();

    Ok(Json(
        rows.into_iter()
            .map(
                |(
                    id,
                    aggregate_type,
                    aggregate_id,
                    action,
                    old_status,
                    new_status,
                    actor_id,
                    comment,
                    metadata,
                    created_at,
                )| {
                serde_json::json!({
                    "id": id,
                    "aggregate_type": aggregate_type,
                    "aggregate_id": aggregate_id,
                    "action": action,
                    "old_status": old_status,
                    "new_status": new_status,
                    "actor_id": actor_id,
                    "comment": comment,
                    "metadata": metadata,
                    "created_at": created_at
                })
            },
            )
            .collect(),
    ))
}

pub async fn get_download_url_fs(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Read)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    let storage_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT storage_file_id FROM file_views WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(file_uuid)
    .fetch_optional(state.pool())
    .await?;
    let Some(storage_file_id) = storage_id else {
        return Err(AppError::NotFound("No binary linked to this file".to_string()));
    };

    let file_path: Option<String> = sqlx::query_scalar("SELECT file_path FROM files WHERE id = $1")
        .bind(storage_file_id)
        .fetch_optional(state.pool())
        .await?;
    let Some(uri) = file_path else {
        return Err(AppError::NotFound("Binary not found".to_string()));
    };

    let signed = state
        .object_storage
        .presign_get_url(&uri, 900)
        .await
        .map_err(|e| AppError::InternalServerError(format!("presign failed: {e}")))?;

    if let Some(url) = signed {
        return Ok(Json(serde_json::json!({ "download_url": url, "expires_in": 900 })));
    }
    Err(AppError::BadRequest(
        "Current storage backend does not support pre-signed URL".to_string(),
    ))
}

/// Stream/proxy the binary file for a FS file.
/// - For local storage (path on disk): stream via tokio fs.
/// - For rustfs:// uri: presign and proxy via reqwest streaming.
pub async fn download_file_fs(
    Extension(actor_id): Extension<String>,
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> AppResult<Response> {
    let file_uuid = parse_uuid(&file_id, "file_id")?;
    let user_uuid = parse_uuid(&actor_id, "actor_id")?;
    let svc = fs_state(&state).service;
    let ok = svc
        .check_permission(file_uuid, user_uuid, crate::core::shared::Permission::Read)
        .await
        .unwrap_or(false);
    if !ok {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    // Resolve storage file info.
    let storage_file_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT storage_file_id FROM file_views WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(file_uuid)
    .fetch_optional(state.pool())
    .await?;
    let Some(storage_id) = storage_file_id else {
        return Err(AppError::NotFound("No binary linked to this file".to_string()));
    };

    let rec: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT file_path, COALESCE(file_type,'application/octet-stream') AS file_type, original_name FROM files WHERE id = $1",
    )
    .bind(storage_id)
    .fetch_optional(state.pool())
    .await?;
    let Some((file_path, file_type, original_name)) = rec else {
        return Err(AppError::NotFound("Binary not found".to_string()));
    };

    let mut headers = HeaderMap::new();
    if let Ok(ct) = HeaderValue::from_str(&file_type) {
        headers.insert(header::CONTENT_TYPE, ct);
    } else {
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    }
    headers.insert(
        header::CONTENT_DISPOSITION,
        content_disposition_attachment(original_name.as_deref().unwrap_or("download")),
    );

    // rustfs: presign then proxy stream
    if file_path.starts_with("rustfs://") {
        let signed = state
            .object_storage
            .presign_get_url(&file_path, 900)
            .await
            .map_err(|e| AppError::InternalServerError(format!("presign failed: {e}")))?;
        let Some(url) = signed else {
            return Err(AppError::BadRequest(
                "Current storage backend does not support pre-signed URL".to_string(),
            ));
        };

        let resp = reqwest::get(url)
            .await
            .map_err(|e| AppError::InternalServerError(format!("proxy fetch failed: {e}")))?;
        if !resp.status().is_success() {
            return Err(AppError::NotFound("File not found".to_string()));
        }
        let stream = resp
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
        let body = Body::from_stream(stream);
        return Ok((StatusCode::OK, headers, body).into_response());
    }

    // local path: allow only known-safe prefixes (match legacy guardrails)
    if !file_path.starts_with("./") && !file_path.starts_with("uploads/") && !file_path.contains(":\\") {
        return Err(AppError::BadRequest("Unsupported storage backend".to_string()));
    }

    let disk = fs::File::open(&file_path).await.map_err(|e| {
        tracing::error!(path = %file_path, error = %e, "open file for fs download");
        AppError::NotFound("File not found".to_string())
    })?;
    let body = Body::from_stream(ReaderStream::new(disk));
    Ok((StatusCode::OK, headers, body).into_response())
}
