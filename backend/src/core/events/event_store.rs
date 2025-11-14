use async_trait::async_trait;
use sqlx::SqlitePool;
use std::error::Error;
use uuid::Uuid;

use crate::core::events::{DomainEvent, EventEnvelope};

/// Event Store - lưu trữ tất cả domain events
#[async_trait]
pub trait EventStore: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// Append events vào event store
    /// expected_version: version hiện tại của aggregate (optimistic locking)
    async fn append<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        aggregate_type: &str,
        expected_version: i64,
        events: Vec<EventEnvelope<E>>,
    ) -> Result<(), Self::Error>;

    /// Load events cho một aggregate
    /// from_version: load từ version nào (0 = từ đầu)
    async fn load<E: DomainEvent>(
        &self,
        aggregate_id: Uuid,
        from_version: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error>;

    /// Load all events (cho projections)
    /// from_position: global position trong event store
    async fn load_all<E: DomainEvent>(
        &self,
        from_position: i64,
        limit: i64,
    ) -> Result<Vec<EventEnvelope<E>>, Self::Error>;

    /// Get current version của aggregate
    async fn get_version(&self, aggregate_id: Uuid) -> Result<i64, Self::Error>;

    /// Get global position (cho projections)
    async fn get_global_position(&self) -> Result<i64, Self::Error>;
}

/// SQLite implementation của Event Store
#[derive(Clone)]
pub struct PostgresEventStore {
    pool: SqlitePool,
}

impl PostgresEventStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: String,  // SQLite stores UUID as TEXT
    aggregate_id: String,  // SQLite stores UUID as TEXT
    aggregate_type: String,
    event_type: String,
    event_data: String,  // JSON stored as TEXT in SQLite
    version: i64,
    occurred_at: String,  // ISO8601 string in SQLite
    metadata: String,  // JSON stored as TEXT in SQLite
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

        // Check current version (optimistic locking)
        let current_version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM event_store WHERE aggregate_id = ?1",
        )
        .bind(aggregate_id.to_string())
        .fetch_optional(&mut *tx)
        .await?;

        let current_version = current_version.unwrap_or(0);
        if current_version != expected_version {
            return Err(sqlx::Error::RowNotFound); // Version mismatch
        }

        // Insert events
        let mut version = expected_version;
        for event in events {
            version += 1;
            sqlx::query(
                r#"
                INSERT INTO event_store (
                    id, aggregate_id, aggregate_type, event_type,
                    event_data, version, occurred_at, metadata
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            )
            .bind(event.id.to_string())
            .bind(event.aggregate_id.to_string())
            .bind(aggregate_type)
            .bind(&event.event_type)
            .bind(serde_json::to_string(&event.event_data).map_err(|e| {
                sqlx::Error::ColumnNotFound(format!("Failed to serialize: {}", e))
            })?)
            .bind(version)
            .bind(event.occurred_at.to_rfc3339())
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
            WHERE aggregate_id = ?1 AND version > ?2
            ORDER BY version ASC
        "#,
        )
        .bind(aggregate_id)
        .bind(from_version)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            // Parse UUIDs from TEXT
            let id = Uuid::parse_str(&row.id)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid UUID id: {}", e)))?;
            let aggregate_id = Uuid::parse_str(&row.aggregate_id)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid UUID aggregate_id: {}", e)))?;
            
            // Parse event data from JSON string
            let event_data: E = serde_json::from_str(&row.event_data)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Failed to deserialize event: {}", e)))?;

            // Parse metadata from JSON string
            let metadata: serde_json::Value = serde_json::from_str(&row.metadata)
                .unwrap_or_else(|_| serde_json::json!({}));

            // Parse occurred_at from ISO8601 string
            let occurred_at = chrono::DateTime::parse_from_rfc3339(&row.occurred_at)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&chrono::Utc);

            events.push(EventEnvelope {
                id,
                aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_data,
                version: row.version,
                occurred_at,
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
            WHERE rowid > ?1
            ORDER BY rowid ASC
            LIMIT ?2
        "#,
        )
        .bind(from_position)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            // Parse UUIDs from TEXT
            let id = Uuid::parse_str(&row.id)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid UUID id: {}", e)))?;
            let aggregate_id = Uuid::parse_str(&row.aggregate_id)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid UUID aggregate_id: {}", e)))?;
            
            // Parse event data from JSON string
            let event_data: E = serde_json::from_str(&row.event_data)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Failed to deserialize: {}", e)))?;

            // Parse metadata from JSON string
            let metadata: serde_json::Value = serde_json::from_str(&row.metadata)
                .unwrap_or_else(|_| serde_json::json!({}));

            // Parse occurred_at from ISO8601 string
            let occurred_at = chrono::DateTime::parse_from_rfc3339(&row.occurred_at)
                .map_err(|e| sqlx::Error::ColumnNotFound(format!("Invalid timestamp: {}", e)))?
                .with_timezone(&chrono::Utc);

            events.push(EventEnvelope {
                id,
                aggregate_id,
                aggregate_type: row.aggregate_type,
                event_type: row.event_type,
                event_data,
                version: row.version,
                occurred_at,
                metadata,
            });
        }

        Ok(events)
    }

    async fn get_version(&self, aggregate_id: Uuid) -> Result<i64, Self::Error> {
        let version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM event_store WHERE aggregate_id = ?1",
        )
        .bind(aggregate_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(version.unwrap_or(0))
    }

    async fn get_global_position(&self) -> Result<i64, Self::Error> {
        let position: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(rowid) FROM event_store",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.unwrap_or(0))
    }
}
