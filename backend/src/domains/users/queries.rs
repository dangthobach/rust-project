use crate::core::cqrs::Query;
use crate::models::User;
use serde::{Deserialize, Serialize};

/// Get User by ID Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserQuery {
    pub id: String,
}

impl Query for GetUserQuery {
    type Response = Option<User>;

    fn query_name(&self) -> &'static str {
        "GetUser"
    }
}

/// Get User by Email Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserByEmailQuery {
    pub email: String,
}

impl Query for GetUserByEmailQuery {
    type Response = Option<User>;

    fn query_name(&self) -> &'static str {
        "GetUserByEmail"
    }
}

/// List Users Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsersQuery {
    pub role: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Query for ListUsersQuery {
    type Response = Vec<User>;

    fn query_name(&self) -> &'static str {
        "ListUsers"
    }
}
