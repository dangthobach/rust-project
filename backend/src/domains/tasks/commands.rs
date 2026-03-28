use crate::authz::data_scope::DataScope;
use crate::core::cqrs::Command;
use crate::models::Task;
use validator::Validate;

/// Create Task Command
#[derive(Debug, Clone, Validate)]
pub struct CreateTaskCommand {
    #[validate(length(min = 1, max = 255))]
    pub title: String,

    pub description: Option<String>,

    pub status: Option<String>,

    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub client_id: Option<String>,
    pub due_date: Option<String>,
    pub created_by: Option<String>,
    pub actor_id: Option<String>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Command for CreateTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "CreateTask"
    }
}

/// Update Task Command
#[derive(Debug, Clone, Validate)]
pub struct UpdateTaskCommand {
    #[validate(length(min = 1))]
    pub id: String,

    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,

    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assigned_to: Option<String>,
    pub client_id: Option<String>,
    pub due_date: Option<String>,
    pub actor_id: Option<String>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Command for UpdateTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "UpdateTask"
    }
}

/// Delete Task Command
#[derive(Debug, Clone, Validate)]
pub struct DeleteTaskCommand {
    #[validate(length(min = 1))]
    pub id: String,
    pub actor_id: Option<String>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Command for DeleteTaskCommand {
    type Response = ();

    fn command_name(&self) -> &'static str {
        "DeleteTask"
    }
}

/// Complete Task Command
#[derive(Debug, Clone, Validate)]
pub struct CompleteTaskCommand {
    #[validate(length(min = 1))]
    pub id: String,
    pub actor_id: Option<String>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Command for CompleteTaskCommand {
    type Response = Task;

    fn command_name(&self) -> &'static str {
        "CompleteTask"
    }
}
