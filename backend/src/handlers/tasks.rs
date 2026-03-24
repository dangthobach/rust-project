use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{CreateTaskRequest, Task, TaskQuery, UpdateTaskRequest};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

pub async fn list_tasks(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<TaskQuery>,
) -> AppResult<Json<PaginatedResponse<Task>>> {
    let pool = state.pool();

    pagination.validate()?;

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    let mut where_sql = String::from("WHERE 1=1");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = query.status {
        bind_values.push(status);
        where_sql.push_str(&format!(" AND status = ?{}", bind_values.len()));
    }

    if let Some(priority) = query.priority {
        bind_values.push(priority);
        where_sql.push_str(&format!(" AND priority = ?{}", bind_values.len()));
    }

    if let Some(assigned_to) = query.assigned_to {
        bind_values.push(assigned_to.to_string());
        where_sql.push_str(&format!(" AND assigned_to = ?{}", bind_values.len()));
    }

    if let Some(client_id) = query.client_id {
        bind_values.push(client_id.to_string());
        where_sql.push_str(&format!(" AND client_id = ?{}", bind_values.len()));
    }

    let count_sql = format!("SELECT COUNT(*) FROM tasks {}", where_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &bind_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let data_sql = format!(
        "SELECT * FROM tasks {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
        where_sql
    );
    let mut data_query = sqlx::query_as::<_, Task>(&data_sql);
    for value in bind_values {
        data_query = data_query.bind(value);
    }
    data_query = data_query.bind(limit).bind(offset);

    let tasks = data_query.fetch_all(pool).await?;

    Ok(Json(PaginatedResponse::new(tasks, page, limit, total)))
}

pub async fn search_tasks(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<TaskQuery>,
) -> AppResult<Json<PaginatedResponse<Task>>> {
    let pool = state.pool();

    pagination.validate()?;

    let search_term =
        query
            .search
            .ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;
    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM tasks t
        INNER JOIN tasks_fts fts ON t.id = fts.id
        WHERE tasks_fts MATCH ?1
        "#,
    )
    .bind(&search_term)
    .fetch_one(pool)
    .await?;

    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT t.* FROM tasks t
        INNER JOIN tasks_fts fts ON t.id = fts.id
        WHERE tasks_fts MATCH ?1
        ORDER BY rank, t.created_at DESC
        LIMIT ?2 OFFSET ?3
        "#,
    )
    .bind(&search_term)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(tasks, page, limit, total)))
}

pub async fn create_task(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> AppResult<Json<Task>> {
    let pool = state.pool();

    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let task_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, title, description, status, priority, assigned_to, client_id, due_date, created_by)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
    )
    .bind(task_id.to_string())
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.status.unwrap_or_else(|| "todo".to_string()))
    .bind(payload.priority.unwrap_or_else(|| "medium".to_string()))
    .bind(payload.assigned_to.map(|id| id.to_string()))
    .bind(payload.client_id.map(|id| id.to_string()))
    .bind(payload.due_date.map(|d| d.to_rfc3339()))
    .bind(user_id.to_string())
    .execute(pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(task_id.to_string())
        .fetch_one(pool)
        .await?;

    Ok(Json(task))
}

pub async fn get_task(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Task>> {
    let pool = state.pool();

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}

pub async fn update_task(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> AppResult<Json<Task>> {
    let pool = state.pool();

    let status_str = payload.status.as_ref().map(|s| s.as_str());
    sqlx::query(
        r#"
        UPDATE tasks
        SET title = COALESCE(?1, title),
            description = COALESCE(?2, description),
            status = COALESCE(?3, status),
            priority = COALESCE(?4, priority),
            assigned_to = COALESCE(?5, assigned_to),
            client_id = COALESCE(?6, client_id),
            due_date = COALESCE(?7, due_date),
            completed_at = CASE WHEN ?3 = 'done' THEN datetime('now') ELSE completed_at END
        WHERE id = ?8
        "#,
    )
    .bind(payload.title.as_ref())
    .bind(payload.description.as_ref())
    .bind(status_str)
    .bind(payload.priority.as_ref())
    .bind(payload.assigned_to.map(|id| id.to_string()).as_ref())
    .bind(payload.client_id.map(|id| id.to_string()).as_ref())
    .bind(payload.due_date.map(|d| d.to_rfc3339()).as_ref())
    .bind(id.to_string())
    .execute(pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}

pub async fn delete_task(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    let result = sqlx::query("DELETE FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({"message": "Task deleted successfully"}),
    ))
}
