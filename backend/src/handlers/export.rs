use axum::{
    extract::{Query, State},
    http::header,
    response::IntoResponse,
    Extension,
};
use serde::Deserialize;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{Client, Task, User};

#[derive(Debug, Deserialize)]
pub struct ExportParams {
    pub format: Option<String>, // csv, json
    pub status: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

/// Export clients to CSV/JSON
pub async fn export_clients(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
    let pool = state.pool();

    // Build query with filters
    let mut query = String::from("SELECT * FROM clients WHERE 1=1");
    
    if let Some(status) = &params.status {
        query.push_str(&format!(" AND status = '{}'", status));
    }
    
    if let Some(start_date) = &params.start_date {
        query.push_str(&format!(" AND created_at >= '{}'", start_date));
    }
    
    if let Some(end_date) = &params.end_date {
        query.push_str(&format!(" AND created_at <= '{}'", end_date));
    }
    
    query.push_str(" ORDER BY created_at DESC");

    let clients = sqlx::query_as::<_, Client>(&query)
        .fetch_all(pool)
        .await?;

    let format = params.format.as_deref().unwrap_or("csv");
    let export_job = serde_json::json!({
        "job": "report.export.clients",
        "requested_by": user_id,
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

/// Export tasks to CSV/JSON
pub async fn export_tasks(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
    let pool = state.pool();

    let mut query = String::from("SELECT * FROM tasks WHERE 1=1");
    
    if let Some(status) = &params.status {
        query.push_str(&format!(" AND status = '{}'", status));
    }
    
    if let Some(start_date) = &params.start_date {
        query.push_str(&format!(" AND created_at >= '{}'", start_date));
    }
    
    if let Some(end_date) = &params.end_date {
        query.push_str(&format!(" AND created_at <= '{}'", end_date));
    }
    
    query.push_str(" ORDER BY created_at DESC");

    let tasks = sqlx::query_as::<_, Task>(&query)
        .fetch_all(pool)
        .await?;

    let format = params.format.as_deref().unwrap_or("csv");
    let export_job = serde_json::json!({
        "job": "report.export.tasks",
        "requested_by": user_id,
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
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Query(params): Query<ExportParams>,
) -> AppResult<impl IntoResponse> {
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
        "requested_by": user_id,
        "format": format
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "report.export.users", &export_job.to_string())
        .await;

    match format {
        "csv" => {
            let mut csv = String::from("ID,Email,Full Name,Role,Avatar URL,Created At,Updated At\n");
            
            for user in users {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    user.id,
                    escape_csv_field(&user.email),
                    escape_csv_field(&user.full_name),
                    user.role,
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
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
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
        "requested_by": user_id
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
