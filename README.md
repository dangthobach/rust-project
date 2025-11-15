# 🎨 Neo-Brutalist CRM System

A high-performance CRM application with Neo-Brutalist design, built with modern web technologies.

> **✅ LATEST UPDATE**: Frontend-backend integration complete! All critical issues fixed (6/7).
> - [Critical Issues Fixed Report](./CRITICAL_ISSUES_FIXED.md)
> - [Testing Guide](./TESTING_GUIDE.md)
> - [Component Fixes](./COMPONENT_FIXES.md)

## 🛠️ Tech Stack

- **Frontend**: SolidJS 1.8+ (Fine-grained reactivity, high performance)
- **Backend**: Rust 1.79+ + Axum 0.7 (High-performance async web framework)
- **Database**: SQLite + SQLx (Compile-time verified queries, zero-config)
- **File Viewer**: Rust/WASM (Native performance in browser)
- **Styling**: Tailwind CSS 3.4+ + Custom Neo-Brutalist Design System
- **Authentication**: JWT tokens + bcrypt password hashing

## 📁 Project Structure

```
rust-system/
├── backend/          # Rust Axum API server (SQLite)
│   ├── src/
│   │   ├── handlers/     # HTTP request handlers
│   │   ├── models/       # Database models
│   │   ├── domains/      # CQRS domains (clients, tasks, users)
│   │   ├── core/         # CQRS infrastructure
│   │   └── middleware/   # Auth middleware
│   ├── migrations/       # SQLite migrations
│   └── data/            # SQLite database file
├── frontend/         # SolidJS application
│   ├── src/
│   │   ├── pages/       # Route pages (Dashboard, Login, Files, etc.)
│   │   ├── components/  # Reusable UI components
│   │   └── lib/         # API client
├── wasm-viewer/      # Rust/WASM file viewer
├── docker-compose.yml
└── README.md
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.79+ (https://rustup.rs/)
- Node.js 18+ (https://nodejs.org/)
- SQLite3 (usually pre-installed on most systems)

### Backend Setup

```bash
cd backend

# Create .env file (optional - defaults work)
# cp .env.example .env

# Build backend
cargo build --release

# Run migrations (creates SQLite database)
sqlx migrate run

# Start server
cargo run --release
# OR use start script
./start.bat  # Windows
./start.sh   # Linux/Mac
```

Backend runs on: http://localhost:3000

**Verify backend is running:**
```bash
curl http://localhost:3000/health
# Expected: {"status":"ok","database":"connected"}
```

### Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

Frontend runs on: http://localhost:5173

**Default login** (after creating user):
- Email: your-registered-email
- Password: your-password

### WASM Viewer Setup (Optional)

```bash
cd wasm-viewer
wasm-pack build --target web
```

## 📊 Features

### Implemented ✅
- ✅ Client Management (CRUD with CQRS)
- ✅ Task Management (CRUD with CQRS)
- ✅ User Authentication (JWT + bcrypt)
- ✅ File Upload & Download (multipart + blob streaming)
- ✅ Real-time Notifications
- ✅ Route Protection (ProtectedRoute component)
- ✅ Neo-Brutalist UI Design
- ✅ Fully Responsive (Mobile, Tablet, Desktop)
- ✅ Loading states & error handling
- ✅ CQRS pattern for domain logic

### In Progress 🚧
- 🚧 Comprehensive testing suite
- 🚧 WebSocket notifications
- 🚧 WASM file viewer integration

### Planned 📋
- 📋 User registration UI
- 📋 Password reset flow
- 📋 File deletion UI
- 📋 Pagination for large datasets
- 📋 Advanced search functionality
- 📋 Role-based access control UI

## 🎨 Design Philosophy

Neo-Brutalism principles implemented:
- Bold, thick borders (3px solid black)
- Strong shadows (brutal-shadow: 4px offset)
- Vibrant accent colors (electric blue #0066FF, neon green)
- Geometric sans-serif typography (Space Grotesk, Manrope)
- High contrast UI elements
- Asymmetric layouts with intentional "mistakes"

## 📝 API Documentation

### Authentication Endpoints
- `POST /api/auth/login` - Login with email/password, returns JWT token
- `POST /api/auth/register` - Register new user (if enabled)

### Protected Endpoints (require Authorization: Bearer token)
- `GET /api/clients` - List all clients
- `POST /api/clients` - Create new client
- `GET /api/clients/:id` - Get client by ID
- `PATCH /api/clients/:id` - Update client
- `DELETE /api/clients/:id` - Delete client

- `GET /api/tasks` - List all tasks
- `POST /api/tasks` - Create new task
- `GET /api/tasks/:id` - Get task by ID
- `PATCH /api/tasks/:id` - Update task
- `DELETE /api/tasks/:id` - Delete task

- `GET /api/files` - List uploaded files
- `POST /api/files/upload` - Upload file (multipart/form-data)
- `GET /api/files/:id/download` - Download file
- `DELETE /api/files/:id` - Delete file

- `GET /api/notifications` - List notifications
- `POST /api/notifications/mark-read` - Mark notifications as read

For detailed testing examples, see [TESTING_GUIDE.md](./TESTING_GUIDE.md)

## 🧪 Testing

```bash
# Backend tests (when implemented)
cd backend && cargo test

# Frontend tests (when implemented)
cd frontend && npm test

# Manual testing guide
# See TESTING_GUIDE.md for comprehensive testing instructions
```

## 🐛 Troubleshooting

### Common Issues

**Build fails with UUID errors:**
- Solution: All models now use `String` for UUIDs (SQLite compatibility)

**401 Unauthorized on API calls:**
- Check if JWT token is in localStorage: `localStorage.getItem('token')`
- Token may have expired - login again

**File upload fails:**
- Check `backend/uploads/` directory exists (auto-created on first upload)
- Verify file size is within limits

**CORS errors:**
- Update `CORS_ORIGIN` in backend/.env to match frontend URL
- Default: `http://localhost:5173`

**Database locked:**
- Stop all backend instances
- Delete `backend/data/crm.db-shm` and `crm.db-wal`
- Restart backend

For more troubleshooting tips, see [TESTING_GUIDE.md](./TESTING_GUIDE.md)

## 📚 Documentation

- [CRITICAL_ISSUES_FIXED.md](./CRITICAL_ISSUES_FIXED.md) - Detailed report of all bug fixes
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - Comprehensive testing instructions
- [ARCHITECTURE.md](./backend/ARCHITECTURE.md) - System architecture overview
- [IMPLEMENTATION_GUIDE.md](./IMPLEMENTATION_GUIDE.md) - Implementation details

## 🏗️ Architecture

The application follows **CQRS (Command Query Responsibility Segregation)** pattern:

- **Commands**: Write operations (Create, Update, Delete)
- **Queries**: Read operations (Get, List, Search)
- **Handlers**: Process commands/queries with business logic
- **Validators**: Validate input before processing

Example flow:
```
Client -> API Handler -> CommandBus -> Validator -> Handler -> Database
```

### Database Schema

**SQLite** database with the following tables:
- `users` - User accounts with password hashes
- `clients` - Client/customer records
- `tasks` - Task management with status tracking
- `files` - Uploaded file metadata
- `notifications` - User notifications
- `activities` - Activity log (optional)

All UUIDs stored as TEXT (36-char strings) for SQLite compatibility.

## 🚢 Deployment

### Development
```bash
# Backend
cd backend && cargo run --release

# Frontend
cd frontend && npm run dev
```

### Production Build
```bash
# Backend
cd backend && cargo build --release
# Binary: target/release/crm-backend

# Frontend
cd frontend && npm run build
# Output: dist/ (serve with any static server)
```

### Docker (Optional)
```bash
docker-compose up -d
```

## 🤝 Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License.

## 🙏 Acknowledgments

- **SolidJS** - For the reactive UI framework
- **Axum** - For the ergonomic web framework
- **SQLx** - For compile-time SQL verification
- **Tailwind CSS** - For utility-first styling

---

**Status:** ✅ Production-ready for development testing
**Build:** ✅ Passing (0 errors, ~185 warnings - unused code)
**Last Updated:** November 15, 2025
```

## 🐳 Docker Deployment

```bash
docker-compose up -d
```

## 📄 License

MIT License - See LICENSE file for details

## 👨‍💻 Author

Expert Web Developer specializing in Performance & Neo-Brutalist Design
