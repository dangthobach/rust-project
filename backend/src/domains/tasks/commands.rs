use crate::core::cqrs::Command;
use crate::models::Task;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Create Task Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateTaskCommand {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    
    pub description: Option<String>,
    
    #[validate(length(min = 1))]
    pub status: String,
    
    pub priority: Option<String>,
    pub assigned_to: Option<i32>,
    pub client_id: Option<i32>,
    pub due_date: Option<String>,
}

impl Command for CreateTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "CreateTask"
    }
}

/// Update Task Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateTaskCommand {
    pub id: i32,
    
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<i32>,
    pub client_id: Option<i32>,
    pub due_date: Option<String>,
}

impl Command for UpdateTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "UpdateTask"
    }
}

/// Delete Task Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteTaskCommand {
    pub id: i32,
}

impl Command for DeleteTaskCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "DeleteTask"
    }
}

/// Complete Task Command
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CompleteTaskCommand {
    pub id: i32,
}

impl Command for CompleteTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "CompleteTask"
    }
}
