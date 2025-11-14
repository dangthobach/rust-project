// This file is kept for future type-specific repository implementations
// Currently using generic PostgresRepository with Rebuildable trait

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::events::{PostgresEventStore, event_store::EventStore};
use crate::core::infrastructure::postgres_repo::Rebuildable;
use crate::domains::file_system::aggregates::{File, Folder};
use crate::domains::file_system::events::FileSystemEvent;

/// File-specific repository implementation
pub struct FileRepository {
    pool: SqlitePool,
    event_store: PostgresEventStore,
}

impl FileRepository {
    pub fn new(pool: SqlitePool, event_store: PostgresEventStore) -> Self {
        Self { pool, event_store }
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<File>, sqlx::Error> {
        // Load events
        let envelopes = self.event_store.load::<FileSystemEvent>(*id, 0).await?;
        let events: Vec<FileSystemEvent> = envelopes.into_iter().map(|e| e.event_data).collect();

        if events.is_empty() {
            return Ok(None);
        }

        // Rebuild using File's rebuild_from_events
        Ok(File::rebuild_from_events(&events))
    }
}

/// Folder-specific repository implementation
pub struct FolderRepository {
    pool: SqlitePool,
    event_store: PostgresEventStore,
}

impl FolderRepository {
    pub fn new(pool: SqlitePool, event_store: PostgresEventStore) -> Self {
        Self { pool, event_store }
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Folder>, sqlx::Error> {
        // Load events
        let envelopes = self.event_store.load::<FileSystemEvent>(*id, 0).await?;
        let events: Vec<FileSystemEvent> = envelopes.into_iter().map(|e| e.event_data).collect();

        if events.is_empty() {
            return Ok(None);
        }

        // Rebuild using Folder's rebuild_from_events
        Ok(Folder::rebuild_from_events(&events))
    }
}


