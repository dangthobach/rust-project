use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::core::events::{EventEnvelope, Projection};
use crate::domains::file_system::events::FileSystemEvent;

/// File View Projection - updates file_views read model
/// Used by event consumer (to be implemented)
#[allow(dead_code)]
pub struct FileViewProjection {
    pool: SqlitePool,
}

impl FileViewProjection {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Projection for FileViewProjection {
    type Event = FileSystemEvent;
    type Error = sqlx::Error;

    fn projection_name(&self) -> &'static str {
        "file_view_projection"
    }

    async fn handle(&self, event: &EventEnvelope<FileSystemEvent>) -> Result<(), Self::Error> {
        match &event.event_data {
            FileSystemEvent::FileCreated(e) => {
                sqlx::query(
                    r#"
                    INSERT INTO file_views (
                        id, name, path, parent_id, size, mime_type, owner_id,
                        item_type, created_at, updated_at, created_by
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'file', ?8, ?8, ?7)
                    ON CONFLICT (id) DO NOTHING
                "#,
                )
                .bind(e.file_id.to_string())
                .bind(&e.name)
                .bind(&e.path)
                .bind(e.parent_id.map(|id| id.to_string()))
                .bind(e.size)
                .bind(&e.mime_type)
                .bind(e.owner_id.to_string())
                .bind(e.occurred_at.to_rfc3339())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FileMoved(e) => {
                sqlx::query(
                    "UPDATE file_views SET parent_id = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                )
                .bind(e.new_parent_id.map(|id| id.to_string()))
                .bind(&e.new_path)
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.file_id.to_string())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FileDeleted(e) => {
                sqlx::query(
                    "UPDATE file_views SET deleted_at = ?1, deleted_by = ?2 WHERE id = ?3",
                )
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.deleted_by.to_string())
                .bind(e.file_id.to_string())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FileRestored(e) => {
                sqlx::query("UPDATE file_views SET deleted_at = NULL, deleted_by = NULL WHERE id = ?1")
                    .bind(e.file_id.to_string())
                    .execute(&self.pool)
                    .await?;
            }
            FileSystemEvent::FileRenamed(e) => {
                sqlx::query(
                    "UPDATE file_views SET name = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                )
                .bind(&e.new_name)
                .bind(&e.new_path)
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.file_id.to_string())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FilePermissionsChanged(e) => {
                // Update permissions in file_permissions table
                // Delete old permissions
                sqlx::query("DELETE FROM file_permissions WHERE file_id = ?1")
                    .bind(e.file_id.to_string())
                    .execute(&self.pool)
                    .await?;

                // Insert new permissions
                let acl: Vec<crate::core::shared::AccessControlEntry> =
                    serde_json::from_value(e.acl.clone()).unwrap_or_default();

                for entry in acl {
                    let (subject_type, subject_id) = match entry.subject {
                        crate::core::shared::Subject::User(id) => ("user", Some(id)),
                        crate::core::shared::Subject::Group(id) => ("group", Some(id)),
                        crate::core::shared::Subject::Everyone => ("everyone", None),
                    };

                    for permission in entry.permissions {
                        sqlx::query(
                            r#"
                            INSERT INTO file_permissions (file_id, subject_type, subject_id, permission, inherited)
                            VALUES (?1, ?2, ?3, ?4, ?5)
                            ON CONFLICT DO NOTHING
                        "#,
                        )
                        .bind(e.file_id.to_string())
                        .bind(subject_type)
                        .bind(subject_id.map(|id| id.to_string()))
                        .bind(format!("{:?}", permission).to_lowercase())
                        .bind(entry.inherited)
                        .execute(&self.pool)
                        .await?;
                    }
                }
            }
            FileSystemEvent::FolderCreated(e) => {
                sqlx::query(
                    r#"
                    INSERT INTO file_views (
                        id, name, path, parent_id, owner_id,
                        item_type, created_at, updated_at, created_by
                    ) VALUES (?1, ?2, ?3, ?4, ?5, 'folder', ?6, ?6, ?5)
                    ON CONFLICT (id) DO NOTHING
                "#,
                )
                .bind(e.folder_id.to_string())
                .bind(&e.name)
                .bind(&e.path)
                .bind(e.parent_id.map(|id| id.to_string()))
                .bind(e.owner_id.to_string())
                .bind(e.occurred_at.to_rfc3339())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FolderMoved(e) => {
                sqlx::query(
                    "UPDATE file_views SET parent_id = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                )
                .bind(e.new_parent_id.map(|id| id.to_string()))
                .bind(&e.new_path)
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.folder_id.to_string())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FolderDeleted(e) => {
                sqlx::query(
                    "UPDATE file_views SET deleted_at = ?1, deleted_by = ?2 WHERE id = ?3",
                )
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.deleted_by.to_string())
                .bind(e.folder_id.to_string())
                .execute(&self.pool)
                .await?;
            }
            FileSystemEvent::FolderRestored(e) => {
                sqlx::query("UPDATE file_views SET deleted_at = NULL, deleted_by = NULL WHERE id = ?1")
                    .bind(e.folder_id.to_string())
                    .execute(&self.pool)
                    .await?;
            }
            FileSystemEvent::FolderRenamed(e) => {
                sqlx::query(
                    "UPDATE file_views SET name = ?1, path = ?2, updated_at = ?3 WHERE id = ?4",
                )
                .bind(&e.new_name)
                .bind(&e.new_path)
                .bind(e.occurred_at.to_rfc3339())
                .bind(e.folder_id.to_string())
                .execute(&self.pool)
                .await?;
            }
        }

        // Update position
        self.update_position(event.version).await?;

        Ok(())
    }

    async fn rebuild(&self) -> Result<(), Self::Error> {
        // Load all events and replay
        // This would be implemented with event store
        todo!("Implement rebuild from event store")
    }

    async fn get_position(&self) -> Result<i64, Self::Error> {
        let position: Option<i64> = sqlx::query_scalar(
            "SELECT position FROM projection_positions WHERE projection_name = ?1",
        )
        .bind(self.projection_name())
        .fetch_optional(&self.pool)
        .await?;

        Ok(position.unwrap_or(0))
    }

    async fn update_position(&self, position: i64) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO projection_positions (projection_name, position, updated_at)
            VALUES (?1, ?2, datetime('now'))
            ON CONFLICT (projection_name) DO UPDATE SET position = ?2, updated_at = datetime('now')
        "#,
        )
        .bind(self.projection_name())
        .bind(position)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

