use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::{Postgres, QueryBuilder};

use crate::app_state::AppState;
use crate::authz::{permissions as perm, AuthContext};
use crate::error::{AppError, AppResult};
use crate::models::{Client, Task, User};

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    pub format: Option<String>, // csv, json
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

fn parse_date(s: &str, field: &str) -> Result<DateTime<Utc>, AppError> {
    s.parse::<DateTime<Utc>>()
        .map_err(|_| AppError::ValidationError(format!("{field} must be RFC3339 (e.g. 2024-01-01T00:00:00Z)")))
}

/// Export clients to CSV/JSON — requires client.write permission
pub async fn export_clients(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
    ctx.require(perm::CLIENT_WRITE)?;
    let pool = state.pool();

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM clients WHERE 1=1");
    if let Some(ref status) = params.status {
        qb.push(" AND status = ");
        qb.push_bind(status);
    }
    if let Some(ref start_date) = params.start_date {
        qb.push(" AND created_at >= ");
        qb.push_bind(parse_date(start_date, "start_date")?);
    }
    if let Some(ref end_date) = params.end_date {
        qb.push(" AND created_at <= ");
        qb.push_bind(parse_date(end_date, "end_date")?);
    }
    qb.push(" ORDER BY created_at DESC");

    let clients = qb.build_query_as::<Client>().fetch_all(pool).await?;

    let format = params.format.as_deref().unwrap_or("csv");
    let export_job = serde_json::json!({
        "job": "report.export.clients",
        "requested_by": ctx.user_id.to_string(),
        "format": format
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "report.export.clients", &export_job.to_string())
        .await;

    match format {
        "csv" => {
            let mut csv = String::from("ID,Name,Email,Phone,Company,Status,Created At,Updated At\n");
            
            for client in clients {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\n",
                    client.id,
                    escape_csv_field(&client.name),
                    escape_csv_field(&client.email.unwrap_or_default()),
                    escape_csv_field(&client.phone.unwrap_or_default()),
                    escape_csv_field(&client.company.unwrap_or_default()),
                    client.status,
                    client.created_at,
                    client.updated_at
                ));
            }

            Ok((
                [(
                    header::CONTENT_TYPE,
                    "text/csv; charset=utf-8",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"clients_export.csv\"",
                )],
                csv,
            ).into_response())
        }
        "json" => {
            Ok((
                [(
                    header::CONTENT_TYPE,
                    "application/json",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"clients_export.json\"",
                )],
                serde_json::to_string_pretty(&clients)?,
            ).into_response())
        }
        _ => Err(AppError::ValidationError("Invalid format. Use 'csv' or 'json'".to_string())),
    }
}

/// Export tasks to CSV/JSON.
/// Users with user.manage see all tasks; others see only their own (assigned or created).
pub async fn export_tasks(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
    let pool = state.pool();
    let admin_view = ctx.superuser() || ctx.has(perm::USER_MANAGE);

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM tasks WHERE 1=1");
    if !admin_view {
        qb.push(" AND (assigned_to = ");
        qb.push_bind(ctx.user_id);
        qb.push(" OR created_by = ");
        qb.push_bind(ctx.user_id);
        qb.push(")");
    }
    if let Some(ref status) = params.status {
        qb.push(" AND status = ");
        qb.push_bind(status);
    }
    if let Some(ref start_date) = params.start_date {
        qb.push(" AND created_at >= ");
        qb.push_bind(parse_date(start_date, "start_date")?);
    }
    if let Some(ref end_date) = params.end_date {
        qb.push(" AND created_at <= ");
        qb.push_bind(parse_date(end_date, "end_date")?);
    }
    qb.push(" ORDER BY created_at DESC");

    let tasks = qb.build_query_as::<Task>().fetch_all(pool).await?;

    let format = params.format.as_deref().unwrap_or("csv");
    let export_job = serde_json::json!({
        "job": "report.export.tasks",
        "requested_by": ctx.user_id.to_string(),
        "format": format
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "report.export.tasks", &export_job.to_string())
        .await;

    match format {
        "csv" => {
            let mut csv = String::from("ID,Title,Description,Status,Priority,Due Date,Client ID,Assigned To,Created By,Created At,Updated At\n");
            
            for task in tasks {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{},{}\n",
                    task.id,
                    escape_csv_field(&task.title),
                    escape_csv_field(&task.description.unwrap_or_default()),
                    task.status,
                    task.priority,
                    task.due_date.map(|d| d.to_string()).unwrap_or_default(),
                    task.client_id.unwrap_or_default(),
                    task.assigned_to.unwrap_or_default(),
                    task.created_by.map(|u| u.to_string()).unwrap_or_default(),
                    task.created_at,
                    task.updated_at
                ));
            }

            Ok((
                [(
                    header::CONTENT_TYPE,
                    "text/csv; charset=utf-8",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"tasks_export.csv\"",
                )],
                csv,
            ).into_response())
        }
        "json" => {
            Ok((
                [(
                    header::CONTENT_TYPE,
                    "application/json",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"tasks_export.json\"",
                )],
                serde_json::to_string_pretty(&tasks)?,
            ).into_response())
        }
        _ => Err(AppError::ValidationError("Invalid format. Use 'csv' or 'json'".to_string())),
    }
}

/// Export users to CSV/JSON (Admin only)
pub async fn export_users(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    let mut query = String::from("SELECT * FROM users WHERE 1=1");
    
    if let Some(status) = &params.status {
        query.push_str(&format!(" AND role = '{}'", status)); // Use role as status
    }
    
    query.push_str(" ORDER BY created_at DESC");

    let users = sqlx::query_as::<_, User>(&query)
        .fetch_all(pool)
        .await?;

    let format = params.format.as_deref().unwrap_or("csv");
    let export_job = serde_json::json!({
        "job": "report.export.users",
        "requested_by": ctx.user_id.to_string(),
        "format": format
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "report.export.users", &export_job.to_string())
        .await;

    match format {
        "csv" => {
            let mut csv = String::from("ID,Email,Full Name,Avatar URL,Created At,Updated At\n");

            for user in users {
                csv.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    user.id,
                    escape_csv_field(&user.email),
                    escape_csv_field(&user.full_name),
                    escape_csv_field(&user.avatar_url.unwrap_or_default()),
                    user.created_at,
                    user.updated_at
                ));
            }

            Ok((
                [(
                    header::CONTENT_TYPE,
                    "text/csv; charset=utf-8",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"users_export.csv\"",
                )],
                csv,
            ).into_response())
        }
        "json" => {
            Ok((
                [(
                    header::CONTENT_TYPE,
                    "application/json",
                ),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"users_export.json\"",
                )],
                serde_json::to_string_pretty(&users)?,
            ).into_response())
        }
        _ => Err(AppError::ValidationError("Invalid format. Use 'csv' or 'json'".to_string())),
    }
}

/// Export dashboard stats report
pub async fn export_dashboard_report(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    ctx.require(perm::USER_MANAGE)?;
    let pool = state.pool();

    // Gather all stats
    let total_clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients")
        .fetch_one(pool)
        .await?;

    let active_clients: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clients WHERE status = 'active'")
        .fetch_one(pool)
        .await?;

    let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(pool)
        .await?;

    let completed_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'completed'")
        .fetch_one(pool)
        .await?;

    let pending_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE status = 'pending'")
        .fetch_one(pool)
        .await?;

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let total_files: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM files")
        .fetch_one(pool)
        .await?;

    let total_size: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(file_size), 0) FROM files")
        .fetch_one(pool)
        .await?;

    let report = serde_json::json!({
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "summary": {
            "clients": {
                "total": total_clients,
                "active": active_clients,
                "inactive": total_clients - active_clients
            },
            "tasks": {
                "total": total_tasks,
                "completed": completed_tasks,
                "pending": pending_tasks,
                "completion_rate": if total_tasks > 0 {
                    (completed_tasks as f64 / total_tasks as f64 * 100.0).round()
                } else {
                    0.0
                }
            },
            "users": {
                "total": total_users
            },
            "storage": {
                "total_files": total_files,
                "total_size_mb": (total_size as f64 / 1_048_576.0).round()
            }
        }
    });

    let export_job = serde_json::json!({
        "job": "report.export.dashboard",
        "requested_by": ctx.user_id.to_string()
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "report.export.dashboard", &export_job.to_string())
        .await;

    Ok((
        [(
            header::CONTENT_TYPE,
            "application/json",
        ),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"dashboard_report.json\"",
        )],
        serde_json::to_string_pretty(&report)?,
    ).into_response())
}

// Helper function to escape CSV fields
fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
