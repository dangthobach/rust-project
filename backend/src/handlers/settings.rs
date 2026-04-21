use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::authz::permissions as perm;
use crate::authz::{invalidate_settings_cache, AuthContext};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SystemSettingDto {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSystemSettingPayload {
    pub value: String,
    pub description: Option<String>,
}

fn require_settings_manage(ctx: &AuthContext) -> Result<(), AppError> {
    if ctx.superuser() || ctx.has(perm::ROLE_MANAGE) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "system.superuser or role.manage permission required".to_string(),
        ))
    }
}

/// List all system settings (admin UI).
/// GET /api/admin/settings
pub async fn list_settings(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<SystemSettingDto>>> {
    require_settings_manage(&ctx)?;

    let rows = sqlx::query_as::<_, SystemSettingDto>(
        r#"
        SELECT key, value, description, updated_at
        FROM system_settings
        ORDER BY key ASC
        "#,
    )
    .fetch_all(state.pool())
    .await?;

    Ok(Json(rows))
}

/// Update a single system setting key (admin UI).
/// PATCH /api/admin/settings/:key
pub async fn patch_setting(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(key): Path<String>,
    Json(payload): Json<UpdateSystemSettingPayload>,
) -> AppResult<(StatusCode, Json<SystemSettingDto>)> {
    require_settings_manage(&ctx)?;

    let k = key.trim().to_string();
    if k.is_empty() {
        return Err(AppError::ValidationError("key must not be empty".to_string()));
    }

    if k.len() > 200 {
        return Err(AppError::ValidationError(
            "key is too long (max 200 chars)".to_string(),
        ));
    }

    let row = sqlx::query_as::<_, SystemSettingDto>(
        r#"
        INSERT INTO system_settings (key, value, description, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (key) DO UPDATE
        SET value = EXCLUDED.value,
            description = EXCLUDED.description,
            updated_at = NOW()
        RETURNING key, value, description, updated_at
        "#,
    )
    .bind(&k)
    .bind(payload.value)
    .bind(payload.description)
    .fetch_one(state.pool())
    .await?;

    invalidate_settings_cache();
    Ok((StatusCode::OK, Json(row)))
}

