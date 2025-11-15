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
use crate::core::events::{EventBus, RedisEventBus};
use crate::domains::file_system::handlers::HandlerState;

/// Create file system routes
/// Returns a stateless Router that can be merged with the main router
pub fn create_file_system_routes(
    pool: SqlitePool,
    config: Config,
) -> Result<Router, anyhow::Error> {
    // Create Redis Event Bus
    let event_bus = RedisEventBus::new(&config.redis_url, "file_system_events".to_string())
        .map_err(|e| anyhow::anyhow!("Failed to create Redis Event Bus: {}", e))?;

    let event_bus: Arc<dyn EventBus + Send + Sync> = Arc::new(event_bus);

    // Create handler state and wrap in Arc for Extension
    let handler_state = Arc::new(HandlerState::new(pool.clone(), event_bus));

    // Create routes WITHOUT state, using Extension for HandlerState
    let fs_routes = Router::new()
        .route("/api/fs/files", post(create_file))
        .route("/api/fs/files/:id", get(get_file))
        .route("/api/fs/files", get(list_files))
        .route("/api/fs/files/:id/move", put(move_file))
        .route("/api/fs/files/:id", delete(delete_file))
        .route("/api/fs/files/:id/rename", put(rename_file))
        .route("/api/fs/folders", post(create_folder))
        .route("/api/fs/folders/:id/tree", get(get_folder_tree))
        .route("/api/fs/files/search", get(search_files))
        .layer(Extension(handler_state))
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
        ));

    Ok(fs_routes)
}
