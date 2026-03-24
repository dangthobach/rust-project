# Neo CRM - Modern Customer Relationship Management System

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Backend: Rust](https://img.shields.io/badge/Backend-Rust-orange.svg)](https://www.rust-lang.org/)
[![Frontend: SolidJS](https://img.shields.io/badge/Frontend-SolidJS-blue.svg)](https://www.solidjs.com/)
[![Database: SQLite](https://img.shields.io/badge/Database-SQLite-green.svg)](https://www.sqlite.org/)

A full-stack, production-ready CRM system built with modern technologies, featuring advanced architecture patterns including CQRS/Event Sourcing, role-based access control, and comprehensive API endpoints.

---

## 🌟 Features

### Core Functionality
- **Customer Management** - Complete CRUD operations with full-text search
- **Task Management** - Assign, track, and manage tasks with priorities and due dates
- **User Management** - Multi-role user system (Admin, Manager, User)
- **File Management** - Secure file upload/download with validation (max 10MB)
- **Notifications** - Real-time notification system
- **Analytics Dashboard** - Business insights and metrics visualization
- **Activity Logging** - Complete audit trail for all operations

### Security & Authentication
- **JWT Authentication** - Access tokens (24h) + Refresh tokens (30 days)
- **Token Rotation** - Automatic refresh with multi-device logout support
- **Role-Based Access Control (RBAC)** - Fine-grained permissions system
- **Rate Limiting** - Protection against brute force attacks
  - Auth endpoints: 5 req/min
  - Upload endpoints: 10 req/min
  - General endpoints: 100 req/min
- **File Upload Validation** - MIME type whitelist, extension blacklist, sanitization
- **bcrypt Password Hashing** - Industry-standard password security

### Performance & Scalability
- **Pagination** - All list endpoints support efficient pagination
- **Full-Text Search** - SQLite FTS5 for fast search across clients, tasks, files
- **Database Indexes** - 13 performance indexes for optimized queries
- **Docker Ready** - Multi-stage builds with health checks
- **Optional Monitoring** - Prometheus + Grafana integration

### Advanced Architecture
- **CQRS/Event Sourcing** - Command Query Responsibility Segregation pattern
- **Event Store** - Complete event history with snapshots
- **Domain-Driven Design** - Clean separation of business domains
- **Repository Pattern** - Abstract database access layer

---

## 🚀 Quick Start

### Option 1: Docker (Recommended)

```bash
# Clone the repository
git clone <repository-url>
cd rust-project

# Set JWT secret (required)
export JWT_SECRET="your-super-secret-jwt-key-min-32-characters"

# Start all services
docker-compose up -d

# Verify services are running
curl http://localhost:3000/health
```

**Services:**
- Backend API: http://localhost:3000
- Frontend: http://localhost:5173
- Health Check: http://localhost:3000/health

### Option 2: Docker with Monitoring

```bash
# Start with Prometheus + Grafana
docker-compose --profile monitoring up -d

# Access monitoring
# - Prometheus: http://localhost:9091
# - Grafana: http://localhost:3001 (admin/admin)
```

### Option 3: Local Development

#### Prerequisites
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 20+ ([Install Node](https://nodejs.org/))
- Redis (optional, for CQRS event bus)

#### Backend Setup

```bash
cd backend

# Copy environment file
cp .env.example .env
# Edit .env and set JWT_SECRET

# Install SQLx CLI (for migrations)
cargo install sqlx-cli --no-default-features --features sqlite

# Run database migrations
sqlx migrate run

# Start backend server
cargo run --release

# Backend will start on http://localhost:3000
```

#### Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Start development server
npm run dev

# Frontend will start on http://localhost:5173
```

---

## 🔧 Configuration

### Environment Variables

Create `.env` file in the `backend/` directory:

```bash
# Database
DATABASE_URL=sqlite:./data/crm.db

# JWT Authentication (REQUIRED - generate a secure random string)
JWT_SECRET=your-super-secret-jwt-key-change-in-production-min-32-chars
JWT_EXPIRATION=86400  # 24 hours in seconds

# Server
HOST=0.0.0.0
PORT=3000

# CORS (set to your frontend URL in production)
CORS_ORIGIN=http://localhost:5173

# File Upload
MAX_FILE_SIZE=10485760  # 10MB in bytes
UPLOAD_DIR=./uploads

# Redis (optional, for CQRS event bus)
REDIS_URL=redis://127.0.0.1:6379

# Logging
RUST_LOG=info  # Options: trace, debug, info, warn, error
```

### Frontend Configuration

Create `.env` file in the `frontend/` directory:

```bash
VITE_API_URL=http://localhost:3000
```

### Generating Secure JWT_SECRET

```bash
# Option 1: Using openssl
openssl rand -hex 32

# Option 2: Using Python
python3 -c "import secrets; print(secrets.token_hex(32))"

# Option 3: Using Node.js
node -e "console.log(require('crypto').randomBytes(32).toString('hex'))"
```

---

## 📖 API Documentation

### Base URL
```
http://localhost:3000/api
```

### Authentication Endpoints

| Method | Endpoint | Description | Rate Limit |
|--------|----------|-------------|------------|
| POST | `/auth/register` | Register new user | 5/min |
| POST | `/auth/login` | Login with credentials | 5/min |
| POST | `/auth/refresh` | Refresh access token | 5/min |
| POST | `/auth/logout` | Logout current device | 5/min |
| POST | `/auth/logout-all` | Logout all devices | 5/min |

### User Endpoints (Authenticated)

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| GET | `/users/me` | Get current user | All |
| GET | `/users/profile` | Get user profile | All |
| GET | `/users/:id` | Get user by ID | All |
| PATCH | `/users/:id` | Update user | All |
| POST | `/users/password` | Change password | All |
| POST | `/users/avatar` | Upload avatar | All |

### Client Endpoints (Authenticated)

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| GET | `/clients` | List clients (paginated) | All |
| GET | `/clients/search` | Search clients | All |
| GET | `/clients/:id` | Get client details | All |
| POST | `/clients` | Create client | Manager/Admin |
| PATCH | `/clients/:id` | Update client | Manager/Admin |
| DELETE | `/clients/:id` | Delete client | Manager/Admin |

### Task Endpoints (Authenticated)

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| GET | `/tasks` | List tasks (paginated) | All |
| GET | `/tasks/search` | Search tasks | All |
| GET | `/tasks/:id` | Get task details | All |
| POST | `/tasks` | Create task | All |
| PATCH | `/tasks/:id` | Update task | All |
| DELETE | `/tasks/:id` | Delete task | Manager/Admin |

### File Endpoints (Authenticated)

| Method | Endpoint | Description | Rate Limit |
|--------|----------|-------------|------------|
| GET | `/files` | List files (paginated) | - |
| GET | `/files/search` | Search files | - |
| GET | `/files/:id` | Get file metadata | - |
| POST | `/files/upload` | Upload file (max 10MB) | 10/min |
| GET | `/files/:id/download` | Download file | - |
| DELETE | `/files/:id` | Delete file | - |

### Admin Endpoints (Admin Only)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/admin/users` | List all users (paginated) |
| GET | `/admin/users/search` | Search users |
| GET | `/admin/users/stats` | User statistics |
| POST | `/admin/users` | Create user |
| POST | `/admin/users/bulk` | Bulk user actions |
| PATCH | `/admin/users/:id` | Update user |
| DELETE | `/admin/users/:id` | Delete user |

### Analytics Endpoints (Admin Only)

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/analytics/user-activity` | User activity metrics |
| GET | `/analytics/task-completion` | Task completion analytics |
| GET | `/analytics/client-engagement` | Client engagement metrics |
| GET | `/analytics/storage-usage` | Storage usage analytics |

### Export Endpoints

| Method | Endpoint | Description | Permission |
|--------|----------|-------------|------------|
| GET | `/export/users` | Export users (CSV/JSON) | Admin |
| GET | `/export/clients` | Export clients (CSV/JSON/PDF) | All |
| GET | `/export/tasks` | Export tasks (CSV/JSON/PDF) | All |
| GET | `/export/dashboard-report` | Dashboard report | All |

### Example API Requests

#### Login
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "admin@example.com", "password": "password123"}'
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "550e8400-e29b-41d4-a716-446655440000",
  "token_type": "Bearer",
  "expires_in": 86400,
  "user": {
    "id": "user-uuid",
    "email": "admin@example.com",
    "name": "Admin User",
    "role": "admin"
  }
}
```

#### List Clients (Paginated)
```bash
curl http://localhost:3000/api/clients?page=1&limit=20 \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

**Response:**
```json
{
  "data": [
    {
      "id": "client-uuid",
      "name": "Acme Corp",
      "email": "contact@acme.com",
      "phone": "+1234567890",
      "status": "active"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 45,
    "total_pages": 3,
    "has_next": true,
    "has_prev": false
  }
}
```

#### Upload File
```bash
curl -X POST http://localhost:3000/api/files/upload \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -F "file=@document.pdf" \
  -F "related_entity_type=client" \
  -F "related_entity_id=client-uuid"
```

---

## 👥 User Roles & Permissions

### Role Hierarchy
```
Admin > Manager > User
```

### Permissions Matrix

| Resource | Create | Read | Update | Delete |
|----------|--------|------|--------|--------|
| **Clients** | Manager/Admin | All | Manager/Admin | Manager/Admin |
| **Tasks** | All | All | All | Manager/Admin |
| **Files** | All | All | All | Manager/Admin |
| **Users** | Admin | All | Owner/Admin | Admin |
| **Analytics** | - | Admin | - | - |

### Default Test Users

After running migrations, you can use these accounts:

```
Admin:
  Email: admin@example.com
  Password: admin123
  Role: admin

Manager:
  Email: manager@example.com
  Password: manager123
  Role: manager

User:
  Email: user@example.com
  Password: user123
  Role: user
```

**⚠️ Change these passwords immediately in production!**

---

## 🏗️ Architecture

### Technology Stack

#### Backend
- **Language:** Rust 1.70+
- **Framework:** Axum 0.7 (async web framework)
- **Database:** SQLite with SQLx (compile-time query verification)
- **Authentication:** JWT (jsonwebtoken 9.2) + bcrypt
- **Event Bus:** Redis Streams (optional, for CQRS)
- **Logging:** tracing + tracing-subscriber

#### Frontend
- **Framework:** SolidJS 1.8.7 (reactive UI framework)
- **Router:** @solidjs/router 0.10.3
- **State Management:** @tanstack/solid-query 5.56.2
- **Charts:** Chart.js 4.5.1 + solid-chartjs
- **Styling:** TailwindCSS 3.4.17 (Neo-Brutalist design)
- **Build Tool:** Vite 5.4.11

#### Infrastructure
- **Containerization:** Docker + Docker Compose
- **Monitoring:** Prometheus + Grafana (optional)
- **Reverse Proxy:** (nginx recommended for production)

### Project Structure

```
rust-project/
├── backend/
│   ├── src/
│   │   ├── api/              # CQRS API handlers
│   │   ├── core/             # Domain-driven design core
│   │   │   ├── cqrs/         # Command/Query infrastructure
│   │   │   ├── domain/       # Domain entities & aggregates
│   │   │   ├── events/       # Event sourcing
│   │   │   └── infrastructure/
│   │   ├── domains/          # Business domains
│   │   │   ├── clients/      # Client domain
│   │   │   ├── tasks/        # Task domain
│   │   │   ├── users/        # User domain
│   │   │   └── file_system/  # File system domain
│   │   ├── handlers/         # HTTP request handlers
│   │   ├── middleware/       # Auth, RBAC, rate limiting
│   │   ├── models/           # Database models
│   │   ├── utils/            # Utilities
│   │   ├── app_state.rs      # Application state
│   │   ├── config.rs         # Configuration
│   │   ├── error.rs          # Error handling
│   │   ├── routes.rs         # Route definitions
│   │   └── main.rs           # Entry point
│   ├── migrations/           # SQLx database migrations
│   ├── tests/                # Integration tests
│   ├── Cargo.toml
│   └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── components/       # Reusable components
│   │   │   ├── crm/          # CRM-specific components
│   │   │   └── ui/           # UI primitives
│   │   ├── pages/            # Page components
│   │   ├── lib/              # API client & utilities
│   │   │   ├── hooks/        # Custom hooks
│   │   │   ├── api.ts        # API client (931 lines)
│   │   │   └── queries.ts    # Query factory
│   │   ├── theme/            # Design tokens
│   │   ├── App.tsx           # Router configuration
│   │   └── index.tsx         # Entry point
│   ├── package.json
│   └── Dockerfile
├── docker-compose.yml
├── prometheus.yml
├── CHANGELOG.md
└── README.md
```

### CQRS/Event Sourcing

The system implements CQRS (Command Query Responsibility Segregation) and Event Sourcing patterns:

- **Commands** - Modify state (CreateClient, UpdateTask, etc.)
- **Queries** - Read state (GetClient, ListTasks, etc.)
- **Events** - Capture state changes (ClientCreated, TaskUpdated, etc.)
- **Event Store** - SQLite-based event log with snapshots
- **Event Bus** - Redis Streams for event publishing (optional)

**Note:** CQRS is partially implemented. Client operations use CQRS handlers, while Tasks/Users use traditional handlers. Redis is required for full CQRS functionality but can be disabled.

For detailed architecture documentation, see [backend/ARCHITECTURE.md](backend/ARCHITECTURE.md).

---

## 🧪 Testing

### Backend Tests

```bash
cd backend

# Run all tests
cargo test

# Run specific test suite
cargo test --test user_management_tests

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

**Test Coverage:**
- Pagination utility: 8 unit tests
- File validator: 10 unit tests
- RBAC middleware: 4 unit tests
- Integration tests: 15 test functions (user management)

### Frontend Tests

```bash
cd frontend

# Type checking
npm run typecheck

# Linting
npm run lint

# Format check
npm run fmt.check

# Format code
npm run fmt
```

**Note:** Unit and E2E tests are planned but not yet implemented.

---

## 🐳 Docker Deployment

### Build Images

```bash
# Build all images
docker-compose build

# Build specific service
docker-compose build backend
docker-compose build frontend
```

### Manage Services

```bash
# Start services
docker-compose up -d

# View logs
docker-compose logs -f backend
docker-compose logs -f frontend

# Stop services
docker-compose down

# Stop and remove volumes
docker-compose down -v

# Restart specific service
docker-compose restart backend
```

### Access Services

- **Backend API:** http://localhost:3000
- **Frontend:** http://localhost:5173
- **Prometheus:** http://localhost:9091 (with monitoring profile)
- **Grafana:** http://localhost:3001 (with monitoring profile)
- **Health Check:** http://localhost:3000/health

---

## 🔨 Development

### Backend Development

```bash
cd backend

# Watch mode (auto-reload on changes)
cargo watch -x run

# Check compilation without building
cargo check

# Run with specific log level
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Frontend Development

```bash
cd frontend

# Development server with hot reload
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Type checking (watch mode)
npm run typecheck -- --watch
```

### Database Management

```bash
cd backend

# Create new migration
sqlx migrate add migration_name

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Generate SQLx offline data (for CI)
cargo sqlx prepare
```

---

## 📊 Database Schema

### Core Tables

- **users** - User accounts with roles (admin, manager, user)
- **clients** - CRM client records
- **tasks** - Task management with status/priority
- **notifications** - User notifications
- **files** - File uploads with metadata
- **activities** - Activity logging
- **refresh_tokens** - JWT refresh token storage
- **audit_logs** - Audit trail for admin actions

### Event Sourcing Tables

- **event_store** - Domain events storage
- **snapshots** - Aggregate snapshots for performance
- **projection_positions** - Event projection checkpoints

### File System Tables

- **file_system_files** - File metadata
- **file_system_folders** - Folder structure
- **folder_tree** - Closure table for efficient tree queries

### Full-Text Search

- SQLite FTS5 tables for clients, tasks, and files with rank-based search

**Total Migrations:** 13 migration files in `backend/migrations/`

---

## 🔒 Security Best Practices

### Production Checklist

- [ ] Generate strong JWT_SECRET (min 32 characters)
- [ ] Change default user passwords
- [ ] Set CORS_ORIGIN to your frontend domain
- [ ] Use HTTPS/TLS in production
- [ ] Configure firewall rules (only expose necessary ports)
- [ ] Enable audit logging
- [ ] Set up database backups
- [ ] Configure rate limiting appropriately
- [ ] Review file upload limits
- [ ] Set secure cookie flags if using sessions
- [ ] Enable monitoring (Prometheus + Grafana)
- [ ] Review and update dependencies regularly

### File Upload Security

The system implements multiple layers of file upload security:

1. **MIME Type Whitelist** - Only 27 safe MIME types allowed
2. **Extension Blacklist** - 18 dangerous extensions blocked (.exe, .bat, etc.)
3. **Size Limit** - Configurable max size (default 10MB)
4. **Filename Sanitization** - Removes path traversal characters
5. **Rate Limiting** - 10 uploads per minute

### Authentication Security

- **bcrypt** password hashing with salt
- **JWT** tokens with configurable expiration
- **Refresh token rotation** on each use
- **Multi-device logout** support
- **Token revocation** on logout
- **Rate limiting** on auth endpoints (5 req/min)

---

## 📈 Performance Considerations

### Database Optimization

- **13 Performance Indexes** - Optimized queries for common operations
- **FTS5 Full-Text Search** - Fast search across large datasets
- **Pagination** - All list endpoints paginated (limit 1-100)
- **Connection Pooling** - SQLx connection pool (max 50 connections)
- **ANALYZE** - Query planner statistics updated

### SQLite Limitations

SQLite is excellent for development and small-to-medium deployments, but has limitations:

- **Single Writer** - Only one write transaction at a time
- **Concurrent Connections** - Limited compared to PostgreSQL/MySQL
- **File Locking** - Can cause issues under heavy concurrent writes

**Recommendation:** For production deployments expecting high concurrency (>100 simultaneous users with write operations), consider migrating to PostgreSQL.

### Caching Strategy

- **Frontend:** @tanstack/solid-query caches API responses automatically
- **Backend:** No application-level caching yet (planned feature)

---

## 🚧 Known Limitations

1. **Activity Routes Disabled** - Implementation exists but commented out due to state management issues
2. **WebSocket Disabled** - Implementation exists but commented out due to futures compilation issues
3. **CQRS Partial** - Only Client domain uses CQRS handlers; Tasks/Users use traditional approach
4. **Redis Required** - Application won't start without Redis (even if CQRS not used)
5. **Integration Tests Incomplete** - Test structure exists but implementations are placeholders
6. **No E2E Tests** - Frontend lacks automated testing
7. **Metrics Commented Out** - Prometheus metrics code exists but disabled

See [CHANGELOG.md](CHANGELOG.md) for detailed status and planned improvements.

---

## 🗺️ Roadmap

### Planned Features

- [ ] Enable WebSocket support for real-time notifications
- [ ] Complete CQRS migration or rollback to traditional handlers
- [ ] Make Redis optional with in-memory fallback
- [ ] Add comprehensive test coverage (target: 80%+)
- [ ] OpenAPI/Swagger documentation
- [ ] Email notifications (SMTP integration)
- [ ] Two-factor authentication (2FA)
- [ ] Advanced analytics with custom reports
- [ ] Mobile app (React Native or Flutter)
- [ ] Import from CSV/Excel
- [ ] Calendar view for tasks
- [ ] Kanban board for task management
- [ ] Custom fields for clients/tasks
- [ ] Workflow automation
- [ ] Integration with third-party services (Zapier, etc.)

### Performance Improvements

- [ ] Response caching (ETag, Last-Modified headers)
- [ ] Lazy loading for Chart.js
- [ ] Database query optimization (eliminate N+1 queries)
- [ ] File streaming for large uploads
- [ ] PostgreSQL migration option
- [ ] Read replicas for scaling

---

## 🤝 Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust formatting standards (`cargo fmt`)
- Run linter before commit (`cargo clippy`)
- Add tests for new features
- Update documentation as needed
- Write meaningful commit messages

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 📞 Support

- **Documentation:** See [CHANGELOG.md](CHANGELOG.md) for detailed release notes
- **Issues:** Report bugs or request features via GitHub Issues
- **Architecture:** See [backend/ARCHITECTURE.md](backend/ARCHITECTURE.md) for detailed architecture documentation

---

## 🙏 Acknowledgments

- **Axum** - Fast and ergonomic web framework for Rust
- **SolidJS** - High-performance reactive UI library
- **SQLite** - Reliable embedded database
- **SQLx** - Compile-time SQL verification for Rust
- **TailwindCSS** - Utility-first CSS framework

---

## 📝 Version History

- **v2.0.0** (2025-11-15) - Production-ready features (RBAC, pagination, rate limiting, comprehensive security)
- **v1.0.0** (2025-11-01) - Initial release (basic CRUD, JWT auth, file upload)

See [CHANGELOG.md](CHANGELOG.md) for detailed version history.

---

**Built with ❤️ using Rust, SolidJS, and modern web technologies**
