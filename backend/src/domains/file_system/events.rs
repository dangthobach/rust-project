use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::events::DomainEvent;

/// File System Events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FileSystemEvent {
    FileCreated(FileCreatedEvent),
    FileMoved(FileMovedEvent),
    FileDeleted(FileDeletedEvent),
    FileRestored(FileRestoredEvent),
    FileRenamed(FileRenamedEvent),
    FilePermissionsChanged(FilePermissionsChangedEvent),
    FolderCreated(FolderCreatedEvent),
    FolderMoved(FolderMovedEvent),
    FolderDeleted(FolderDeletedEvent),
    FolderRestored(FolderRestoredEvent),
    FolderRenamed(FolderRenamedEvent),
}

impl DomainEvent for FileSystemEvent {
    fn event_type(&self) -> &'static str {
        match self {
            FileSystemEvent::FileCreated(_) => "file_created",
            FileSystemEvent::FileMoved(_) => "file_moved",
            FileSystemEvent::FileDeleted(_) => "file_deleted",
            FileSystemEvent::FileRestored(_) => "file_restored",
            FileSystemEvent::FileRenamed(_) => "file_renamed",
            FileSystemEvent::FilePermissionsChanged(_) => "file_permissions_changed",
            FileSystemEvent::FolderCreated(_) => "folder_created",
            FileSystemEvent::FolderMoved(_) => "folder_moved",
            FileSystemEvent::FolderDeleted(_) => "folder_deleted",
            FileSystemEvent::FolderRestored(_) => "folder_restored",
            FileSystemEvent::FolderRenamed(_) => "folder_renamed",
        }
    }

    fn aggregate_id(&self) -> Uuid {
        match self {
            FileSystemEvent::FileCreated(e) => e.file_id,
            FileSystemEvent::FileMoved(e) => e.file_id,
            FileSystemEvent::FileDeleted(e) => e.file_id,
            FileSystemEvent::FileRestored(e) => e.file_id,
            FileSystemEvent::FileRenamed(e) => e.file_id,
            FileSystemEvent::FilePermissionsChanged(e) => e.file_id,
            FileSystemEvent::FolderCreated(e) => e.folder_id,
            FileSystemEvent::FolderMoved(e) => e.folder_id,
            FileSystemEvent::FolderDeleted(e) => e.folder_id,
            FileSystemEvent::FolderRestored(e) => e.folder_id,
            FileSystemEvent::FolderRenamed(e) => e.folder_id,
        }
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        match self {
            FileSystemEvent::FileCreated(e) => e.occurred_at,
            FileSystemEvent::FileMoved(e) => e.occurred_at,
            FileSystemEvent::FileDeleted(e) => e.occurred_at,
            FileSystemEvent::FileRestored(e) => e.occurred_at,
            FileSystemEvent::FileRenamed(e) => e.occurred_at,
            FileSystemEvent::FilePermissionsChanged(e) => e.occurred_at,
            FileSystemEvent::FolderCreated(e) => e.occurred_at,
            FileSystemEvent::FolderMoved(e) => e.occurred_at,
            FileSystemEvent::FolderDeleted(e) => e.occurred_at,
            FileSystemEvent::FolderRestored(e) => e.occurred_at,
            FileSystemEvent::FolderRenamed(e) => e.occurred_at,
        }
    }

    fn version(&self) -> i64 {
        match self {
            FileSystemEvent::FileCreated(e) => e.version,
            FileSystemEvent::FileMoved(e) => e.version,
            FileSystemEvent::FileDeleted(e) => e.version,
            FileSystemEvent::FileRestored(e) => e.version,
            FileSystemEvent::FileRenamed(e) => e.version,
            FileSystemEvent::FilePermissionsChanged(e) => e.version,
            FileSystemEvent::FolderCreated(e) => e.version,
            FileSystemEvent::FolderMoved(e) => e.version,
            FileSystemEvent::FolderDeleted(e) => e.version,
            FileSystemEvent::FolderRestored(e) => e.version,
            FileSystemEvent::FolderRenamed(e) => e.version,
        }
    }
}

// File Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCreatedEvent {
    pub file_id: Uuid,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMovedEvent {
    pub file_id: Uuid,
    pub old_parent_id: Option<Uuid>,
    pub new_parent_id: Option<Uuid>,
    pub old_path: String,
    pub new_path: String,
    pub moved_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeletedEvent {
    pub file_id: Uuid,
    pub deleted_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRestoredEvent {
    pub file_id: Uuid,
    pub restored_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRenamedEvent {
    pub file_id: Uuid,
    pub old_name: String,
    pub new_name: String,
    pub new_path: String,
    pub renamed_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissionsChangedEvent {
    pub file_id: Uuid,
    pub acl: serde_json::Value, // Vec<AccessControlEntry>
    pub changed_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

// Folder Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCreatedEvent {
    pub folder_id: Uuid,
    pub name: String,
    pub path: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used in Folder aggregate
pub struct FolderMovedEvent {
    pub folder_id: Uuid,
    pub old_parent_id: Option<Uuid>,
    pub new_parent_id: Option<Uuid>,
    pub old_path: String,
    pub new_path: String,
    pub moved_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used in Folder aggregate
pub struct FolderDeletedEvent {
    pub folder_id: Uuid,
    pub deleted_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used in Folder aggregate
pub struct FolderRestoredEvent {
    pub folder_id: Uuid,
    pub restored_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Used in Folder aggregate
pub struct FolderRenamedEvent {
    pub folder_id: Uuid,
    pub old_name: String,
    pub new_name: String,
    pub new_path: String,
    pub renamed_by: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub version: i64,
}

