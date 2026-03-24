use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use sqlx::SqlitePool;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::authz::{
    can_delete_file, can_download_file, can_read_file_meta, file_list_scope, file_search_scope,
    require_file_upload, AuthContext,
};
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{File, FileQuery};

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

/// Reduce FTS5 syntax surprises / abuse; keep query as a single literal token phrase where possible.
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
    let escaped = t.replace('"', "\"\"");
    Ok(escaped)
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
    State((pool, _)): State<(SqlitePool, Config)>,
) -> AppResult<Json<Vec<File>>> {
    let unrestricted = file_list_scope(&ctx)?;
    let files = if unrestricted {
        sqlx::query_as::<_, File>(
            "SELECT * FROM files ORDER BY created_at DESC LIMIT 50",
        )
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as::<_, File>(
            "SELECT * FROM files WHERE uploaded_by = ?1 ORDER BY created_at DESC LIMIT 50",
        )
        .bind(ctx.user_id.to_string())
        .fetch_all(&pool)
        .await?
    };

    Ok(Json(files))
}

pub async fn search_files(
    Extension(ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Query(query): Query<FileQuery>,
) -> AppResult<Json<Vec<File>>> {
    let search_raw = query
        .search
        .ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;
    let search_term = sanitize_fts_query(&search_raw)?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1).saturating_mul(limit);

    let unrestricted = file_search_scope(&ctx)?;
    let user_id = ctx.user_id.to_string();
    let scope_all: i64 = if unrestricted { 1 } else { 0 };

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
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "FTS search failed (syntax or DB)");
        AppError::BadRequest(
            "Search could not be executed; simplify the search term".to_string(),
        )
    })?;

    Ok(Json(files))
}

pub async fn upload_file(
    Extension(ctx): Extension<AuthContext>,
    State((pool, config)): State<(SqlitePool, Config)>,
    mut multipart: Multipart,
) -> AppResult<Json<File>> {
    require_file_upload(&ctx)?;

    let upload_dir = "uploads";
    fs::create_dir_all(upload_dir).await.map_err(|e| {
        AppError::InternalServerError(format!("Failed to create upload directory: {}", e))
    })?;

    let mut file_name: Option<String> = None;
    let mut file_field_consumed = false;
    let mut temp_path: Option<String> = None;
    let mut stored_name: Option<String> = None;
    let file_id = Uuid::new_v4().to_string();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
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

        let original = field
            .file_name()
            .map(|s| s.to_string())
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| AppError::BadRequest("No file name provided".to_string()))?;

        file_name = Some(original.clone());

        let extension = std::path::Path::new(&original)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let stored = if !extension.is_empty() {
            format!("{}_{}.{}", file_id, Uuid::new_v4().simple(), extension)
        } else {
            format!("{}_{}", file_id, Uuid::new_v4().simple())
        };

        let path = format!("{}/{}", upload_dir, stored);
        let mut dst = fs::File::create(&path).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to create file: {}", e))
        })?;

        let max = config.max_file_size as u64;
        let mut written: u64 = 0;

        loop {
            let chunk = field.chunk().await.map_err(|e| {
                let _ = std::fs::remove_file(&path);
                AppError::BadRequest(format!("Failed to read upload chunk: {}", e))
            })?;
            let Some(bytes) = chunk else { break };
            let n = bytes.len() as u64;
            written = written.saturating_add(n);
            if written > max {
                let _ = fs::remove_file(&path).await;
                return Err(AppError::BadRequest(format!(
                    "File exceeds maximum size of {} bytes",
                    max
                )));
            }
            dst.write_all(&bytes).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to write file: {}", e))
            })?;
        }

        temp_path = Some(path);
        stored_name = Some(stored);
    }

    if !file_field_consumed {
        return Err(AppError::BadRequest("No file provided".to_string()));
    }

    let original_name = file_name.expect("validated");
    let stored_name = stored_name.expect("set with path");
    let file_path = temp_path.expect("set");

    let file_size = fs::metadata(&file_path)
        .await
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    let file_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    let file_record = sqlx::query_as::<_, File>(
        r#"INSERT INTO files (id, name, original_name, file_path, file_type, file_size, uploaded_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
         RETURNING *"#,
    )
    .bind(&file_id)
    .bind(&stored_name)
    .bind(&original_name)
    .bind(&file_path)
    .bind(&file_type)
    .bind(file_size)
    .bind(ctx.user_id.to_string())
    .fetch_one(&pool)
    .await;

    match file_record {
        Ok(rec) => Ok(Json(rec)),
        Err(e) => {
            let _ = fs::remove_file(&file_path).await;
            Err(e.into())
        }
    }
}

pub async fn get_file(
    Extension(ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Path(id): Path<String>,
) -> AppResult<Json<File>> {
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&pool)
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
    State((pool, _)): State<(SqlitePool, Config)>,
    Path(id): Path<String>,
) -> AppResult<Response> {
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_download_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
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

pub async fn delete_file(
    Extension(ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let id = parse_file_id(&id)?;
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(&id)
        .fetch_optional(&pool)
        .await?;

    let Some(file) = file else {
        return Err(AppError::NotFound("File not found".to_string()));
    };

    if !can_delete_file(&ctx, &file) {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    sqlx::query("DELETE FROM files WHERE id = ?1")
        .bind(&id)
        .execute(&pool)
        .await?;

    let _ = fs::remove_file(&file.file_path).await;

    Ok(Json(serde_json::json!({"message": "File deleted successfully"})))
}
