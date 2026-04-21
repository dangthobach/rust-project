use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::app_state::AppState;
use crate::authz::load_system_settings;
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

    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    tracing::info!(email = %payload.email, "User registration attempt");

    // ── Guard: registration gate ──────────────────────────────────────────────
    let settings = load_system_settings(pool).await?;
    if !settings.registration_enabled {
        return Err(AppError::Forbidden(
            "Self-service registration is currently disabled".to_string(),
        ));
    }

    // ── Duplicate check ───────────────────────────────────────────────────────
    let existing: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM users WHERE email = $1 LIMIT 1")
            .bind(&payload.email)
            .fetch_optional(pool)
            .await?;
    if existing.is_some() {
        tracing::warn!(email = %payload.email, "Email already registered");
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    // ── Create user ───────────────────────────────────────────────────────────
    let password_hash = password::hash(&payload.password)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    let user_id = uuid::Uuid::new_v4();

    // `role` column is a legacy label; RBAC comes from user_roles.
    // Use the first slug from settings as the display label.
    let role_label = settings
        .default_role_slugs
        .first()
        .map(|s| s.as_str())
        .unwrap_or("user");

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, full_name, role) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(role_label)
    .execute(pool)
    .await?;

    // ── Assign all default roles (dynamic, from system_settings) ─────────────
    for slug in &settings.default_role_slugs {
        let assigned: Option<(i64,)> =
            sqlx::query_as("SELECT 1 FROM roles WHERE slug = $1 AND is_active = TRUE LIMIT 1")
                .bind(slug)
                .fetch_optional(pool)
                .await?;

        if assigned.is_none() {
            tracing::warn!(slug = %slug, "default_role_slugs contains unknown role slug; skipping");
            continue;
        }

        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id)
            SELECT $1, r.id FROM roles r WHERE r.slug = $2 AND r.is_active = TRUE LIMIT 1
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(slug)
        .execute(pool)
        .await?;
    }

    // ── Assign default branch (dynamic, from system_settings) ─────────────────
    let branch_uuid = uuid::Uuid::parse_str(&settings.default_branch_id)
        .map_err(|_| {
            AppError::InternalServerError(format!(
                "system_settings.default_branch_id is not a valid UUID: {}",
                settings.default_branch_id
            ))
        })?;

    let branch_exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM branches WHERE id = $1 AND is_active = TRUE LIMIT 1")
            .bind(branch_uuid)
            .fetch_optional(pool)
            .await?;

    if branch_exists.is_none() {
        tracing::warn!(
            branch_id = %settings.default_branch_id,
            "system_settings.default_branch_id references a non-existent or inactive branch"
        );
    } else {
        sqlx::query(
            "INSERT INTO user_branches (user_id, branch_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(branch_uuid)
        .execute(pool)
        .await?;
    }

    // ── Fetch created user & generate tokens ──────────────────────────────────
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let access_token = jwt::generate_token(&user.id.to_string())?;
    let refresh_token = RefreshToken::create(pool, &user.id.to_string()).await?;

    tracing::info!(user_id = %user.id, role = %user.role, "User registered successfully");

    let welcome_job = serde_json::json!({
        "job": "email.welcome",
        "user_id": user.id,
        "email": user.email
    });
    let _ = state
        .rabbitmq_publisher
        .publish("crm.jobs", "email.welcome", &welcome_job.to_string())
        .await;

    let domain_event = serde_json::json!({
        "event_type": "UserRegistered",
        "user_id": user.id,
        "email": user.email,
        "occurred_at": chrono::Utc::now().to_rfc3339()
    });
    let _ = state
        .kafka_publisher
        .publish("crm.domain.user", &user.id.to_string(), &domain_event.to_string())
        .await;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            access_token,
            refresh_token: refresh_token.token,
            token_type: "Bearer".to_string(),
            expires_in: 86400,
            user,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let pool = state.pool();

    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    tracing::info!(email = %payload.email, "Login attempt");

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| {
            tracing::warn!(email = %payload.email, "User not found");
            AppError::Unauthorized("Invalid credentials".to_string())
        })?;

    let is_valid = password::verify(&payload.password, &user.password_hash)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    if !is_valid {
        tracing::warn!(user_id = %user.id, "Invalid password");
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    if !user.is_active {
        tracing::warn!(user_id = %user.id, "Inactive account login attempt");
        return Err(AppError::Unauthorized("Account is inactive".to_string()));
    }

    let access_token = jwt::generate_token(&user.id.to_string())?;
    let refresh_token = RefreshToken::create(pool, &user.id.to_string()).await?;

    tracing::info!(user_id = %user.id, role = %user.role, "Login successful");

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: refresh_token.token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        user,
    }))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<AuthResponse>> {
    let pool = state.pool();

    tracing::debug!("Token refresh attempt");

    let old_refresh_token = RefreshToken::find_valid(pool, &payload.refresh_token).await?;
    old_refresh_token.revoke(pool).await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = TRUE")
        .bind(old_refresh_token.user_id)
        .fetch_one(pool)
        .await?;

    let access_token = jwt::generate_token(&user.id.to_string())?;
    let new_refresh_token = RefreshToken::create(pool, &user.id.to_string()).await?;

    tracing::info!(user_id = %user.id, "Token refreshed successfully");

    Ok(Json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token.token,
        token_type: "Bearer".to_string(),
        expires_in: 86400,
        user,
    }))
}

pub async fn logout(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    tracing::info!(user_id = %user_id, "Logout attempt");

    if let Ok(refresh_token) = RefreshToken::find_valid(pool, &payload.refresh_token).await {
        refresh_token.revoke(pool).await?;
        tracing::info!(user_id = %user_id, "Logout successful");
    }

    Ok(Json(serde_json::json!({"message": "Logged out successfully"})))
}

pub async fn logout_all(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    tracing::info!(user_id = %user_id, "Logout all devices attempt");

    RefreshToken::revoke_all_for_user(pool, &user_id).await?;

    tracing::info!(user_id = %user_id, "All devices logged out");

    Ok(Json(serde_json::json!({"message": "Logged out from all devices"})))
}
