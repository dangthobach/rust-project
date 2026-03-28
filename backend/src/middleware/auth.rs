use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::authz::permissions;
use crate::authz::{load_accessible_branch_ids, load_effective_permissions, AuthContext};
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

    // No-JWT dev mode:
    // - If Authorization header is missing -> inject a superuser admin principal
    // - If Authorization header is present but invalid -> still reject
    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => h[7..].trim().to_string(),
        None => {
            tracing::warn!("Authorization header missing; allow anonymous dev principal");
            let user_id = Uuid::nil();
            let perms: BTreeSet<String> = BTreeSet::from([permissions::SYSTEM_SUPERUSER.to_string()]);
            let branches: BTreeSet<String> = BTreeSet::new();
            let now = Utc::now();
            let user = User {
                id: user_id,
                email: "anonymous@local".to_string(),
                password_hash: "".to_string(),
                full_name: "Anonymous".to_string(),
                role: "admin".to_string(),
                avatar_url: None,
                is_active: true,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
            };
            let ctx = AuthContext::new(user_id, perms, branches);
            req.extensions_mut().insert(user_id);
            req.extensions_mut().insert(user.id.to_string());
            req.extensions_mut().insert(user);
            req.extensions_mut().insert(ctx);
            return Ok(next.run(req).await);
        }
        _ => {
            tracing::warn!("Invalid Authorization header format");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id_str = jwt::verify_token(&token).map_err(|e| {
        tracing::warn!("Invalid JWT: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    let user_id = Uuid::parse_str(user_id_str.trim()).map_err(|_| {
        tracing::warn!("JWT sub is not a valid UUID");
        StatusCode::UNAUTHORIZED
    })?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND is_active = TRUE")
        .bind(user_id)
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

    let branches = load_accessible_branch_ids(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to load branch scope: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let ctx = AuthContext::from_arcs(user_id, perms, branches);

    req.extensions_mut().insert(user_id);
    req.extensions_mut().insert(user.id.to_string());
    req.extensions_mut().insert(user);
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
