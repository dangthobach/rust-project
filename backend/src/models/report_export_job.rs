use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportExportJob {
    pub id: String,
    pub user_id: String,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub object_uri: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

