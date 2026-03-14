mod api;
mod app_state;
mod config;
mod core;
mod domains;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::net::SocketAddr;
use std::str::FromStr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::app_state::AppState;
use crate::config::Config;
use crate::routes::create_router_with_state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Starting Neo-Brutalist CRM Backend Server...");
    tracing::info!("Environment loaded from .env");

    // Create database connection pool (SQLite)
    tracing::info!("Connecting to database...");
    let connect_options = SqliteConnectOptions::from_str(&config.database_url)?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(50) // Increased for 30k CCU
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with(connect_options)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("✅ Migrations completed successfully");

    // ============================================================
    // CQRS + Event Sourcing Infrastructure
    // ============================================================
    tracing::info!("Initializing CQRS/Event Sourcing infrastructure...");

    // Create AppState with all CQRS buses and Event Store
    let state = AppState::new(pool, config.clone()).await?;

    // Perform health check
    tracing::info!("Running health checks...");
    match state.health_check().await {
        Ok(_) => tracing::info!("✅ All systems healthy (Database + Redis)"),
        Err(e) => {
            tracing::error!("❌ Health check failed: {}", e);
            tracing::error!("Please ensure:");
            tracing::error!("  1. Redis is running: docker run -d -p 6379:6379 redis:alpine");
            tracing::error!("  2. Database is accessible");
            return Err(e);
        }
    }

    tracing::info!("✅ CQRS Infrastructure ready:");
    tracing::info!("  - Command Bus: Initialized");
    tracing::info!("  - Query Bus: Initialized");
    tracing::info!("  - Event Store: SQLite-based");
    tracing::info!("  - Event Bus: Redis Streams (neo_crm_events)");

    // ✅ Create router with CQRS support
    let app = create_router_with_state(state);

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("API available at http://{}/api", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
