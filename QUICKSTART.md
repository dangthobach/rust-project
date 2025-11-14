# ⚡ Quick Start Guide

## 🚀 Cài đặt nhanh (5 phút)

### Bước 1: Cài dependencies

```bash
cd d:\project\rust-system\frontend

# Xóa node_modules cũ (nếu có)
rm -rf node_modules package-lock.json

# Cài lại với versions mới
npm install
```

### Bước 2: Chạy Frontend

```bash
# Trong thư mục frontend
npm run dev
```

Frontend sẽ chạy tại: **http://localhost:5173**

### Bước 3: Chạy Backend (Terminal mới)

```bash
cd d:\project\rust-system\backend

# Tạo file .env
cp .env.example .env

# Sửa DATABASE_URL trong .env (nếu cần)
# DATABASE_URL=postgresql://postgres:password@localhost:5432/crm_db

# Tạo database
sqlx database create

# Chạy migrations
sqlx migrate run

# Chạy backend
cargo run
```

Backend API sẽ chạy tại: **http://localhost:3000**

### Bước 4: Test

Mở browser và vào: **http://localhost:5173**

Bạn sẽ thấy trang landing page Neo-Brutalist đầy màu sắc!

---

## 🎨 Các thay đổi quan trọng

### ✅ Packages đã update lên LTS versions

- **Qwik**: 1.5.0 → **1.9.0** (stable LTS)
- **TypeScript**: 5.3.3 → **5.6.3** (latest LTS)
- **Vite**: 5.0.12 → **5.4.11** (stable)
- **Tailwind**: 3.4.1 → **3.4.17** (latest)
- **Autoprefixer**: 10.4.17 → **10.4.20**

### ✅ Đã xóa dependencies không cần thiết

- ❌ `@modular-forms/qwik` (không dùng)
- ❌ `clsx` (replaced với custom `cn()` function)

### ✅ Đã fix

- ✅ Added `"type": "module"` vào package.json
- ✅ Fixed `cn()` utility function (không phụ thuộc clsx)
- ✅ Tạo entry points (entry.dev.tsx, entry.ssr.tsx)
- ✅ Tạo layout.tsx cho routing
- ✅ Tạo public assets (favicon, manifest)

---

## 📦 Demo Accounts

Sau khi chạy migrations, login với:

| Email | Password | Role |
|-------|----------|------|
| admin@crm.local | admin123 | admin |
| manager@crm.local | manager123 | manager |
| user@crm.local | user123 | user |

---

## 🔧 Troubleshooting

### Lỗi: "Cannot find module"

```bash
cd frontend
rm -rf node_modules package-lock.json .turbo
npm install
```

### Lỗi: "Database does not exist"

```bash
cd backend
sqlx database create
sqlx migrate run
```

### Lỗi: "Port 5173 already in use"

```bash
# Kill process trên port 5173
# Windows:
netstat -ano | findstr :5173
taskkill /PID <PID> /F

# Linux/Mac:
lsof -ti:5173 | xargs kill -9
```

### Lỗi: SQLx compile error

```bash
cd backend
cargo clean
cargo sqlx prepare --database-url="postgresql://postgres:password@localhost:5432/crm_db"
cargo build
```

---

## 🎯 Next Steps

1. ✅ Frontend đang chạy
2. ✅ Backend đang chạy
3. ✅ Database đã setup

**Giờ bạn có thể:**

- Xem UI components tại: [http://localhost:5173](http://localhost:5173)
- Test API tại: [http://localhost:3000/health](http://localhost:3000/health)
- Đọc [API.md](./API.md) để biết các endpoints
- Đọc [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) để bắt đầu code

---

## 💡 Quick Commands

```bash
# Frontend
npm run dev          # Development server
npm run build        # Production build
npm run preview      # Preview production build
npm run typecheck    # Type checking
npm run lint         # Lint code
npm run fmt          # Format code

# Backend
cargo run           # Run dev server
cargo build         # Build
cargo test          # Run tests
cargo fmt           # Format code
cargo clippy        # Lint code

# Database
sqlx migrate add <name>    # Create new migration
sqlx migrate run           # Run migrations
sqlx database create       # Create database
sqlx database drop         # Drop database
```

---

Happy Coding! 🎨✨
