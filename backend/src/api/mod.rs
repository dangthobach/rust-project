// pub mod file_system; // Disabled - requires PostgreSQL
pub mod cqrs_handlers;

// Re-export only what's needed
// pub use file_system::*;
pub use cqrs_handlers::*;

