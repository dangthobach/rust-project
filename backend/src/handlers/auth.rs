use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use validator::Validate;

use crate::app_state::AppState;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, LoginRequest, RefreshToken, User};
use crate::utils::jwt;
use crate::utils::password;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: User,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let pool = state.pool();

    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    tracing::info!(email = %payload.email, "User registration attempt");

    // Check if user already exists
    let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?1")
        .bind(&payload.email)
        .fetch_optional(pool)
        .await?;

    if existing_user.is_some() {
        tracing::warn!(email = %payload.email, "Email already registered");
        return Err(AppError::BadRequest(
            "Email already registered".to_string(),
        ));
    }

    // Hash password
    let password_hash = password::hash(&payload.password)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Create user
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
    .execute(pool)
    .await?;

    // Fetch created user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;

    // Generate tokens
    let access_token = jwt::generate_token(&user.id)?;
    let refresh_token = RefreshToken::create(pool, &user.id).await?;

    tracing::info!(user_id = %user.id, role = %user.role, "User registered successfully");

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            refresh_token: refresh_token.token,
            token_type: "Bearer".to_string(),
            expires_in: 86400, // 24 hours
            user,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let pool = state.pool();

    // Validate input
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    tracing::info!(email = %payload.email, "Login attempt");

    // Find user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?1")
        .bind(&payload.email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            tracing::warn!(email = %payload.email, "User not found");
            AppError::Unauthorized("Invalid credentials".to_string())
        })?;

    // Verify password
    let is_valid = password::verify(&payload.password, &user.password_hash)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if !is_valid {
        tracing::warn!(user_id = %user.id, "Invalid password");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    // Check if user is active
    if !user.is_active {
        tracing::warn!(user_id = %user.id, "Inactive account login attempt");
        return Err(AppError::Unauthorized("Account is inactive".to_string()));
    }

    // Generate tokens
    let access_token = jwt::generate_token(&user.id)?;
    let refresh_token = RefreshToken::create(pool, &user.id).await?;

    tracing::info!(user_id = %user.id, role = %user.role, "Login successful");

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: refresh_token.token,
        token_type: "Bearer".to_string(),
        expires_in: 86400, // 24 hours
        user,
    }))
}

/// Refresh access token using refresh token
pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<AuthResponse>> {
    let pool = state.pool();

    tracing::debug!("Token refresh attempt");

    // Find and validate refresh token
    let old_refresh_token = RefreshToken::find_valid(pool, &payload.refresh_token).await?;

    // Revoke old refresh token
    old_refresh_token.revoke(pool).await?;

    // Fetch user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
        .bind(&old_refresh_token.user_id)
        .fetch_one(pool)
        .await?;

    // Generate new tokens
    let access_token = jwt::generate_token(&user.id)?;
    let new_refresh_token = RefreshToken::create(pool, &user.id).await?;

    tracing::info!(user_id = %user.id, "Token refreshed successfully");

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token.token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        user,
    }))
}

/// Logout (revoke refresh token)
pub async fn logout(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    tracing::info!(user_id = %user_id, "Logout attempt");

    // Revoke the specified refresh token
    if let Ok(refresh_token) = RefreshToken::find_valid(pool, &payload.refresh_token).await {
        refresh_token.revoke(pool).await?;
        tracing::info!(user_id = %user_id, "Logout successful");
    }

    Ok(Json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}

/// Logout from all devices (revoke all refresh tokens)
pub async fn logout_all(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    tracing::info!(user_id = %user_id, "Logout all devices attempt");

    RefreshToken::revoke_all_for_user(pool, &user_id).await?;

    tracing::info!(user_id = %user_id, "All devices logged out");

    Ok(Json(serde_json::json!({
        "message": "Logged out from all devices"
    })))
}
