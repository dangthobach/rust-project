# 🏗️ CQRS/Event Sourcing Architecture

## 📋 Tổng quan

Hệ thống được xây dựng với kiến trúc **CQRS (Command Query Responsibility Segregation)** và **Event Sourcing** để:

- ✅ Scale đến 30k CCU
- ✅ Audit trail hoàn chỉnh
- ✅ Tách biệt read/write models
- ✅ Rebuild state từ events
- ✅ Common base cho tất cả entities

## 🎯 Cấu trúc Core Components

### 1. Domain Layer (`core/domain/`)

#### Entity Trait
```rust
pub trait Entity {
    type Id;
    fn id(&self) -> &Self::Id;
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}
```

#### Aggregate Root
```rust
pub trait AggregateRoot: Entity {
    type Event: DomainEvent;
    fn version(&self) -> i64;
    fn uncommitted_events(&self) -> &[Self::Event];
    fn mark_events_as_committed(&mut self);
    fn apply(&mut self, event: &Self::Event);
}
```

**Sử dụng:**
- Mỗi aggregate có version cho optimistic locking
- Events được collect trong `uncommitted_events()`
- `apply()` để rebuild state từ events

#### Repository Pattern
```rust
#[async_trait]
pub trait Repository<T: AggregateRoot> {
    async fn find_by_id(&self, id: &T::Id) -> Result<Option<T>, Error>;
    async fn save(&self, aggregate: &mut T) -> Result<(), Error>;
    async fn rebuild_from_events(&self, id: &T::Id) -> Result<Option<T>, Error>;
}
```

### 2. Event Sourcing (`core/events/`)

#### Event Store
- Lưu tất cả domain events
- Optimistic locking với version
- Support snapshots

#### Event Bus
- Redis Streams cho guaranteed delivery
- Pub/Sub cho projections

#### Snapshots
- Tăng tốc rebuild aggregates
- Tự động tạo mỗi N events

### 3. CQRS (`core/cqrs/`)

#### Commands
```rust
pub trait Command: Validate {
    type Response;
    fn command_name(&self) -> &'static str;
}
```

#### Queries
```rust
pub trait Query {
    type Response;
    fn query_name(&self) -> &'static str;
}
```

**Flow:**
1. Command → CommandHandler → Aggregate → Events
2. Events → Event Store → Event Bus
3. Event Bus → Projections → Read Models
4. Query → QueryHandler → Read Model

### 4. Shared Components (`core/shared/`)

#### Auditable
```rust
pub trait Auditable {
    fn created_by(&self) -> Option<Uuid>;
    fn updated_by(&self) -> Option<Uuid>;
    fn touch(&mut self, user_id: Uuid);
}
```

#### SoftDeletable
```rust
pub trait SoftDeletable {
    fn is_deleted(&self) -> bool;
    fn mark_as_deleted(&mut self, by: Uuid);
    fn restore(&mut self);
}
```

#### Securable (ACL)
```rust
pub trait Securable {
    fn get_acl(&self) -> &[AccessControlEntry];
    fn has_permission(&self, user_id: Uuid, permission: Permission) -> bool;
}
```

## 📁 File System Domain Example

### Aggregate Implementation

```rust
use crate::core::domain::*;
use crate::core::shared::*;

pub struct File {
    // Entity fields
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Domain fields
    pub name: String,
    pub path: FilePath,
    pub parent_id: Option<Uuid>,
    pub size: i64,
    pub mime_type: String,
    pub owner_id: Uuid,
    
    // Shared components
    pub audit: AuditFields,
    pub soft_delete: SoftDeleteFields,
    pub acl: Vec<AccessControlEntry>,
    
    // Event sourcing
    version: i64,
    uncommitted_events: Vec<FileEvent>,
}

impl Entity for File {
    type Id = Uuid;
    fn id(&self) -> &Uuid { &self.id }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}

impl AggregateRoot for File {
    type Event = FileEvent;
    
    fn version(&self) -> i64 { self.version }
    fn uncommitted_events(&self) -> &[FileEvent] { &self.uncommitted_events }
    
    fn mark_events_as_committed(&mut self) {
        self.uncommitted_events.clear();
    }
    
    fn apply(&mut self, event: &FileEvent) {
        match event {
            FileEvent::FileCreated(e) => {
                self.id = e.file_id;
                self.name = e.name.clone();
                // ... apply other fields
            }
            // ... other events
        }
        self.version += 1;
    }
}

impl Auditable for File { /* ... */ }
impl SoftDeletable for File { /* ... */ }
impl Securable for File { /* ... */ }
```

### Command Handler

```rust
pub struct CreateFileCommand {
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub owner_id: Uuid,
}

impl Command for CreateFileCommand {
    type Response = Uuid;
    fn command_name(&self) -> &'static str { "create_file" }
}

#[async_trait]
impl CommandHandler<CreateFileCommand> for FileCommandHandler {
    async fn handle(&self, cmd: CreateFileCommand) -> Result<Uuid, Error> {
        // 1. Validate
        // 2. Create aggregate
        let mut file = File::create(cmd.name, cmd.parent_id, cmd.owner_id)?;
        
        // 3. Save (persist events)
        self.repository.save(&mut file).await?;
        
        // 4. Publish events to bus
        for event in file.uncommitted_events() {
            self.event_bus.publish(event.clone()).await?;
        }
        
        Ok(file.id)
    }
}
```

### Projection

```rust
#[async_trait]
impl Projection for FileViewProjection {
    type Event = FileEvent;
    
    async fn handle(&self, event: &EventEnvelope<FileEvent>) -> Result<(), Error> {
        match &event.event_data {
            FileEvent::FileCreated(e) => {
                // Update read model
                sqlx::query("INSERT INTO file_views ...")
                    .execute(&self.pool)
                    .await?;
            }
            // ... other events
        }
        
        // Update position
        self.update_position(event.version).await?;
        Ok(())
    }
}
```

## 🚀 Sử dụng Common Components

### 1. Tạo Entity mới với Audit + Soft Delete

```rust
pub struct MyEntity {
    pub id: Uuid,
    pub audit: AuditFields,
    pub soft_delete: SoftDeleteFields,
    // ... other fields
}

impl Entity for MyEntity { /* ... */ }
impl Auditable for MyEntity {
    fn created_by(&self) -> Option<Uuid> { self.audit.created_by }
    // ... delegate to audit fields
}
impl SoftDeletable for MyEntity {
    fn is_deleted(&self) -> bool { self.soft_delete.is_deleted() }
    // ... delegate to soft_delete fields
}
```

### 2. Thêm Permissions

```rust
impl Securable for MyEntity {
    fn get_acl(&self) -> &[AccessControlEntry] { &self.acl }
    fn set_acl(&mut self, acl: Vec<AccessControlEntry>) { self.acl = acl; }
}

// Usage
entity.grant_permission(Subject::User(user_id), Permission::Read);
entity.revoke_permission(Subject::User(user_id), Permission::Write);
```

### 3. Repository Pattern

```rust
let repo = PostgresRepository::new(pool, event_store, "file".to_string());

// Find
let file = repo.find_by_id(&file_id).await?;

// Save (persist events)
let mut file = File::create(...)?;
repo.save(&mut file).await?;
```

## 📊 Database Schema

### Event Store
- `event_store`: Lưu tất cả events
- `snapshots`: Lưu snapshots của aggregates
- `projection_positions`: Track position của projections

### Read Models
- `file_views`: Denormalized file data cho queries
- `folder_tree`: Closure table cho tree queries
- `file_permissions`: Denormalized permissions

## 🔧 Performance Optimizations

1. **Snapshots**: Rebuild nhanh hơn từ snapshot thay vì tất cả events
2. **Read Models**: Denormalized data cho queries nhanh
3. **Closure Table**: Tree queries O(1) thay vì recursive
4. **Caching**: Redis cache cho hot data
5. **Connection Pooling**: PgPool với max connections

## 📝 Next Steps

1. ✅ Core infrastructure - **DONE**
2. ⏳ File/Folder aggregates implementation
3. ⏳ Command/Query handlers
4. ⏳ Projections
5. ⏳ API endpoints
6. ⏳ Testing & Load testing

## 💡 Best Practices

1. **Always use events**: Không thay đổi state trực tiếp, dùng events
2. **Idempotent commands**: Commands có thể retry an toàn
3. **Event versioning**: Support event schema evolution
4. **Snapshot strategy**: Tạo snapshot mỗi N events
5. **Projection error handling**: Retry failed projections

