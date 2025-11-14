use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Auditable trait - track ai tạo/sửa entity
pub trait Auditable: Send + Sync {
    fn created_by(&self) -> Option<Uuid>;
    fn updated_by(&self) -> Option<Uuid>;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;

    /// Set creator
    fn set_created_by(&mut self, user_id: Uuid);

    /// Set updater
    fn set_updated_by(&mut self, user_id: Uuid);

    /// Touch - update updated_at và updated_by
    fn touch(&mut self, user_id: Uuid) {
        self.set_updated_by(user_id);
    }
}

/// Base audit fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFields {
    pub created_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<Uuid>,
}

impl AuditFields {
    pub fn new(user_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            created_at: now,
            created_by: user_id,
            updated_at: now,
            updated_by: user_id,
        }
    }

    pub fn touch(&mut self, user_id: Uuid) {
        self.updated_at = Utc::now();
        self.updated_by = Some(user_id);
    }
}

impl Auditable for AuditFields {
    fn created_by(&self) -> Option<Uuid> {
        self.created_by
    }

    fn updated_by(&self) -> Option<Uuid> {
        self.updated_by
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn set_created_by(&mut self, user_id: Uuid) {
        self.created_by = Some(user_id);
    }

    fn set_updated_by(&mut self, user_id: Uuid) {
        self.updated_by = Some(user_id);
    }
}

