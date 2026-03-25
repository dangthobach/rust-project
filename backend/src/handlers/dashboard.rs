use axum::{extract::{Query, State}, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::app_state::AppState;
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub clients: ClientStats,
    pub tasks: TaskStats,
    pub files: FileStats,
    pub notifications: NotificationStats,
    pub activities: ActivityStats,
}

#[derive(Debug, Serialize)]
pub struct ClientStats {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub new_this_week: i64,
    pub new_this_month: i64,
}

#[derive(Debug, Serialize)]
pub struct TaskStats {
    pub total: i64,
    pub completed: i64,
    pub in_progress: i64,
    pub pending: i64,
    pub cancelled: i64,
    pub overdue: i64,
    pub due_today: i64,
    pub due_this_week: i64,
}

#[derive(Debug, Serialize)]
pub struct FileStats {
    pub total: i64,
    pub total_size: i64,
    pub uploaded_this_week: i64,
}

#[derive(Debug, Serialize)]
pub struct NotificationStats {
    pub total: i64,
    pub unread: i64,
}

#[derive(Debug, Serialize)]
pub struct ActivityStats {
    pub total: i64,
    pub today: i64,
    pub this_week: i64,
}

/// Get comprehensive dashboard statistics
pub async fn get_dashboard_stats(
    State(state): State<AppState>,
    Extension(_user_id): Extension<String>,
) -> Result<Json<DashboardStats>, AppError> {
    let pool = state.pool();

    // Run all queries in parallel using tokio::try_join!
    let (client_stats, task_stats, file_stats, notification_stats, activity_stats) =
        tokio::try_join!(
            get_client_stats(pool),
            get_task_stats(pool),
            get_file_stats(pool),
            get_notification_stats(pool),
            get_activity_stats(pool),
        )?;

    Ok(Json(DashboardStats {
        clients: client_stats,
        tasks: task_stats,
        files: file_stats,
        notifications: notification_stats,
        activities: activity_stats,
    }))
}

async fn get_client_stats(pool: &SqlitePool) -> Result<ClientStats, AppError> {
    // Total clients
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count clients: {}", e);
            AppError::InternalServerError("Failed to fetch client stats".to_string())
        })?;

    // Active clients
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE status = 'active'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Inactive clients
    let inactive: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE status = 'inactive'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // New this week (last 7 days)
    let new_this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM clients WHERE created_at >= datetime('now', '-7 days')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // New this month (last 30 days)
    let new_this_month: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM clients WHERE created_at >= datetime('now', '-30 days')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(ClientStats {
        total,
        active,
        inactive,
        new_this_week,
        new_this_month,
    })
}

async fn get_task_stats(pool: &SqlitePool) -> Result<TaskStats, AppError> {
    // Total tasks
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count tasks: {}", e);
            AppError::InternalServerError("Failed to fetch task stats".to_string())
        })?;

    // Status counts
    let completed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'done'")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    let in_progress: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'in_progress'")
            .fetch_one(pool)
            .await
            .unwrap_or(0);

    let pending: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'todo'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let cancelled: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'cancelled'")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Overdue tasks (due_date < now AND status != completed)
    let overdue: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE due_date < datetime('now') AND status != 'done'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Due today
    let due_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE date(due_date) = date('now') AND status != 'done'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Due this week
    let due_this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE due_date BETWEEN datetime('now') AND datetime('now', '+7 days') AND status != 'done'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(TaskStats {
        total,
        completed,
        in_progress,
        pending,
        cancelled,
        overdue,
        due_today,
        due_this_week,
    })
}

async fn get_file_stats(pool: &SqlitePool) -> Result<FileStats, AppError> {
    // Total files
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Total size (sum of all file sizes)
    let total_size: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(file_size), 0) FROM files")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Uploaded this week
    let uploaded_this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM files WHERE created_at >= datetime('now', '-7 days')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(FileStats {
        total,
        total_size,
        uploaded_this_week,
    })
}

async fn get_notification_stats(pool: &SqlitePool) -> Result<NotificationStats, AppError> {
    // Total notifications
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Unread notifications
    let unread: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE read = 0")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    Ok(NotificationStats { total, unread })
}

async fn get_activity_stats(pool: &SqlitePool) -> Result<ActivityStats, AppError> {
    // Total activities
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activities")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Today's activities
    let today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activities WHERE date(created_at) = date('now')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // This week's activities
    let this_week: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activities WHERE created_at >= datetime('now', '-7 days')",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(ActivityStats {
        total,
        today,
        this_week,
    })
}

// ===== Sprint 3 Admin Dashboard Endpoints =====

/// Activity feed item with user details
#[derive(Debug, Serialize)]
pub struct ActivityItem {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub created_at: String,
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
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

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub database: String,
    pub timestamp: String,
    pub version: String,
}

/// Get activity feed with pagination
/// GET /api/dashboard/activity-feed?page=1&limit=20
pub async fn get_activity_feed(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
    Extension(_user_id): Extension<String>,
) -> Result<Json<PaginatedResponse<ActivityItem>>, AppError> {
    let pool = state.pool();

    let page = params.page.max(1);
    let limit = params.limit.clamp(1, 100);
    let offset = (page - 1) * limit;

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activities")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count activities: {}", e);
            AppError::InternalServerError("Failed to fetch activity feed".to_string())
        })?;

    // Get activities with user information
    let activities = sqlx::query_as::<_, (String, String, String, Option<String>, String, String, Option<String>, Option<String>, String)>(
        "SELECT 
            a.id,
            a.user_id,
            u.full_name as user_name,
            u.avatar_url as user_avatar,
            a.action,
            a.resource_type,
            a.resource_id,
            a.details,
            a.created_at
         FROM activities a
         JOIN users u ON a.user_id = u.id
         ORDER BY a.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch activities: {}", e);
        AppError::InternalServerError("Failed to fetch activity feed".to_string())
    })?
    .into_iter()
    .map(|(id, user_id, user_name, user_avatar, action, resource_type, resource_id, details, created_at)| {
        ActivityItem {
            id,
            user_id,
            user_name,
            user_avatar,
            action,
            resource_type,
            resource_id,
            details,
            created_at,
        }
    })
    .collect();

    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    let response = PaginatedResponse {
        data: activities,
        pagination: PaginationInfo {
            page,
            limit,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        },
    };

    Ok(Json(response))
}

/// Health check endpoint for monitoring
/// GET /api/dashboard/health
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthCheckResponse>, AppError> {
    let pool = state.pool();

    // Test database connection
    let db_status = match sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
    {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    let response = HealthCheckResponse {
        status: if db_status == "healthy" { "ok".to_string() } else { "error".to_string() },
        database: db_status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };

    Ok(Json(response))
}
