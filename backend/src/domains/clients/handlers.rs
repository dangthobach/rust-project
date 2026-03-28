use async_trait::async_trait;
use chrono::Utc;
use sqlx::{QueryBuilder, Postgres, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::core::events::{EventBus, EventBusExt, EventEnvelope, EventMetadata};
use crate::core::shared::append_aggregate_history;
use crate::domains::clients::{
    CreateClientCommand, UpdateClientCommand, DeleteClientCommand,
    GetClientQuery, ListClientsQuery, SearchClientsQuery,
    ClientCreatedEvent, ClientUpdatedEvent, ClientDeletedEvent,
};
use crate::authz::data_scope::{self, can_read_client};
use crate::error::AppError;
use crate::models::Client;

// ============ Command Handlers ============

pub struct CreateClientHandler {
    pool: Arc<PgPool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl CreateClientHandler {
    pub fn new(pool: Arc<PgPool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<CreateClientCommand> for CreateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: CreateClientCommand) -> Result<Client, Self::Error> {
        let status = normalize_status(command.status.as_deref())?;
        if let Some(assigned_to) = &command.assigned_to {
            validate_user_exists(&self.pool, assigned_to).await?;
        }

        let branch_id = data_scope::resolve_client_branch_id(command.branch_id.as_ref());
        data_scope::ensure_branch_allowed(&command.data_scope, &branch_id)?;
        validate_branch_exists(&self.pool, &branch_id).await?;

        let client_id = Uuid::new_v4();
        let assigned_to = command
            .assigned_to
            .as_ref()
            .and_then(|s| Uuid::parse_str(s).ok());
        let branch_uuid = Uuid::parse_str(&branch_id).unwrap_or_else(|_| {
            Uuid::parse_str(data_scope::ROOT_BRANCH_ID).expect("root branch uuid")
        });
        // 1. Execute the command (database write)
        let client = sqlx::query_as::<_, Client>(
            r#"
            INSERT INTO clients (id, name, email, phone, address, company, position, status, assigned_to, notes, branch_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#,
        )
        .bind(client_id)
        .bind(&command.name)
        .bind(&command.email)
        .bind(&command.phone)
        .bind(&command.address)
        .bind(&command.company)
        .bind(&command.position)
        .bind(&status)
        .bind(assigned_to)
        .bind(&command.notes)
        .bind(branch_uuid)
        .fetch_one(&*self.pool)
        .await?;

        append_aggregate_history(
            &self.pool,
            "client",
            &client.id.to_string(),
            "CREATE",
            None,
            Some(client.status.as_str()),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "name": client.name,
                "email": client.email,
                "assigned_to": client.assigned_to
            })),
        )
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
    pool: Arc<PgPool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl UpdateClientHandler {
    pub fn new(pool: Arc<PgPool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<UpdateClientCommand> for UpdateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateClientCommand) -> Result<Client, Self::Error> {
        // 1. Get original client for previous values
        let original = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

        if !can_read_client(
            &self.pool,
            &command.actor_user_id,
            &command.data_scope,
            &original,
        )
        .await?
        {
            return Err(AppError::NotFound("Client not found".to_string()));
        }

        if command.name.is_none()
            && command.email.is_none()
            && command.phone.is_none()
            && command.address.is_none()
            && command.company.is_none()
            && command.position.is_none()
            && command.status.is_none()
            && command.assigned_to.is_none()
            && command.notes.is_none()
        {
            return Err(AppError::ValidationError("No fields to update".to_string()));
        }

        if let Some(assigned_to) = &command.assigned_to {
            validate_user_exists(&self.pool, assigned_to).await?;
        }

        let mut qb = QueryBuilder::<Postgres>::new("UPDATE clients SET ");
        let mut separated = qb.separated(", ");

        if let Some(v) = &command.name {
            separated.push("name = ").push_bind(v);
        }
        if let Some(v) = &command.email {
            separated.push("email = ").push_bind(v);
        }
        if let Some(v) = &command.phone {
            separated.push("phone = ").push_bind(v);
        }
        if let Some(v) = &command.address {
            separated.push("address = ").push_bind(v);
        }
        if let Some(v) = &command.company {
            separated.push("company = ").push_bind(v);
        }
        if let Some(v) = &command.position {
            separated.push("position = ").push_bind(v);
        }
        if let Some(v) = &command.status {
            separated
                .push("status = ")
                .push_bind(normalize_status(Some(v.as_str()))?);
        }
        if let Some(v) = &command.assigned_to {
            separated.push("assigned_to = ").push_bind(v);
        }
        if let Some(v) = &command.notes {
            separated.push("notes = ").push_bind(v);
        }
        separated.push("updated_at = NOW()");
        drop(separated);

        qb.push(" WHERE id = ").push_bind(&command.id);
        qb.build().execute(&*self.pool).await?;

        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(&command.id)
            .fetch_one(&*self.pool)
            .await?;

        append_aggregate_history(
            &self.pool,
            "client",
            &client.id.to_string(),
            "UPDATE",
            Some(original.status.as_str()),
            Some(client.status.as_str()),
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "before": {
                    "name": original.name,
                    "email": original.email,
                    "phone": original.phone,
                    "status": original.status,
                    "assigned_to": original.assigned_to
                },
                "after": {
                    "name": client.name,
                    "email": client.email,
                    "phone": client.phone,
                    "status": client.status,
                    "assigned_to": client.assigned_to
                }
            })),
        )
        .await?;

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
    pool: Arc<PgPool>,
    event_bus: Arc<dyn EventBus + Send + Sync>,
}

impl DeleteClientHandler {
    pub fn new(pool: Arc<PgPool>, event_bus: Arc<dyn EventBus + Send + Sync>) -> Self {
        Self { pool, event_bus }
    }
}

#[async_trait]
impl CommandHandler<DeleteClientCommand> for DeleteClientHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteClientCommand) -> Result<(), Self::Error> {
        // 1. Get client info before deletion (for event)
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(&command.id)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::NotFound("Client not found".to_string()))?;

        if !can_read_client(
            &self.pool,
            &command.actor_user_id,
            &command.data_scope,
            &client,
        )
        .await?
        {
            return Err(AppError::NotFound("Client not found".to_string()));
        }

        // 2. Delete from database
        let result = sqlx::query("DELETE FROM clients WHERE id = $1")
            .bind(&command.id)
            .execute(&*self.pool)
            .await?;
        append_aggregate_history(
            &self.pool,
            "client",
            &client.id.to_string(),
            "DELETE",
            Some(client.status.as_str()),
            None,
            command.actor_id.as_deref(),
            None,
            Some(serde_json::json!({
                "name": client.name,
                "email": client.email
            })),
        )
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
    pool: Arc<PgPool>,
}

impl GetClientHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<GetClientQuery> for GetClientHandler {
    type Error = AppError;

    async fn handle(&self, query: GetClientQuery) -> Result<Option<Client>, Self::Error> {
        let client = sqlx::query_as::<_, Client>("SELECT * FROM clients WHERE id = $1")
            .bind(&query.id)
            .fetch_optional(&*self.pool)
            .await?;

        let Some(c) = client else {
            return Ok(None);
        };
        if !can_read_client(
            &self.pool,
            &query.actor_user_id,
            &query.data_scope,
            &c,
        )
        .await?
        {
            return Ok(None);
        }
        Ok(Some(c))
    }
}

pub struct ListClientsHandler {
    pool: Arc<PgPool>,
}

impl ListClientsHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<ListClientsQuery> for ListClientsHandler {
    type Error = AppError;

    async fn handle(&self, query: ListClientsQuery) -> Result<Vec<Client>, Self::Error> {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM clients WHERE 1=1");

        if let Some(status) = &query.status {
            qb.push(" AND status = ")
                .push_bind(normalize_status(Some(status.as_str()))?);
        }

        if let Some(assigned_to) = &query.assigned_to {
            qb.push(" AND assigned_to = ").push_bind(assigned_to);
        }

        data_scope::push_client_scope_filter(
            &mut qb,
            &query.data_scope,
            &query.actor_user_id,
        );

        qb.push(" ORDER BY created_at DESC");
        qb.push(" LIMIT ").push_bind(query.limit.unwrap_or(50).max(1));
        qb.push(" OFFSET ").push_bind(query.offset.unwrap_or(0).max(0));

        let clients = qb
            .build_query_as::<Client>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(clients)
    }
}

pub struct SearchClientsHandler {
    pool: Arc<PgPool>,
}

impl SearchClientsHandler {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl QueryHandler<SearchClientsQuery> for SearchClientsHandler {
    type Error = AppError;

    async fn handle(&self, query: SearchClientsQuery) -> Result<Vec<Client>, Self::Error> {
        let search = format!("%{}%", query.search_term);
        let limit = query.limit.unwrap_or(50).max(1);

        let mut qb = QueryBuilder::<Postgres>::new(
            "SELECT * FROM clients WHERE (name LIKE ",
        );
        qb.push_bind(&search);
        qb.push(" OR email LIKE ");
        qb.push_bind(&search);
        qb.push(" OR company LIKE ");
        qb.push_bind(&search);
        qb.push(") ");
        data_scope::push_client_scope_filter(
            &mut qb,
            &query.data_scope,
            &query.actor_user_id,
        );
        qb.push(" ORDER BY created_at DESC LIMIT ");
        qb.push_bind(limit);

        let clients = qb
            .build_query_as::<Client>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(clients)
    }
}

fn normalize_status(status: Option<&str>) -> Result<String, AppError> {
    let value = status.unwrap_or("active").to_ascii_lowercase();
    match value.as_str() {
        "active" | "inactive" | "prospect" | "customer" => Ok(value),
        _ => Err(AppError::ValidationError("Invalid client status".to_string())),
    }
}

async fn validate_branch_exists(pool: &PgPool, branch_id: &str) -> Result<(), AppError> {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM branches WHERE id = $1 AND is_active = 1",
    )
    .bind(branch_id)
    .fetch_one(pool)
    .await?;
    if n == 0 {
        return Err(AppError::ValidationError(
            "branch_id does not exist or is inactive".to_string(),
        ));
    }
    Ok(())
}

async fn validate_user_exists(pool: &PgPool, user_id: &str) -> Result<(), AppError> {
    Uuid::parse_str(user_id)
        .map_err(|_| AppError::ValidationError("assigned_to must be UUID".to_string()))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    if exists == 0 {
        return Err(AppError::ValidationError("Assigned user not found".to_string()));
    }
    Ok(())
}
