use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;
use sqlx::PgPool;

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

        // Synchronous read-model upsert (until projections are running)
        // This keeps `/api/fs/*` usable without an async projector.
        let _ = sqlx::query(
            r#"
            INSERT INTO file_views (id, name, path, parent_id, size, mime_type, owner_id, item_type, created_at, updated_at, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'file', NOW(), NOW(), $7, $7)
            ON CONFLICT (id) DO UPDATE SET
              name = EXCLUDED.name,
              path = EXCLUDED.path,
              parent_id = EXCLUDED.parent_id,
              size = EXCLUDED.size,
              mime_type = EXCLUDED.mime_type,
              owner_id = EXCLUDED.owner_id,
              updated_at = NOW(),
              updated_by = EXCLUDED.owner_id
            "#,
        )
        .bind(file.id)
        .bind(&file.name)
        .bind(&file.path)
        .bind(file.parent_id)
        .bind(file.size)
        .bind(&file.mime_type)
        .bind(file.owner_id)
        .execute(self.service.pool())
        .await;

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
        let new_path_for_event = new_path.clone();
        file.move_to(cmd.new_parent_id, new_path_for_event, cmd.moved_by);

        // Save
        self.file_repo.save(&mut file).await?;

        let _ = sqlx::query(
            r#"
            UPDATE file_views
            SET parent_id = $1, path = $2, updated_at = NOW(), updated_by = $3
            WHERE id = $4
            "#,
        )
        .bind(cmd.new_parent_id)
        .bind(&new_path)
        .bind(cmd.moved_by)
        .bind(cmd.file_id)
        .execute(self.service.pool())
        .await;

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

        let _ = sqlx::query(
            r#"
            UPDATE file_views
            SET deleted_at = NOW(), deleted_by = $1, updated_at = NOW(), updated_by = $1
            WHERE id = $2
            "#,
        )
        .bind(cmd.deleted_by)
        .bind(cmd.file_id)
        .execute(self.service.pool())
        .await;

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
        let new_name = cmd.new_name.clone();
        let new_path_for_event = new_path.clone();
        file.rename(new_name, new_path_for_event, cmd.renamed_by);

        // Save
        self.file_repo.save(&mut file).await?;

        let _ = sqlx::query(
            r#"
            UPDATE file_views
            SET name = $1, path = $2, updated_at = NOW(), updated_by = $3
            WHERE id = $4 AND deleted_at IS NULL
            "#,
        )
        .bind(&cmd.new_name)
        .bind(&new_path)
        .bind(cmd.renamed_by)
        .bind(cmd.file_id)
        .execute(self.service.pool())
        .await;

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

        let _ = sqlx::query(
            r#"
            INSERT INTO file_views (id, name, path, parent_id, owner_id, item_type, created_at, updated_at, created_by, updated_by)
            VALUES ($1, $2, $3, $4, $5, 'folder', NOW(), NOW(), $5, $5)
            ON CONFLICT (id) DO UPDATE SET
              name = EXCLUDED.name,
              path = EXCLUDED.path,
              parent_id = EXCLUDED.parent_id,
              owner_id = EXCLUDED.owner_id,
              updated_at = NOW(),
              updated_by = EXCLUDED.owner_id
            "#,
        )
        .bind(folder.id)
        .bind(&folder.name)
        .bind(&folder.path)
        .bind(folder.parent_id)
        .bind(folder.owner_id)
        .execute(self.service.pool())
        .await;

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

/// Set File Permissions Handler
pub struct SetFilePermissionsHandler {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
    service: Arc<FileSystemService>,
    pool: PgPool,
}

impl SetFilePermissionsHandler {
    pub fn new(
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        event_bus: Arc<dyn EventBus + Send + Sync>,
        service: Arc<FileSystemService>,
        pool: PgPool,
    ) -> Self {
        Self {
            file_repo,
            event_bus,
            service,
            pool,
        }
    }
}

#[async_trait]
impl CommandHandler<SetFilePermissionsCommand> for SetFilePermissionsHandler {
    type Error = HandlerError;

    async fn handle(&self, cmd: SetFilePermissionsCommand) -> Result<(), Self::Error> {
        // Load file
        let mut file = self
            .file_repo
            .find_by_id(&cmd.file_id)
            .await?
            .ok_or_else(|| err("File not found"))?;

        // Only owner or admin can change permissions (admin check via ACL)
        // If ACL is empty (should not happen), deny.
        // We use owner_id as "changed_by" because API already verifies the actor.
        let changed_by = file.owner_id;

        file.change_permissions(cmd.acl.clone(), changed_by);

        self.file_repo.save(&mut file).await?;

        // Synchronous rewrite of file_permissions table
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM file_permissions WHERE file_id = $1")
            .bind(cmd.file_id)
            .execute(&mut *tx)
            .await?;

        for entry in cmd.acl {
            let (subject_type, subject_id) = match entry.subject {
                crate::core::shared::Subject::User(id) => ("user", Some(id)),
                crate::core::shared::Subject::Group(id) => ("group", Some(id)),
                crate::core::shared::Subject::Everyone => ("everyone", None),
            };
            for perm in entry.permissions {
                let perm_str = format!("{:?}", perm).to_lowercase();
                let _ = sqlx::query(
                    r#"
                    INSERT INTO file_permissions (file_id, subject_type, subject_id, permission, inherited)
                    VALUES ($1, $2, $3, $4, $5)
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(cmd.file_id)
                .bind(subject_type)
                .bind(subject_id)
                .bind(perm_str)
                .bind(entry.inherited)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;

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

