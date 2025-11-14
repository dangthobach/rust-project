use async_trait::async_trait;
use std::error::Error;

use crate::core::events::{DomainEvent, EventEnvelope};

/// Projection - xử lý events để update read models
#[async_trait]
pub trait Projection: Send + Sync {
    type Event: DomainEvent;
    type Error: Error + Send + Sync + 'static;

    /// Projection name (unique identifier)
    fn projection_name(&self) -> &'static str;

    /// Handle một event
    async fn handle(&self, event: &EventEnvelope<Self::Event>) -> Result<(), Self::Error>;

    /// Rebuild toàn bộ projection từ events
    async fn rebuild(&self) -> Result<(), Self::Error>;

    /// Get last processed position
    async fn get_position(&self) -> Result<i64, Self::Error>;

    /// Update position sau khi process event
    async fn update_position(&self, position: i64) -> Result<(), Self::Error>;
}

/// Projection Runner - quản lý và chạy các projections
pub struct ProjectionRunner {
    // Will manage multiple projections
    // Track positions
    // Handle errors and retries
}

impl ProjectionRunner {
    pub fn new() -> Self {
        Self {}
    }

    // Register projection
    // Start processing
    // Handle errors
}

