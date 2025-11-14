use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::domain::{AggregateRoot, Entity};
use crate::core::infrastructure::Rebuildable;
use crate::core::shared::{
    AccessControlEntry, AuditFields, Permission, Securable, SoftDeleteFields, SoftDeletable,
    Subject,
};
use crate::domains::file_system::events::*;

/// File Aggregate - represents a file in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    // Entity fields
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Domain fields
    pub name: String,
    pub path: String,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid,

    // Shared components
    pub audit: AuditFields,
    pub soft_delete: SoftDeleteFields,
    pub acl: Vec<AccessControlEntry>,

    // Event sourcing
    version: i64,
    uncommitted_events: Vec<FileSystemEvent>,
}

impl Entity for File {
    type Id = Uuid;

    fn id(&self) -> &Uuid {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl AggregateRoot for File {
    type Event = FileSystemEvent;

    fn version(&self) -> i64 {
        self.version
    }

    fn uncommitted_events(&self) -> &[FileSystemEvent] {
        &self.uncommitted_events
    }

    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }

    fn apply(&mut self, event: &FileSystemEvent) {
        match event {
            FileSystemEvent::FileCreated(e) => {
                self.id = e.file_id;
                self.name = e.name.clone();
                self.path = e.path.clone();
                self.parent_id = e.parent_id;
                self.size = e.size;
                self.mime_type = e.mime_type.clone();
                self.owner_id = e.owner_id;
                self.created_at = e.occurred_at;
                self.updated_at = e.occurred_at;
                self.audit = AuditFields::new(Some(e.owner_id));
                // Default ACL: owner has all permissions
                self.acl = vec![AccessControlEntry::new(
                    Subject::User(e.owner_id),
                    Permission::all(),
                )];
            }
            FileSystemEvent::FileMoved(e) => {
                self.parent_id = e.new_parent_id;
                self.path = e.new_path.clone();
                self.updated_at = e.occurred_at;
                self.audit.touch(e.moved_by);
            }
            FileSystemEvent::FileDeleted(e) => {
                self.soft_delete.mark_as_deleted(e.deleted_by);
                self.updated_at = e.occurred_at;
            }
            FileSystemEvent::FileRestored(e) => {
                self.soft_delete.restore();
                self.updated_at = e.occurred_at;
                self.audit.touch(e.restored_by);
            }
            FileSystemEvent::FileRenamed(e) => {
                self.name = e.new_name.clone();
                self.path = e.new_path.clone();
                self.updated_at = e.occurred_at;
                self.audit.touch(e.renamed_by);
            }
            FileSystemEvent::FilePermissionsChanged(e) => {
                self.acl = serde_json::from_value(e.acl.clone())
                    .unwrap_or_else(|_| vec![]);
                self.updated_at = e.occurred_at;
                self.audit.touch(e.changed_by);
            }
            _ => {} // Ignore folder events
        }
        self.version += 1;
    }
}

impl Securable for File {
    fn get_acl(&self) -> &[AccessControlEntry] {
        &self.acl
    }

    fn set_acl(&mut self, acl: Vec<AccessControlEntry>) {
        self.acl = acl;
    }
}

impl SoftDeletable for File {
    fn is_deleted(&self) -> bool {
        self.soft_delete.is_deleted()
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.soft_delete.deleted_at()
    }

    fn deleted_by(&self) -> Option<Uuid> {
        self.soft_delete.deleted_by()
    }

    fn mark_as_deleted(&mut self, by: Uuid) {
        let event = FileSystemEvent::FileDeleted(FileDeletedEvent {
            file_id: self.id,
            deleted_by: by,
            occurred_at: Utc::now(),
            version: self.version + 1,
        });
        self.apply(&event);
        self.uncommitted_events.push(event);
    }

    fn restore(&mut self) {
        let event = FileSystemEvent::FileRestored(FileRestoredEvent {
            file_id: self.id,
            restored_by: self.audit.updated_by.unwrap_or(self.owner_id),
            occurred_at: Utc::now(),
            version: self.version + 1,
        });
        self.apply(&event);
        self.uncommitted_events.push(event);
    }
}

// Business methods
impl File {
    /// Create new file
    pub fn create(
        name: String,
        path: String,
        parent_id: Option<Uuid>,
        size: i64,
        mime_type: String,
        owner_id: Uuid,
    ) -> Result<Self, String> {
        if name.is_empty() {
            return Err("File name cannot be empty".to_string());
        }

        let id = Uuid::new_v4();
        let now = Utc::now();

        let event = FileSystemEvent::FileCreated(FileCreatedEvent {
            file_id: id,
            name: name.clone(),
            path: path.clone(),
            parent_id,
            size,
            mime_type: mime_type.clone(),
            owner_id,
            occurred_at: now,
            version: 1,
        });

        let mut file = Self {
            id,
            created_at: now,
            updated_at: now,
            name: String::new(),
            path: String::new(),
            parent_id: None,
            size: 0,
            mime_type: String::new(),
            owner_id,
            audit: AuditFields::new(None),
            soft_delete: SoftDeleteFields::new(),
            acl: vec![],
            version: 0,
            uncommitted_events: vec![],
        };

        file.apply(&event);
        file.uncommitted_events.push(event);

        Ok(file)
    }

    /// Move file to new parent
    pub fn move_to(&mut self, new_parent_id: Option<Uuid>, new_path: String, moved_by: Uuid) {
        let event = FileSystemEvent::FileMoved(FileMovedEvent {
            file_id: self.id,
            old_parent_id: self.parent_id,
            new_parent_id,
            old_path: self.path.clone(),
            new_path: new_path.clone(),
            moved_by,
            occurred_at: Utc::now(),
            version: self.version + 1,
        });

        self.apply(&event);
        self.uncommitted_events.push(event);
    }

    /// Rename file
    pub fn rename(&mut self, new_name: String, new_path: String, renamed_by: Uuid) {
        if new_name.is_empty() {
            return;
        }

        let old_name = self.name.clone();
        let event = FileSystemEvent::FileRenamed(FileRenamedEvent {
            file_id: self.id,
            old_name,
            new_name: new_name.clone(),
            new_path: new_path.clone(),
            renamed_by,
            occurred_at: Utc::now(),
            version: self.version + 1,
        });

        self.apply(&event);
        self.uncommitted_events.push(event);
    }

    /// Change permissions
    pub fn change_permissions(&mut self, acl: Vec<AccessControlEntry>, changed_by: Uuid) {
        let event = FileSystemEvent::FilePermissionsChanged(FilePermissionsChangedEvent {
            file_id: self.id,
            acl: serde_json::to_value(&acl).unwrap_or(serde_json::json!([])),
            changed_by,
            occurred_at: Utc::now(),
            version: self.version + 1,
        });

        self.apply(&event);
        self.uncommitted_events.push(event);
    }

}

impl Rebuildable for File {
    /// Rebuild from events (for event sourcing)
    fn rebuild_from_events(events: &[FileSystemEvent]) -> Option<Self> {
        if events.is_empty() {
            return None;
        }

        let mut file = Self {
            id: Uuid::nil(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            name: String::new(),
            path: String::new(),
            parent_id: None,
            size: 0,
            mime_type: String::new(),
            owner_id: Uuid::nil(),
            audit: AuditFields::new(None),
            soft_delete: SoftDeleteFields::new(),
            acl: vec![],
            version: 0,
            uncommitted_events: vec![],
        };

        for event in events {
            file.apply(event);
        }

        Some(file)
    }
}

