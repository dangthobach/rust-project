use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::error::AppError;
use crate::models::User;

/// User roles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
    Admin,
    Manager,
    User,
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "manager" => Role::Manager,
            _ => Role::User,
        }
    }
}

/// Permission definition
#[derive(Debug, Clone)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: &str, action: &str) -> Self {
        Self {
            resource: resource.to_string(),
            action: action.to_string(),
        }
    }
}

/// Check if a role has a specific permission
pub fn check_permission(role: &Role, permission: &Permission) -> bool {
    match role {
        Role::Admin => true, // Admins can do everything
        Role::Manager => match (permission.resource.as_str(), permission.action.as_str()) {
            // Managers can CRUD clients
            ("clients", "create" | "read" | "update" | "delete") => true,
            // Managers can CRUD tasks
            ("tasks", "create" | "read" | "update" | "delete") => true,
            // Managers can read users
            ("users", "read") => true,
            // Managers can upload/download files
            ("files", "upload" | "download" | "read" | "delete") => true,
            // Managers can manage notifications
            ("notifications", "read" | "delete") => true,
            _ => false,
        },
        Role::User => match (permission.resource.as_str(), permission.action.as_str()) {
            // Users can read clients
            ("clients", "read") => true,
            // Users can CRUD their own tasks (checked at handler level)
            ("tasks", "create" | "read" | "update") => true,
            // Users can read their own profile
            ("users", "read") => true,
            // Users can upload/download files
            ("files", "upload" | "download" | "read") => true,
            // Users can manage their own notifications
            ("notifications", "read" | "delete") => true,
            _ => false,
        },
    }
}

/// Middleware to require a specific permission
pub async fn require_permission(
    permission: Permission,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Get user from request extensions (set by auth middleware)
    let user = request
        .extensions()
        .get::<User>()
        .ok_or(AppError::Unauthorized(
            "User not authenticated".to_string(),
        ))?;

    let role = Role::from_str(&user.role);

    // Check permission
    if !check_permission(&role, &permission) {
        tracing::warn!(
            user_id = %user.id,
            role = ?role,
            resource = %permission.resource,
            action = %permission.action,
            "Permission denied"
        );
        return Err(AppError::Forbidden(format!(
            "You don't have permission to {} {}",
            permission.action, permission.resource
        )));
    }

    Ok(next.run(request).await)
}

/// Middleware to require admin role
pub async fn require_admin(request: Request, next: Next) -> Result<Response, AppError> {
    let user = request
        .extensions()
        .get::<User>()
        .ok_or(AppError::Unauthorized(
            "User not authenticated".to_string(),
        ))?;

    let role = Role::from_str(&user.role);

    if role != Role::Admin {
        tracing::warn!(
            user_id = %user.id,
            role = ?role,
            "Admin access denied"
        );
        return Err(AppError::Forbidden(
            "Admin access required".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

/// Middleware to require manager or admin role
pub async fn require_manager_or_admin(request: Request, next: Next) -> Result<Response, AppError> {
    let user = request
        .extensions()
        .get::<User>()
        .ok_or(AppError::Unauthorized(
            "User not authenticated".to_string(),
        ))?;

    let role = Role::from_str(&user.role);

    if role != Role::Admin && role != Role::Manager {
        tracing::warn!(
            user_id = %user.id,
            role = ?role,
            "Manager/Admin access denied"
        );
        return Err(AppError::Forbidden(
            "Manager or Admin access required".to_string(),
        ));
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_has_all_permissions() {
        let admin = Role::Admin;
        assert!(check_permission(&admin, &Permission::new("clients", "create")));
        assert!(check_permission(&admin, &Permission::new("tasks", "delete")));
        assert!(check_permission(&admin, &Permission::new("users", "update")));
    }

    #[test]
    fn test_manager_permissions() {
        let manager = Role::Manager;
        assert!(check_permission(&manager, &Permission::new("clients", "create")));
        assert!(check_permission(&manager, &Permission::new("tasks", "update")));
        assert!(check_permission(&manager, &Permission::new("users", "read")));
        assert!(!check_permission(&manager, &Permission::new("users", "delete")));
    }

    #[test]
    fn test_user_permissions() {
        let user = Role::User;
        assert!(check_permission(&user, &Permission::new("clients", "read")));
        assert!(check_permission(&user, &Permission::new("tasks", "create")));
        assert!(!check_permission(&user, &Permission::new("clients", "delete")));
        assert!(!check_permission(&user, &Permission::new("tasks", "delete")));
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(Role::from_str("admin"), Role::Admin);
        assert_eq!(Role::from_str("ADMIN"), Role::Admin);
        assert_eq!(Role::from_str("manager"), Role::Manager);
        assert_eq!(Role::from_str("user"), Role::User);
        assert_eq!(Role::from_str("invalid"), Role::User);
    }
}
