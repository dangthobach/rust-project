use async_trait::async_trait;
use sqlx::PgPool;
use std::error::Error;
use uuid::Uuid;

use crate::core::events::{DomainEvent, EventEnvelope};

/// Event Store — persistent domain events (PostgreSQL implementation).
#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    async fn append<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        expected_version: i64,
        events: Vec<EventEnvelope<E>>,
    ) -> Result<(), Self::Error>;

    async fn load<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error>;

    async fn load_all<E: DomainEvent>(
        &self,
        from_position: i64,
        limit: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error>;

    async fn get_version(&self, aggregate_id: Uuid) -> Result<i64, Self::Error>;

    async fn get_global_position(&self) -> Result<i64, Self::Error>;
}

#[derive(Clone)]
pub struct PostgresEventStore {
    pool: PgPool,
}

impl PostgresEventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: Uuid,
    aggregate_id: Uuid,
    aggregate_type: String,
    event_type: String,
    event_data: String,
    version: i64,
    occurred_at: chrono::DateTime<chrono::Utc>,
    metadata: String,
}

#[async_trait]
impl EventStore for PostgresEventStore {
    type Error = sqlx::Error;

    async fn append<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        expected_version: i64,
        events: Vec<EventEnvelope<E>>,
    ) -> Result<(), Self::Error> {
        let mut tx = self.pool.begin().await?;

        let current_version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM event_store WHERE aggregate_id = $1",
        )
        .bind(aggregate_id)
        .fetch_optional(&mut *tx)
        .await?;

        let current_version = current_version.unwrap_or(0);
        if current_version != expected_version {
            return Err(sqlx::Error::RowNotFound);
        }

        let mut version = expected_version;
        for event in events {
            version += 1;
            sqlx::query(
                r#"
                INSERT INTO event_store (
                    id, aggregate_id, aggregate_type, event_type,
                    event_data, version, occurred_at, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            )
            .bind(event.id)
            .bind(aggregate_id)
            .bind(aggregate_type)
            .bind(&event.event_type)
            .bind(serde_json::to_string(&event.event_data).map_err(|e| {
                sqlx::Error::ColumnNotFound(format!("Failed to serialize: {}", e))
            })?)
            .bind(version)
            .bind(event.occurred_at)
            .bind(serde_json::to_string(&event.metadata).unwrap_or_else(|_| "{}".to_string()))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn load<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT id, aggregate_id, aggregate_type, event_type,
                   event_data, version, occurred_at, metadata
            FROM event_store
            WHERE aggregate_id = $1 AND version > $2
            ORDER BY version ASC
        "#,
        )
        .bind(aggregate_id)
        .bind(from_version)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let event_data: E = serde_json::from_str(&row.event_data)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Failed to deserialize event: {}", e)))?;

            let metadata: serde_json::Value = serde_json::from_str(&row.metadata)
                .unwrap_or_else(|_| serde_json::json!({}));

            events.push(EventEnvelope {
                id: row.id,
                aggregate_id: row.aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_data,
                version: row.version,
                occurred_at: row.occurred_at,
                metadata,
            });
        }

        Ok(events)
    }

    async fn load_all<E: DomainEvent>(
        &self,
        from_position: i64,
        limit: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT id, aggregate_id, aggregate_type, event_type,
                   event_data, version, occurred_at, metadata
            FROM event_store
            WHERE seq > $1
            ORDER BY seq ASC
            LIMIT $2
        "#,
        )
        .bind(from_position)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let event_data: E = serde_json::from_str(&row.event_data).map_err(|e| {
                sqlx::Error::ColumnNotFound(format!("Failed to deserialize: {}", e))
            })?;

            let metadata: serde_json::Value = serde_json::from_str(&row.metadata)
                .unwrap_or_else(|_| serde_json::json!({}));

            events.push(EventEnvelope {
                id: row.id,
                aggregate_id: row.aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_data,
                version: row.version,
                occurred_at: row.occurred_at,
                metadata,
            });
        }

        Ok(events)
    }

    async fn get_version(&self, aggregate_id: Uuid) -> Result<i64, Self::Error> {
        let version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM event_store WHERE aggregate_id = $1",
        )
        .bind(aggregate_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(version.unwrap_or(0))
    }

    async fn get_global_position(&self) -> Result<i64, Self::Error> {
        let position: Option<i64> = sqlx::query_scalar("SELECT COALESCE(MAX(seq), 0) FROM event_store")
            .fetch_optional(&self.pool)
            .await?;

        Ok(position.unwrap_or(0))
    }
}
