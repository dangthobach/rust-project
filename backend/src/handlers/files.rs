use axum::{extract::State, Extension, Json};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::File;

// Placeholder implementations - file upload requires multipart form handling
pub async fn list_files(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
) -> AppResult<Json<Vec<File>>> {
    let files = sqlx::query_as::<_, File>("SELECT * FROM files ORDER BY created_at DESC LIMIT 50")
        .fetch_all(&pool)
        .await?;

    Ok(Json(files))
}

pub async fn upload_file(
    Extension(_user_id): Extension<Uuid>,
    State((_pool, _config)): State<(SqlitePool, Config)>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Implement multipart file upload
    Ok(Json(serde_json::json!({"message": "File upload endpoint - implement multipart"})))
}

pub async fn get_file(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<File>> {
    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("File not found".to_string()))?;

    Ok(Json(file))
}

pub async fn download_file(
    Extension(_user_id): Extension<Uuid>,
    State((_pool, _config)): State<(SqlitePool, Config)>,
    axum::extract::Path(_id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    // TODO: Implement file download
    Ok(Json(serde_json::json!({"message": "File download endpoint"})))
}

pub async fn delete_file(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM files WHERE id = ?1")
        .bind(id.to_string())
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("File not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "File deleted successfully"})))
}
