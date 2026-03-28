use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportExportJob {
    pub id: Uuid,
    pub user_id: Uuid,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub object_uri: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

