# SQLite Migration Status

## ✅ Completed

1. **Cargo.toml**: Switched from PostgreSQL to SQLite
2. **Event Store**: Updated to use SQLite syntax (?1 instead of $1)
3. **Repository**: Updated PostgresRepository to use SqlitePool
4. **Rebuildable Trait**: Implemented for File and Folder aggregates
5. **Main.rs**: Updated to use SqlitePoolOptions with 50 max connections
6. **Config**: Default DATABASE_URL set to `sqlite:./data/crm.db`
7. **Migrations**: Created SQLite-compatible migrations
8. **Core Infrastructure**: All core files updated
9. **File System Domain**: All handlers, services, projections updated
10. **Routes**: Updated to use SqlitePool

## 🔄 In Progress

### SQL Syntax Updates Needed

All handlers need SQL syntax updated from PostgreSQL ($1, $2) to SQLite (?1, ?2):

- [ ] `backend/src/handlers/auth.rs` - 3 queries need fixing
- [ ] `backend/src/handlers/users.rs` - Multiple queries
- [ ] `backend/src/handlers/clients.rs` - Multiple queries
- [ ] `backend/src/handlers/tasks.rs` - Multiple queries
- [ ] `backend/src/handlers/notifications.rs` - Multiple queries
- [ ] `backend/src/handlers/files.rs` - Already fixed ✅

### UUID Handling

SQLite stores UUIDs as TEXT, so all UUID bindings need `.to_string()`:
- [x] File System handlers ✅
- [ ] Other handlers need UUID conversion

### Boolean Handling

SQLite uses INTEGER (0/1) instead of BOOLEAN:
- [x] Auth middleware ✅
- [ ] Other handlers need boolean conversion

## 📋 Migration Checklist

### Critical Files
- [x] Cargo.toml
- [x] main.rs
- [x] config.rs
- [x] core/infrastructure/postgres_repo.rs
- [x] core/events/event_store.rs
- [x] domains/file_system/* (all files)
- [x] routes.rs
- [x] routes/file_system.rs
- [x] middleware/auth.rs
- [ ] handlers/auth.rs (SQL syntax)
- [ ] handlers/users.rs
- [ ] handlers/clients.rs
- [ ] handlers/tasks.rs
- [ ] handlers/notifications.rs
- [x] handlers/files.rs

### Migrations
- [x] 008_create_event_store.sql (SQLite compatible)
- [x] 009_create_file_system_tables.sql (SQLite compatible)
- [ ] Need to update existing migrations (001-007) for SQLite

## 🚀 Next Steps

1. Fix SQL syntax in all remaining handlers
2. Update UUID bindings to use `.to_string()`
3. Update boolean handling (true -> 1, false -> 0)
4. Update existing migrations (001-007) for SQLite compatibility
5. Test compilation
6. Test database operations

## 📝 Notes

- SQLite doesn't support RETURNING clause, need to use separate SELECT
- SQLite uses TEXT for UUIDs instead of UUID type
- SQLite uses INTEGER for booleans (0/1)
- SQLite uses ?1, ?2 for parameters instead of $1, $2
- SQLite uses datetime('now') instead of NOW()

