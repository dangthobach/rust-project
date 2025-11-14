use async_trait::async_trait;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::cqrs::QueryHandler;
use crate::domains::file_system::queries::*;

/// Get File Handler
pub struct GetFileHandler {
    pool: SqlitePool,
}

impl GetFileHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetFileQuery> for GetFileHandler {
    type Error = sqlx::Error;

    async fn handle(&self, query: GetFileQuery) -> Result<FileView, Self::Error> {
        use crate::domains::file_system::read_models::FileViewRow;
        
        let row: FileViewRow = sqlx::query_as(
            r#"
            SELECT id, name, path, parent_id, size, mime_type, owner_id, item_type,
                   created_at, updated_at
            FROM file_views
            WHERE id = ?1 AND deleted_at IS NULL
        "#,
        )
        .bind(query.file_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(FileView::from(row))
    }
}

/// List Files Handler
pub struct ListFilesHandler {
    pool: SqlitePool,
}

impl ListFilesHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListFilesQuery> for ListFilesHandler {
    type Error = sqlx::Error;

    async fn handle(&self, query: ListFilesQuery) -> Result<Vec<FileView>, Self::Error> {
        use crate::domains::file_system::read_models::FileViewRow;
        
        let parent_id_str = query.parent_id.map(|id| id.to_string());
        let rows: Vec<FileViewRow> = sqlx::query_as(
            r#"
            SELECT id, name, path, parent_id, size, mime_type, owner_id, item_type,
                   created_at, updated_at
            FROM file_views
            WHERE (parent_id = ?1 OR (parent_id IS NULL AND ?1 IS NULL)) AND deleted_at IS NULL
            ORDER BY item_type DESC, name ASC
            LIMIT ?2 OFFSET ?3
        "#,
        )
        .bind(&parent_id_str)
        .bind(query.pagination.limit())
        .bind(query.pagination.offset())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(FileView::from).collect())
    }
}

/// Get Folder Tree Handler
pub struct GetFolderTreeHandler {
    pool: SqlitePool,
}

impl GetFolderTreeHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetFolderTreeQuery> for GetFolderTreeHandler {
    type Error = sqlx::Error;

    async fn handle(&self, query: GetFolderTreeQuery) -> Result<FolderTreeView, Self::Error> {
        use crate::domains::file_system::read_models::{FileView, FileViewRow};
        
        // Get folder
        let row: FileViewRow = sqlx::query_as(
            r#"
            SELECT id, name, path, parent_id, size, mime_type, owner_id, item_type,
                   created_at, updated_at
            FROM file_views
            WHERE id = ?1 AND item_type = 'folder' AND deleted_at IS NULL
        "#,
        )
        .bind(query.folder_id.to_string())
        .fetch_one(&self.pool)
        .await?;
        
        let folder = FileView::from(row);

        // Get children recursively
        let children = self.get_children_recursive(query.folder_id, query.depth.unwrap_or(10)).await?;

        Ok(FolderTreeView {
            folder,
            children,
        })
    }
}

impl GetFolderTreeHandler {
    #[allow(dead_code)] // Used internally by handle method
    async fn get_children_recursive(
            &self,
            parent_id: Uuid,
            depth: i32,
        ) -> Result<Vec<FolderTreeView>, sqlx::Error> {
            if depth <= 0 {
                return Ok(vec![]);
            }

            use crate::domains::file_system::read_models::{FileView, FileViewRow};
            
            // Get direct children (folders only for tree)
            let rows: Vec<FileViewRow> = sqlx::query_as(
                r#"
                SELECT id, name, path, parent_id, size, mime_type, owner_id, item_type,
                       created_at, updated_at
                FROM file_views
                WHERE parent_id = ?1 AND item_type = 'folder' AND deleted_at IS NULL
                ORDER BY name ASC
            "#,
            )
            .bind(parent_id.to_string())
            .fetch_all(&self.pool)
            .await?;
            
            let folders: Vec<FileView> = rows.into_iter().map(FileView::from).collect();

            let mut result = Vec::new();
            for folder in folders {
                // Use Box::pin for recursive async call to avoid infinite future size
                let future = self.get_children_recursive(folder.id, depth - 1);
                let children = Box::pin(future).await?;
                result.push(FolderTreeView {
                    folder,
                    children,
                });
            }

            Ok(result)
        }
    }

/// Search Files Handler
pub struct SearchFilesHandler {
    pool: SqlitePool,
}

impl SearchFilesHandler {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<SearchFilesQuery> for SearchFilesHandler {
    type Error = sqlx::Error;

    async fn handle(&self, query: SearchFilesQuery) -> Result<Vec<FileView>, Self::Error> {
        use crate::domains::file_system::read_models::{FileView, FileViewRow};
        
        // SQLite doesn't have full-text search like PostgreSQL
        // Use LIKE for simple search
        let search_pattern = format!("%{}%", query.query);
        let rows: Vec<FileViewRow> = sqlx::query_as(
            r#"
            SELECT id, name, path, parent_id, size, mime_type, owner_id, item_type,
                   created_at, updated_at
            FROM file_views
            WHERE deleted_at IS NULL
            AND (name LIKE ?1 OR path LIKE ?1)
            ORDER BY name ASC
            LIMIT ?2 OFFSET ?3
        "#,
        )
        .bind(&search_pattern)
        .bind(query.pagination.limit())
        .bind(query.pagination.offset())
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(FileView::from).collect())
    }
}

