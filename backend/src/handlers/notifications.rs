use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{MarkAsReadRequest, Notification};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

#[derive(Debug, serde::Deserialize)]
pub struct NotificationQuery {
    /// Filter by read state (true/false)
    pub read: Option<bool>,
    /// Filter by notification type (info/success/warning/error/...)
    #[serde(rename = "type")]
    pub notification_type: Option<String>,
}

pub async fn list_notifications(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<NotificationQuery>,
) -> AppResult<Json<PaginatedResponse<Notification>>> {
    let pool = state.pool();

    pagination.validate()?;

    let mut where_sql = String::from("WHERE user_id = ?1");
    let mut bind_values: Vec<String> = Vec::new();
    bind_values.push(user_id.to_string());

    if let Some(read) = query.read {
        bind_values.push(if read { "1".to_string() } else { "0".to_string() });
        where_sql.push_str(&format!(" AND is_read = ?{}", bind_values.len()));
    }

    if let Some(t) = query.notification_type.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty())
    {
        bind_values.push(t.to_string());
        where_sql.push_str(&format!(" AND type = ?{}", bind_values.len()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM notifications {}", where_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for v in &bind_values {
        count_query = count_query.bind(v);
    }
    let total = count_query.fetch_one(pool).await?;

    let data_sql = format!(
        "SELECT * FROM notifications {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_sql
    );
    let mut data_query = sqlx::query_as::<_, Notification>(&data_sql);
    for v in bind_values {
        data_query = data_query.bind(v);
    }
    let notifications = data_query
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
