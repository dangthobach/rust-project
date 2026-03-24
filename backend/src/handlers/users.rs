use axum::{extract::State, Extension, Json};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::authz::AuthContext;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{UpdateUserRequest, User};

pub async fn get_current_user(
    Extension(ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
) -> AppResult<Json<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(ctx.user_id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn get_user(
    Extension(_ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> AppResult<Json<User>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn update_user(
    Extension(ctx): Extension<AuthContext>,
    State((pool, _)): State<(SqlitePool, Config)>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<User>> {
    // Only allow users to update their own profile (or add admin check later)
    if ctx.user_id != id {
        return Err(AppError::Unauthorized("Cannot update other users".to_string()));
    }

    // SQLite doesn't support RETURNING, update then select
    sqlx::query(
        r#"
        UPDATE users
        SET full_name = COALESCE(?1, full_name),
            avatar_url = COALESCE(?2, avatar_url)
        WHERE id = ?3
        "#,
    )
    .bind(payload.full_name.as_ref())
    .bind(payload.avatar_url.as_ref())
    .bind(id.to_string())
    .execute(&pool)
    .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(id.to_string())
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}
