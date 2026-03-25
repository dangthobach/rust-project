use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use tokio::fs;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::authz::{
    can_delete_file, can_download_file, can_read_file_meta, file_list_scope, file_search_scope,
    require_file_upload, AuthContext,
};
use crate::error::{AppError, AppResult};
use crate::models::{File, FileQuery};
use crate::utils::file_validator;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

const FTS_TERM_MAX_BYTES: usize = 200;
const FILE_ID_MAX_LEN: usize = 64;

fn parse_file_id(raw: &str) -> AppResult<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(AppError::BadRequest("File id is required".to_string()));
    }
    if t.len() > FILE_ID_MAX_LEN {
        return Err(AppError::BadRequest("Invalid file id".to_string()));
    }
    let u = Uuid::parse_str(t).map_err(|_| AppError::BadRequest("Invalid file id".to_string()))?;
    Ok(u.to_string())
}

fn sanitize_fts_query(raw: &str) -> AppResult<String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(AppError::ValidationError("Search term required".to_string()));
    }
    if t.len() > FTS_TERM_MAX_BYTES {
        return Err(AppError::ValidationError(format!(
            "Search term too long (max {} bytes)",
            FTS_TERM_MAX_BYTES
        )));
    }
    if t.chars().any(|c| c == '\0') {
        return Err(AppError::ValidationError("Invalid search term".to_string()));
    }
    Ok(t.replace('"', "\"\""))
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

pub async fn list_files(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<File>>> {
    let pool = state.pool();
    pagination.validate()?;

    let unrestricted = file_list_scope(&ctx)?;
    let uid = ctx.user_id.to_string();

    let total: i64 = if unrestricted {
        sqlx::query_scalar("SELECT COUNT(*) FROM files")
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM files WHERE uploaded_by = ?1")
            .bind(&uid)
            .fetch_one(pool)
            .await?
    };

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    let files = if unrestricted {
        sqlx::query_as::<_, File>(
            "SELECT * FROM files ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, File>(
            "SELECT * FROM files WHERE uploaded_by = ?1 ORDER BY created_at DESC LIMIT ? OFFSET ?",
        )
        .bind(&uid)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(Json(PaginatedResponse::new(files, page, limit, total)))
}

pub async fn search_files(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<FileQuery>,
) -> AppResult<Json<PaginatedResponse<File>>> {
    let pool = state.pool();
    pagination.validate()?;

    let search_raw = query
        .search
        .ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;
    let search_term = sanitize_fts_query(&search_raw)?;

    let unrestricted = file_search_scope(&ctx)?;
    let user_id = ctx.user_id.to_string();
    let scope_all: i64 = if unrestricted { 1 } else { 0 };

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM files f
        INNER JOIN files_fts fts ON f.id = fts.id
        WHERE files_fts MATCH ?1
          AND (?2 = 1 OR f.uploaded_by = ?3)
        "#,
    )
    .bind(&search_term)
    .bind(scope_all)
    .bind(&user_id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "FTS count failed");
        AppError::BadRequest(
            "Search could not be executed; simplify the search term".to_string(),
        )
    })?;

    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT f.* FROM files f
        INNER JOIN files_fts fts ON f.id = fts.id
        WHERE files_fts MATCH ?1
          AND (?2 = 1 OR f.uploaded_by = ?3)
        ORDER BY rank, f.created_at DESC
        LIMIT ?4 OFFSET ?5
        "#,
    )
    .bind(&search_term)
    .bind(scope_all)
    .bind(&user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "FTS search failed");
        AppError::BadRequest(
            "Search could not be executed; simplify the search term".to_string(),
        )
    })?;

    Ok(Json(PaginatedResponse::new(files, page, limit, total)))
}

pub async fn upload_file(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<File>> {
    require_file_upload(&ctx)?;

    let pool = state.pool();
    let config = state.config();
    let max = config.max_file_size;

    tracing::info!(user_id = %ctx.user_id, "File upload initiated");

    let mut file_name: Option<String> = None;
    let mut file_field_consumed = false;
    let mut buf: Vec<u8> = Vec::new();

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let field_name = field.name().unwrap_or("").trim();
        if field_name.is_empty() {
            continue;
        }
        if field_name != "file" {
            return Err(AppError::BadRequest(format!(
                "Unexpected multipart field: {}",
                field_name
            )));
        }
        if file_field_consumed {
            return Err(AppError::BadRequest(
                "Multiple file fields are not allowed".to_string(),
            ));
        }
        file_field_consumed = true;

        file_name = field
            .file_name()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty());

        loop {
            let chunk = field.chunk().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read upload chunk: {}", e))
            })?;
            let Some(bytes) = chunk else { break };
            if buf.len().saturating_add(bytes.len()) > max {
                return Err(AppError::BadRequest(format!(
                    "File exceeds maximum size of {} bytes",
                    max
                )));
            }
            buf.extend_from_slice(&bytes);
        }
    }

    if !file_field_consumed {
        return Err(AppError::BadRequest("No file provided".to_string()));
    }

    let original_name =
        file_name.ok_or_else(|| AppError::BadRequest("No file name provided".to_string()))?;

    let file_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    let safe_filename = file_validator::validate_upload(
        &original_name,
        &buf,
        &file_type,
        max,
    )?;

    let file_id = Uuid::new_v4().to_string();
    let extension = std::path::Path::new(&safe_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let object_name = if !extension.is_empty() {
        format!("{}_{}.{}", file_id, Uuid::new_v4().simple(), extension)
    } else {
        format!("{}_{}", file_id, Uuid::new_v4().simple())
    };

    use chrono::Datelike;
    let now = chrono::Utc::now();
    let tenant_id = state.config().default_tenant_id.clone();
    let object_key = format!(
        "{}/{}/{:04}/{:02}/{:02}/{}",
        tenant_id,
        ctx.user_id,
        now.year(),
        now.month(),
        now.day(),
        object_name
    );
    let file_path = state
        .object_storage
        .put_object(&object_key, &buf, &file_type)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Object storage put failed: {}", e)))?;

    let file_size = buf.len() as i64;

    let file_record = sqlx::query_as::<_, File>(
        r#"INSERT INTO files (id, name, original_name, file_path, file_type, file_size, uploaded_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         RETURNING *"#,
    )
    .bind(&file_id)
    .bind(&object_name)
    .bind(&safe_filename)
    .bind(&file_path)
    .bind(&file_type)
    .bind(file_size)
    .bind(ctx.user_id.to_string())
    .fetch_one(pool)
    .await;

    match file_record {
        Ok(rec) => {
            let thumb_job = serde_json::json!({
                "job": "thumbnail.generate",
                "file_id": rec.id,
                "object_key": object_key,
                "file_type": rec.file_type
            });
            let _ = state
                .rabbitmq_publisher
                .publish("crm.jobs", "thumbnail.generate", &thumb_job.to_string())
                .await;

            let domain_event = serde_json::json!({
                "event_type": "FileUploaded",
                "file_id": rec.id,
                "uploaded_by": rec.uploaded_by,
                "file_type": rec.file_type,
                "file_size": rec.file_size,
                "occurred_at": chrono::Utc::now().to_rfc3339()
            });
            let _ = state
                .kafka_publisher
                .publish("crm.domain.file", &rec.id, &domain_event.to_string())
                .await;

            tracing::info!(
                file_id = %rec.id,
                original_name = %safe_filename,
                size_bytes = file_size,
                "File uploaded successfully"
            );
            Ok(Json(rec))
        }
        Err(e) => {
            Err(e.into())
        }
    }
}

pub async fn get_file(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<File>> {
    let pool = state.pool();
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_read_file_meta(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    Ok(Json(file))
}

pub async fn download_file(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let pool = state.pool();
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_download_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    if file.file_path.starts_with("rustfs://") {
        let signed = state
            .object_storage
            .presign_get_url(&file.file_path, 900)
            .await
            .map_err(|e| AppError::InternalServerError(format!("presign failed: {}", e)))?;
        if let Some(url) = signed {
            return Ok(Json(serde_json::json!({
                "download_url": url,
                "expires_in": 900
            }))
            .into_response());
        }
    } else if !file.file_path.starts_with("./")
        && !file.file_path.starts_with("uploads/")
        && !file.file_path.contains(":\\")
    {
        return Err(AppError::BadRequest("Unsupported storage backend".to_string()));
    }

    let disk = fs::File::open(&file.file_path).await.map_err(|e| {
        tracing::error!(path = %file.file_path, error = %e, "open file for download");
        AppError::NotFound("File not found".to_string())
    })?;

    let body = Body::from_stream(ReaderStream::new(disk));

    let mut headers = HeaderMap::new();
    if let Ok(ct) = HeaderValue::from_str(&file.file_type) {
        headers.insert(header::CONTENT_TYPE, ct);
    } else {
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
    }
    headers.insert(
        header::CONTENT_DISPOSITION,
        content_disposition_attachment(&file.original_name),
    );

    Ok((StatusCode::OK, headers, body).into_response())
}

pub async fn get_download_url(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };
    if !can_download_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    let signed = state
        .object_storage
        .presign_get_url(&file.file_path, 900)
        .await
        .map_err(|e| AppError::InternalServerError(format!("presign failed: {}", e)))?;

    if let Some(url) = signed {
        return Ok(Json(serde_json::json!({
            "download_url": url,
            "expires_in": 900
        })));
    }

    Err(AppError::BadRequest(
        "Current storage backend does not support pre-signed URL".to_string(),
    ))
}

pub async fn get_thumbnail_url(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_download_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    let Some(thumbnail_uri) = file.thumbnail_path.as_deref() else {
        return Err(AppError::NotFound("Thumbnail not found".to_string()));
    };

    let signed = state
        .object_storage
        .presign_get_url(thumbnail_uri, 900)
        .await
        .map_err(|e| AppError::InternalServerError(format!("presign failed: {}", e)))?;

    if let Some(url) = signed {
        return Ok(Json(serde_json::json!({
            "download_url": url,
            "expires_in": 900
        })));
    }

    Err(AppError::BadRequest(
        "Current storage backend does not support pre-signed thumbnail URL".to_string(),
    ))
}

pub async fn delete_file(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_delete_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    sqlx::query("DELETE FROM files WHERE id = ?1")
        .bind(&id)
        .execute(pool)
        .await?;

    if file.file_path.starts_with("./")
        || file.file_path.starts_with("uploads/")
        || file.file_path.contains(":\\")
    {
        let _ = fs::remove_file(&file.file_path).await;
    }

    Ok(Json(serde_json::json!({"message": "File deleted successfully"})))
}
