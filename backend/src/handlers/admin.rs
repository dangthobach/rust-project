use axum::{extract::State, Extension, Json};
use axum::extract::{Path, Query};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder};

use crate::app_state::AppState;
use crate::authz::permissions as perm;
use crate::authz::AuthContext;
use crate::error::{AppError, AppResult};
use crate::models::User;
use crate::utils::pagination::{PaginatedResponse, Pagination, PaginationParams};
use crate::utils::password;

// ── Request / response DTOs ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub full_name: String,
    pub password: String,
    /// Must be the slug of an existing active role.
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAdminRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    /// If provided, must be the slug of an existing active role.
    pub role: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BulkUserAction {
    pub user_ids: Vec<String>,
    /// Supported: "delete", "activate", "deactivate", "change_role"
    pub action: String,
    /// Required when action = "change_role"; must be a valid role slug.
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BulkActionResponse {
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

// ── Shared helpers ────────────────────────────────────────────────────────────

/// Validate that `slug` is an existing active role in the DB.
/// Eliminates all hardcoded role lists — new roles work automatically.
async fn ensure_role_slug_valid(state: &AppState, slug: &str) -> AppResult<()> {
    let exists: Option<(i64,)> =
        sqlx::query_as("SELECT 1 FROM roles WHERE slug = $1 AND is_active = TRUE LIMIT 1")
            .bind(slug)
            .fetch_optional(state.pool())
            .await?;
    if exists.is_none() {
        return Err(AppError::ValidationError(format!(
            "Role '{}' does not exist or is inactive",
            slug
        )));
    }
    Ok(())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Search users (requires user.manage).
pub async fn search_users(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<PaginatedResponse<User>>> {
    ctx.require(perm::USER_MANAGE)?;

    pagination.validate()?;

    let search_term = query
        .get("search")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("search query param is required".to_string()))?;

    let pattern = format!("%{}%", search_term);

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email ILIKE $1 OR full_name ILIKE $1",
    )
    .bind(&pattern)
    .fetch_one(state.pool())
    .await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email ILIKE $1 OR full_name ILIKE $1
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(&pattern)
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    let p = Pagination::new(pagination.page, pagination.limit);
    Ok(Json(PaginatedResponse::new(users, total, p)))
}

/// Create a new user (admin panel).  Requires user.manage.
pub async fn create_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<User>> {
    ctx.require(perm::USER_MANAGE)?;

    // Validate role against DB — no hardcoded list.
    ensure_role_slug_valid(&state, &payload.role).await?;

    if !payload.email.contains('@') {
        return Err(AppError::ValidationError("Invalid email format".to_string()));
    }
    if payload.password.len() < 6 {
        return Err(AppError::ValidationError(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(state.pool())
        .await?;
    if exists > 0 {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    let password_hash = password::hash(&payload.password)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    let user_id = uuid::Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, full_name, role, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
    )
    .bind(user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .execute(state.pool())
    .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(state.pool())
        .await?;

    Ok(Json(user))
}

/// Update a user (admin panel).  Requires user.manage.
pub async fn update_user_admin(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserAdminRequest>,
) -> AppResult<Json<User>> {
    ctx.require(perm::USER_MANAGE)?;

    if let Some(ref role) = payload.role {
        ensure_role_slug_valid(&state, role).await?;
    }

    if let Some(ref status) = payload.status {
        if status != "active" && status != "inactive" {
            return Err(AppError::ValidationError(
                "status must be 'active' or 'inactive'".to_string(),
            ));
        }
    }

    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(&id)
        .fetch_one(state.pool())
        .await?;
    if exists == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    let mut qb = QueryBuilder::<Postgres>::new("UPDATE users SET updated_at = NOW()");
    if let Some(ref full_name) = payload.full_name {
        qb.push(", full_name = ").push_bind(full_name);
    }
    if let Some(ref email) = payload.email {
        qb.push(", email = ").push_bind(email);
    }
    if let Some(ref role) = payload.role {
        qb.push(", role = ").push_bind(role);
    }
    if let Some(ref status) = payload.status {
        qb.push(", status = ").push_bind(status);
    }
    qb.push(" WHERE id = ").push_bind(&id);
    qb.build().execute(state.pool()).await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(&id)
        .fetch_one(state.pool())
        .await?;

    Ok(Json(user))
}

/// Delete a user (admin panel).  Requires user.manage.
pub async fn delete_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require(perm::USER_MANAGE)?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(&id)
        .execute(state.pool())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(Json(serde_json::json!({"message": "User deleted successfully"})))
}

/// Bulk user actions.  Requires user.manage.
pub async fn bulk_user_actions(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<BulkUserAction>,
) -> AppResult<Json<BulkActionResponse>> {
    ctx.require(perm::USER_MANAGE)?;

    if payload.user_ids.is_empty() {
        return Err(AppError::ValidationError("user_ids must not be empty".to_string()));
    }

    // Pre-validate role for change_role action before touching any rows.
    if payload.action == "change_role" {
        match &payload.role {
            None => {
                return Err(AppError::ValidationError(
                    "role is required for action 'change_role'".to_string(),
                ));
            }
            Some(slug) => ensure_role_slug_valid(&state, slug).await?,
        }
    }

    let mut success = 0usize;
    let mut failed = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for user_id in &payload.user_ids {
        let result = match payload.action.as_str() {
            "delete" => {
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(user_id)
                    .execute(state.pool())
                    .await
            }
            "activate" => {
                sqlx::query(
                    "UPDATE users SET status = 'active', is_active = TRUE, updated_at = NOW()
                     WHERE id = $1",
                )
                .bind(user_id)
                .execute(state.pool())
                .await
            }
            "deactivate" => {
                sqlx::query(
                    "UPDATE users SET status = 'inactive', is_active = FALSE, updated_at = NOW()
                     WHERE id = $1",
                )
                .bind(user_id)
                .execute(state.pool())
                .await
            }
            "change_role" => {
                // Safety: validated above; unwrap is safe here.
                let role = payload.role.as_deref().unwrap();
                sqlx::query(
                    "UPDATE users SET role = $1, updated_at = NOW() WHERE id = $2",
                )
                .bind(role)
                .bind(user_id)
                .execute(state.pool())
                .await
            }
            _ => {
                failed += 1;
                errors.push(format!("User {user_id}: unsupported action '{}'", payload.action));
                continue;
            }
        };

        match result {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("User {user_id}: {e}"));
            }
        }
    }

    Ok(Json(BulkActionResponse { success, failed, errors }))
}

/// User statistics (requires user.manage).
pub async fn get_user_stats(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require(perm::USER_MANAGE)?;

    let pool = state.pool();

    let (total, active, inactive): (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*)                                       AS total,
            COUNT(*) FILTER (WHERE status = 'active')     AS active,
            COUNT(*) FILTER (WHERE status = 'inactive')   AS inactive
        FROM users
        "#,
    )
    .fetch_one(pool)
    .await?;

    // Role breakdown is dynamic — group by the legacy `role` column.
    let role_rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT role, COUNT(*) FROM users GROUP BY role ORDER BY role")
            .fetch_all(pool)
            .await?;

    let by_role: serde_json::Value = role_rows
        .into_iter()
        .map(|(r, c)| (r, serde_json::Value::from(c)))
        .collect::<serde_json::Map<_, _>>()
        .into();

    Ok(Json(serde_json::json!({
        "total":    total,
        "active":   active,
        "inactive": inactive,
        "by_role":  by_role,
    })))
}
