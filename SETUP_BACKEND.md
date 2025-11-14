# 🚀 Backend Setup Guide

## 📋 Prerequisites

### 1. Install Rust

**Windows:**
```powershell
# Download and run rustup-init.exe from:
https://rustup.rs/

# Or use winget:
winget install Rustlang.Rustup
```

**Linux/Mac:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify installation:
```bash
cargo --version
rustc --version
```

### 2. Install SQLite (Optional - SQLx will handle it)

SQLite is embedded, no separate installation needed!

### 3. Install Redis (for Event Bus)

**Windows:**
```powershell
# Option 1: Use WSL
wsl --install
sudo apt update && sudo apt install redis-server

# Option 2: Download Windows port
# https://github.com/microsoftarchive/redis/releases
```

**Linux:**
```bash
sudo apt update
sudo apt install redis-server
sudo systemctl start redis
```

**Mac:**
```bash
brew install redis
brew services start redis
```

## 🔧 Setup Steps

### Step 1: Create Environment File

```bash
cd backend
cp .env.example .env
```

Edit `.env`:
```bash
# Database
DATABASE_URL=sqlite:./data/crm.db

# JWT
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION=86400

# Server
HOST=0.0.0.0
PORT=3000

# CORS
CORS_ORIGIN=http://localhost:5173

# File Upload
MAX_FILE_SIZE=10485760
UPLOAD_DIR=./uploads

# Redis (Event Bus)
REDIS_URL=redis://127.0.0.1:6379
```

### Step 2: Install SQLx CLI (Optional but recommended)

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

### Step 3: Create Database Directory

```bash
mkdir -p data uploads
```

### Step 4: Build Project

```bash
cd backend

# Download dependencies and build
cargo build

# Or build with optimizations
cargo build --release
```

### Step 5: Run Migrations

Migrations will run automatically when you start the server, but you can also run manually:

```bash
sqlx migrate run
```

### Step 6: Start Server

**Development mode:**
```bash
cargo run
```

**Production mode:**
```bash
cargo run --release
```

Server will start on `http://localhost:3000`

## ✅ Verify Installation

### 1. Check Health Endpoint
```bash
curl http://localhost:3000/health
```
Expected: `OK`

### 2. Test Registration
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123",
    "full_name": "Admin User",
    "role": "admin"
  }'
```

Expected: JSON with `token` and `user` object

### 3. Test Login
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "password123"
  }'
```

## 🐛 Troubleshooting

### Issue: "JWT_SECRET not found"
**Solution:** Create `.env` file with `JWT_SECRET=your-secret-key`

### Issue: "Cannot connect to Redis"
**Solution:**
- Redis is optional for basic functionality
- Start Redis: `redis-server`
- Or comment out Redis features temporarily

### Issue: "Permission denied" on uploads folder
**Solution:**
```bash
chmod 755 uploads
```

### Issue: Build errors
**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build
```

### Issue: SQLite locked
**Solution:**
```bash
# Stop all running instances
pkill -9 crm-backend

# Remove lock file
rm data/crm.db-shm data/crm.db-wal
```

## 📊 Database Schema

The migrations will create these tables:
- `users` - User accounts
- `clients` - CRM clients
- `tasks` - Tasks and todos
- `notifications` - User notifications
- `files` - File metadata
- `activities` - Audit log
- `event_store` - Event sourcing
- `file_views` - Denormalized file/folder views
- `folder_tree` - Closure table for tree queries
- `file_permissions` - ACL

## 🎯 Next Steps

1. ✅ Backend running
2. Start frontend: `cd frontend && npm run dev`
3. Open browser: `http://localhost:5173`
4. Login with test credentials

## 📝 Development Tips

### Watch mode (auto-reload)
```bash
cargo install cargo-watch
cargo watch -x run
```

### Run with logs
```bash
RUST_LOG=debug cargo run
```

### Format code
```bash
cargo fmt
```

### Lint
```bash
cargo clippy
```

### Run tests
```bash
cargo test
```

## 🚀 Performance Tuning

For 30k CCU, adjust in `main.rs`:
```rust
SqlitePoolOptions::new()
    .max_connections(100)  // Increase based on load
    .acquire_timeout(std::time::Duration::from_secs(10))
```

Enable WAL mode for better concurrency:
```rust
sqlx::query("PRAGMA journal_mode = WAL")
    .execute(&pool)
    .await?;
```

## 🔒 Security Checklist

- [ ] Change JWT_SECRET in production
- [ ] Use strong passwords
- [ ] Enable HTTPS in production
- [ ] Set proper CORS_ORIGIN
- [ ] Limit file upload sizes
- [ ] Enable rate limiting (TODO)
- [ ] Audit sensitive operations

Happy coding! 🎉
