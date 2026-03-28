use crate::authz::data_scope::DataScope;
use crate::core::cqrs::Query;
use crate::models::Client;
use serde::{Deserialize, Serialize};

/// Get Client by ID Query
#[derive(Debug, Clone)]
pub struct GetClientQuery {
    pub id: String,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for GetClientQuery {
    type Response = Option<Client>;

    fn query_name(&self) -> &'static str {
        "GetClient"
    }
}

/// List Clients Query
#[derive(Debug, Clone)]
pub struct ListClientsQuery {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for ListClientsQuery {
    type Response = Vec<Client>;

    fn query_name(&self) -> &'static str {
        "ListClients"
    }
}

/// Search Clients Query
#[derive(Debug, Clone)]
pub struct SearchClientsQuery {
    pub search_term: String,
    pub limit: Option<i64>,
    pub data_scope: DataScope,
    pub actor_user_id: String,
}

impl Query for SearchClientsQuery {
    type Response = Vec<Client>;

    fn query_name(&self) -> &'static str {
        "SearchClients"
    }
}

/// Query params from HTTP (mapped to `ListClientsQuery` in API layer).
#[derive(Debug, Deserialize, Serialize)]
pub struct ListClientsParams {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
