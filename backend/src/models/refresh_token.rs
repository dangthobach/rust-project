use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl RefreshToken {
    pub async fn create(pool: &PgPool, user_id: &str) -> AppResult<Self> {
        let id = Uuid::new_v4();
        let token = Uuid::new_v4().to_string();
        let uid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::ValidationError("Invalid user id".to_string()))?;
        let expires_at = Utc::now() + Duration::days(30);

        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token, expires_at)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(uid)
        .bind(&token)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok(refresh_token)
    }

    pub async fn find_valid(pool: &PgPool, token: &str) -> AppResult<Self> {
        let refresh_token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT * FROM refresh_tokens
            WHERE token = $1
            AND revoked_at IS NULL
            AND expires_at > NOW()
            "#,
        )
        .bind(token)
        .fetch_one(pool)
        .await?;

        Ok(refresh_token)
    }

    pub async fn revoke(&self, pool: &PgPool) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn revoke_all_for_user(pool: &PgPool, user_id: &str) -> AppResult<()> {
        let uid = Uuid::parse_str(user_id)
            .map_err(|_| AppError::ValidationError("Invalid user id".to_string()))?;
        sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked_at = NOW()
            WHERE user_id = $1 AND revoked_at IS NULL
            "#,
        )
        .bind(uid)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn cleanup_expired(pool: &PgPool) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW() - INTERVAL '7 days'
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
