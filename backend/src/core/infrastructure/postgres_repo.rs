use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::domain::{AggregateRoot, Repository};
use crate::core::events::event_store::EventStore;

/// Generic SQLite Repository implementation
/// Sử dụng Event Sourcing để persist aggregates
#[derive(Clone)]
pub struct PostgresRepository<T: AggregateRoot> {
    pool: SqlitePool,
    event_store: crate::core::events::PostgresEventStore,
    aggregate_type: String,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: AggregateRoot> PostgresRepository<T> {
    pub fn new(
        pool: SqlitePool,
        event_store: crate::core::events::PostgresEventStore,
        aggregate_type: String,
    ) -> Self {
        Self {
            pool,
            event_store,
            aggregate_type,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T> Repository<T> for PostgresRepository<T>
where
    T: AggregateRoot<Id = Uuid> + Rebuildable,
    T::Event: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    type Error = sqlx::Error;

    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, Self::Error> {
        // Try to load from snapshot first
        // If not found, rebuild from events
        self.rebuild_from_events(id).await
    }

    async fn save(&self, aggregate: &mut T) -> Result<(), Self::Error> {
        let events: Vec<_> = aggregate
            .uncommitted_events()
            .iter()
            .map(|e| {
                crate::core::events::EventEnvelope::new(
                    *aggregate.id(),
                    self.aggregate_type.clone(),
                    e.clone(),
                    serde_json::json!({}),
                )
            })
            .collect();

        if events.is_empty() {
            return Ok(()); // No changes
        }

        let expected_version = aggregate.version() - events.len() as i64;

        // Append events to event store
        self.event_store
            .append(
                *aggregate.id(),
                &self.aggregate_type,
                expected_version,
                events,
            )
            .await?;

        // Mark events as committed
        aggregate.mark_events_as_committed();

        Ok(())
    }

    async fn delete(&self, _aggregate: &mut T) -> Result<(), Self::Error> {
        // Soft delete through event
        // Will be handled by domain logic
        Ok(())
    }

    async fn get_events(&self, id: &T::Id) -> Result<Vec<T::Event>, Self::Error> {
        let envelopes = self.event_store.load(*id, 0).await?;
        Ok(envelopes.into_iter().map(|e| e.event_data).collect())
    }

    async fn rebuild_from_events(&self, id: &T::Id) -> Result<Option<T>, Self::Error> {
        // Load events
        let events = self.get_events(id).await?;

        if events.is_empty() {
            return Ok(None);
        }

        // Rebuild aggregate from events
        // This will be handled by type-specific implementations
        // For File and Folder, use their rebuild_from_events methods
        rebuild_aggregate_from_events::<T>(&events)
    }
}

/// Helper trait for aggregates that can rebuild from events
pub trait Rebuildable: AggregateRoot {
    fn rebuild_from_events(events: &[Self::Event]) -> Option<Self>;
}

/// Helper function to rebuild aggregate from events
fn rebuild_aggregate_from_events<T: Rebuildable>(
    events: &[T::Event],
) -> Result<Option<T>, sqlx::Error> {
    Ok(T::rebuild_from_events(events))
}

