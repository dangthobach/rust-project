use sqlx::PgPool;
use std::sync::Arc;
use crate::core::cqrs::{CommandBus, QueryBus};
use crate::core::events::{EventBus, PostgresEventStore, RedisEventBus, InMemoryEventBus};
use crate::config::Config;
use crate::handlers::websocket::WsConnectionManager;
use crate::integrations::{
    KafkaPublisher, KafkaPublisherAdapter, LocalObjectStorage, NoopKafkaPublisher, NoopRabbitMqPublisher,
    ObjectStorage, RabbitMqPublisher, RabbitMqPublisherAdapter, RustfsObjectStorage,
};

/// Application State - Centralized dependency injection container
///
/// Contains all shared resources needed across the application:
/// - Database pool
/// - CQRS buses (Command & Query)
/// - Event Sourcing infrastructure (EventStore & EventBus)
/// - WebSocket connection manager
/// - Configuration
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub pool: Arc<PgPool>,

    /// Command Bus - for handling write operations
    pub command_bus: Arc<CommandBus>,

    /// Query Bus - for handling read operations
    pub query_bus: Arc<QueryBus>,

    /// Event Store - persistent storage for domain events
    pub event_store: Arc<PostgresEventStore>,

    /// Event Bus - publish/subscribe for domain events (Redis Streams)
    pub event_bus: Arc<dyn EventBus + Send + Sync>,

    /// WebSocket connection manager for real-time notifications
    pub ws_manager: Arc<WsConnectionManager>,

    /// Application configuration
    pub config: Arc<Config>,

    /// Kafka publisher (event streaming integration)
    pub kafka_publisher: Arc<dyn KafkaPublisher + Send + Sync>,

    /// RabbitMQ publisher (queue/workflow integration)
    pub rabbitmq_publisher: Arc<dyn RabbitMqPublisher + Send + Sync>,

    /// Object storage integration (local or s3-compatible)
    pub object_storage: Arc<dyn ObjectStorage + Send + Sync>,

    /// Redis Client for generic caching
    pub redis_client: Option<redis::Client>,
}

impl AppState {
    /// Create new AppState with all dependencies initialized
    ///
    /// # Arguments
    /// * `pool` - PostgreSQL connection pool
    /// * `config` - Application configuration
    ///
    /// # Returns
    /// * `AppState` instance with all buses and stores initialized
    ///
    /// # Errors
    /// * Redis connection errors
    /// * Configuration errors
    pub async fn new(pool: PgPool, config: Config) -> anyhow::Result<Self> {
        tracing::info!("Initializing AppState...");

        // 1. Get Redis URL from environment or use default
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| {
                tracing::warn!("REDIS_URL not set, using default: redis://127.0.0.1:6379");
                "redis://127.0.0.1:6379".to_string()
            });

        tracing::info!("Attempting to connect to Redis at: {}", redis_url);
        
        let redis_client = redis::Client::open(redis_url.clone()).ok();

        // 2. Initialize Event Bus (Redis with InMemory fallback if server unreachable)
        let event_bus: Arc<dyn EventBus + Send + Sync> =
            match RedisEventBus::new(&redis_url, "neo_crm_events".to_string()) {
                Ok(bus) => match bus.verify_connection().await {
                    Ok(()) => {
                        tracing::info!("✅ Redis Event Bus initialized successfully");
                        Arc::new(bus)
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️  Redis unreachable: {} — using InMemoryEventBus (events not persisted)",
                            e
                        );
                        tracing::info!("💡 To enable Redis: docker run -d -p 6379:6379 redis:alpine");
                        Arc::new(InMemoryEventBus::new())
                    }
                },
                Err(e) => {
                    tracing::warn!("⚠️  Failed to create Redis client: {}", e);
                    tracing::warn!("⚠️  Falling back to InMemoryEventBus (events won't be persisted)");
                    tracing::info!("💡 To enable Redis: docker run -d -p 6379:6379 redis:alpine");
                    Arc::new(InMemoryEventBus::new())
                }
            };

        // 3. Initialize Event Store (SQLite-based)
        let event_store = Arc::new(PostgresEventStore::new(pool.clone()));
        tracing::info!("✅ Event Store initialized");

        // 4. Initialize CQRS buses
        let command_bus = Arc::new(CommandBus::new());
        let query_bus = Arc::new(QueryBus::new());
        tracing::info!("✅ Command Bus and Query Bus initialized");

        // 5. Initialize WebSocket connection manager
        let ws_manager = Arc::new(WsConnectionManager::new());
        tracing::info!("✅ WebSocket Connection Manager initialized");

        // 6. Initialize Kafka publisher (adapter with noop fallback)
        let kafka_publisher: Arc<dyn KafkaPublisher + Send + Sync> =
            if config.kafka_brokers.trim().is_empty() {
                tracing::warn!("⚠️  KAFKA_BROKERS not configured, using NoopKafkaPublisher");
                Arc::new(NoopKafkaPublisher)
            } else {
                match KafkaPublisherAdapter::new(config.kafka_brokers.clone()) {
                    Ok(v) => Arc::new(v),
                    Err(e) => {
                        tracing::warn!("⚠️  Kafka init failed: {}, fallback noop", e);
                        Arc::new(NoopKafkaPublisher)
                    }
                }
            };

        // 7. Initialize RabbitMQ publisher (adapter with noop fallback)
        let rabbitmq_publisher: Arc<dyn RabbitMqPublisher + Send + Sync> =
            if config.rabbitmq_url.trim().is_empty() {
                tracing::warn!("⚠️  RABBITMQ_URL not configured, using NoopRabbitMqPublisher");
                Arc::new(NoopRabbitMqPublisher)
            } else {
                Arc::new(RabbitMqPublisherAdapter::new(config.rabbitmq_url.clone()))
            };

        // 8. Initialize Object Storage (local or s3-compatible adapter)
        let object_storage: Arc<dyn ObjectStorage + Send + Sync> =
            if config.object_storage_provider.eq_ignore_ascii_case("rustfs")
            {
                Arc::new(RustfsObjectStorage::new(
                    config.object_storage_endpoint.clone(),
                    config.object_storage_bucket.clone(),
                    config.object_storage_access_key.clone(),
                    config.object_storage_secret_key.clone(),
                ))
            } else {
                Arc::new(LocalObjectStorage::new(config.upload_dir.clone()))
            };

        // 9. Build AppState
        let state = Self {
            pool: Arc::new(pool),
            command_bus,
            query_bus,
            event_store,
            event_bus,
            ws_manager,
            config: Arc::new(config),
            kafka_publisher,
            rabbitmq_publisher,
            object_storage,
            redis_client,
        };

        tracing::info!("✅ AppState initialized successfully");
        Ok(state)
    }

    /// Get database pool reference
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get command bus reference
    pub fn command_bus(&self) -> &CommandBus {
        &self.command_bus
    }

    /// Get query bus reference
    pub fn query_bus(&self) -> &QueryBus {
        &self.query_bus
    }

    /// Get event store reference
    pub fn event_store(&self) -> &PostgresEventStore {
        &self.event_store
    }

    /// Get event bus reference
    pub fn event_bus(&self) -> &Arc<dyn EventBus + Send + Sync> {
        &self.event_bus
    }

    /// Get configuration reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get WebSocket connection manager reference
    pub fn ws_manager(&self) -> &WsConnectionManager {
        &self.ws_manager
    }

    pub fn kafka_publisher(&self) -> &Arc<dyn KafkaPublisher + Send + Sync> {
        &self.kafka_publisher
    }

    pub fn rabbitmq_publisher(&self) -> &Arc<dyn RabbitMqPublisher + Send + Sync> {
        &self.rabbitmq_publisher
    }

    pub fn object_storage(&self) -> &Arc<dyn ObjectStorage + Send + Sync> {
        &self.object_storage
    }

    /// Test connections (for health checks)
    pub async fn health_check(&self) -> anyhow::Result<()> {
        // Test database connection
        sqlx::query("SELECT 1")
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| anyhow::anyhow!("Database health check failed: {}", e))?;

        // Test Redis connection by publishing a test event
        let test_event = serde_json::json!({
            "type": "HealthCheck",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        self.event_bus
            .publish_json(test_event.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Redis health check failed: {}", e))?;

        self.kafka_publisher
            .health_check()
            .await
            .map_err(|e| anyhow::anyhow!("Kafka health check failed: {}", e))?;

        self.rabbitmq_publisher
            .health_check()
            .await
            .map_err(|e| anyhow::anyhow!("RabbitMQ health check failed: {}", e))?;

        self.object_storage
            .health_check()
            .await
            .map_err(|e| anyhow::anyhow!("Object storage health check failed: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Redis and TEST_DATABASE_URL (PostgreSQL)"]
    async fn test_app_state_creation() {
        // This test requires Redis running
        // Skip in CI if Redis not available
        if std::env::var("CI").is_ok() {
            return;
        }

        let url = match std::env::var("TEST_DATABASE_URL") {
            Ok(u) if !u.is_empty() => u,
            _ => return,
        };

        let pool = PgPool::connect(&url).await.unwrap();

        // Create test config
        let config = Config {
            database_url: url,
            jwt_secret: "test-secret".to_string(),
            jwt_expiration: 86400,
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_origin: "http://localhost:5173".to_string(),
            max_file_size: 10485760,
            upload_dir: "./uploads".to_string(),
            redis_url: "redis://127.0.0.1:6379".to_string(),
            kafka_brokers: "".to_string(),
            rabbitmq_url: "".to_string(),
            object_storage_provider: "local".to_string(),
            object_storage_endpoint: "".to_string(),
            object_storage_bucket: "crm-objects".to_string(),
            object_storage_access_key: "".to_string(),
            object_storage_secret_key: "".to_string(),
            default_tenant_id: "public".to_string(),
        };

        // Create AppState (will fallback to InMemoryEventBus if Redis not available)
        let result = AppState::new(pool, config).await;

        // Should always succeed now with InMemory fallback
        assert!(result.is_ok());
    }
}
