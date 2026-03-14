use axum::{extract::State, Extension, Json};
use axum::extract::{Multipart, Path, Query};
use sqlx::SqlitePool;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{UpdateUserRequest, User};
use crate::utils::password;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePreferencesRequest {
    pub theme: Option<String>,
    pub notifications_enabled: Option<bool>,
    pub email_notifications: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct UserProfileResponse {
    #[serde(flatten)]
    pub user: User,
    pub stats: UserStats,
}

#[derive(Debug, Serialize)]
pub struct UserStats {
    pub tasks_created: i64,
    pub tasks_completed: i64,
    pub clients_assigned: i64,
    pub files_uploaded: i64,
}

pub async fn get_current_user(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<User>> {
    let pool = state.pool();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

/// Get user profile with statistics
pub async fn get_user_profile(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
) -> AppResult<Json<UserProfileResponse>> {
    let pool = state.pool();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Get user statistics
    let tasks_created: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE created_by = ?1"
    )
    .bind(&user_id)
    .fetch_one(pool)
    .await?;

    let tasks_completed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE assigned_to = ?1 AND status = 'completed'"
    )
    .bind(&user_id)
    .fetch_one(pool)
    .await?;

    let clients_assigned: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM clients WHERE assigned_to = ?1"
    )
    .bind(&user_id)
    .fetch_one(pool)
    .await?;

    let files_uploaded: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM files WHERE uploaded_by = ?1"
    )
    .bind(&user_id)
    .fetch_one(pool)
    .await?;

    Ok(Json(UserProfileResponse {
        user,
        stats: UserStats {
            tasks_created,
            tasks_completed,
            clients_assigned,
            files_uploaded,
        },
    }))
}

pub async fn get_user(
    Extension(_user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> AppResult<Json<User>> {
    let pool = state.pool();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn update_user(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> AppResult<Json<User>> {
    let pool = state.pool();

    // Only allow users to update their own profile (or admin can update any)
    if user_id != id {
        return Err(AppError::Forbidden("Cannot update other users".to_string()));
    }

    // Update user fields
    sqlx::query(
        r#"
        UPDATE users
        SET full_name = COALESCE(?1, full_name),
            avatar_url = COALESCE(?2, avatar_url),
            updated_at = datetime('now')
        WHERE id = ?3
        "#,
    )
    .bind(&payload.full_name)
    .bind(&payload.avatar_url)
    .bind(&id)
    .execute(pool)
    .await?;

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

/// Change user password
pub async fn change_password(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();

    // Validate new password
    if payload.new_password.len() < 6 {
        return Err(AppError::ValidationError(
            "Password must be at least 6 characters".to_string(),
        ));
    }

    // Get current user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(&user_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Verify current password
    if !password::verify(&payload.current_password, &user.password_hash)? {
        return Err(AppError::Unauthorized("Current password is incorrect".to_string()));
    }

    // Hash new password
    let new_hash = password::hash(&payload.new_password)?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(&new_hash)
        .bind(&user_id)
        .execute(pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Password changed successfully"
    })))
}

/// Upload user avatar
pub async fn upload_avatar(
    Extension(user_id): Extension<String>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let pool = state.pool();
    use tokio::fs;
    use tokio::io::AsyncWriteExt;

    // Ensure avatars directory exists
    let avatar_dir = "uploads/avatars";
    fs::create_dir_all(avatar_dir).await.map_err(|e| {
        tracing::error!("Failed to create avatar directory: {}", e);
        AppError::InternalServerError(format!("Failed to create avatar directory: {}", e))
    })?;

    let mut avatar_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    // Process multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read multipart field: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "avatar" {
            file_name = field.file_name().map(|s| s.to_string());
            avatar_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Failed to read avatar data: {}", e)))?
                    .to_vec(),
            );
        }
    }

    let original_name = file_name.ok_or_else(|| AppError::BadRequest("No avatar file provided".to_string()))?;
    let avatar_bytes = avatar_data.ok_or_else(|| AppError::BadRequest("No avatar data provided".to_string()))?;

    // Validate file type (images only)
    let file_type = mime_guess::from_path(&original_name)
        .first_or_octet_stream()
        .to_string();

    if !file_type.starts_with("image/") {
        return Err(AppError::ValidationError("Only image files are allowed".to_string()));
    }

    // Validate file size (max 2MB for avatars)
    if avatar_bytes.len() > 2 * 1024 * 1024 {
        return Err(AppError::ValidationError("Avatar size must be less than 2MB".to_string()));
    }

    // Generate unique filename
    let extension = std::path::Path::new(&original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let stored_name = format!("{}_{}.{}", user_id, Uuid::new_v4().simple(), extension);
    let avatar_path = format!("{}/{}", avatar_dir, stored_name);

    // Write avatar to disk
    let mut file = fs::File::create(&avatar_path).await.map_err(|e| {
        tracing::error!("Failed to create avatar file: {}", e);
        AppError::InternalServerError(format!("Failed to create avatar file: {}", e))
    })?;

    file.write_all(&avatar_bytes).await.map_err(|e| {
        tracing::error!("Failed to write avatar data: {}", e);
        AppError::InternalServerError(format!("Failed to write avatar: {}", e))
    })?;

    // Delete old avatar if exists
    let old_avatar: Option<String> = sqlx::query_scalar("SELECT avatar_url FROM users WHERE id = ?1")
        .bind(&user_id)
        .fetch_optional(pool)
        .await?;

    if let Some(old_url) = old_avatar {
        if !old_url.is_empty() && old_url.starts_with("/uploads/avatars/") {
            let _ = fs::remove_file(&old_url[1..]).await; // Remove leading /
        }
    }

    // Update user avatar URL
    let avatar_url = format!("/{}", avatar_path);
    sqlx::query("UPDATE users SET avatar_url = ?1, updated_at = datetime('now') WHERE id = ?2")
        .bind(&avatar_url)
        .bind(&user_id)
        .execute(pool)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Avatar uploaded successfully",
        "avatar_url": avatar_url
    })))
}
