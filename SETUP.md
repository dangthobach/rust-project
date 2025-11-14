# 🚀 Setup Guide - Neo-Brutalist CRM

## Prerequisites

Trước khi bắt đầu, hãy đảm bảo bạn đã cài đặt:

- **Rust** 1.75+ → [https://rustup.rs/](https://rustup.rs/)
- **Node.js** 20+ → [https://nodejs.org/](https://nodejs.org/)
- **PostgreSQL** 15+ → [https://www.postgresql.org/](https://www.postgresql.org/)
- **wasm-pack** → `cargo install wasm-pack`
- **sqlx-cli** → `cargo install sqlx-cli --features postgres`

## 📦 Installation Steps

### 1. Clone và Setup Project

```bash
cd d:\project\rust-system
```

### 2. Setup Backend (Rust + Axum)

```bash
cd backend

# Tạo file .env từ template
cp .env.example .env

# Chỉnh sửa .env với thông tin database của bạn
# DATABASE_URL=postgresql://username:password@localhost:5432/crm_db

# Tạo database
sqlx database create

# Chạy migrations
sqlx migrate run

# Build backend
cargo build

# Run backend (development)
cargo run

# Backend chạy tại: http://localhost:3000
```

**Test Backend:**
```bash
curl http://localhost:3000/health
# Expected: {"status":"ok","service":"CRM Backend","version":"0.1.0"}
```

### 3. Setup WASM File Viewer

```bash
cd ../wasm-viewer

# Build WASM module cho web
wasm-pack build --target web --release

# Output sẽ ở folder: pkg/
```

### 4. Setup Frontend (Qwik)

```bash
cd ../frontend

# Tạo file .env
cp .env.example .env

# Cài đặt dependencies
npm install

# Copy WASM files vào frontend (nếu cần)
mkdir -p src/wasm
cp -r ../wasm-viewer/pkg src/wasm/

# Run development server
npm run dev

# Frontend chạy tại: http://localhost:5173
```

### 5. Verify Setup

Mở browser và truy cập:
- Frontend: http://localhost:5173
- Backend API: http://localhost:3000/health

## 🐳 Docker Setup (Alternative)

Nếu muốn chạy toàn bộ hệ thống bằng Docker:

```bash
# Từ root directory
docker-compose up -d

# Xem logs
docker-compose logs -f

# Stop services
docker-compose down
```

Services sẽ chạy tại:
- PostgreSQL: localhost:5432
- Backend API: localhost:3000
- Frontend: localhost:5173

## 🧪 Testing

### Backend Tests
```bash
cd backend
cargo test
```

### Frontend Tests
```bash
cd frontend
npm test
```

## 🎨 Development Workflow

### Backend Development

```bash
cd backend

# Watch mode (auto-reload khi code thay đổi)
cargo install cargo-watch
cargo watch -x run

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Frontend Development

```bash
cd frontend

# Dev server với hot reload
npm run dev

# Type checking
npm run typecheck

# Format code
npm run fmt

# Lint
npm run lint
```

### WASM Development

```bash
cd wasm-viewer

# Rebuild WASM khi thay đổi
wasm-pack build --target web --dev

# Watch mode (cần cargo-watch)
cargo watch -s 'wasm-pack build --target web'
```

## 📝 Demo Accounts

Sau khi chạy migrations, bạn có thể đăng nhập với các tài khoản demo:

| Email | Password | Role |
|-------|----------|------|
| admin@crm.local | admin123 | admin |
| manager@crm.local | manager123 | manager |
| user@crm.local | user123 | user |

## 🔧 Troubleshooting

### Database Connection Error

```bash
# Kiểm tra PostgreSQL đang chạy
psql -U postgres -c "SELECT version();"

# Reset database
sqlx database drop
sqlx database create
sqlx migrate run
```

### WASM Build Error

```bash
# Cài lại wasm-pack
cargo install wasm-pack --force

# Thử build lại
cd wasm-viewer
wasm-pack build --target web
```

### Frontend Build Error

```bash
# Xóa node_modules và cài lại
cd frontend
rm -rf node_modules package-lock.json
npm install
```

### Port Already in Use

```bash
# Backend (port 3000)
lsof -ti:3000 | xargs kill -9

# Frontend (port 5173)
lsof -ti:5173 | xargs kill -9
```

## 🌐 Environment Variables

### Backend (.env)
```env
DATABASE_URL=postgresql://user:password@localhost:5432/crm_db
JWT_SECRET=your-super-secret-key-min-32-chars
JWT_EXPIRATION=86400
HOST=0.0.0.0
PORT=3000
CORS_ORIGIN=http://localhost:5173
MAX_FILE_SIZE=10485760
UPLOAD_DIR=./uploads
```

### Frontend (.env)
```env
VITE_API_URL=http://localhost:3000
VITE_WS_URL=ws://localhost:3000
VITE_APP_NAME=Neo-Brutalist CRM
```

## 📚 Next Steps

1. Đọc [API Documentation](./API.md) để hiểu các endpoints
2. Xem [Component Guide](./frontend/COMPONENTS.md) để biết cách sử dụng UI components
3. Tham khảo [Design System](./frontend/DESIGN_SYSTEM.md) cho Neo-Brutalist principles

## 🆘 Need Help?

- Check [README.md](./README.md) cho overview
- Xem [GitHub Issues](https://github.com/your-repo/issues)
- Đọc [Qwik Docs](https://qwik.builder.io/)
- Đọc [Axum Docs](https://docs.rs/axum/)

Happy Coding! 🎨✨
