use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::CommandHandler;
use crate::core::domain::{AggregateRoot, Entity, Repository};
use crate::core::events::{EventBus, EventBusExt, EventEnvelope};
use crate::core::shared::SoftDeletable;
use crate::domains::file_system::aggregates::{File, Folder};
use crate::domains::file_system::commands::*;
// FileSystemEvent is used in EventEnvelope, no need to import directly
use crate::domains::file_system::services::FileSystemService;

// Type alias for handler errors (kept for future use)
#[allow(dead_code)]
type HandlerError = Box<dyn std::error::Error + Send + Sync>;

// Helper function to create boxed errors from strings (kept for future use)
#[allow(dead_code)]
fn err(msg: impl Into<String>) -> HandlerError {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
}

/// Create File Handler
pub struct CreateFileHandler {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
}

impl CreateFileHandler {
    pub fn new(
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
    ) -> Self {
        Self {
            file_repo,
            event_bus,
            service,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateFileCommand> for CreateFileHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: CreateFileCommand) -> Result<Uuid, Self::Error> {
        // Check if name exists
        if self.service.name_exists(&cmd.name, cmd.parent_id, None).await? {
            return Err("File with this name already exists".into());
        }

        // Calculate path
        let path = self.service.calculate_path(&cmd.name, cmd.parent_id).await?;

        // Create aggregate with owner_id from command
        let mut file = File::create(
            cmd.name.clone(),
            path,
            cmd.parent_id,
            cmd.size,
            cmd.mime_type,
            cmd.owner_id,
        ).map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)) })?;

        // Save (persist events)
        self.file_repo.save(&mut file).await?;

        // Publish events
        for event in file.uncommitted_events() {
            let envelope = EventEnvelope::new(
                file.id().clone(),
                "file".to_string(),
                event.clone(),
                serde_json::json!({}),
            );
            self.event_bus.publish(envelope).await?;
        }

        Ok(file.id().clone())
    }
}

/// Move File Handler
pub struct MoveFileHandler {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
}

impl MoveFileHandler {
    pub fn new(
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
    ) -> Self {
        Self {
            file_repo,
            event_bus,
            service,
        }
    }
}

#[async_trait]
impl CommandHandler<MoveFileCommand> for MoveFileHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: MoveFileCommand) -> Result<(), Self::Error> {
        // Load file
        let mut file = self
            .file_repo
            .find_by_id(&cmd.file_id)
            .await?
            .ok_or_else(|| err("File not found"))?;

        // Check permission
        if !self
            .service
            .check_permission(cmd.file_id, cmd.moved_by, crate::core::shared::Permission::Write)
            .await?
        {
            return Err(err("Permission denied"));
        }

        // Calculate new path
        let new_path = self
            .service
            .calculate_path(&file.name, cmd.new_parent_id)
            .await?;

        // Move
        file.move_to(cmd.new_parent_id, new_path, cmd.moved_by);

        // Save
        self.file_repo.save(&mut file).await?;

        // Publish events
        for event in file.uncommitted_events() {
            let envelope = EventEnvelope::new(
                file.id().clone(),
                "file".to_string(),
                event.clone(),
                serde_json::json!({}),
            );
            self.event_bus.publish(envelope).await?;
        }

        Ok(())
    }
}

/// Rename File Handler
pub struct DeleteFileHandler {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
}

impl DeleteFileHandler {
    pub fn new(
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
    ) -> Self {
        Self {
            file_repo,
            event_bus,
            service,
        }
    }
}

#[async_trait]
impl CommandHandler<DeleteFileCommand> for DeleteFileHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: DeleteFileCommand) -> Result<(), Self::Error> {
        // Load file
        let mut file = self
            .file_repo
            .find_by_id(&cmd.file_id)
            .await?
            .ok_or_else(|| err("File not found"))?;

        // Check permission
        if !self
            .service
            .check_permission(cmd.file_id, cmd.deleted_by, crate::core::shared::Permission::Delete)
            .await?
        {
            return Err(err("Permission denied"));
        }

        // Delete (soft delete)
        file.mark_as_deleted(cmd.deleted_by);

        // Save
        self.file_repo.save(&mut file).await?;

        // Publish events
        for event in file.uncommitted_events() {
            let envelope = EventEnvelope::new(
                file.id().clone(),
                "file".to_string(),
                event.clone(),
                serde_json::json!({}),
            );
            self.event_bus.publish(envelope).await?;
        }

        Ok(())
    }
}

/// Rename File Handler
pub struct RenameFileHandler {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
}

impl RenameFileHandler {
    pub fn new(
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
    ) -> Self {
        Self {
            file_repo,
            event_bus,
            service,
        }
    }
}

#[async_trait]
impl CommandHandler<RenameFileCommand> for RenameFileHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: RenameFileCommand) -> Result<(), Self::Error> {
        // Load file
        let mut file = self
            .file_repo
            .find_by_id(&cmd.file_id)
            .await?
            .ok_or_else(|| err("File not found"))?;

        // Check permission
        if !self
            .service
            .check_permission(cmd.file_id, cmd.renamed_by, crate::core::shared::Permission::Write)
            .await?
        {
            return Err(err("Permission denied"));
        }

        // Check if new name exists in same parent
        if self.service.name_exists(&cmd.new_name, file.parent_id, Some(file.id)).await? {
            return Err(err("A file/folder with the same name already exists"));
        }

        // Calculate new path
        let new_path = self
            .service
            .calculate_path(&cmd.new_name, file.parent_id)
            .await?;

        // Rename
        file.rename(cmd.new_name, new_path, cmd.renamed_by);

        // Save
        self.file_repo.save(&mut file).await?;

        // Publish events
        for event in file.uncommitted_events() {
            let envelope = EventEnvelope::new(
                file.id().clone(),
                "file".to_string(),
                event.clone(),
                serde_json::json!({}),
            );
            self.event_bus.publish(envelope).await?;
        }
        file.mark_events_as_committed();

        Ok(())
    }
}

/// Create Folder Handler
pub struct CreateFolderHandler {
    folder_repo: Arc<crate::core::infrastructure::PostgresRepository<Folder>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
}

impl CreateFolderHandler {
    pub fn new(
        folder_repo: Arc<crate::core::infrastructure::PostgresRepository<Folder>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
    ) -> Self {
        Self {
            folder_repo,
            event_bus,
            service,
        }
    }
}

#[async_trait]
impl CommandHandler<CreateFolderCommand> for CreateFolderHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: CreateFolderCommand) -> Result<Uuid, Self::Error> {
        // Check if name exists
        if self.service.name_exists(&cmd.name, cmd.parent_id, None).await? {
            return Err(err("Folder with this name already exists"));
        }

        // Calculate path
        let path = self.service.calculate_path(&cmd.name, cmd.parent_id).await?;

        // Create aggregate with owner_id from command
        let mut folder = Folder::create(cmd.name.clone(), path, cmd.parent_id, cmd.owner_id)
            .map_err(|e| err(format!("{}", e)))?;

        // Save
        self.folder_repo.save(&mut folder).await?;

        // Publish events
        for event in folder.uncommitted_events() {
            let envelope = EventEnvelope::new(
                folder.id().clone(),
                "folder".to_string(),
                event.clone(),
                serde_json::json!({}),
            );
            self.event_bus.publish(envelope).await?;
        }

        Ok(folder.id().clone())
    }
}

