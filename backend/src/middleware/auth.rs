use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;

use crate::app_state::AppState;
use crate::config::Config;
use crate::models::User;
use crate::utils::jwt;

pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let pool = state.pool();

    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            tracing::warn!("Missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Verify token
    let user_id = jwt::verify_token(token).map_err(|e| {
        tracing::warn!("Invalid JWT token: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Fetch full user from database
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = 1")
        .bind(&user_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let user = user.ok_or_else(|| {
        tracing::warn!("User not found or inactive: {}", user_id);
        StatusCode::UNAUTHORIZED
    })?;

    // Add both user_id (for backward compatibility) and full User object to extensions
    req.extensions_mut().insert(user_id);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
