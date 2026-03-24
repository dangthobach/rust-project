use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::authz::{load_effective_permissions, AuthContext};
use crate::config::Config;
use crate::utils::jwt;

pub async fn auth(
    State((pool, _config)): State<(SqlitePool, Config)>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => h[7..].trim(),
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user_id_str = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user_id = Uuid::parse_str(user_id_str.trim()).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user_exists: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = ?1 AND is_active = 1",
    )
    .bind(user_id.to_string())
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_exists = user_exists.map(|c| c > 0).unwrap_or(false);
    if !user_exists {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let perms = load_effective_permissions(&pool, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ctx = AuthContext::from_arc(user_id, perms);
    req.extensions_mut().insert(ctx);

    Ok(next.run(req).await)
}
