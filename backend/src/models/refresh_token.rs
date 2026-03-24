use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
    pub revoked_at: Option<String>,
}

impl RefreshToken {
    /// Create a new refresh token for a user (30 days expiry)
    pub async fn create(pool: &SqlitePool, user_id: &str) -> AppResult<Self> {
        let id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let expires_at = (Utc::now() + Duration::days(30)).to_rfc3339();

        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at)
            VALUES (?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(&token)
        .bind(&expires_at)
        .fetch_one(pool)
        .await?;

        Ok(refresh_token)
    }

    /// Find a valid (non-revoked, non-expired) refresh token
    pub async fn find_valid(pool: &SqlitePool, token: &str) -> AppResult<Self> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens
            WHERE token = ?
            AND revoked_at IS NULL
            AND datetime(expires_at) > datetime('now')
            "#,
        )
        .bind(token)
        .fetch_one(pool)
        .await?;

        Ok(refresh_token)
    }

    /// Revoke this refresh token
    pub async fn revoke(&self, pool: &SqlitePool) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(&self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Revoke all refresh tokens for a user (logout all devices)
    pub async fn revoke_all_for_user(pool: &SqlitePool, user_id: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = datetime('now')
            WHERE user_id = ? AND revoked_at IS NULL
            "#,
        )
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Clean up expired tokens (run periodically)
    pub async fn cleanup_expired(pool: &SqlitePool) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE datetime(expires_at) < datetime('now', '-7 days')
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
