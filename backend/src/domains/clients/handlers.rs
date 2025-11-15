use async_trait::async_trait;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::core::cqrs::{CommandHandler, QueryHandler};
use crate::domains::clients::{
    CreateClientCommand, UpdateClientCommand, DeleteClientCommand,
    GetClientQuery, ListClientsQuery, SearchClientsQuery,
};
use crate::error::AppError;
use crate::models::Client;

// ============ Command Handlers ============

pub struct CreateClientHandler {
    pool: Arc<SqlitePool>,
}

impl CreateClientHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<CreateClientCommand> for CreateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: CreateClientCommand) -> Result<Client, Self::Error> {
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
        .bind(command.status.unwrap_or_else(|| "active".to_string()))
        .fetch_one(&*self.pool)
        .await?;

        Ok(client)
    }
}

pub struct UpdateClientHandler {
    pool: Arc<SqlitePool>,
}

impl UpdateClientHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<UpdateClientCommand> for UpdateClientHandler {
    type Error = AppError;

    async fn handle(&self, command: UpdateClientCommand) -> Result<Client, Self::Error> {
        // Build dynamic update query
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
        
        if let Some(name) = command.name {
            q = q.bind(name);
        }
        if let Some(email) = command.email {
            q = q.bind(email);
        }
        if let Some(phone) = command.phone {
            q = q.bind(phone);
        }
        if let Some(address) = command.address {
            q = q.bind(address);
        }
        if let Some(company) = command.company {
            q = q.bind(company);
        }
        if let Some(status) = command.status {
            q = q.bind(status);
        }

        let client = q.fetch_one(&*self.pool).await?;
        Ok(client)
    }
}

pub struct DeleteClientHandler {
    pool: Arc<SqlitePool>,
}

impl DeleteClientHandler {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CommandHandler<DeleteClientCommand> for DeleteClientHandler {
    type Error = AppError;

    async fn handle(&self, command: DeleteClientCommand) -> Result<(), Self::Error> {
        let result = sqlx::query("DELETE FROM clients WHERE id = ?")
            .bind(command.id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Client not found".to_string()));
        }

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
