// pub mod file_system; // Disabled - requires PostgreSQL
pub mod cqrs_handlers;
pub mod file_system_cqrs;
pub mod rbac_cqrs;
pub mod users_cqrs;
pub mod tasks_cqrs;

// Re-export only what's needed
// pub use file_system::*;

