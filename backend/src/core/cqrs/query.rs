use async_trait::async_trait;
use std::sync::Arc;
use crate::error::AppError;

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

    /// Dispatch query with handler
    pub async fn dispatch_with_handler<Q, H>(
        &self,
        query: Q,
        handler: Arc<H>,
    ) -> Result<Q::Response, AppError>
    where
        Q: Query,
        H: QueryHandler<Q>,
        H::Error: Into<AppError>,
    {
        // 1. Execute query handler
        let result = handler.handle(query).await
            .map_err(|e| e.into())?;

        // 2. Return result
        Ok(result)
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

