use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::handlers::{auth, clients, files, health, notifications, tasks, users};
use crate::middleware::auth as auth_middleware;

pub fn create_router(pool: SqlitePool, config: Config) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register));

    // Protected routes (auth required)
    let protected_routes = Router::new()
        // User routes
        .route("/api/users/me", get(users::get_current_user))
        .route("/api/users/:id", get(users::get_user))
        .route("/api/users/:id", patch(users::update_user))
        // Client routes
        .route("/api/clients", get(clients::list_clients))
        .route("/api/clients", post(clients::create_client))
        .route("/api/clients/search", get(clients::search_clients))
        .route("/api/clients/:id", get(clients::get_client))
        .route("/api/clients/:id", patch(clients::update_client))
        .route("/api/clients/:id", delete(clients::delete_client))
        // Task routes
        .route("/api/tasks", get(tasks::list_tasks))
        .route("/api/tasks", post(tasks::create_task))
        .route("/api/tasks/search", get(tasks::search_tasks))
        .route("/api/tasks/:id", get(tasks::get_task))
        .route("/api/tasks/:id", patch(tasks::update_task))
        .route("/api/tasks/:id", delete(tasks::delete_task))
        // Notification routes
        .route("/api/notifications", get(notifications::list_notifications))
        .route("/api/notifications/mark-read", post(notifications::mark_as_read))
        .route("/api/notifications/:id", delete(notifications::delete_notification))
        // File routes (traditional)
        .route("/api/files", get(files::list_files))
        .route("/api/files/upload", post(files::upload_file))
        .route("/api/files/search", get(files::search_files))
        .route("/api/files/:id", get(files::get_file))
        .route("/api/files/:id/download", get(files::download_file))
        .route("/api/files/:id", delete(files::delete_file))
        // File System CQRS routes disabled temporarily (needs PostgreSQL)
        // .route("/api/fs/files", post(file_system::create_file))
        // .route("/api/fs/files/:id", get(file_system::get_file))
        // .route("/api/fs/files", get(file_system::list_files))
        // .route("/api/fs/files/:id/move", put(file_system::move_file))
        // .route("/api/fs/files/:id", delete(file_system::delete_file))
        // .route("/api/fs/files/:id/rename", put(file_system::rename_file))
        // .route("/api/fs/folders", post(file_system::create_folder))
        // .route("/api/fs/folders/:id/tree", get(file_system::get_folder_tree))
        // .route("/api/fs/files/search", get(file_system::search_files))
        .layer(middleware::from_fn_with_state(
            (pool.clone(), config.clone()),
            auth_middleware::auth,
        ));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(DefaultBodyLimit::max(config.max_file_size))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state((pool, config))
}
