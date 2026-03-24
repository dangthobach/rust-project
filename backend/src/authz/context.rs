use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::authz::permissions as perm;

/// Authenticated principal + effective permission set (loaded once per request; may be cached briefly).
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
    permissions: Arc<BTreeSet<String>>,
}

impl AuthContext {
    pub fn new(user_id: Uuid, permissions: BTreeSet<String>) -> Self {
        Self {
            user_id,
            permissions: Arc::new(permissions),
        }
    }

    pub fn from_arc(user_id: Uuid, permissions: Arc<BTreeSet<String>>) -> Self {
        Self {
            user_id,
            permissions,
        }
    }

    #[inline]
    pub fn has(&self, code: &str) -> bool {
        self.permissions.contains(code)
    }

    #[inline]
    pub fn superuser(&self) -> bool {
        self.has(perm::SYSTEM_SUPERUSER)
    }
}
