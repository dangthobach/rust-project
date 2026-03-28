use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use sqlx::{QueryBuilder, Postgres};
use uuid::Uuid;
use validator::Validate;

use crate::app_state::AppState;
use crate::authz::data_scope::DataScope;
use crate::authz::AuthContext;
use crate::error::{AppError, AppResult};
use crate::models::{CreateTaskRequest, Task, TaskQuery, UpdateTaskRequest};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

pub async fn list_tasks(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<TaskQuery>,
) -> AppResult<Json<PaginatedResponse<Task>>> {
    let pool = state.pool();

    pagination.validate()?;

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();
    let scope = DataScope::from_auth_context(&ctx);

    let mut count_qb = QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM tasks WHERE 1=1");
    if let Some(status) = &query.status {
        count_qb.push(" AND status = ").push_bind(status.clone());
    }
    if let Some(priority) = &query.priority {
        count_qb.push(" AND priority = ").push_bind(priority.clone());
    }
    if let Some(assigned_to) = &query.assigned_to {
        count_qb.push(" AND assigned_to = ").push_bind(*assigned_to);
    }
    if let Some(client_id) = &query.client_id {
        count_qb.push(" AND client_id = ").push_bind(*client_id);
    }
    crate::authz::data_scope::push_task_scope_filter(
        &mut count_qb,
        &scope,
        &actor_id,
    );
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM tasks WHERE 1=1");
    if let Some(status) = &query.status {
        qb.push(" AND status = ").push_bind(status.clone());
    }
    if let Some(priority) = &query.priority {
        qb.push(" AND priority = ").push_bind(priority.clone());
    }
    if let Some(assigned_to) = &query.assigned_to {
        qb.push(" AND assigned_to = ").push_bind(*assigned_to);
    }
    if let Some(client_id) = &query.client_id {
        qb.push(" AND client_id = ").push_bind(*client_id);
    }
    crate::authz::data_scope::push_task_scope_filter(&mut qb, &scope, &actor_id);
    qb.push(" ORDER BY created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let tasks = qb.build_query_as::<Task>().fetch_all(pool).await?;

    Ok(Json(PaginatedResponse::new(tasks, page, limit, total)))
}

pub async fn search_tasks(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
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
    let scope = DataScope::from_auth_context(&ctx);

    let mut count_qb = QueryBuilder::<Postgres>::new(
        "SELECT COUNT(*) FROM tasks t WHERE t.search_vector @@ plainto_tsquery('simple', ",
    );
    count_qb.push_bind(&search_term);
    count_qb.push(")");
    crate::authz::data_scope::push_task_scope_filter_aliased(
        &mut count_qb,
        &scope,
        &actor_id,
        Some("t"),
    );
    let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;

    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT t.* FROM tasks t WHERE t.search_vector @@ plainto_tsquery('simple', ",
    );
    qb.push_bind(&search_term);
    qb.push(")");
    crate::authz::data_scope::push_task_scope_filter_aliased(
        &mut qb,
        &scope,
        &actor_id,
        Some("t"),
    );
    qb.push(" ORDER BY ts_rank_cd(t.search_vector, plainto_tsquery('simple', ")
        .push_bind(&search_term)
        .push(")) DESC NULLS LAST, t.created_at DESC LIMIT ")
        .push_bind(limit)
        .push(" OFFSET ")
        .push_bind(offset);

    let tasks = qb.build_query_as::<Task>().fetch_all(pool).await?;

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
    let branch_id = if let Some(cid) = &payload.client_id {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT branch_id::text FROM clients WHERE id = $1",
        )
        .bind(*cid)
        .fetch_optional(pool)
        .await?
        .flatten()
    } else {
        None
    }
    .unwrap_or_else(|| crate::authz::data_scope::ROOT_BRANCH_ID.to_string());

    sqlx::query(
        r#"
        INSERT INTO tasks (id, title, description, status, priority, assigned_to, client_id, due_date, created_by, branch_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(task_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.status.unwrap_or_else(|| "todo".to_string()))
    .bind(payload.priority.unwrap_or_else(|| "medium".to_string()))
    .bind(payload.assigned_to)
    .bind(payload.client_id)
    .bind(payload.due_date)
    .bind(user_id)
    .bind(
        uuid::Uuid::parse_str(&branch_id).unwrap_or_else(|_| {
            uuid::Uuid::parse_str(crate::authz::data_scope::ROOT_BRANCH_ID).expect("root branch uuid")
        }),
    )
    .execute(pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(task_id)
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

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
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
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            status = COALESCE($3, status),
            priority = COALESCE($4, priority),
            assigned_to = COALESCE($5, assigned_to),
            client_id = COALESCE($6, client_id),
            due_date = COALESCE($7, due_date),
            completed_at = CASE WHEN $3::text = 'done' THEN NOW() ELSE completed_at END
        WHERE id = $8
        "#,
    )
    .bind(payload.title.as_ref())
    .bind(payload.description.as_ref())
    .bind(status_str)
    .bind(payload.priority.as_ref())
    .bind(payload.assigned_to.as_ref())
    .bind(payload.client_id.as_ref())
    .bind(payload.due_date.as_ref())
    .bind(id)
    .execute(pool)
    .await?;

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
        .bind(id)
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

    let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Task not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({"message": "Task deleted successfully"}),
    ))
}
