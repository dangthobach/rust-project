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
        let mut conn = self.client.get_async_connection().await
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
