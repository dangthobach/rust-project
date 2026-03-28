use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::{Client, ClientQuery, CreateClientRequest, UpdateClientRequest};
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

pub async fn list_clients(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<ClientQuery>,
) -> AppResult<Json<PaginatedResponse<Client>>> {
    let pool = state.pool();

    pagination.validate()?;

    let page = pagination.page;
    let limit = pagination.limit;
    let offset = pagination.offset();

    let mut where_sql = String::from("WHERE 1=1");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = query.status {
        bind_values.push(status);
        where_sql.push_str(&format!(" AND status = ${}", bind_values.len()));
    }

    if let Some(assigned_to) = query.assigned_to {
        bind_values.push(assigned_to.to_string());
        where_sql.push_str(&format!(" AND assigned_to = ${}", bind_values.len()));
    }

    if let Some(search) = query.search {
        let search_pattern = format!("%{}%", search);
        bind_values.push(search_pattern.clone());
        let pos = bind_values.len();
        where_sql.push_str(&format!(
            " AND (UPPER(name) LIKE UPPER(${0}) OR UPPER(company) LIKE UPPER(${0}) OR UPPER(email) LIKE UPPER(${0}))",
            pos
        ));
    }

    let count_sql = format!("SELECT COUNT(*) FROM clients {}", where_sql);
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
    for value in &bind_values {
        count_query = count_query.bind(value);
    }
    let total = count_query.fetch_one(pool).await?;

    let data_sql = format!(
        "SELECT * FROM clients {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_sql,
        bind_values.len() + 1,
        bind_values.len() + 2
    );
    let mut data_query = sqlx::query_as::<_, Client>(&data_sql);
    for value in bind_values {
        data_query = data_query.bind(value);
    }
    data_query = data_query.bind(limit).bind(offset);

    let clients = data_query.fetch_all(pool).await?;

    Ok(Json(PaginatedResponse::new(clients, page, limit, total)))
}

pub async fn search_clients(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<ClientQuery>,
) -> AppResult<Json<PaginatedResponse<Client>>> {
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
        SELECT COUNT(*) FROM clients c
        WHERE c.search_vector @@ plainto_tsquery('simple', $1)
        "#,
    )
    .bind(&search_term)
    .fetch_one(pool)
    .await?;

    let clients = sqlx::query_as::<_, Client>(
        r#"
        SELECT c.* FROM clients c
        WHERE c.search_vector @@ plainto_tsquery('simple', $1)
        ORDER BY ts_rank_cd(c.search_vector, plainto_tsquery('simple', $1)) DESC NULLS LAST,
                 c.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&search_term)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(clients, page, limit, total)))
}

pub async fn create_client(
    Extension(user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<CreateClientRequest>,
) -> AppResult<Json<Client>> {
    let pool = state.pool();

    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let client_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO clients (id, name, email, phone, company, position, address, status, assigned_to, notes)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(client_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.company)
    .bind(&payload.position)
    .bind(&payload.address)
    .bind(payload.status.unwrap_or_else(|| "active".to_string()))
    .bind(payload.assigned_to.or(Some(user_id)))
    .bind(&payload.notes)
    .execute(pool)
    .await?;

    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(client_id)
        .fetch_one(pool)
        .await?;

    Ok(Json(client))
}

pub async fn get_client(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Client>> {
    let pool = state.pool();

    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

    Ok(Json(client))
}

pub async fn update_client(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateClientRequest>,
) -> AppResult<Json<Client>> {
    let pool = state.pool();

    sqlx::query(
        r#"
        UPDATE clients
        SET name = COALESCE($1, name),
            email = COALESCE($2, email),
            phone = COALESCE($3, phone),
            company = COALESCE($4, company),
            position = COALESCE($5, position),
            address = COALESCE($6, address),
            status = COALESCE($7, status),
            assigned_to = COALESCE($8, assigned_to),
            notes = COALESCE($9, notes)
        WHERE id = $10
        "#,
    )
    .bind(payload.name.as_ref())
    .bind(payload.email.as_ref())
    .bind(payload.phone.as_ref())
    .bind(payload.company.as_ref())
    .bind(payload.position.as_ref())
    .bind(payload.address.as_ref())
    .bind(payload.status.as_ref())
    .bind(payload.assigned_to.as_ref())
    .bind(payload.notes.as_ref())
    .bind(id)
    .execute(pool)
    .await?;

    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

    Ok(Json(client))
}

pub async fn delete_client(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    let result = sqlx::query("DELETE FROM clients WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Client not found".to_string()));
    }

    Ok(Json(
        serde_json::json!({"message": "Client deleted successfully"}),
    ))
}
