use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: Uuid,
    pub name: String,
    pub original_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub uploaded_by: Option<Uuid>,
    pub client_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub description: Option<String>,
    pub thumbnail_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in API handlers
pub struct FileQuery {
    pub client_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub file_type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
