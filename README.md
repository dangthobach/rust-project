# 🎨 Neo-Brutalist CRM System

A high-performance CRM application with Neo-Brutalist design, built with modern web technologies.

> **✅ LATEST UPDATE**: All frontend issues fixed! Using LTS versions.
> - [Package Updates](./FIXES_APPLIED.md)
> - [Component Syntax Fixes](./COMPONENT_FIXES.md)

## 🛠️ Tech Stack

- **Frontend**: Qwik 1.9.0 (Ultra-fast, Resumable)
- **Backend**: Rust + Axum (High-performance async web framework)
- **Database**: PostgreSQL + SQLx (Compile-time verified queries)
- **File Viewer**: Rust/WASM (Native performance in browser)
- **Styling**: Tailwind CSS 3.4.17 + Custom Neo-Brutalist Design System

## 📁 Project Structure

```
rust-system/
├── backend/          # Rust Axum API server
├── frontend/         # Qwik application
├── wasm-viewer/      # Rust/WASM file viewer
├── docker-compose.yml
└── README.md
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (https://rustup.rs/)
- Node.js 20+ (https://nodejs.org/)
- PostgreSQL 15+ (https://www.postgresql.org/)
- wasm-pack (https://rustwasm.github.io/wasm-pack/)

### Backend Setup

```bash
cd backend
cp .env.example .env
# Edit .env with your database credentials
cargo build
sqlx database create
sqlx migrate run
cargo run
```

Backend runs on: http://localhost:3000

### Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

Frontend runs on: http://localhost:5173

### WASM Viewer Setup

```bash
cd wasm-viewer
wasm-pack build --target web
```

## 📊 Features

- ✅ Client Management
- ✅ Task Management with Checkboxes
- ✅ Real-time Notifications (WebSocket)
- ✅ File Upload & Management
- ✅ WASM-powered File Viewer
- ✅ Neo-Brutalist UI Design
- ✅ Fully Responsive (Mobile, Tablet, Desktop)
- ✅ JWT Authentication
- ✅ Role-based Access Control

## 🎨 Design Philosophy

Neo-Brutalism principles:
- Bold, thick borders (3-5px solid black)
- Strong shadows (8px offset)
- Vibrant accent colors (neon green, electric blue)
- Asymmetric grid layouts
- Geometric sans-serif typography
- High contrast UI elements

## 📝 API Documentation

API docs available at: http://localhost:3000/api/docs (coming soon)

## 🧪 Testing

```bash
# Backend tests
cd backend && cargo test

# Frontend tests
cd frontend && npm test
```

## 🐳 Docker Deployment

```bash
docker-compose up -d
```

## 📄 License

MIT License - See LICENSE file for details

## 👨‍💻 Author

Expert Web Developer specializing in Performance & Neo-Brutalist Design
