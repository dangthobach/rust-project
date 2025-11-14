use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Soft Deletable trait - không xóa thật, chỉ đánh dấu deleted
pub trait SoftDeletable: Send + Sync {
    fn is_deleted(&self) -> bool;
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn deleted_by(&self) -> Option<Uuid>;

    /// Mark as deleted
    fn mark_as_deleted(&mut self, by: Uuid);

    /// Restore deleted entity
    fn restore(&mut self);
}

/// Base soft delete fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SoftDeleteFields {
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

impl SoftDeleteFields {
    pub fn new() -> Self {
        Self {
            deleted_at: None,
            deleted_by: None,
        }
    }

    pub fn mark_as_deleted(&mut self, by: Uuid) {
        self.deleted_at = Some(Utc::now());
        self.deleted_by = Some(by);
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.deleted_by = None;
    }
}

impl SoftDeletable for SoftDeleteFields {
    fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn deleted_by(&self) -> Option<Uuid> {
        self.deleted_by
    }

    fn mark_as_deleted(&mut self, by: Uuid) {
        self.mark_as_deleted(by);
    }

    fn restore(&mut self) {
        self.restore();
    }
}

