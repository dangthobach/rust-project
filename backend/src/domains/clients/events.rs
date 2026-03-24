use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::events::DomainEvent;

/// Client Created Event
///
/// Published when a new client is added to the system.
/// This event triggers:
/// - Welcome email notification
/// - Client list projection update
/// - Activity feed entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCreatedEvent {
    /// Aggregate ID (unique identifier for event sourcing)
    pub aggregate_id: Uuid,

    /// Client ID from database
    pub client_id: Uuid,

    /// Client name
    pub name: String,

    /// Client email
    pub email: Option<String>,

    /// Client phone
    pub phone: Option<String>,

    /// Company name (optional)
    pub company: Option<String>,

    /// Client status (active, inactive, prospect, customer)
    pub status: String,

    /// Address (optional)
    pub address: Option<String>,

    /// Position/title (optional)
    pub position: Option<String>,

    /// User ID who created this client
    pub created_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ClientCreatedEvent {
    fn event_type(&self) -> &'static str {
        "ClientCreated"
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

/// Client Updated Event
///
/// Published when client details are modified.
/// Contains a JSON object with only the changed fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientUpdatedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Client ID from database
    pub client_id: Uuid,

    /// Changed fields (JSON object with field_name: new_value)
    /// Example: {"name": "New Name", "email": "new@email.com"}
    pub changes: serde_json::Value,

    /// Previous values (for audit trail)
    pub previous_values: Option<serde_json::Value>,

    /// User ID who updated this client
    pub updated_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ClientUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "ClientUpdated"
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

/// Client Deleted Event
///
/// Published when a client is removed from the system.
/// This is a hard delete (soft delete would be ClientStatusChanged).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDeletedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Client ID from database
    pub client_id: Uuid,

    /// Client name (for audit trail)
    pub name: String,

    /// User ID who deleted this client
    pub deleted_by: String,

    /// Reason for deletion (optional)
    pub reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ClientDeletedEvent {
    fn event_type(&self) -> &'static str {
        "ClientDeleted"
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

/// Client Status Changed Event
///
/// Published when client status changes (active, inactive, prospect, customer).
/// This is useful for tracking client lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStatusChangedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Client ID from database
    pub client_id: Uuid,

    /// Previous status
    pub old_status: String,

    /// New status
    pub new_status: String,

    /// User ID who changed the status
    pub changed_by: String,

    /// Reason for status change (optional)
    pub reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ClientStatusChangedEvent {
    fn event_type(&self) -> &'static str {
        "ClientStatusChanged"
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
    fn test_client_created_event_serialization() {
        let event = ClientCreatedEvent {
            aggregate_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "John Doe".to_string(),
            email: Some("john@example.com".to_string()),
            phone: Some("+1234567890".to_string()),
            company: Some("Acme Corp".to_string()),
            status: "active".to_string(),
            address: None,
            position: Some("CEO".to_string()),
            created_by: "user_456".to_string(),
            version: 1,
            occurred_at: Utc::now(),
        };

        // Test serialization
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ClientCreated") == false); // event_type not in JSON

        // Test deserialization
        let deserialized: ClientCreatedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "John Doe");
        assert_eq!(deserialized.email, Some("john@example.com".to_string()));
    }

    #[test]
    fn test_event_type() {
        let event = ClientCreatedEvent {
            aggregate_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            name: "Test".to_string(),
            email: Some("test@test.com".to_string()),
            phone: Some("123".to_string()),
            company: None,
            status: "active".to_string(),
            address: None,
            position: None,
            created_by: "user".to_string(),
            version: 1,
            occurred_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "ClientCreated");
    }
}
