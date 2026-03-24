use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::authz::{load_effective_permissions, AuthContext};
use crate::models::User;
use crate::utils::jwt;

/// JWT auth: loads `User`, `Uuid`, `String` (user id), and `AuthContext` (DB RBAC permissions).
pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let pool = state.pool();

    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => h[7..].trim(),
        _ => {
            tracing::warn!("Missing or invalid Authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id_str = jwt::verify_token(token).map_err(|e| {
        tracing::warn!("Invalid JWT: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let user_id = Uuid::parse_str(user_id_str.trim()).map_err(|_| {
        tracing::warn!("JWT sub is not a valid UUID");
        StatusCode::UNAUTHORIZED
    })?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1 AND is_active = 1")
        .bind(&user_id_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching user: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!("User not found or inactive: {}", user_id_str);
            StatusCode::UNAUTHORIZED
        })?;

    let perms = load_effective_permissions(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load RBAC permissions: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let ctx = AuthContext::from_arc(user_id, perms);

    req.extensions_mut().insert(user_id);
    req.extensions_mut().insert(user.id.clone());
    req.extensions_mut().insert(user);
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
