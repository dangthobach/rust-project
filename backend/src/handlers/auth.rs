use axum::{extract::State, http::StatusCode, Json};
use sqlx::SqlitePool;
use validator::Validate;

use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, LoginRequest, LoginResponse, User};
use crate::utils::jwt;
use crate::utils::password;

pub async fn register(
    State((pool, _)): State<(SqlitePool, Config)>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<LoginResponse>)> {
    // Validate input
    payload.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    // Hash password
    let password_hash = password::hash(&payload.password)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Create user (SQLite doesn't support RETURNING, use last_insert_rowid)
    let user_id = uuid::Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, role)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
    )
    .bind(user_id.to_string())
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(payload.role.unwrap_or_else(|| "user".to_string()))
    .execute(&pool)
    .await?;

    // Fetch created user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(user_id.to_string())
        .fetch_one(&pool)
        .await?;

    // Generate JWT token
    let token = jwt::generate_token(&user.id)?;

    Ok((
        StatusCode::CREATED,
        Json(LoginResponse { token, user }),
    ))
}

pub async fn login(
    State((pool, _)): State<(SqlitePool, Config)>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    // Validate input
    payload.validate().map_err(|e| AppError::ValidationError(e.to_string()))?;

    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?1")
        .bind(&payload.email)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    // Verify password
    let is_valid = password::verify(&payload.password, &user.password_hash)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if !is_valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Check if user is active
    if !user.is_active {
        return Err(AppError::Unauthorized("Account is inactive".to_string()));
    }

    // Generate JWT token
    let token = jwt::generate_token(&user.id)?;

    Ok(Json(LoginResponse { token, user }))
}
