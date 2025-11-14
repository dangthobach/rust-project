use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

use crate::core::domain::AggregateRoot;

/// Repository trait cho Aggregate Root
/// Sử dụng Event Sourcing để persist aggregates
#[async_trait]
pub trait Repository<T: AggregateRoot>: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    /// Find aggregate by ID
    /// Load từ Event Store và rebuild aggregate từ events
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, Self::Error>;

    /// Save aggregate (insert or update)
    /// Persist uncommitted events vào Event Store
    async fn save(&self, aggregate: &mut T) -> Result<(), Self::Error>;

    /// Delete aggregate (soft delete thông qua event)
    async fn delete(&self, aggregate: &mut T) -> Result<(), Self::Error>;

    /// Get all events cho aggregate (for debugging/replay)
    async fn get_events(&self, id: &T::Id) -> Result<Vec<T::Event>, Self::Error>;

    /// Rebuild aggregate from events (event sourcing)
    async fn rebuild_from_events(&self, id: &T::Id) -> Result<Option<T>, Self::Error>;
}

/// Query Repository - cho Read Models
/// Không dùng Event Sourcing, query trực tiếp từ denormalized tables
#[async_trait]
pub trait QueryRepository<T>: Send + Sync {
    type Error: Error + Send + Sync + 'static;
    type Filter: Send + Sync;

    /// Find by ID từ read model
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<T>, Self::Error>;

    /// Find all với filter
    async fn find_all(&self, filter: &Self::Filter) -> Result<Vec<T>, Self::Error>;

    /// Count với filter
    async fn count(&self, filter: &Self::Filter) -> Result<i64, Self::Error>;

    /// Pagination support
    async fn find_paginated(
        &self,
        filter: &Self::Filter,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<T>, i64), Self::Error> {
        let offset = (page - 1) * page_size;
        // Default implementation - override nếu cần optimize
        let items = self.find_all(filter).await?;
        let total = items.len() as i64;
        let paginated = items
            .into_iter()
            .skip(offset as usize)
            .take(page_size as usize)
            .collect();
        Ok((paginated, total))
    }
}

