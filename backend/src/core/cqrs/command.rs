use async_trait::async_trait;
use std::error::Error;
use validator::Validate;

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
pub struct CommandBus {
    // Registry of handlers
    // Will use type map or similar pattern
}

impl CommandBus {
    pub fn new() -> Self {
        Self {}
    }

    /// Register command handler
    pub fn register<C: Command, H: CommandHandler<C>>(&mut self, _handler: H) {
        // Register handler
        todo!()
    }

    /// Dispatch command
    pub async fn dispatch<C: Command>(
        &self,
        command: C,
    ) -> Result<C::Response, Box<dyn Error>> {
        // 1. Validate command
        command.validate()?;

        // 2. Find handler
        // 3. Execute in transaction
        // 4. Publish events
        // 5. Return result
        todo!()
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

