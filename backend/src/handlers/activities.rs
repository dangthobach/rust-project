use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    error::AppError,
    models::activity::{Activity, CreateActivityRequest},
};

#[derive(Debug, Deserialize)]
pub struct ActivityQueryParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub entity_type: Option<String>,
    pub user_id: Option<String>,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    10
}

#[derive(Debug, Serialize)]
pub struct PaginatedActivities {
    pub data: Vec<ActivityResponse>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize)]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub icon: String,
    pub action: String,
    pub detail: String,
    pub time: String,
    pub color: String,
    pub user_id: Option<String>,
    pub related_id: String,
    pub created_at: String,
}

/// Get recent activities with pagination
pub async fn get_activities(
    State(state): State<AppState>,
    Query(params): Query<ActivityQueryParams>,
) -> Result<Json<PaginatedActivities>, AppError> {
    let pool = state.pool();
    let offset = (params.page - 1) * params.limit;

    // Build query with optional filters
    let mut query = String::from("SELECT * FROM activities WHERE 1=1");
    let mut count_query = String::from("SELECT COUNT(*) as count FROM activities WHERE 1=1");

    if params.entity_type.is_some() {
        query.push_str(" AND entity_type = ?");
        count_query.push_str(" AND entity_type = ?");
    }

    if params.user_id.is_some() {
        query.push_str(" AND user_id = ?");
        count_query.push_str(" AND user_id = ?");
    }

    query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");

    // Get total count
    let total: i64 = sqlx::query_scalar::<_, i64>(&count_query)
        .bind(params.entity_type.as_ref())
        .bind(params.user_id.as_ref())
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count activities: {}", e);
            AppError::InternalServerError("Failed to count activities".to_string())
        })?;

    // Get activities
    let activities = sqlx::query_as::<_, Activity>(&query)
        .bind(params.entity_type.as_ref())
        .bind(params.user_id.as_ref())
        .bind(params.limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch activities: {}", e);
            AppError::InternalServerError("Failed to fetch activities".to_string())
        })?;

    // Convert to response format
    let data: Vec<ActivityResponse> = activities
        .into_iter()
        .map(|activity| {
            let activity_type = activity.entity_type.clone();
            let (icon, color) = get_activity_icon_and_color(&activity_type, &activity.action);
            
            let time = format_relative_time(&activity.created_at);

            ActivityResponse {
                id: activity.id.to_string(),
                activity_type: activity_type.clone(),
                icon,
                action: activity.action.clone(),
                detail: activity.description.clone(),
                time,
                color,
                user_id: activity.user_id.map(|id| id.to_string()),
                related_id: activity.entity_id.to_string(),
                created_at: activity.created_at.to_rfc3339(),
            }
        })
        .collect();

    let total_pages = (total as f64 / params.limit as f64).ceil() as i64;

    Ok(Json(PaginatedActivities {
        data,
        pagination: PaginationInfo {
            page: params.page,
            limit: params.limit,
            total,
            total_pages,
            has_next: params.page < total_pages,
            has_prev: params.page > 1,
        },
    }))
}

/// Log a new activity
pub async fn create_activity(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(request): Json<CreateActivityRequest>,
) -> Result<(StatusCode, Json<Activity>), AppError> {
    let pool = state.pool();
    let activity_id = Uuid::new_v4();
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let now = Utc::now();

    let activity = sqlx::query_as::<_, Activity>(
        r#"
        INSERT INTO activities (id, user_id, entity_type, entity_id, action, description, metadata, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(activity_id)
    .bind(user_uuid)
    .bind(&request.entity_type)
    .bind(&request.entity_id)
    .bind(&request.action)
    .bind(&request.description)
    .bind(&request.metadata)
    .bind(now)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create activity: {}", e);
        AppError::InternalServerError("Failed to create activity".to_string())
    })?;

    tracing::info!(
        activity_id = %activity.id,
        user_id = %user_id,
        entity_type = %request.entity_type,
        "Activity logged"
    );

    Ok((StatusCode::CREATED, Json(activity)))
}

/// Helper function to get icon and color based on activity type
fn get_activity_icon_and_color(entity_type: &str, action: &str) -> (String, String) {
    match (entity_type, action) {
        ("client", "created") => ("👤".to_string(), "bg-primary".to_string()),
        ("client", "updated") => ("✏️".to_string(), "bg-neutral-concrete".to_string()),
        ("client", "deleted") => ("🗑️".to_string(), "bg-red-400".to_string()),
        ("task", "created") => ("📋".to_string(), "bg-primary".to_string()),
        ("task", "updated") => ("✏️".to_string(), "bg-neutral-concrete".to_string()),
        ("task", "completed") => ("✅".to_string(), "bg-green-400".to_string()),
        ("task", "deleted") => ("🗑️".to_string(), "bg-red-400".to_string()),
        ("file", "uploaded") => ("📁".to_string(), "bg-accent-yellow".to_string()),
        ("file", "deleted") => ("🗑️".to_string(), "bg-red-400".to_string()),
        ("email", "sent") => ("📧".to_string(), "bg-secondary".to_string()),
        ("note", "added") => ("📝".to_string(), "bg-green-400".to_string()),
        ("user", "login") => ("🔐".to_string(), "bg-primary".to_string()),
        ("user", "logout") => ("🚪".to_string(), "bg-neutral-concrete".to_string()),
        _ => ("📌".to_string(), "bg-neutral-concrete".to_string()),
    }
}

/// Format datetime to relative time string
fn format_relative_time(dt: &chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    let seconds = duration.num_seconds();
    let minutes = duration.num_minutes();
    let hours = duration.num_hours();
    let days = duration.num_days();

    if seconds < 60 {
        "Just now".to_string()
    } else if minutes < 60 {
        format!("{} minute{} ago", minutes, if minutes > 1 { "s" } else { "" })
    } else if hours < 24 {
        format!("{} hour{} ago", hours, if hours > 1 { "s" } else { "" })
    } else if days < 7 {
        format!("{} day{} ago", days, if days > 1 { "s" } else { "" })
    } else {
        dt.format("%Y-%m-%d").to_string()
    }
}

/// Helper function to log activity (can be called from other handlers)
pub async fn log_activity(
    pool: &SqlitePool,
    user_id: Uuid,
    entity_type: &str,
    entity_id: Uuid,
    action: &str,
    description: &str,
) -> Result<(), AppError> {
    let activity_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO activities (id, user_id, entity_type, entity_id, action, description, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(activity_id)
    .bind(user_id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(action)
    .bind(description)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::warn!("Failed to log activity: {}", e);
        // Don't fail the main operation if logging fails
        AppError::InternalServerError("Failed to log activity".to_string())
    })?;

    Ok(())
}
