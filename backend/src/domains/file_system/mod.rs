pub mod aggregates;
pub mod commands;
pub mod queries;
pub mod events;
pub mod projections;
pub mod handlers;
pub mod read_models;
pub mod repository;
pub mod services;

// Re-export only what's needed
pub use read_models::{FileView, FolderTreeView};

