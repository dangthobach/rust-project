use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::authz::permissions as perm;
use crate::error::AppError;

/// Authenticated principal + effective permission set (loaded once per request).
/// `accessible_branches` is the expanded subtree from `user_branches` (see `branch_loader`).
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
        Self { user_id, permissions, accessible_branches }
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    #[inline]
    pub fn accessible_branches(&self) -> &Arc<BTreeSet<String>> {
        &self.accessible_branches
    }

    #[inline]
    pub fn permission_codes(&self) -> &Arc<BTreeSet<String>> {
        &self.permissions
    }

    /// Returns `true` if the caller holds `code`.
    #[inline]
    pub fn has(&self, code: &str) -> bool {
        self.permissions.contains(code)
    }

    /// Returns `true` if the caller holds `system.superuser`.
    #[inline]
    pub fn superuser(&self) -> bool {
        self.has(perm::SYSTEM_SUPERUSER)
    }

    // ── Guard helpers ─────────────────────────────────────────────────────────

    /// Succeeds if the caller is a superuser **or** holds `code`.
    /// Returns `AppError::Forbidden` otherwise.
    #[inline]
    pub fn require(&self, code: &str) -> Result<(), AppError> {
        if self.superuser() || self.has(code) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!("Permission required: {code}")))
        }
    }

    /// Succeeds if the caller is a superuser **or** holds **any** of `codes`.
    /// Returns `AppError::Forbidden` (listing all required codes) otherwise.
    #[inline]
    pub fn require_any(&self, codes: &[&str]) -> Result<(), AppError> {
        if self.superuser() || codes.iter().any(|c| self.has(c)) {
            Ok(())
        } else {
            Err(AppError::Forbidden(format!(
                "One of these permissions is required: {}",
                codes.join(", ")
            )))
        }
    }
}
