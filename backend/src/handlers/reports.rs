use axum::{
    extract::{Extension, Json, Path, Query, State},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::ReportExportJob;
use crate::utils::pagination::{PaginatedResponse, Pagination, PaginationParams};

#[derive(Debug, Deserialize)]
pub struct StartReportExportRequest {
    /// clients | tasks | users | dashboard
    pub report_type: String,
    pub format: Option<String>, // csv | json
    /// Inclusive start (YYYY-MM-DD), filters source rows by `created_at`
    pub start_date: Option<chrono::NaiveDate>,
    /// Inclusive end (YYYY-MM-DD), filters source rows by `created_at`
    pub end_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct StartReportExportResponse {
    pub job_id: String,
}

#[derive(Debug, Serialize)]
pub struct ReportExportStatusResponse {
    pub job_id: String,
    pub status: String,
    pub download_url: Option<String>,
    pub expires_in: Option<i64>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReportExportListItem {
    pub job_id: String,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub download_url: Option<String>,
}

fn normalize_format(format: Option<String>) -> AppResult<String> {
    let f = format.unwrap_or_else(|| "csv".to_string());
    match f.to_ascii_lowercase().as_str() {
        "csv" | "json" => Ok(f.to_ascii_lowercase()),
        other => Err(AppError::ValidationError(format!(
            "Invalid report format: {}",
            other
        ))),
    }
}

fn normalize_report_type(report_type: &str) -> AppResult<String> {
    let rt = report_type.to_ascii_lowercase();
    match rt.as_str() {
        "clients" | "tasks" | "users" | "dashboard" => Ok(rt),
        other => Err(AppError::ValidationError(format!(
            "Invalid report_type: {}",
            other
        ))),
    }
}

fn report_type_to_routing_key(report_type: &str) -> &'static str {
    match report_type {
        "clients" => "report.export.clients",
        "tasks" => "report.export.tasks",
        "users" => "report.export.users",
        // Worker falls back to dashboard generation for the "other" routing keys.
        "dashboard" => "report.export.dashboard",
        _ => "report.export.dashboard",
    }
}

pub async fn start_report_export(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<StartReportExportRequest>,
) -> AppResult<impl IntoResponse> {
    let report_type = normalize_report_type(&payload.report_type)?;
    let format = normalize_format(payload.format)?;

    let job_id = Uuid::new_v4().to_string();
    let uid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::ValidationError("Invalid user id".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO report_export_jobs (id, user_id, report_type, format, status, start_date, end_date)
        VALUES ($1, $2, $3, $4, 'queued', $5, $6)
        "#,
    )
    .bind(&job_id)
    .bind(&uid)
    .bind(&report_type)
    .bind(&format)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .execute(state.pool())
    .await?;

    let routing_key = report_type_to_routing_key(&report_type);
    let job_payload = serde_json::json!({
        "job": routing_key,
        "job_id": job_id,
        "requested_by": user_id,
        "format": format,
    });

    state
        .rabbitmq_publisher
        .publish("crm.jobs", routing_key, &job_payload.to_string())
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to enqueue report job: {e}")))?;

    Ok(axum::Json(StartReportExportResponse { job_id }))
}

pub async fn list_report_exports(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<ReportExportListItem>>> {
    pagination.validate()?;

    let uid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::ValidationError("Invalid user id".to_string()))?;

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();
    let p = Pagination::new(page, limit);

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*)::bigint FROM report_export_jobs WHERE user_id = $1"#,
    )
    .bind(&uid)
    .fetch_one(state.pool())
    .await?;

    let rows: Vec<ReportExportJob> = sqlx::query_as(
        r#"
        SELECT * FROM report_export_jobs
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&uid)
    .bind(limit)
    .bind(offset)
    .fetch_all(state.pool())
    .await?;

    let mut items = Vec::with_capacity(rows.len());
    for job in rows {
        let mut download_url: Option<String> = None;
        if job.status == "ready" {
            if let Some(object_uri) = job.object_uri.as_deref() {
                let signed = state
                    .object_storage()
                    .presign_get_url(object_uri, 3600)
                    .await
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                download_url = signed;
            }
        }
        items.push(ReportExportListItem {
            job_id: job.id.to_string(),
            report_type: job.report_type,
            format: job.format,
            status: job.status,
            created_at: job.created_at,
            start_date: job.start_date,
            end_date: job.end_date,
            download_url,
        });
    }

    Ok(Json(PaginatedResponse::new(items, total, p)))
}

pub async fn get_report_export_status(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> AppResult<Json<ReportExportStatusResponse>> {
    let job = sqlx::query_as::<_, ReportExportJob>(
        r#"
        SELECT * FROM report_export_jobs WHERE id = $1
        "#,
    )
    .bind(&job_id)
    .fetch_optional(state.pool())
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Report export job not found: {job_id}")))?;

    // For dev/no-auth mode, we skip ownership checks.
    // If you want strict behavior later, enforce: job.user_id == user_id || ctx.superuser.
    let mut download_url: Option<String> = None;
    let mut expires_in: Option<i64> = None;

    if job.status == "ready" {
        if let Some(object_uri) = job.object_uri.as_deref() {
            let signed = state
                .object_storage()
                .presign_get_url(object_uri, 3600)
                .await
                .map_err(|e| AppError::InternalServerError(e.to_string()))?;
            download_url = signed.or_else(|| None);
            expires_in = Some(3600);
        }
    }

    Ok(axum::Json(ReportExportStatusResponse {
        job_id: job.id.to_string(),
        status: job.status,
        download_url,
        expires_in,
        error_message: job.error_message,
    }))
}

