use axum::{extract::{Query, State}, Extension, Json};
use sqlx::SqlitePool;
use uuid::Uuid;
use validator::Validate;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{Client, ClientQuery, CreateClientRequest, UpdateClientRequest};

pub async fn list_clients(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Query(query): Query<ClientQuery>,
) -> AppResult<Json<Vec<Client>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    let offset = (page - 1) * limit;

    let mut sql = String::from("SELECT * FROM clients WHERE 1=1");
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(status) = query.status {
        bind_values.push(status);
        sql.push_str(&format!(" AND status = ?{}", bind_values.len()));
    }

    if let Some(assigned_to) = query.assigned_to {
        bind_values.push(assigned_to.to_string());
        sql.push_str(&format!(" AND assigned_to = ?{}", bind_values.len()));
    }

    if let Some(search) = query.search {
        let search_pattern = format!("%{}%", search);
        bind_values.push(search_pattern.clone());
        let pos = bind_values.len();
        sql.push_str(&format!(" AND (UPPER(name) LIKE UPPER(?{}) OR UPPER(company) LIKE UPPER(?{}))", pos, pos));
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT ?");
    sql.push_str(&(bind_values.len() + 1).to_string());
    sql.push_str(" OFFSET ?");
    sql.push_str(&(bind_values.len() + 2).to_string());

    let mut query = sqlx::query_as::<_, Client>(&sql);
    for value in bind_values {
        query = query.bind(value);
    }
    query = query.bind(limit).bind(offset);

    let clients = query.fetch_all(&pool).await?;

    Ok(Json(clients))
}

pub async fn create_client(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    Json(payload): Json<CreateClientRequest>,
) -> AppResult<Json<Client>> {
    payload.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    let client_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO clients (id, name, email, phone, company, position, address, status, assigned_to, notes)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#,
    )
    .bind(client_id.to_string())
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.company)
    .bind(&payload.position)
    .bind(&payload.address)
    .bind(payload.status.unwrap_or_else(|| "active".to_string()))
    .bind(payload.assigned_to.or(Some(user_id)).map(|id| id.to_string()))
    .bind(&payload.notes)
    .execute(&pool)
    .await?;

    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?1")
        .bind(client_id.to_string())
        .fetch_one(&pool)
        .await?;

    Ok(Json(client))
}

pub async fn get_client(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<Client>> {
    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

    Ok(Json(client))
}

pub async fn update_client(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateClientRequest>,
) -> AppResult<Json<Client>> {
    sqlx::query(
        r#"
        UPDATE clients
        SET name = COALESCE(?1, name),
            email = COALESCE(?2, email),
            phone = COALESCE(?3, phone),
            company = COALESCE(?4, company),
            position = COALESCE(?5, position),
            address = COALESCE(?6, address),
            status = COALESCE(?7, status),
            assigned_to = COALESCE(?8, assigned_to),
            notes = COALESCE(?9, notes)
        WHERE id = ?10
        "#,
    )
    .bind(payload.name.as_ref())
    .bind(payload.email.as_ref())
    .bind(payload.phone.as_ref())
    .bind(payload.company.as_ref())
    .bind(payload.position.as_ref())
    .bind(payload.address.as_ref())
    .bind(payload.status.as_ref())
    .bind(payload.assigned_to.map(|id| id.to_string()).as_ref())
    .bind(payload.notes.as_ref())
    .bind(id.to_string())
    .execute(&pool)
    .await?;

    let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

    Ok(Json(client))
}

pub async fn delete_client(
    Extension(_user_id): Extension<Uuid>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let result = sqlx::query("DELETE FROM clients WHERE id = ?1")
        .bind(id.to_string())
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Client not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "Client deleted successfully"})))
}
