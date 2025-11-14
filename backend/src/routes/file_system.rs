use axum::{
    extract::{Extension, Request},
    middleware::Next,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::file_system::*;
use crate::config::Config;
use crate::core::events::{DomainEvent, EventBus, EventEnvelope, EventStream, RedisEventBus};
use crate::domains::file_system::handlers::HandlerState;
use async_trait::async_trait;

/// Create file system routes
pub fn create_file_system_routes(
    pool: SqlitePool,
    config: Config,
) -> Result<Router, anyhow::Error> {
    // Create Redis Event Bus
    // Wrap EventBusError into anyhow::Error
    let redis_bus = RedisEventBus::new(&config.redis_url, "file_system_events".to_string())
        .map_err(|e| anyhow::anyhow!("Failed to create Redis Event Bus: {}", e))?;
    
    // Create wrapper to convert EventBusError to anyhow::Error
    struct EventBusWrapper {
        inner: RedisEventBus,
    }
    
    #[async_trait::async_trait]
    impl EventBus for EventBusWrapper {
        type Error = anyhow::Error;
        
        async fn publish<E: DomainEvent>(&self, event: EventEnvelope<E>) -> Result<(), Self::Error> {
            self.inner.publish(event).await.map_err(|e| anyhow::anyhow!("{}", e))
        }
        
        async fn subscribe(&self, consumer_group: &str, consumer_name: &str) -> Result<EventStream, Self::Error> {
            self.inner.subscribe(consumer_group, consumer_name).await.map_err(|e| anyhow::anyhow!("{}", e))
        }
    }
    
    let event_bus: Arc<dyn EventBus<Error = anyhow::Error> + Send + Sync> =
        Arc::new(EventBusWrapper { inner: redis_bus });

    // Create handler state
    let handler_state = Arc::new(HandlerState::new(pool.clone(), event_bus));

    // Create routes
    let routes = Router::new()
        .route("/api/files", post(create_file))
        .route("/api/files/:id", get(get_file))
        .route("/api/files", get(list_files))
        .route("/api/files/:id/move", put(move_file))
        .route("/api/files/:id", delete(delete_file))
        .route("/api/files/:id/rename", put(rename_file))
        .route("/api/folders", post(create_folder))
        .route("/api/folders/:id/tree", get(get_folder_tree))
        .route("/api/files/search", get(search_files))
        .layer(axum::middleware::from_fn(
            |mut req: Request, next: Next| async move {
                // Extract user_id from auth middleware (already set by auth middleware)
                let user_id = req
                    .extensions()
                    .get::<Uuid>()
                    .copied()
                    .unwrap_or_else(|| {
                        // Fallback for testing - in production this should not happen
                        tracing::warn!("No user_id in request extensions, using fallback");
                        Uuid::new_v4()
                    });

                // Ensure user_id is in extensions for handlers
                req.extensions_mut().insert(user_id);
                next.run(req).await
            },
        ))
        .layer(Extension(handler_state.clone()))
        .with_state(handler_state);

    Ok(routes)
}
