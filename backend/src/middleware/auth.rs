use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use sqlx::SqlitePool;

use crate::config::Config;
use crate::utils::jwt;

pub async fn auth(
    State((pool, _config)): State<(SqlitePool, Config)>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Verify token
    let user_id = jwt::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Verify user exists and is active
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

    // Add user_id to request extensions
    req.extensions_mut().insert(user_id);

    Ok(next.run(req).await)
}
