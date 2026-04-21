///! Analytics Handler
///! Provides advanced analytics endpoints for admin dashboard

use axum::{
    extract::{Query, State},
    Extension,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::authz::AuthContext;
use crate::authz::permissions as perm;
use crate::error::AppError;

/// Date range query parameters
#[derive(Debug, Deserialize)]
pub struct DateRangeParams {
    #[serde(default = "default_start_date")]
    pub start_date: String,
    #[serde(default = "default_end_date")]
    pub end_date: String,
}

fn default_start_date() -> String {
    // 30 days ago
    chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(30))
        .unwrap()
        .format("%Y-%m-%d")
        .to_string()
}

fn default_end_date() -> String {
    // Today
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// User activity analytics
#[derive(Debug, Serialize)]
pub struct UserActivityAnalytics {
    pub total_activities: i64,
    pub unique_active_users: i64,
    pub average_activities_per_user: f64,
    pub most_active_users: Vec<ActiveUserInfo>,
    pub activity_by_type: Vec<ActivityTypeCount>,
    pub daily_activity: Vec<DailyActivity>,
}

#[derive(Debug, Serialize)]
pub struct ActiveUserInfo {
    pub user_id: String,
    pub user_name: String,
    pub avatar_url: Option<String>,
    pub activity_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ActivityTypeCount {
    pub action: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct DailyActivity {
    pub date: String,
    pub count: i64,
}

/// Task completion analytics
#[derive(Debug, Serialize)]
pub struct TaskCompletionAnalytics {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub completion_rate: f64,
    pub average_completion_time_hours: f64,
    pub completion_by_status: Vec<StatusCount>,
    pub completion_by_priority: Vec<PriorityCount>,
    pub daily_completions: Vec<DailyCompletion>,
}

#[derive(Debug, Serialize)]
pub struct StatusCount {
    pub status: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct PriorityCount {
    pub priority: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct DailyCompletion {
    pub date: String,
    pub completed_count: i64,
    pub created_count: i64,
}

/// Client engagement analytics
#[derive(Debug, Serialize)]
pub struct ClientEngagementAnalytics {
    pub total_clients: i64,
    pub active_clients: i64,
    pub inactive_clients: i64,
    pub engagement_rate: f64,
    pub average_tasks_per_client: f64,
    pub clients_with_recent_activity: i64,
    pub top_clients_by_tasks: Vec<ClientTaskCount>,
    pub new_clients_trend: Vec<NewClientsTrend>,
}

#[derive(Debug, Serialize)]
pub struct ClientTaskCount {
    pub client_id: String,
    pub client_name: String,
    pub task_count: i64,
    pub completed_tasks: i64,
}

#[derive(Debug, Serialize)]
pub struct NewClientsTrend {
    pub date: String,
    pub new_clients: i64,
}

/// Storage usage analytics
#[derive(Debug, Serialize)]
pub struct StorageAnalytics {
    pub total_files: i64,
    pub total_size_bytes: i64,
    pub total_size_mb: f64,
    pub total_size_gb: f64,
    pub average_file_size_bytes: f64,
    pub files_by_type: Vec<FileTypeCount>,
    pub storage_by_user: Vec<UserStorageInfo>,
    pub daily_upload_trend: Vec<DailyUpload>,
}

#[derive(Debug, Serialize)]
pub struct FileTypeCount {
    pub file_type: String,
    pub count: i64,
    pub total_size_bytes: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct UserStorageInfo {
    pub user_id: String,
    pub user_name: String,
    pub file_count: i64,
    pub total_size_mb: f64,
}

#[derive(Debug, Serialize)]
pub struct DailyUpload {
    pub date: String,
    pub file_count: i64,
    pub total_size_mb: f64,
}

/// Get user activity analytics
/// GET /api/analytics/user-activity?start_date=2024-01-01&end_date=2024-12-31
pub async fn get_user_activity_analytics(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
    Extension(ctx): Extension<AuthContext>,
) -> Result<Json<UserActivityAnalytics>, AppError> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    // Total activities in date range
    let total_activities: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activities
         WHERE (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Unique active users
    let unique_active_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM activities 
         WHERE (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(1);

    let average_activities_per_user = 
        total_activities as f64 / unique_active_users.max(1) as f64;

    // Most active users (top 10)
    let most_active_users = sqlx::query_as::<_, (String, String, Option<String>, i64)>(
        "SELECT a.user_id::text, u.full_name, u.avatar_url, COUNT(*) as activity_count
         FROM activities a
         JOIN users u ON a.user_id = u.id
         WHERE (a.created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY a.user_id, u.full_name, u.avatar_url
         ORDER BY activity_count DESC
         LIMIT 10"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(user_id, user_name, avatar_url, activity_count)| ActiveUserInfo {
        user_id,
        user_name,
        avatar_url,
        activity_count,
    })
    .collect();

    // Activity by type
    let activity_by_type = sqlx::query_as::<_, (String, i64)>(
        "SELECT action, COUNT(*) as count
         FROM activities
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY action
         ORDER BY count DESC"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(action, count)| ActivityTypeCount {
        action,
        count,
        percentage: (count as f64 / total_activities.max(1) as f64) * 100.0,
    })
    .collect();

    // Daily activity trend
    let daily_activity = sqlx::query_as::<_, (String, i64)>(
        "SELECT (created_at::date)::text as date, COUNT(*) as count
         FROM activities
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY created_at::date
         ORDER BY created_at::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(date, count)| DailyActivity { date, count })
    .collect();

    Ok(Json(UserActivityAnalytics {
        total_activities,
        unique_active_users,
        average_activities_per_user,
        most_active_users,
        activity_by_type,
        daily_activity,
    }))
}

/// Get task completion analytics
/// GET /api/analytics/task-completion?start_date=2024-01-01&end_date=2024-12-31
pub async fn get_task_completion_analytics(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
    Extension(ctx): Extension<AuthContext>,
) -> Result<Json<TaskCompletionAnalytics>, AppError> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    // Total tasks created in range
    let total_tasks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Completed tasks
    let completed_tasks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks 
         WHERE status = 'done'
         AND (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let completion_rate = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Average completion time (hours between created and updated for completed tasks)
    let average_completion_time_hours: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) / 3600.0), 0)
         FROM tasks
         WHERE status = 'done'
         AND (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);

    // Completion by status
    let completion_by_status = sqlx::query_as::<_, (String, i64)>(
        "SELECT status, COUNT(*) as count
         FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY status
         ORDER BY count DESC"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(status, count)| StatusCount {
        status,
        count,
        percentage: (count as f64 / total_tasks.max(1) as f64) * 100.0,
    })
    .collect();

    // Completion by priority
    let completion_by_priority = sqlx::query_as::<_, (String, i64)>(
        "SELECT priority, COUNT(*) as count
         FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY priority
         ORDER BY count DESC"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(priority, count)| PriorityCount {
        priority,
        count,
        percentage: (count as f64 / total_tasks.max(1) as f64) * 100.0,
    })
    .collect();

    // Daily completion trend
    let daily_completions = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT 
            (created_at::date)::text as date,
            SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as completed_count,
            COUNT(*) as created_count
         FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY created_at::date
         ORDER BY created_at::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(date, completed_count, created_count)| DailyCompletion {
        date,
        completed_count,
        created_count,
    })
    .collect();

    Ok(Json(TaskCompletionAnalytics {
        total_tasks,
        completed_tasks,
        completion_rate,
        average_completion_time_hours,
        completion_by_status,
        completion_by_priority,
        daily_completions,
    }))
}

/// Get client engagement analytics
/// GET /api/analytics/client-engagement?start_date=2024-01-01&end_date=2024-12-31
pub async fn get_client_engagement_analytics(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
    Extension(ctx): Extension<AuthContext>,
) -> Result<Json<ClientEngagementAnalytics>, AppError> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    // Total clients
    let total_clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Active/inactive clients
    let active_clients: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM clients WHERE status = 'active'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let inactive_clients = total_clients - active_clients;

    let engagement_rate = if total_clients > 0 {
        (active_clients as f64 / total_clients as f64) * 100.0
    } else {
        0.0
    };

    // Average tasks per client
    let average_tasks_per_client: f64 = sqlx::query_scalar(
        "SELECT CAST(COUNT(*) AS REAL) / NULLIF(COUNT(DISTINCT client_id), 0)
         FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0.0);

    // Clients with recent activity
    let clients_with_recent_activity: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT client_id) FROM tasks
         WHERE (created_at::date) BETWEEN $1::date AND $2::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // Top clients by task count
    let top_clients_by_tasks = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT 
            c.id, c.name,
            COUNT(t.id) as task_count,
            SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) as completed_tasks
         FROM clients c
         LEFT JOIN tasks t ON c.id = t.client_id
         WHERE (t.created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY c.id
         ORDER BY task_count DESC
         LIMIT 10"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(client_id, client_name, task_count, completed_tasks)| ClientTaskCount {
        client_id,
        client_name,
        task_count,
        completed_tasks,
    })
    .collect();

    // New clients trend
    let new_clients_trend = sqlx::query_as::<_, (String, i64)>(
        "SELECT (created_at::date)::text as date, COUNT(*) as new_clients
         FROM clients
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY created_at::date
         ORDER BY created_at::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(date, new_clients)| NewClientsTrend { date, new_clients })
    .collect();

    Ok(Json(ClientEngagementAnalytics {
        total_clients,
        active_clients,
        inactive_clients,
        engagement_rate,
        average_tasks_per_client,
        clients_with_recent_activity,
        top_clients_by_tasks,
        new_clients_trend,
    }))
}

/// Get storage usage analytics
/// GET /api/analytics/storage-usage?start_date=2024-01-01&end_date=2024-12-31
pub async fn get_storage_analytics(
    State(state): State<AppState>,
    Query(params): Query<DateRangeParams>,
    Extension(ctx): Extension<AuthContext>,
) -> Result<Json<StorageAnalytics>, AppError> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    // Total files
    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    // Total storage
    let total_size_bytes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(file_size), 0) FROM files"
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let total_size_mb = total_size_bytes as f64 / 1_048_576.0;
    let total_size_gb = total_size_mb / 1024.0;

    let average_file_size_bytes = if total_files > 0 {
        total_size_bytes as f64 / total_files as f64
    } else {
        0.0
    };

    // Files by type
    let files_by_type = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT 
            COALESCE(file_type, 'unknown') as file_type,
            COUNT(*) as count,
            COALESCE(SUM(file_size), 0) as total_size
         FROM files
         GROUP BY file_type
         ORDER BY count DESC
         LIMIT 15"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(file_type, count, total_size_bytes)| FileTypeCount {
        file_type,
        count,
        total_size_bytes,
        percentage: (count as f64 / total_files.max(1) as f64) * 100.0,
    })
    .collect();

    // Storage by user (top 10)
    let storage_by_user = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT 
            f.uploaded_by::text as user_id,
            u.full_name as user_name,
            COUNT(f.id) as file_count,
            COALESCE(SUM(f.file_size), 0) as total_size
         FROM files f
         JOIN users u ON f.uploaded_by = u.id
         GROUP BY f.uploaded_by, u.full_name
         ORDER BY total_size DESC
         LIMIT 10"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(user_id, user_name, file_count, total_size_bytes)| UserStorageInfo {
        user_id,
        user_name,
        file_count,
        total_size_mb: total_size_bytes as f64 / 1_048_576.0,
    })
    .collect();

    // Daily upload trend
    let daily_upload_trend = sqlx::query_as::<_, (String, i64, i64)>(
        "SELECT 
            (created_at::date)::text as date,
            COUNT(*) as file_count,
            COALESCE(SUM(file_size), 0) as total_size
         FROM files
         WHERE (created_at::date) BETWEEN $1::date AND $2::date
         GROUP BY created_at::date
         ORDER BY created_at::date"
    )
    .bind(&params.start_date)
    .bind(&params.end_date)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(date, file_count, total_size_bytes)| DailyUpload {
        date,
        file_count,
        total_size_mb: total_size_bytes as f64 / 1_048_576.0,
    })
    .collect();

    Ok(Json(StorageAnalytics {
        total_files,
        total_size_bytes,
        total_size_mb,
        total_size_gb,
        average_file_size_bytes,
        files_by_type,
        storage_by_user,
        daily_upload_trend,
    }))
}
