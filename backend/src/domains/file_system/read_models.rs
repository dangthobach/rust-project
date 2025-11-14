use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// FileView - Read model for files
/// SQLite stores UUIDs as TEXT, so we need custom FromRow implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileView {
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid,
    pub item_type: String, // "file" or "folder"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLite row structure (UUIDs as TEXT)
/// Used internally for database queries
#[derive(FromRow)]
pub struct FileViewRow {
    id: String,
    name: String,
    path: String,
    parent_id: Option<String>,
    size: i64,
    mime_type: String,
    owner_id: String,
    item_type: String,
    created_at: String,  // ISO8601 string
    updated_at: String,  // ISO8601 string
}

impl From<FileViewRow> for FileView {
    fn from(row: FileViewRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id)
                .unwrap_or_else(|_| Uuid::nil()), // Fallback for invalid UUID
            name: row.name,
            path: row.path,
            parent_id: row.parent_id.and_then(|s| Uuid::parse_str(&s).ok()),
            size: row.size,
            mime_type: row.mime_type,
            owner_id: Uuid::parse_str(&row.owner_id)
                .unwrap_or_else(|_| Uuid::nil()), // Fallback for invalid UUID
            item_type: row.item_type,
            created_at: DateTime::parse_from_rfc3339(&row.created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()), // Fallback to current time
            updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()), // Fallback to current time
        }
    }
}

/// FolderTreeView - Tree structure for folders
/// Used in query handlers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderTreeView {
    pub folder: FileView,
    pub children: Vec<FolderTreeView>,
}

