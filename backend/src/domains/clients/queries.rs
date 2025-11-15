use crate::core::cqrs::Query;
use crate::models::Client;
use serde::{Deserialize, Serialize};

/// Get Client by ID Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetClientQuery {
    pub id: i32,
}

impl Query for GetClientQuery {
    type Response = Option<Client>;

    fn query_name(&self) -> &'static str {
        "GetClient"
    }
}

/// List Clients Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListClientsQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Query for ListClientsQuery {
    type Response = Vec<Client>;

    fn query_name(&self) -> &'static str {
        "ListClients"
    }
}

/// Search Clients Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchClientsQuery {
    pub search_term: String,
    pub limit: Option<i64>,
}

impl Query for SearchClientsQuery {
    type Response = Vec<Client>;

    fn query_name(&self) -> &'static str {
        "SearchClients"
    }
}
