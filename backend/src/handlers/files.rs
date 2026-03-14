use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{File, FileQuery};
use crate::utils::{file_validator, pagination::{PaginationParams, PaginatedResponse}};

pub async fn list_files(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<File>>> {
    let pool = state.pool();

    pagination.validate()?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await?;

    let files = sqlx::query_as::<_, File>(
        "SELECT * FROM files ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(files, pagination.page, pagination.limit, total)))
}

pub async fn search_files(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<FileQuery>,
) -> AppResult<Json<PaginatedResponse<File>>> {
    let pool = state.pool();

    pagination.validate()?;

    let search_term = query.search.ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;
    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    // Get total count
    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM files f
        INNER JOIN files_fts fts ON f.id = fts.id
        WHERE files_fts MATCH ?1
        "#,
    )
    .bind(&search_term)
    .fetch_one(pool)
    .await?;

    // Use FTS5 full-text search with MATCH operator
    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT f.* FROM files f
        INNER JOIN files_fts fts ON f.id = fts.id
        WHERE files_fts MATCH ?1
        ORDER BY rank, f.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(&search_term)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(files, page, limit, total)))
}

pub async fn upload_file(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<File>> {
    let pool = state.pool();
    let config = state.config();

    tracing::info!(user_id = %user_id, "File upload initiated");

    // Ensure uploads directory exists
    let upload_dir = "uploads";
    fs::create_dir_all(upload_dir).await.map_err(|e| {
        tracing::error!("Failed to create upload directory: {}", e);
        AppError::InternalServerError(format!("Failed to create upload directory: {}", e))
    })?;

    let mut file_name: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            file_name = field.file_name().map(|s| s.to_string());
            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read file data: {}", e)))?
                    .to_vec(),
            );
        }
    }

    let original_name = file_name.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;

    let file_bytes =
        file_data.ok_or_else(|| AppError::BadRequest("No file data provided".to_string()))?;

    // Detect MIME type
    let file_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    // VALIDATE FILE (type, size, extension, sanitize filename)
    let safe_filename = file_validator::validate_upload(
        &original_name,
        &file_bytes,
        &file_type,
        config.max_file_size,
    )?;

    tracing::debug!(
        original_name = %original_name,
        safe_filename = %safe_filename,
        file_type = %file_type,
        size_bytes = file_bytes.len(),
        "File validation passed"
    );

    // Generate unique file name
    let file_id = Uuid::new_v4().to_string();
    let extension = std::path::Path::new(&safe_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let stored_name = if !extension.is_empty() {
        format!("{}_{}.{}", file_id, Uuid::new_v4().simple(), extension)
    } else {
        format!("{}_{}", file_id, Uuid::new_v4().simple())
    };

    let file_path = format!("{}/{}", upload_dir, stored_name);

    // Write file to disk
    let mut file = fs::File::create(&file_path).await.map_err(|e| {
        tracing::error!("Failed to create file on disk: {}", e);
        AppError::InternalServerError(format!("Failed to create file: {}", e))
    })?;

    file.write_all(&file_bytes).await.map_err(|e| {
        tracing::error!("Failed to write file data: {}", e);
        AppError::InternalServerError(format!("Failed to write file: {}", e))
    })?;

    let file_size = file_bytes.len() as i64;

    // Insert into database
    let file_record = sqlx::query_as::<_, File>(
        "INSERT INTO files (id, name, original_name, file_path, file_type, file_size, uploaded_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         RETURNING *"
    )
    .bind(&file_id)
    .bind(&stored_name)
    .bind(&safe_filename)
    .bind(&file_path)
    .bind(&file_type)
    .bind(file_size)
    .bind(&user_id)
    .fetch_one(pool)
    .await?;

    tracing::info!(
        file_id = %file_record.id,
        original_name = %safe_filename,
        size_mb = ?(file_size as f64 / 1_048_576.0),
        "File uploaded successfully"
    );

    Ok(Json(file_record))
}

pub async fn get_file(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<File>> {
    let pool = state.pool();

    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(file))
}

pub async fn download_file(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let pool = state.pool();

    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // Read file from disk
    let file_bytes = fs::read(&file.file_path).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to read file: {}", e))
    })?;

    // Return file as response
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, file.file_type.as_str()),
            (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", file.original_name)),
        ],
        file_bytes,
    ).into_response())
}

pub async fn delete_file(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    // Get file to delete from disk
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    // Delete from database
    sqlx::query("DELETE FROM files WHERE id = ?1")
        .bind(&id)
        .execute(pool)
        .await?;

    // Delete file from disk (ignore errors if file doesn't exist)
    let _ = fs::remove_file(&file.file_path).await;

    Ok(Json(serde_json::json!({"message": "File deleted successfully"})))
}
