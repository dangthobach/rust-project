use axum::{extract::State, Extension, Json};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{MarkAsReadRequest, Notification};

pub async fn list_notifications(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
) -> AppResult<Json<Vec<Notification>>> {
    let notifications = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = ?1 ORDER BY created_at DESC LIMIT 50"
    )
    .bind(user_id.to_string())
    .fetch_all(&pool)
    .await?;

    Ok(Json(notifications))
}

pub async fn mark_as_read(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Json(payload): Json<MarkAsReadRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // SQLite doesn't support ANY(), use IN with placeholders
    let placeholders: Vec<String> = (1..=payload.notification_ids.len())
        .map(|i| format!("?{}", i))
        .collect();
    let placeholders_str = placeholders.join(", ");
    
    let query_str = format!(
        "UPDATE notifications SET is_read = 1 WHERE id IN ({}) AND user_id = ?{}",
        placeholders_str,
        payload.notification_ids.len() + 1
    );
    
    let mut query = sqlx::query(&query_str);
    for id in &payload.notification_ids {
        query = query.bind(id.to_string());
    }
    query = query.bind(user_id.to_string());
    query
    .execute(&pool)
    .await?;

    Ok(Json(serde_json::json!({"message": "Notifications marked as read"})))
}

pub async fn delete_notification(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM notifications WHERE id = ?1 AND user_id = ?2")
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "Notification deleted"})))
}
