use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::SqlitePool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::api::{cqrs_handlers as cqrs, file_system_cqrs as fs_cqrs, rbac_cqrs, tasks_cqrs, users_cqrs};
use crate::app_state::AppState;
use crate::config::Config;
use crate::handlers::{activities, admin, analytics, auth, dashboard, export, files, health, notifications, tasks, users, websocket};
use crate::middleware::{auth as auth_middleware, deprecation, rate_limit, rbac};

/// Create router with uniform AppState
///
/// All handlers now use AppState extraction.
/// CQRS handlers use full AppState with CommandBus/QueryBus.
pub fn create_router_with_state(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(state.config().cors_origin.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods(Any)
        .allow_headers(Any);

    // Rate limiters
    let auth_limiter = rate_limit::create_auth_limiter();
    let upload_limiter = rate_limit::create_upload_limiter();

    // Public routes (no auth required) with rate limiting on auth endpoints
    let auth_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh))
        .route_layer(middleware::from_fn(move |req, next| {
            let limiter = auth_limiter.clone();
            async move { rate_limit::apply_rate_limit(limiter, req, next).await }
        }));

    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .merge(auth_routes);

    // Protected routes (auth required)
    // File upload with separate rate limiting
    let file_upload_routes = Router::new()
        .route("/api/files/upload", post(files::upload_file))
        .route_layer(middleware::from_fn(move |req, next| {
            let limiter = upload_limiter.clone();
            async move { rate_limit::apply_rate_limit(limiter, req, next).await }
        }));

    // Legacy file routes (controlled deprecation period)
    let legacy_file_routes = Router::new()
        .route("/api/files", get(files::list_files))
        .merge(file_upload_routes) // keep upload behavior stable for clients
        .route("/api/files/search", get(files::search_files))
        .route("/api/files/:id", get(files::get_file))
        .route("/api/files/:id/download", get(files::download_file))
        .route("/api/files/:id", delete(files::delete_file))
        .route_layer(middleware::from_fn(
            deprecation::legacy_files_api_deprecation,
        ));

    // Auth-protected logout routes
    let logout_routes = Router::new()
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/logout-all", post(auth::logout_all));

    // Manager/Admin only routes (write operations for clients and tasks)
    // Using CQRS handlers for clients
    let manager_routes = Router::new()
        .route("/api/clients", post(cqrs::create_client))
        .route("/api/clients/:id", patch(cqrs::update_client))
        .route("/api/clients/:id", delete(cqrs::delete_client))
        .route("/api/tasks/:id", delete(tasks_cqrs::delete_task))
        .route("/api/files/:id", delete(files::delete_file))
        .route_layer(middleware::from_fn(rbac::require_manager_or_admin));

    // Admin only routes
    let admin_routes = Router::new()
        .route("/api/admin/users", get(users_cqrs::list_users_admin))
        .route("/api/admin/users/search", get(admin::search_users))
        .route("/api/admin/users/stats", get(admin::get_user_stats))
        .route("/api/admin/users", post(users_cqrs::create_user_admin))
        .route("/api/admin/users/bulk", post(admin::bulk_user_actions))
        .route("/api/admin/users/:id", patch(users_cqrs::update_user_admin))
        .route("/api/admin/users/:id", delete(users_cqrs::delete_user_admin))
        // RBAC CRUD (roles/permissions and assignments)
        .route("/api/admin/rbac/roles", get(rbac_cqrs::list_roles))
        .route("/api/admin/rbac/roles", post(rbac_cqrs::create_role))
        .route("/api/admin/rbac/roles/:role_id", patch(rbac_cqrs::update_role))
        .route("/api/admin/rbac/roles/:role_id", delete(rbac_cqrs::delete_role))
        .route("/api/admin/rbac/permissions", get(rbac_cqrs::list_permissions))
        .route("/api/admin/rbac/permissions", post(rbac_cqrs::create_permission))
        .route(
            "/api/admin/rbac/permissions/:code",
            patch(rbac_cqrs::update_permission),
        )
        .route(
            "/api/admin/rbac/permissions/:code",
            delete(rbac_cqrs::delete_permission),
        )
        .route(
            "/api/admin/rbac/roles/:role_id/permissions/:code",
            post(rbac_cqrs::assign_permission_to_role),
        )
        .route(
            "/api/admin/rbac/roles/:role_id/permissions/:code",
            delete(rbac_cqrs::revoke_permission_from_role),
        )
        .route(
            "/api/admin/rbac/users/:user_id/roles/:role_id",
            post(rbac_cqrs::assign_role_to_user),
        )
        .route(
            "/api/admin/rbac/users/:user_id/roles/:role_id",
            delete(rbac_cqrs::revoke_role_from_user),
        )
        // Export routes (Admin only)
        .route("/api/export/users", get(export::export_users))
        .route("/api/export/dashboard-report", get(export::export_dashboard_report))
        .route_layer(middleware::from_fn(rbac::require_admin));

    let protected_routes = Router::new()
        // Auth routes (require authentication)
        .merge(logout_routes)
        // User routes (all authenticated users)
        .route("/api/users/me", get(users_cqrs::get_current_user))
        .route("/api/users/profile", get(users::get_user_profile))
        .route("/api/users/:id", get(users_cqrs::get_user))
        .route("/api/users/:id", patch(users_cqrs::update_user_self))
        .route("/api/users/password", post(users_cqrs::change_password))
        .route("/api/users/avatar", post(users::upload_avatar))
        // Dashboard routes
        .route("/api/dashboard/stats", get(dashboard::get_dashboard_stats))
        .route("/api/dashboard/activity-feed", get(dashboard::get_activity_feed))
        .route("/api/dashboard/health", get(dashboard::health_check))
        // Analytics routes (admin only)
        .route("/api/analytics/user-activity", get(analytics::get_user_activity_analytics))
        .route("/api/analytics/task-completion", get(analytics::get_task_completion_analytics))
        .route("/api/analytics/client-engagement", get(analytics::get_client_engagement_analytics))
        .route("/api/analytics/storage-usage", get(analytics::get_storage_analytics))
        // Activity routes
        .route("/api/activities", get(activities::get_activities))
        .route("/api/activities", post(activities::create_activity))
        // Client routes (read for all, write for managers/admins)
        // Using CQRS handlers
        .route("/api/clients", get(cqrs::list_clients))
        .route("/api/clients/search", get(cqrs::search_clients))
        .route("/api/clients/:id", get(cqrs::get_client))
        // Task routes (all users can CRUD own tasks)
        .route("/api/tasks", get(tasks_cqrs::list_tasks))
        .route("/api/tasks", post(tasks_cqrs::create_task))
        .route("/api/tasks/search", get(tasks::search_tasks))
        .route("/api/tasks/:id", get(tasks_cqrs::get_task))
        .route("/api/tasks/:id", patch(tasks_cqrs::update_task))
        .route("/api/tasks/:id/complete", post(tasks_cqrs::complete_task))
        // Notification routes
        .route("/api/notifications", get(notifications::list_notifications))
        .route("/api/notifications/mark-read", post(notifications::mark_as_read))
        .route("/api/notifications/:id", delete(notifications::delete_notification))
        // File routes (legacy - deprecated with headers)
        .merge(legacy_file_routes)
        // File system CQRS routes (new standardized endpoints)
        .route("/api/fs/files", get(fs_cqrs::list_files))
        .route("/api/fs/files", post(fs_cqrs::create_file))
        .route("/api/fs/files/search", get(fs_cqrs::search_files))
        .route("/api/fs/files/:id", get(fs_cqrs::get_file))
        .route("/api/fs/files/:id", delete(fs_cqrs::delete_file))
        .route("/api/fs/files/:id/move", patch(fs_cqrs::move_file))
        .route("/api/fs/files/:id/rename", patch(fs_cqrs::rename_file))
        .route("/api/fs/folders", post(fs_cqrs::create_folder))
        .route("/api/fs/folders/:id/tree", get(fs_cqrs::get_folder_tree))
        // Export routes (authenticated users)
        .route("/api/export/clients", get(export::export_clients))
        .route("/api/export/tasks", get(export::export_tasks))
        // WebSocket route for real-time notifications
        .route("/api/ws", get(websocket::websocket_handler))
        // Merge manager-only routes
        .merge(manager_routes)
        // Merge admin-only routes
        .merge(admin_routes)
        // Apply auth middleware to all protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth,
        ));

    // Combine all routes
    let router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(DefaultBodyLimit::max(state.config().max_file_size))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    tracing::info!("✅ Routes configured with CQRS/Event Sourcing support");
    router.with_state(state)
}

/// Legacy create_router (backward compatible)
///
/// Deprecated: Use create_router_with_state() instead.
#[deprecated(since = "1.0.0", note = "Use create_router_with_state instead")]
pub fn create_router(pool: SqlitePool, config: Config) -> Router {
    // Create AppState from pool and config for backward compatibility
    let runtime = tokio::runtime::Handle::current();
    let state = runtime.block_on(async {
        AppState::new(pool, config).await.expect("Failed to initialize AppState")
    });
    create_router_with_state(state)
}
