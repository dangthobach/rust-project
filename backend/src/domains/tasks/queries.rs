use serde::Serialize;

use crate::authz::data_scope::DataScope;
use crate::core::cqrs::Query;
use crate::models::Task;

/// Paginated list result for `ListTasksQuery` (total matches the same filters as `tasks`).
#[derive(Debug, Clone, Serialize)]
pub struct ListTasksResult {
    pub tasks: Vec<Task>,
    pub total: i64,
}

/// Get Task by ID Query
#[derive(Debug, Clone)]
pub struct GetTaskQuery {
    pub id: String,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for GetTaskQuery {
    type Response = Option<Task>;

    fn query_name(&self) -> &'static str {
        "GetTask"
    }
}

/// List Tasks Query
#[derive(Debug, Clone)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub client_id: Option<String>,
    pub priority: Option<String>,
    /// If true: tasks due today (date(due_date)=date('now')) and not done
    pub due_today: Option<bool>,
    /// If true: tasks overdue (due_date < now) and not done
    pub overdue: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for ListTasksQuery {
    type Response = ListTasksResult;

    fn query_name(&self) -> &'static str {
        "ListTasks"
    }
}

/// Get Tasks by User Query
#[derive(Debug, Clone)]
pub struct GetTasksByUserQuery {
    pub user_id: String,
    pub status: Option<String>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for GetTasksByUserQuery {
    type Response = Vec<Task>;

    fn query_name(&self) -> &'static str {
        "GetTasksByUser"
    }
}

/// Get Tasks by Client Query
#[derive(Debug, Clone)]
pub struct GetTasksByClientQuery {
    pub client_id: String,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for GetTasksByClientQuery {
    type Response = Vec<Task>;

    fn query_name(&self) -> &'static str {
        "GetTasksByClient"
    }
}
