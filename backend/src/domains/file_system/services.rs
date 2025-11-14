use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

// Repository trait is used internally, no need to import
use crate::domains::file_system::aggregates::{File, Folder};

/// File System Service - business logic
/// Used by command handlers
#[derive(Clone)]
pub struct FileSystemService {
    file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
    folder_repo: Arc<crate::core::infrastructure::PostgresRepository<Folder>>,
    pool: SqlitePool,
}

impl FileSystemService {
    pub fn new(
        pool: SqlitePool,
        file_repo: Arc<crate::core::infrastructure::PostgresRepository<File>>,
        folder_repo: Arc<crate::core::infrastructure::PostgresRepository<Folder>>,
    ) -> Self {
        Self {
            file_repo,
            folder_repo,
            pool,
        }
    }

    /// Calculate path from parent
    pub async fn calculate_path(
        &self,
        name: &str,
        parent_id: Option<Uuid>,
    ) -> Result<String, sqlx::Error> {
        if let Some(parent_id) = parent_id {
            let parent_path: Option<(String,)> = sqlx::query_as(
                "SELECT path FROM file_views WHERE id = ?1 AND item_type = 'folder'",
            )
            .bind(parent_id.to_string())
            .fetch_optional(&self.pool)
            .await?;

            if let Some((parent_path,)) = parent_path {
                Ok(if parent_path.ends_with('/') {
                    format!("{}{}", parent_path, name)
                } else {
                    format!("{}/{}", parent_path, name)
                })
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        } else {
            Ok(format!("/{}", name))
        }
    }

    /// Check if name exists in parent
    pub async fn name_exists(
        &self,
        name: &str,
        parent_id: Option<Uuid>,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, sqlx::Error> {
        let parent_id_str = parent_id.map(|id| id.to_string());
        
        let exists: Option<i64> = if let Some(exclude_id) = exclude_id {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM file_views WHERE name = ?1 AND (parent_id = ?2 OR (parent_id IS NULL AND ?2 IS NULL)) AND id != ?3 AND deleted_at IS NULL",
            )
            .bind(name)
            .bind(&parent_id_str)
            .bind(exclude_id.to_string())
            .fetch_optional(&self.pool)
            .await?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM file_views WHERE name = ?1 AND (parent_id = ?2 OR (parent_id IS NULL AND ?2 IS NULL)) AND deleted_at IS NULL",
            )
            .bind(name)
            .bind(&parent_id_str)
            .fetch_optional(&self.pool)
            .await?
        };

        Ok(exists.map(|c| c > 0).unwrap_or(false))
    }

    /// Check permissions
    pub async fn check_permission(
        &self,
        file_id: Uuid,
        user_id: Uuid,
        permission: crate::core::shared::Permission,
    ) -> Result<bool, sqlx::Error> {
        // Check if user is owner
        let owner: Option<String> = sqlx::query_scalar(
            "SELECT owner_id FROM file_views WHERE id = ?1",
        )
        .bind(file_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        let owner_id = owner.and_then(|s| Uuid::parse_str(&s).ok());

        if let Some(oid) = owner_id {
            if oid == user_id {
                return Ok(true); // Owner has all permissions
            }
        }

        // Check ACL
        let permission_str = format!("{:?}", permission).to_lowercase();
        let has_permission: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM file_permissions
            WHERE file_id = ?1
            AND (
                (subject_type = 'user' AND subject_id = ?2)
                OR (subject_type = 'everyone')
                OR (subject_type = 'group' AND subject_id IN (
                    SELECT group_id FROM user_groups WHERE user_id = ?2
                ))
            )
            AND permission IN (?3, 'admin')
        "#,
        )
        .bind(file_id.to_string())
        .bind(user_id.to_string())
        .bind(&permission_str)
        .fetch_optional(&self.pool)
        .await?;

        Ok(has_permission.map(|c| c > 0).unwrap_or(false))
    }
}

