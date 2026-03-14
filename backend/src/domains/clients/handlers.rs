use async_trait::async_trait;
use chrono::Utc;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::events::{EventBus, EventBusExt, EventEnvelope, EventMetadata};
use crate::domains::clients::{
    CreateClientCommand, UpdateClientCommand, DeleteClientCommand,
    GetClientQuery, ListClientsQuery, SearchClientsQuery,
    ClientCreatedEvent, ClientUpdatedEvent, ClientDeletedEvent,
};
use crate::error::AppError;
use crate::models::Client;

// ============ Command Handlers ============

pub struct CreateClientHandler {
    pool: Arc<SqlitePool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl CreateClientHandler {
    pub fn new(pool: Arc<SqlitePool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<CreateClientCommand> for CreateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: CreateClientCommand) -> Result<Client, Self::Error> {
        // 1. Execute the command (database write)
        let status = command.status.clone().unwrap_or_else(|| "active".to_string());
        let client = sqlx::query_as::<_, Client>(
            r#"
            INSERT INTO clients (name, email, phone, address, company, status)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&command.name)
        .bind(&command.email)
        .bind(&command.phone)
        .bind(&command.address)
        .bind(&command.company)
        .bind(&status)
        .fetch_one(&*self.pool)
        .await?;

        // 2. Create domain event
        let aggregate_id = Uuid::new_v4();
        let event = ClientCreatedEvent {
            aggregate_id,
            client_id: client.id,
            name: client.name.clone(),
            email: client.email.clone(),
            phone: client.phone.clone(),
            company: client.company.clone(),
            status: client.status.clone(),
            address: client.address.clone(),
            position: client.position.clone(),
            created_by: "system".to_string(), // TODO: Get from auth context
            version: 1,
            occurred_at: Utc::now(),
        };

        // 3. Create event envelope with metadata
        let metadata = EventMetadata::new(None);
        let envelope = EventEnvelope::new(
            aggregate_id,
            "Client".to_string(),
            event,
            metadata.to_json(),
        );

        // 4. Publish event (async, non-blocking)
        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            if let Err(e) = event_bus.publish(envelope).await {
                tracing::error!("Failed to publish ClientCreatedEvent: {}", e);
            } else {
                tracing::debug!("ClientCreatedEvent published successfully");
            }
        });

        tracing::info!("Client created: {} (id: {})", client.name, client.id);
        Ok(client)
    }
}

pub struct UpdateClientHandler {
    pool: Arc<SqlitePool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl UpdateClientHandler {
    pub fn new(pool: Arc<SqlitePool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<UpdateClientCommand> for UpdateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateClientCommand) -> Result<Client, Self::Error> {
        // 1. Get original client for previous values
        let original = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

        // 2. Build dynamic update query
        let mut query = String::from("UPDATE clients SET ");
        let mut updates = Vec::new();
        let mut bind_count = 0;

        if command.name.is_some() {
            updates.push("name = ?");
            bind_count += 1;
        }
        if command.email.is_some() {
            updates.push("email = ?");
            bind_count += 1;
        }
        if command.phone.is_some() {
            updates.push("phone = ?");
            bind_count += 1;
        }
        if command.address.is_some() {
            updates.push("address = ?");
            bind_count += 1;
        }
        if command.company.is_some() {
            updates.push("company = ?");
            bind_count += 1;
        }
        if command.status.is_some() {
            updates.push("status = ?");
            bind_count += 1;
        }

        if bind_count == 0 {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        query.push_str(&updates.join(", "));
        query.push_str(&format!(" WHERE id = {} RETURNING *", command.id));

        let mut q = sqlx::query_as::<_, Client>(&query);

        if let Some(ref name) = command.name {
            q = q.bind(name);
        }
        if let Some(ref email) = command.email {
            q = q.bind(email);
        }
        if let Some(ref phone) = command.phone {
            q = q.bind(phone);
        }
        if let Some(ref address) = command.address {
            q = q.bind(address);
        }
        if let Some(ref company) = command.company {
            q = q.bind(company);
        }
        if let Some(ref status) = command.status {
            q = q.bind(status);
        }

        let client = q.fetch_one(&*self.pool).await?;

        // 3. Create change map (only changed fields)
        let mut changes = serde_json::Map::new();
        if let Some(ref name) = command.name {
            changes.insert("name".to_string(), serde_json::json!(name));
        }
        if let Some(ref email) = command.email {
            changes.insert("email".to_string(), serde_json::json!(email));
        }
        if let Some(ref phone) = command.phone {
            changes.insert("phone".to_string(), serde_json::json!(phone));
        }
        if let Some(ref address) = command.address {
            changes.insert("address".to_string(), serde_json::json!(address));
        }
        if let Some(ref company) = command.company {
            changes.insert("company".to_string(), serde_json::json!(company));
        }
        if let Some(ref status) = command.status {
            changes.insert("status".to_string(), serde_json::json!(status));
        }

        // 4. Create event
        let aggregate_id = Uuid::new_v4();
        let event = ClientUpdatedEvent {
            aggregate_id,
            client_id: client.id,
            changes: serde_json::Value::Object(changes),
            previous_values: Some(serde_json::json!({
                "name": original.name,
                "email": original.email,
                "phone": original.phone,
                "status": original.status,
            })),
            updated_by: "system".to_string(),
            version: 1,
            occurred_at: Utc::now(),
        };

        // 5. Publish event
        let metadata = EventMetadata::new(None);
        let envelope = EventEnvelope::new(
            aggregate_id,
            "Client".to_string(),
            event,
            metadata.to_json(),
        );

        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            if let Err(e) = event_bus.publish(envelope).await {
                tracing::error!("Failed to publish ClientUpdatedEvent: {}", e);
            } else {
                tracing::debug!("ClientUpdatedEvent published successfully");
            }
        });

        tracing::info!("Client updated: {} (id: {})", client.name, client.id);
        Ok(client)
    }
}

pub struct DeleteClientHandler {
    pool: Arc<SqlitePool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl DeleteClientHandler {
    pub fn new(pool: Arc<SqlitePool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<DeleteClientCommand> for DeleteClientHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteClientCommand) -> Result<(), Self::Error> {
        // 1. Get client info before deletion (for event)
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

        // 2. Delete from database
        let result = sqlx::query("DELETE FROM clients WHERE id = ?")
            .bind(command.id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Client not found".to_string()));
        }

        // 3. Create event
        let aggregate_id = Uuid::new_v4();
        let event = ClientDeletedEvent {
            aggregate_id,
            client_id: client.id,
            name: client.name.clone(),
            deleted_by: "system".to_string(),
            reason: None,
            version: 1,
            occurred_at: Utc::now(),
        };

        // 4. Publish event
        let metadata = EventMetadata::new(None);
        let envelope = EventEnvelope::new(
            aggregate_id,
            "Client".to_string(),
            event,
            metadata.to_json(),
        );

        let event_bus = self.event_bus.clone();
        tokio::spawn(async move {
            if let Err(e) = event_bus.publish(envelope).await {
                tracing::error!("Failed to publish ClientDeletedEvent: {}", e);
            } else {
                tracing::debug!("ClientDeletedEvent published successfully");
            }
        });

        tracing::info!("Client deleted: {} (id: {})", client.name, client.id);
        Ok(())
    }
}

// ============ Query Handlers ============

pub struct GetClientHandler {
    pool: Arc<SqlitePool>,
}

impl GetClientHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetClientQuery> for GetClientHandler {
    type Error = AppError;

    async fn handle(&self, query: GetClientQuery) -> Result<Option<Client>, Self::Error> {
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = ?")
            .bind(query.id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(client)
    }
}

pub struct ListClientsHandler {
    pool: Arc<SqlitePool>,
}

impl ListClientsHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListClientsQuery> for ListClientsHandler {
    type Error = AppError;

    async fn handle(&self, query: ListClientsQuery) -> Result<Vec<Client>, Self::Error> {
        let mut sql = String::from("SELECT * FROM clients");

        if let Some(status) = &query.status {
            sql.push_str(&format!(" WHERE status = '{}'", status));
        }

        sql.push_str(" ORDER BY created_at DESC");

        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = query.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        let clients = sqlx::query_as::<_, Client>(&sql)
            .fetch_all(&*self.pool)
            .await?;

        Ok(clients)
    }
}

pub struct SearchClientsHandler {
    pool: Arc<SqlitePool>,
}

impl SearchClientsHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<SearchClientsQuery> for SearchClientsHandler {
    type Error = AppError;

    async fn handle(&self, query: SearchClientsQuery) -> Result<Vec<Client>, Self::Error> {
        let search = format!("%{}%", query.search_term);
        let limit = query.limit.unwrap_or(50);

        let clients = sqlx::query_as::<_, Client>(
            r#"
            SELECT * FROM clients
            WHERE name LIKE ? OR email LIKE ? OR company LIKE ?
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(&search)
        .bind(&search)
        .bind(&search)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        Ok(clients)
    }
}
