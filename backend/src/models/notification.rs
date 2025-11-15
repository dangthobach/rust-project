use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: String,  // SQLite stores UUID as TEXT
    pub user_id: String,  // SQLite stores UUID as TEXT
    #[sqlx(rename = "type")]
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in API handlers
pub struct CreateNotificationRequest {
    pub user_id: String,  // SQLite stores UUID as TEXT
    pub notification_type: String,
    pub title: String,
    pub message: String,
    pub link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MarkAsReadRequest {
    pub notification_ids: Vec<String>,  // SQLite stores UUID as TEXT
}
