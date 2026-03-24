use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{MarkAsReadRequest, Notification};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

pub async fn list_notifications(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Notification>>> {
    let pool = state.pool();

    pagination.validate()?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = ?1")
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

    let notifications = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = ?1 ORDER BY created_at DESC LIMIT ? OFFSET ?",
    )
    .bind(user_id.to_string())
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(
        notifications,
        pagination.page,
        pagination.limit,
        total,
    )))
}

pub async fn mark_as_read(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<MarkAsReadRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

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
    query.execute(pool).await?;

    Ok(Json(
        serde_json::json!({"message": "Notifications marked as read"}),
    ))
}

pub async fn delete_notification(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    let result = sqlx::query("DELETE FROM notifications WHERE id = ?1 AND user_id = ?2")
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "Notification deleted"})))
}
