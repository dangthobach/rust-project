use sqlx::SqlitePool;
use uuid::Uuid;

use crate::error::AppError;

pub async fn append_aggregate_history(
    pool: &SqlitePool,
    aggregate_type: &str,
    aggregate_id: &str,
    action: &str,
    old_status: Option<&str>,
    new_status: Option<&str>,
    actor_id: Option<&str>,
    comment: Option<&str>,
    metadata: Option<serde_json::Value>,
) -> Result<(), AppError> {
    sqlx::query(
        r#"
        INSERT INTO aggregate_history (
            id, aggregate_type, aggregate_id, action, old_status, new_status, actor_id, comment, metadata
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(aggregate_type)
    .bind(aggregate_id)
    .bind(action)
    .bind(old_status)
    .bind(new_status)
    .bind(actor_id)
    .bind(comment)
    .bind(metadata.map(|m| m.to_string()))
    .execute(pool)
    .await?;
    Ok(())
}
