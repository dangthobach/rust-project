use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::events::DomainEvent;

/// User Created Event
///
/// Published when a new user is registered or created by admin.
/// Triggers:
/// - Welcome email
/// - User onboarding workflow
/// - Activity feed entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreatedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Username
    pub username: String,

    /// Email address
    pub email: String,

    /// Full name
    pub name: String,

    /// User role (Admin, Manager, User)
    pub role: String,

    /// Registration method (manual, oauth, invite)
    pub registration_method: String,

    /// Created by (user_id if created by admin, or "self" if self-registration)
    pub created_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserCreatedEvent {
    fn event_type(&self) -> &'static str {
        "UserCreated"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Updated Event
///
/// Published when user profile is updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUpdatedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Changed fields
    pub changes: serde_json::Value,

    /// Previous values
    pub previous_values: Option<serde_json::Value>,

    /// User ID who made the update
    pub updated_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "UserUpdated"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Logged In Event
///
/// Published on successful login.
/// Useful for:
/// - Security monitoring
/// - User activity analytics
/// - Last login tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedInEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Username
    pub username: String,

    /// IP address
    pub ip_address: String,

    /// User agent (browser info)
    pub user_agent: String,

    /// Login method (password, oauth, 2fa)
    pub login_method: String,

    /// Was this login successful?
    pub success: bool,

    /// Failure reason (if success = false)
    pub failure_reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserLoggedInEvent {
    fn event_type(&self) -> &'static str {
        "UserLoggedIn"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Logged Out Event
///
/// Published when user explicitly logs out.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedOutEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Was this logout for all devices?
    pub all_devices: bool,

    /// Number of sessions terminated
    pub sessions_terminated: usize,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserLoggedOutEvent {
    fn event_type(&self) -> &'static str {
        "UserLoggedOut"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Password Changed Event
///
/// Published when user changes their password.
/// Triggers:
/// - Security notification email
/// - Invalidate all sessions (force re-login)
/// - Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPasswordChangedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Username
    pub username: String,

    /// Who initiated the change (user_id or "self")
    pub changed_by: String,

    /// Was this a password reset (vs normal change)?
    pub is_reset: bool,

    /// IP address where change was made
    pub ip_address: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserPasswordChangedEvent {
    fn event_type(&self) -> &'static str {
        "UserPasswordChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Role Changed Event
///
/// Published when user role is upgraded/downgraded.
/// Critical security event that requires audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRoleChangedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Username
    pub username: String,

    /// Previous role
    pub old_role: String,

    /// New role
    pub new_role: String,

    /// Admin user who made the change
    pub changed_by: String,

    /// Reason for role change
    pub reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserRoleChangedEvent {
    fn event_type(&self) -> &'static str {
        "UserRoleChanged"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

/// User Deleted Event
///
/// Published when user account is deleted.
/// GDPR compliance event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// User ID from database
    pub user_id: String,

    /// Username
    pub username: String,

    /// Email (for audit)
    pub email: String,

    /// Admin who deleted the user
    pub deleted_by: String,

    /// Deletion reason
    pub reason: Option<String>,

    /// Was personal data anonymized (GDPR)?
    pub anonymized: bool,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for UserDeletedEvent {
    fn event_type(&self) -> &'static str {
        "UserDeleted"
    }

    fn aggregate_id(&self) -> Uuid {
        self.aggregate_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn version(&self) -> i64 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_created_event() {
        let event = UserCreatedEvent {
            aggregate_id: Uuid::new_v4(),
            user_id: "user_123".to_string(),
            username: "johndoe".to_string(),
            email: "john@example.com".to_string(),
            name: "John Doe".to_string(),
            role: "User".to_string(),
            registration_method: "manual".to_string(),
            created_by: "self".to_string(),
            version: 1,
            occurred_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "UserCreated");
        assert_eq!(event.username, "johndoe");
    }

    #[test]
    fn test_user_logged_in_event() {
        let event = UserLoggedInEvent {
            aggregate_id: Uuid::new_v4(),
            user_id: "user_123".to_string(),
            username: "johndoe".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            login_method: "password".to_string(),
            success: true,
            failure_reason: None,
            version: 1,
            occurred_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "UserLoggedIn");
        assert!(event.success);
    }
}
