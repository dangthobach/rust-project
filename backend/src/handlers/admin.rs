use axum::{extract::State, Extension, Json};
use axum::extract::{Path, Query};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::config::Config;
use crate::app_state::AppState;
use crate::error::{AppError, AppResult};
use crate::models::User;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};
use crate::utils::password;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub full_name: String,
    pub password: String,
    pub role: String, // admin, manager, user
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAdminRequest {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>, // active, inactive
}

#[derive(Debug, Deserialize)]
pub struct BulkUserAction {
    pub user_ids: Vec<String>,
    pub action: String, // delete, activate, deactivate, change_role
    pub role: Option<String>, // For change_role action
}

#[derive(Debug, Serialize)]
pub struct BulkActionResponse {
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

/// List all users (Admin only)
pub async fn list_users(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<User>>> {
    let pool = state.pool();

    pagination.validate()?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(
        users,
        pagination.page,
        pagination.limit,
        total,
    )))
}

/// Search users (Admin only)
pub async fn search_users(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
    Query(query): Query<serde_json::Value>,
) -> AppResult<Json<PaginatedResponse<User>>> {
    let pool = state.pool();

    pagination.validate()?;

    let search_term = query
        .get("search")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ValidationError("Search term required".to_string()))?;

    let search_pattern = format!("%{}%", search_term);

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE email LIKE ?1 OR full_name LIKE ?1"
    )
    .bind(&search_pattern)
    .fetch_one(pool)
    .await?;

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email LIKE ?1 OR full_name LIKE ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
    )
    .bind(&search_pattern)
    .bind(pagination.limit)
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;

    Ok(Json(PaginatedResponse::new(
        users,
        pagination.page,
        pagination.limit,
        total,
    )))
}

/// Create new user (Admin only)
pub async fn create_user(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> AppResult<Json<User>> {
    let pool = state.pool();

    // Validate role
    if !["admin", "manager", "user"].contains(&payload.role.as_str()) {
        return Err(AppError::ValidationError("Invalid role".to_string()));
    }

    // Validate email
    if !payload.email.contains('@') {
        return Err(AppError::ValidationError("Invalid email format".to_string()));
    }

    // Validate password
    if payload.password.len() < 6 {
        return Err(AppError::ValidationError(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    // Check if email already exists
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE email = ?1")
        .bind(&payload.email)
        .fetch_one(pool)
        .await?;

    if exists > 0 {
        return Err(AppError::ValidationError("Email already exists".to_string()));
    }

    // Hash password
    let password_hash = password::hash(&payload.password)?;

    // Generate user ID
    let user_id = uuid::Uuid::new_v4().to_string();

    // Insert user
    sqlx::query(
        r#"
        INSERT INTO users (id, email, password_hash, full_name, role, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))
        "#,
    )
    .bind(&user_id)
    .bind(&payload.email)
    .bind(&password_hash)
    .bind(&payload.full_name)
    .bind(&payload.role)
    .execute(pool)
    .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&user_id)
        .fetch_one(pool)
        .await?;

    Ok(Json(user))
}

/// Update user (Admin only)
pub async fn update_user_admin(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserAdminRequest>,
) -> AppResult<Json<User>> {
    let pool = state.pool();

    // Validate role if provided
    if let Some(ref role) = payload.role {
        if !["admin", "manager", "user"].contains(&role.as_str()) {
            return Err(AppError::ValidationError("Invalid role".to_string()));
        }
    }

    // Validate status if provided
    if let Some(ref status) = payload.status {
        if !["active", "inactive"].contains(&status.as_str()) {
            return Err(AppError::ValidationError("Invalid status".to_string()));
        }
    }

    // Check if user exists
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await?;

    if exists == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Update user fields
    let mut query = String::from("UPDATE users SET updated_at = datetime('now')");
    let mut bindings = Vec::new();

    if let Some(full_name) = &payload.full_name {
        query.push_str(", full_name = ?");
        bindings.push(full_name.clone());
    }

    if let Some(email) = &payload.email {
        query.push_str(", email = ?");
        bindings.push(email.clone());
    }

    if let Some(role) = &payload.role {
        query.push_str(", role = ?");
        bindings.push(role.clone());
    }

    if let Some(status) = &payload.status {
        query.push_str(", status = ?");
        bindings.push(status.clone());
    }

    query.push_str(" WHERE id = ?");
    bindings.push(id.clone());

    // Build dynamic query
    let mut sql_query = sqlx::query(&query);
    for binding in bindings {
        sql_query = sql_query.bind(binding);
    }

    sql_query.execute(pool).await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await?;

    Ok(Json(user))
}

/// Delete user (Admin only)
pub async fn delete_user(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    // Check if user exists
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?1")
        .bind(&id)
        .fetch_one(pool)
        .await?;

    if exists == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // Delete user
    sqlx::query("DELETE FROM users WHERE id = ?1")
        .bind(&id)
        .execute(pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "User deleted successfully"
    })))
}

/// Bulk user actions (Admin only)
pub async fn bulk_user_actions(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<BulkUserAction>,
) -> AppResult<Json<BulkActionResponse>> {
    let pool = state.pool();

    let mut success = 0;
    let mut failed = 0;
    let mut errors = Vec::new();

    for user_id in &payload.user_ids {
        let result = match payload.action.as_str() {
            "delete" => {
                sqlx::query("DELETE FROM users WHERE id = ?1")
                    .bind(user_id)
                    .execute(pool)
                    .await
            }
            "activate" => {
                sqlx::query("UPDATE users SET status = 'active', updated_at = datetime('now') WHERE id = ?1")
                    .bind(user_id)
                    .execute(pool)
                    .await
            }
            "deactivate" => {
                sqlx::query("UPDATE users SET status = 'inactive', updated_at = datetime('now') WHERE id = ?1")
                    .bind(user_id)
                    .execute(pool)
                    .await
            }
            "change_role" => {
                if let Some(ref role) = payload.role {
                    sqlx::query("UPDATE users SET role = ?1, updated_at = datetime('now') WHERE id = ?2")
                        .bind(role)
                        .bind(user_id)
                        .execute(pool)
                        .await
                } else {
                    failed += 1;
                    errors.push(format!("User {}: No role provided", user_id));
                    continue;
                }
            }
            _ => {
                failed += 1;
                errors.push(format!("User {}: Invalid action", user_id));
                continue;
            }
        };

        match result {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(format!("User {}: {}", user_id, e));
            }
        }
    }

    Ok(Json(BulkActionResponse {
        success,
        failed,
        errors,
    }))
}

/// Get user statistics (Admin only)
pub async fn get_user_stats(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    let active_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE status = 'active'")
        .fetch_one(pool)
        .await?;

    let inactive_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE status = 'inactive'")
        .fetch_one(pool)
        .await?;

    let admins: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'admin'")
        .fetch_one(pool)
        .await?;

    let managers: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'manager'")
        .fetch_one(pool)
        .await?;

    let regular_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'user'")
        .fetch_one(pool)
        .await?;

    Ok(Json(serde_json::json!({
        "total": total_users,
        "active": active_users,
        "inactive": inactive_users,
        "by_role": {
            "admin": admins,
            "manager": managers,
            "user": regular_users
        }
    })))
}
