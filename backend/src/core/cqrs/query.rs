use async_trait::async_trait;
use std::error::Error;

/// Query - request để đọc data (không thay đổi state)
pub trait Query: Send + Sync {
    type Response: Send + Sync;

    /// Query name (for logging/debugging)
    fn query_name(&self) -> &'static str;
}

/// Query Handler - xử lý queries
#[async_trait]
pub trait QueryHandler<Q: Query>: Send + Sync {
    type Error: Send + Sync + 'static;

    /// Handle query
    async fn handle(&self, query: Q) -> Result<Q::Response, Self::Error>;
}

/// Query Bus - dispatch queries đến handlers
pub struct QueryBus {
    // Registry of handlers
}

impl QueryBus {
    pub fn new() -> Self {
        Self {}
    }

    /// Register query handler
    pub fn register<Q: Query, H: QueryHandler<Q>>(&mut self, _handler: H) {
        // Register handler
        todo!()
    }

    /// Dispatch query
    pub async fn dispatch<Q: Query>(&self, _query: Q) -> Result<Q::Response, Box<dyn Error>> {
        // 1. Find handler
        // 2. Execute query
        // 3. Return result
        todo!()
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

