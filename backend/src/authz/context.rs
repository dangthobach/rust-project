use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::authz::permissions as perm;

/// Authenticated principal + effective permission set (loaded once per request; may be cached briefly).
/// `accessible_branches` is the expanded tree from `user_branches` (see `branch_loader`).
#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: Uuid,
    permissions: Arc<BTreeSet<String>>,
    accessible_branches: Arc<BTreeSet<String>>,
}

impl AuthContext {
    pub fn new(
        user_id: Uuid,
        permissions: BTreeSet<String>,
        accessible_branches: BTreeSet<String>,
    ) -> Self {
        Self {
            user_id,
            permissions: Arc::new(permissions),
            accessible_branches: Arc::new(accessible_branches),
        }
    }

    pub fn from_arcs(
        user_id: Uuid,
        permissions: Arc<BTreeSet<String>>,
        accessible_branches: Arc<BTreeSet<String>>,
    ) -> Self {
        Self {
            user_id,
            permissions,
            accessible_branches,
        }
    }

    #[inline]
    pub fn accessible_branches(&self) -> &Arc<BTreeSet<String>> {
        &self.accessible_branches
    }

    #[inline]
    pub fn permission_codes(&self) -> &Arc<BTreeSet<String>> {
        &self.permissions
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
