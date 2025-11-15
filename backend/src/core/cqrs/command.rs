use async_trait::async_trait;
use validator::Validate;
use std::sync::Arc;
use crate::error::AppError;

/// Command - request để thay đổi state
pub trait Command: Validate + Send + Sync {
    type Response: Send + Sync;

    /// Command name (for logging/debugging)
    fn command_name(&self) -> &'static str;
}

/// Command Handler - xử lý commands
#[async_trait]
pub trait CommandHandler<C: Command>: Send + Sync {
    type Error: Send + Sync + 'static;

    /// Handle command
    async fn handle(&self, command: C) -> Result<C::Response, Self::Error>;
}

/// Command Bus - dispatch commands đến handlers
/// Note: This is a simplified implementation. For production, consider using a type map
/// or dependency injection container
pub struct CommandBus {
    // In real implementation, this would be a type map of handlers
}

impl CommandBus {
    pub fn new() -> Self {
        Self {}
    }

    /// Dispatch command with validation
    /// This is a helper that validates and then delegates to specific handler
    pub async fn dispatch_with_handler<C, H>(
        &self,
        command: C,
        handler: Arc<H>,
    ) -> Result<C::Response, AppError>
    where
        C: Command,
        H: CommandHandler<C>,
        H::Error: Into<AppError>,
    {
        // 1. Validate command
        command.validate()
            .map_err(|e| AppError::ValidationError(e.to_string()))?;

        // 2. Execute handler (in production, this would be in a transaction)
        let result = handler.handle(command).await
            .map_err(|e| e.into())?;

        // 3. Publish events (if any) - handled by handlers themselves
        
        Ok(result)
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

