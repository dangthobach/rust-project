use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::cqrs::Query;
use crate::core::shared::Pagination;

// Re-export read models
pub use crate::domains::file_system::read_models::{FileView, FolderTreeView};

/// Get File Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFileQuery {
    pub file_id: Uuid,
    pub user_id: Uuid, // User requesting the file (for permission check)
}

impl Query for GetFileQuery {
    type Response = FileView;

    fn query_name(&self) -> &'static str {
        "get_file"
    }
}

/// List Files Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFilesQuery {
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid, // User requesting the list (for permission filtering)
    pub pagination: Pagination,
}

impl Query for ListFilesQuery {
    type Response = Vec<FileView>;

    fn query_name(&self) -> &'static str {
        "list_files"
    }
}

/// Get Folder Tree Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFolderTreeQuery {
    pub folder_id: Uuid,
    pub depth: Option<i32>,
    pub user_id: Uuid, // User requesting the tree (for permission filtering)
}

impl Query for GetFolderTreeQuery {
    type Response = FolderTreeView;

    fn query_name(&self) -> &'static str {
        "get_folder_tree"
    }
}

/// Search Files Query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilesQuery {
    pub query: String,
    pub user_id: Uuid, // User performing the search (for permission filtering)
    pub pagination: Pagination,
}

impl Query for SearchFilesQuery {
    type Response = Vec<FileView>;

    fn query_name(&self) -> &'static str {
        "search_files"
    }
}

// Read models are defined in read_models.rs and re-exported above

