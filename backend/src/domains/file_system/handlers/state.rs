use sqlx::SqlitePool;
use std::sync::Arc;

use crate::core::events::{EventBus, PostgresEventStore};
use crate::core::infrastructure::PostgresRepository;
use crate::domains::file_system::aggregates::{File, Folder};
use crate::domains::file_system::handlers::*;
use crate::domains::file_system::services::FileSystemService;

/// Handler State - contains all dependencies for handlers
/// Used in API routes for dependency injection
#[derive(Clone)]
pub struct HandlerState {
    pub pool: SqlitePool,
    pub file_repo: Arc<PostgresRepository<File>>,
    pub folder_repo: Arc<PostgresRepository<Folder>>,
    pub event_bus: Arc<dyn EventBus + Send + Sync>,
    pub service: Arc<FileSystemService>,
}

impl HandlerState {
    pub fn new(
        pool: SqlitePool,
        event_bus: Arc<dyn EventBus + Send + Sync>,
    ) -> Self {
        let event_store = PostgresEventStore::new(pool.clone());
        
        let file_repo = Arc::new(PostgresRepository::new(
            pool.clone(),
            event_store.clone(),
            "file".to_string(),
        ));
        
        let folder_repo = Arc::new(PostgresRepository::new(
            pool.clone(),
            event_store.clone(),
            "folder".to_string(),
        ));
        
        let service = Arc::new(FileSystemService::new(
            pool.clone(),
            file_repo.clone(),
            folder_repo.clone(),
        ));

        Self {
            pool,
            file_repo,
            folder_repo,
            event_bus,
            service,
        }
    }

    /// Create command handlers
    pub fn create_file_handler(&self) -> CreateFileHandler {
        CreateFileHandler::new(
            self.file_repo.clone(),
            self.event_bus.clone(),
            self.service.clone(),
        )
    }

    pub fn move_file_handler(&self) -> MoveFileHandler {
        MoveFileHandler::new(
            self.file_repo.clone(),
            self.event_bus.clone(),
            self.service.clone(),
        )
    }

    pub fn delete_file_handler(&self) -> DeleteFileHandler {
        DeleteFileHandler::new(
            self.file_repo.clone(),
            self.event_bus.clone(),
            self.service.clone(),
        )
    }

    pub fn create_folder_handler(&self) -> CreateFolderHandler {
        CreateFolderHandler::new(
            self.folder_repo.clone(),
            self.event_bus.clone(),
            self.service.clone(),
        )
    }

    /// Create query handlers
    pub fn get_file_handler(&self) -> GetFileHandler {
        GetFileHandler::new(self.pool.clone())
    }

    pub fn list_files_handler(&self) -> ListFilesHandler {
        ListFilesHandler::new(self.pool.clone())
    }

    pub fn get_folder_tree_handler(&self) -> GetFolderTreeHandler {
        GetFolderTreeHandler::new(self.pool.clone())
    }

    pub fn search_files_handler(&self) -> SearchFilesHandler {
        SearchFilesHandler::new(self.pool.clone())
    }

    pub fn rename_file_handler(&self) -> RenameFileHandler {
        RenameFileHandler::new(
            self.file_repo.clone(),
            self.event_bus.clone(),
            self.service.clone(),
        )
    }
}

