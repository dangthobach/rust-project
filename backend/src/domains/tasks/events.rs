use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::events::DomainEvent;

/// Task Created Event
///
/// Published when a new task is created.
/// Triggers:
/// - Email notification to assignee
/// - Task list projection update
/// - Activity feed entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreatedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Task title
    pub title: String,

    /// Task description
    pub description: Option<String>,

    /// Task status (todo, in_progress, done, cancelled)
    pub status: String,

    /// Task priority (low, medium, high, urgent)
    pub priority: String,

    /// Assigned user ID
    pub assigned_to: String,

    /// Related client ID (optional)
    pub client_id: Option<String>,

    /// Due date (optional)
    pub due_date: Option<DateTime<Utc>>,

    /// User ID who created this task
    pub created_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskCreatedEvent {
    fn event_type(&self) -> &'static str {
        "TaskCreated"
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

/// Task Updated Event
///
/// Published when task details are modified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdatedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Changed fields (JSON object)
    pub changes: serde_json::Value,

    /// Previous values (for audit)
    pub previous_values: Option<serde_json::Value>,

    /// User ID who updated this task
    pub updated_by: String,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskUpdatedEvent {
    fn event_type(&self) -> &'static str {
        "TaskUpdated"
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

/// Task Completed Event
///
/// Published when a task is marked as completed.
/// This is a special event (not just TaskUpdated with status=done)
/// because completion often triggers workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Task title (for notifications)
    pub title: String,

    /// User ID who completed this task
    pub completed_by: String,

    /// When the task was completed
    pub completed_at: DateTime<Utc>,

    /// Completion notes (optional)
    pub notes: Option<String>,

    /// Was it completed on time?
    pub on_time: bool,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskCompletedEvent {
    fn event_type(&self) -> &'static str {
        "TaskCompleted"
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

/// Task Assigned Event
///
/// Published when a task is assigned to a different user.
/// Triggers:
/// - Email notification to new assignee
/// - Notification to old assignee (if changed)
/// - Activity feed entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Task title (for notifications)
    pub title: String,

    /// Previous assignee (if any)
    pub previous_assignee: Option<String>,

    /// New assignee
    pub new_assignee: String,

    /// User ID who made the assignment
    pub assigned_by: String,

    /// Assignment reason/notes (optional)
    pub notes: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskAssignedEvent {
    fn event_type(&self) -> &'static str {
        "TaskAssigned"
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

/// Task Deleted Event
///
/// Published when a task is removed from the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDeletedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Task title (for audit)
    pub title: String,

    /// Task status at deletion
    pub status: String,

    /// User ID who deleted this task
    pub deleted_by: String,

    /// Deletion reason (optional)
    pub reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskDeletedEvent {
    fn event_type(&self) -> &'static str {
        "TaskDeleted"
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

/// Task Priority Changed Event
///
/// Published when task priority is escalated or de-escalated.
/// Useful for tracking priority changes over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPriorityChangedEvent {
    /// Aggregate ID
    pub aggregate_id: Uuid,

    /// Task ID from database
    pub task_id: String,

    /// Task title
    pub title: String,

    /// Previous priority
    pub old_priority: String,

    /// New priority
    pub new_priority: String,

    /// User ID who changed priority
    pub changed_by: String,

    /// Reason for change (optional)
    pub reason: Option<String>,

    /// Event version
    pub version: i64,

    /// When this event occurred
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for TaskPriorityChangedEvent {
    fn event_type(&self) -> &'static str {
        "TaskPriorityChanged"
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
    fn test_task_created_event() {
        let event = TaskCreatedEvent {
            aggregate_id: Uuid::new_v4(),
            task_id: "task_123".to_string(),
            title: "Complete documentation".to_string(),
            description: Some("Write comprehensive docs".to_string()),
            status: "todo".to_string(),
            priority: "high".to_string(),
            assigned_to: "user_456".to_string(),
            client_id: Some("client_789".to_string()),
            due_date: None,
            created_by: "user_123".to_string(),
            version: 1,
            occurred_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "TaskCreated");
        assert_eq!(event.title, "Complete documentation");
    }

    #[test]
    fn test_task_completed_event() {
        let event = TaskCompletedEvent {
            aggregate_id: Uuid::new_v4(),
            task_id: "task_123".to_string(),
            title: "Test task".to_string(),
            completed_by: "user_456".to_string(),
            completed_at: Utc::now(),
            notes: Some("All tests passing".to_string()),
            on_time: true,
            version: 2,
            occurred_at: Utc::now(),
        };

        assert_eq!(event.event_type(), "TaskCompleted");
        assert!(event.on_time);
    }
}
