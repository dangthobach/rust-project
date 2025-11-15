use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: String,
    pub name: String,
    pub original_name: String,
    pub file_path: String,
    pub file_type: String,
    pub file_size: i64,
    pub uploaded_by: Option<String>,
    pub client_id: Option<String>,
    pub task_id: Option<String>,
    pub description: Option<String>,
    pub thumbnail_path: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in API handlers
pub struct FileQuery {
    pub client_id: Option<String>,
    pub task_id: Option<String>,
    pub file_type: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}
