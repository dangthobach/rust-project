use axum::{extract::{Query, State}, Extension, Json};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{CreateTaskRequest, Task, TaskQuery, UpdateTaskRequest};

pub async fn list_tasks(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Query(query): Query<TaskQuery>,
) -> AppResult<Json<Vec<Task>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut sql = String::from("SELECT * FROM tasks WHERE 1=1");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = query.status {
        bind_values.push(status);
        sql.push_str(&format!(" AND status = ?{}", bind_values.len()));
    }

    if let Some(priority) = query.priority {
        bind_values.push(priority);
        sql.push_str(&format!(" AND priority = ?{}", bind_values.len()));
    }

    if let Some(assigned_to) = query.assigned_to {
        bind_values.push(assigned_to.to_string());
        sql.push_str(&format!(" AND assigned_to = ?{}", bind_values.len()));
    }

    if let Some(client_id) = query.client_id {
        bind_values.push(client_id.to_string());
        sql.push_str(&format!(" AND client_id = ?{}", bind_values.len()));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ?");
    sql.push_str(&(bind_values.len() + 1).to_string());
    sql.push_str(" OFFSET ?");
    sql.push_str(&(bind_values.len() + 2).to_string());

    let mut query = sqlx::query_as::<_, Task>(&sql);
    for value in bind_values {
        query = query.bind(value);
    }
    query = query.bind(limit).bind(offset);

    let tasks = query.fetch_all(&pool).await?;

    Ok(Json(tasks))
}

pub async fn search_tasks(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Query(query): Query<TaskQuery>,
) -> AppResult<Json<Vec<Task>>> {
    let search_term = query.search.ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    // Use FTS5 full-text search with MATCH operator
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
    .fetch_all(&pool)
    .await?;

    Ok(Json(tasks))
}

pub async fn create_task(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Json(payload): Json<CreateTaskRequest>,
) -> AppResult<Json<Task>> {
    payload.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

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
    .execute(&pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(task_id.to_string())
        .fetch_one(&pool)
        .await?;

    Ok(Json(task))
}

pub async fn get_task(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<Task>> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}

pub async fn update_task(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> AppResult<Json<Task>> {
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
    .execute(&pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    Ok(Json(task))
}

pub async fn delete_task(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?1")
        .bind(id.to_string())
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "Task deleted successfully"})))
}
