use async_trait::async_trait;
use redis::AsyncCommands;
use std::error::Error;
use std::fmt;

use crate::core::events::{DomainEvent, EventEnvelope};

/// Event Bus - publish/subscribe events
/// Sử dụng Redis Streams cho guaranteed delivery
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish event vào bus (serialized as JSON)
    async fn publish_json(&self, event_json: String) -> Result<(), anyhow::Error>;
}

/// Helper extension trait for EventBus với generic methods
#[async_trait]
pub trait EventBusExt: EventBus {
    /// Publish event vào bus
    async fn publish<E: DomainEvent>(&self, event: EventEnvelope<E>) -> Result<(), anyhow::Error> {
        let event_json = serde_json::to_string(&event)
            .map_err(|e| anyhow::anyhow!("Failed to serialize event: {}", e))?;
        self.publish_json(event_json).await
    }
}

// Blanket implementation
impl<T: EventBus + ?Sized> EventBusExt for T {}

/// Event stream từ Redis
pub struct EventStream {
    // Will be implemented with Redis Streams
}

/// Error type for Event Bus
#[derive(Debug)]
pub enum EventBusError {
    Redis(redis::RedisError),
    Serialization(String),
}

impl fmt::Display for EventBusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventBusError::Redis(e) => write!(f, "Redis error: {}", e),
            EventBusError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl Error for EventBusError {}

impl From<redis::RedisError> for EventBusError {
    fn from(e: redis::RedisError) -> Self {
        EventBusError::Redis(e)
    }
}

/// Redis implementation của Event Bus
pub struct RedisEventBus {
    client: redis::Client,
    stream_name: String,
}

impl RedisEventBus {
    pub fn new(redis_url: &str, stream_name: String) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client,
            stream_name,
        })
    }
}

#[async_trait]
impl EventBus for RedisEventBus {
    async fn publish_json(&self, event_json: String) -> Result<(), anyhow::Error> {
        let mut conn = self.client.get_multiplexed_async_connection().await
            .map_err(|e| anyhow::anyhow!("Redis connection error: {}", e))?;
        
        // Publish to Redis Stream
        let _: String = conn.xadd(
            &self.stream_name,
            "*",
            &[("event", event_json)],
        )
        .await
        .map_err(|e| anyhow::anyhow!("Redis xadd error: {}", e))?;

        Ok(())
    }
}

/// In-memory implementation of Event Bus (fallback when Redis unavailable)
///
/// This is a simple implementation that logs events but doesn't persist them.
/// Suitable for development or when Redis is not available.
pub struct InMemoryEventBus {
    // Could add a channel here for actual event processing if needed
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        tracing::warn!("⚠️  Using InMemoryEventBus - events will not be persisted!");
        tracing::warn!("⚠️  For production, please configure Redis for event persistence");
        Self {}
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish_json(&self, event_json: String) -> Result<(), anyhow::Error> {
        // Log event for debugging
        tracing::debug!("📤 Event published (in-memory): {}", event_json);

        // In a real implementation, you might:
        // - Store events in a Vec with Arc<RwLock<>>
        // - Send to tokio channels for processing
        // - Trigger event handlers directly

        // For now, just log and return success
        Ok(())
    }
}
