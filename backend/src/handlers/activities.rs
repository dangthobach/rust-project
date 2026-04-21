use axum::{
    extract::{Query, State},
    http::StatusCode,
    Extension, Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    app_state::AppState,
    authz::{permissions as perm, AuthContext},
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

/// Get recent activities with pagination.
/// Non-admin callers are automatically scoped to their own activities;
/// the `user_id` query param is ignored for them.
/// Admins (user.manage or superuser) may filter by any user_id.
pub async fn get_activities(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ActivityQueryParams>,
) -> Result<Json<PaginatedActivities>, AppError> {
    let pool = state.pool();
    let offset = (params.page - 1) * params.limit;
    let admin_view = ctx.superuser() || ctx.has(perm::USER_MANAGE);

    // Resolve the effective user_id scope:
    // - admins: honour the query param (None = all users)
    // - regular users: always force to caller's own id
    let effective_uid: Option<Uuid> = if admin_view {
        params
            .user_id
            .as_deref()
            .map(|uid| {
                Uuid::parse_str(uid)
                    .map_err(|_| AppError::BadRequest("Invalid user_id filter".to_string()))
            })
            .transpose()?
    } else {
        Some(ctx.user_id)
    };

    let mut count_qb = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM activities WHERE 1=1");
    if let Some(ref et) = params.entity_type {
        count_qb.push(" AND entity_type = ");
        count_qb.push_bind(et);
    }
    if let Some(uid) = effective_uid {
        count_qb.push(" AND user_id = ");
        count_qb.push_bind(uid);
    }
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count activities: {}", e);
            AppError::InternalServerError("Failed to count activities".to_string())
        })?;

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM activities WHERE 1=1");
    if let Some(ref et) = params.entity_type {
        qb.push(" AND entity_type = ");
        qb.push_bind(et);
    }
    if let Some(uid) = effective_uid {
        qb.push(" AND user_id = ");
        qb.push_bind(uid);
    }
    qb.push(" ORDER BY created_at DESC LIMIT ");
    qb.push_bind(params.limit);
    qb.push(" OFFSET ");
    qb.push_bind(offset);

    let activities = qb
        .build_query_as::<Activity>()
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

    let metadata_str = request
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m).unwrap_or_default());

    let activity = sqlx::query_as::<_, Activity>(
        r#"
        INSERT INTO activities (id, user_id, entity_type, entity_id, action, description, metadata, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(activity_id)
    .bind(user_uuid)
    .bind(&request.entity_type)
    .bind(request.entity_id.to_string())
    .bind(&request.action)
    .bind(&request.description)
    .bind(metadata_str)
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
    pool: &PgPool,
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
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(activity_id)
    .bind(user_id)
    .bind(entity_type)
    .bind(entity_id.to_string())
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
