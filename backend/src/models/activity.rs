use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[allow(dead_code)] // Used in API handlers
pub struct Activity {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Used in API handlers
pub struct CreateActivityRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub description: String,
    pub metadata: Option<serde_json::Value>,
}
