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

#[derive(FromRow)]
pub struct FileViewRow {
    id: Uuid,
    name: String,
    path: String,
    parent_id: Option<Uuid>,
    size: Option<i64>,
    mime_type: Option<String>,
    owner_id: Uuid,
    item_type: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<FileViewRow> for FileView {
    fn from(row: FileViewRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            path: row.path,
            parent_id: row.parent_id,
            size: row.size.unwrap_or(0),
            mime_type: row.mime_type.unwrap_or_default(),
            owner_id: row.owner_id,
            item_type: row.item_type,
            created_at: row.created_at,
            updated_at: row.updated_at,
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

