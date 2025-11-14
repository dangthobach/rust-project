use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

impl Permission {
    pub fn all() -> Vec<Self> {
        vec![
            Permission::Read,
            Permission::Write,
            Permission::Delete,
            Permission::Share,
            Permission::Admin,
        ]
    }

    /// Check if permission includes another (Admin includes all)
    pub fn includes(&self, other: &Permission) -> bool {
        matches!(self, Permission::Admin) || self == other
    }
}

/// Subject - ai có quyền (User, Group, hoặc Everyone)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Subject {
    User(Uuid),
    Group(Uuid),
    Everyone,
}

/// Access Control Entry - một entry trong ACL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    pub subject: Subject,
    pub permissions: Vec<Permission>,
    pub inherited: bool, // Permission được inherit từ parent
}

impl AccessControlEntry {
    pub fn new(subject: Subject, permissions: Vec<Permission>) -> Self {
        Self {
            subject,
            permissions,
            inherited: false,
        }
    }

    pub fn inherited(subject: Subject, permissions: Vec<Permission>) -> Self {
        Self {
            subject,
            permissions,
            inherited: true,
        }
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission) || self.permissions.contains(&Permission::Admin)
    }
}

/// Securable trait - entity có thể set permissions
pub trait Securable: Send + Sync {
    /// Get ACL (Access Control List)
    fn get_acl(&self) -> &[AccessControlEntry];

    /// Set ACL
    fn set_acl(&mut self, acl: Vec<AccessControlEntry>);

    /// Check if user has permission
    fn has_permission(&self, user_id: Uuid, permission: Permission) -> bool {
        let acl = self.get_acl();

        // Check direct permissions
        for entry in acl {
            let matches = match &entry.subject {
                Subject::User(id) => *id == user_id,
                Subject::Group(_) => {
                    // TODO: Check if user is in group
                    false
                }
                Subject::Everyone => true,
            };

            if matches && entry.has_permission(&permission) {
                return true;
            }
        }

        false
    }

    /// Add permission cho subject
    fn grant_permission(&mut self, subject: Subject, permission: Permission) {
        let mut acl = self.get_acl().to_vec();

        // Find existing entry for subject
        if let Some(entry) = acl.iter_mut().find(|e| e.subject == subject) {
            if !entry.permissions.contains(&permission) {
                entry.permissions.push(permission);
            }
        } else {
            acl.push(AccessControlEntry::new(subject, vec![permission]));
        }

        self.set_acl(acl);
    }

    /// Remove permission từ subject
    fn revoke_permission(&mut self, subject: Subject, permission: Permission) {
        let mut acl = self.get_acl().to_vec();

        if let Some(entry) = acl.iter_mut().find(|e| e.subject == subject) {
            entry.permissions.retain(|p| p != &permission);
            if entry.permissions.is_empty() {
                acl.retain(|e| e.subject != subject);
            }
        }

        self.set_acl(acl);
    }
}

