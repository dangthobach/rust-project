use sqlx::PgPool;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: String,
}

/// Log an audit event
pub async fn log_audit(
    pool: &PgPool,
    user_id: &str,
    action: &str,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
) {
    let id = Uuid::new_v4();
    let Ok(uid) = Uuid::parse_str(user_id) else {
        tracing::warn!("audit: skip invalid user_id");
        return;
    };

    let result = sqlx::query(
        r#"
        INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, ip_address, user_agent)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(id)
    .bind(uid)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(details)
    .bind(ip_address)
    .bind(user_agent)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::error!("Failed to log audit event: {}", e);
    }
}

/// Audit action types
pub mod actions {
    pub const LOGIN: &str = "login";
    pub const LOGOUT: &str = "logout";
    pub const CREATE: &str = "create";
    pub const UPDATE: &str = "update";
    pub const DELETE: &str = "delete";
    pub const READ: &str = "read";
    pub const BULK_DELETE: &str = "bulk_delete";
    pub const BULK_UPDATE: &str = "bulk_update";
    pub const ROLE_CHANGE: &str = "role_change";
    pub const STATUS_CHANGE: &str = "status_change";
    pub const PASSWORD_CHANGE: &str = "password_change";
}

/// Resource types
pub mod resources {
    pub const USER: &str = "user";
    pub const CLIENT: &str = "client";
    pub const TASK: &str = "task";
    pub const FILE: &str = "file";
    pub const NOTIFICATION: &str = "notification";
}
