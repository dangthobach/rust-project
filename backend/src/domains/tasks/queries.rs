use crate::core::cqrs::Query;
use crate::models::Task;
use serde::{Deserialize, Serialize};

/// Get Task by ID Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTaskQuery {
    pub id: String,
}

impl Query for GetTaskQuery {
    type Response = Option<Task>;

    fn query_name(&self) -> &'static str {
        "GetTask"
    }
}

/// List Tasks Query
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Query for ListTasksQuery {
    type Response = Vec<Task>;

    fn query_name(&self) -> &'static str {
        "ListTasks"
    }
}

/// Get Tasks by User Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTasksByUserQuery {
    pub user_id: String,
    pub status: Option<String>,
}

impl Query for GetTasksByUserQuery {
    type Response = Vec<Task>;

    fn query_name(&self) -> &'static str {
        "GetTasksByUser"
    }
}

/// Get Tasks by Client Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTasksByClientQuery {
    pub client_id: String,
}

impl Query for GetTasksByClientQuery {
    type Response = Vec<Task>;

    fn query_name(&self) -> &'static str {
        "GetTasksByClient"
    }
}
