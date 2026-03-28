use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{MarkAsReadRequest, Notification};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

#[derive(Debug, serde::Deserialize)]
pub struct NotificationQuery {
    pub read: Option<bool>,
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

    let mut count_qb =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM notifications WHERE user_id = ");
    count_qb.push_bind(user_id);
    if let Some(read) = query.read {
        count_qb.push(" AND is_read = ").push_bind(read);
    }
    if let Some(t) = query.notification_type.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty())
    {
        count_qb.push(" AND ").push("\"type\" = ").push_bind(t);
    }
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM notifications WHERE user_id = ");
    qb.push_bind(user_id);
    if let Some(read) = query.read {
        qb.push(" AND is_read = ").push_bind(read);
    }
    if let Some(t) = query.notification_type.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty())
    {
        qb.push(" AND ").push("\"type\" = ").push_bind(t);
    }
    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(pagination.limit)
        .push(" OFFSET ")
        .push_bind(pagination.offset());

    let notifications = qb.build_query_as::<Notification>().fetch_all(pool).await?;

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

    if payload.notification_ids.is_empty() {
        return Ok(Json(serde_json::json!({"message": "No notifications selected"})));
    }

    let mut qb =
        QueryBuilder::<Postgres>::new("UPDATE notifications SET is_read = TRUE WHERE user_id = ");
    qb.push_bind(user_id);
    qb.push(" AND id IN (");
    {
        let mut sep = qb.separated(", ");
        for id in &payload.notification_ids {
            sep.push_bind(id);
        }
    }
    qb.push(")");

    qb.build().execute(pool).await?;

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

    let result = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "Notification deleted"})))
}
