use async_trait::async_trait;
use sqlx::SqlitePool;
use std::error::Error;

use crate::core::domain::AggregateRoot;

/// Snapshot Store - lưu snapshot của aggregates để tăng tốc rebuild
#[async_trait]
pub trait SnapshotStore<T: AggregateRoot>: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// Save snapshot của aggregate
    async fn save_snapshot(&self, aggregate: &T) -> Result<(), Self::Error>;

    /// Load snapshot của aggregate
    async fn load_snapshot(&self, id: &T::Id) -> Result<Option<T>, Self::Error>;

    /// Delete snapshot (khi rebuild từ events)
    async fn delete_snapshot(&self, id: &T::Id) -> Result<(), Self::Error>;
}

/// SQLite implementation của Snapshot Store
pub struct PostgresSnapshotStore<T: AggregateRoot> {
    pool: SqlitePool,
    aggregate_type: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AggregateRoot> PostgresSnapshotStore<T> {
    pub fn new(pool: SqlitePool, aggregate_type: String) -> Self {
        Self {
            pool,
            aggregate_type,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> SnapshotStore<T>
    for PostgresSnapshotStore<T>
where
    T: AggregateRoot + serde::Serialize + for<'de> serde::Deserialize<'de>,
    T::Id: ToString + std::str::FromStr,
{
    type Error = sqlx::Error;

    async fn save_snapshot(&self, aggregate: &T) -> Result<(), Self::Error> {
        let aggregate_data = serde_json::to_string(aggregate).map_err(|e| {
            sqlx::Error::ColumnNotFound(format!("Failed to serialize aggregate: {}", e))
        })?;

        sqlx::query(
            r#"
            INSERT INTO snapshots (aggregate_id, aggregate_type, aggregate_data, version)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT (aggregate_id) 
            DO UPDATE SET 
                aggregate_data = ?3,
                version = ?4,
                created_at = CURRENT_TIMESTAMP
        "#,
        )
        .bind(aggregate.id().to_string())
        .bind(&self.aggregate_type)
        .bind(aggregate_data)
        .bind(aggregate.version())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_snapshot(&self, id: &T::Id) -> Result<Option<T>, Self::Error> {
        let row: Option<(String, i64)> = sqlx::query_as(
            r#"
            SELECT aggregate_data, version
            FROM snapshots
            WHERE aggregate_id = ?1 AND aggregate_type = ?2
        "#,
        )
        .bind(id.to_string())
        .bind(&self.aggregate_type)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((data, _version)) = row {
            let aggregate: T = serde_json::from_str(&data).map_err(|e| {
                sqlx::Error::ColumnNotFound(format!("Failed to deserialize aggregate: {}", e))
            })?;
            Ok(Some(aggregate))
        } else {
            Ok(None)
        }
    }

    async fn delete_snapshot(&self, id: &T::Id) -> Result<(), Self::Error> {
        sqlx::query(
            "DELETE FROM snapshots WHERE aggregate_id = ?1 AND aggregate_type = ?2",
        )
        .bind(id.to_string())
        .bind(&self.aggregate_type)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

