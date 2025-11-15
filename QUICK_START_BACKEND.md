# ⚡ QUICK START - Backend Development
**Hướng dẫn bắt đầu triển khai Backend Roadmap**

---

## 🎯 MỤC TIÊU TUẦN NÀY (Week 1)

**Phase 1: Stabilization & Testing**
- ✅ Setup testing infrastructure
- ✅ Write 60+ tests (unit + integration)
- ✅ Achieve 50%+ test coverage
- ✅ Complete API documentation

**Timeline:** 5 working days
**Effort:** ~40 hours total

---

## 📋 DAY-BY-DAY CHECKLIST

### 🔴 DAY 1: Testing Setup & Model Tests (8h)

#### Morning (4h)
- [ ] **Task 1.1: Add test dependencies** (30min)
  ```bash
  cd backend
  # Edit Cargo.toml, add to [dev-dependencies]:
  ```
  ```toml
  [dev-dependencies]
  reqwest = { version = "0.11", features = ["json"] }
  tokio-test = "0.4"
  mockall = "0.12"
  fake = "2.9"
  ```
  ```bash
  cargo build --dev
  ```

- [ ] **Task 1.2: Create test utilities** (2h)
  ```bash
  # Create file: backend/src/test_utils.rs
  ```
  **Must have:**
  - `async fn setup_test_db() -> SqlitePool`
  - `async fn create_test_user() -> User`
  - `fn generate_test_jwt() -> String`
  - `async fn cleanup_test_db(pool: &SqlitePool)`

- [ ] **Task 1.3: Add test module to main** (30min)
  ```rust
  // In backend/src/main.rs
  #[cfg(test)]
  mod test_utils;
  ```

- [ ] **Task 1.4: Verify test setup** (1h)
  ```bash
  cargo test --lib
  # Should show "0 tests, 0 passed"
  ```

#### Afternoon (4h)
- [ ] **Task 1.5: User model tests** (2h)
  ```bash
  # Create: backend/src/models/user_test.rs
  ```
  **Tests to write:**
  - [x] `test_user_serialization`
  - [x] `test_user_deserialization`
  - [x] `test_user_from_row`
  - [x] `test_user_creation`
  - [x] `test_user_validation`

- [ ] **Task 1.6: Client model tests** (1h)
  **Tests to write:**
  - [x] `test_client_creation`
  - [x] `test_client_validation`
  - [x] `test_client_email_format`
  - [x] `test_client_phone_format`

- [ ] **Task 1.7: Task model tests** (1h)
  **Tests to write:**
  - [x] `test_task_creation`
  - [x] `test_task_status_transition`
  - [x] `test_task_priority_validation`
  - [x] `test_task_assignment`

**End of Day 1 Goal:**
```bash
cargo test
# Expected: 20+ tests passing
```

---

### 🟡 DAY 2: Integration Test Framework (8h)

#### Morning (4h)
- [ ] **Task 2.1: Create integration test structure** (1h)
  ```bash
  mkdir -p backend/tests
  touch backend/tests/common/mod.rs
  touch backend/tests/auth_tests.rs
  ```

- [ ] **Task 2.2: Test database helper** (2h)
  ```rust
  // In backend/tests/common/mod.rs

  pub async fn setup_test_app() -> (Router, SqlitePool) {
      // Create in-memory test database
      // Run migrations
      // Create test app
      // Return router + pool
  }

  pub async fn create_auth_header(pool: &SqlitePool) -> String {
      // Create test user
      // Generate JWT
      // Return "Bearer <token>"
  }
  ```

- [ ] **Task 2.3: Verify test app works** (1h)
  ```bash
  cargo test --test auth_tests
  # Should compile successfully
  ```

#### Afternoon (4h)
- [ ] **Task 2.4: Auth tests - Register** (2h)
  ```rust
  // In backend/tests/auth_tests.rs

  #[tokio::test]
  async fn test_register_success() { }

  #[tokio::test]
  async fn test_register_duplicate_email() { }

  #[tokio::test]
  async fn test_register_invalid_email() { }

  #[tokio::test]
  async fn test_register_weak_password() { }
  ```

- [ ] **Task 2.5: Auth tests - Login** (2h)
  ```rust
  #[tokio::test]
  async fn test_login_success() { }

  #[tokio::test]
  async fn test_login_wrong_password() { }

  #[tokio::test]
  async fn test_login_user_not_found() { }

  #[tokio::test]
  async fn test_login_returns_jwt() { }
  ```

**End of Day 2 Goal:**
```bash
cargo test
# Expected: 30+ tests passing
# Auth flow fully tested
```

---

### 🟢 DAY 3: Handler Tests (8h)

#### Morning (4h)
- [ ] **Task 3.1: Client handler tests** (2h)
  ```rust
  // In backend/tests/client_tests.rs

  #[tokio::test]
  async fn test_create_client() { }

  #[tokio::test]
  async fn test_list_clients() { }

  #[tokio::test]
  async fn test_get_client() { }

  #[tokio::test]
  async fn test_update_client() { }

  #[tokio::test]
  async fn test_delete_client() { }

  #[tokio::test]
  async fn test_client_requires_auth() { }
  ```

- [ ] **Task 3.2: Task handler tests** (2h)
  ```rust
  // In backend/tests/task_tests.rs

  #[tokio::test]
  async fn test_create_task() { }

  #[tokio::test]
  async fn test_list_tasks() { }

  #[tokio::test]
  async fn test_update_task_status() { }

  #[tokio::test]
  async fn test_assign_task() { }

  #[tokio::test]
  async fn test_delete_task() { }
  ```

#### Afternoon (4h)
- [ ] **Task 3.3: File handler tests** (2h)
  ```rust
  // In backend/tests/file_tests.rs

  #[tokio::test]
  async fn test_upload_file() { }

  #[tokio::test]
  async fn test_list_files() { }

  #[tokio::test]
  async fn test_download_file() { }

  #[tokio::test]
  async fn test_delete_file() { }
  ```

- [ ] **Task 3.4: Notification handler tests** (1h)
  ```rust
  // In backend/tests/notification_tests.rs

  #[tokio::test]
  async fn test_list_notifications() { }

  #[tokio::test]
  async fn test_mark_as_read() { }

  #[tokio::test]
  async fn test_delete_notification() { }
  ```

- [ ] **Task 3.5: User handler tests** (1h)
  ```rust
  // In backend/tests/user_tests.rs

  #[tokio::test]
  async fn test_get_current_user() { }

  #[tokio::test]
  async fn test_get_user() { }

  #[tokio::test]
  async fn test_update_user() { }
  ```

**End of Day 3 Goal:**
```bash
cargo test
# Expected: 50+ tests passing
# All CRUD operations tested
```

---

### 🔵 DAY 4: API Documentation & Coverage (8h)

#### Morning (4h)
- [ ] **Task 4.1: Generate test coverage report** (1h)
  ```bash
  # Install tarpaulin
  cargo install cargo-tarpaulin

  # Generate coverage
  cargo tarpaulin --out Html --output-dir coverage

  # Open coverage/index.html
  # Target: 50%+ coverage
  ```

- [ ] **Task 4.2: Write missing tests** (3h)
  Based on coverage report, write tests for:
  - Uncovered edge cases
  - Error handling paths
  - Validation logic
  - Middleware functions

#### Afternoon (4h)
- [ ] **Task 4.3: Create API documentation** (3h)
  ```bash
  # Create: backend/API.md
  ```
  **Document for each endpoint:**
  - Method + Path
  - Description
  - Request body (JSON example)
  - Response body (JSON example)
  - Auth required (Yes/No)
  - Error codes
  - Example curl command

  **Example format:**
  ```markdown
  ### POST /api/auth/register

  **Description:** Register a new user

  **Request:**
  ```json
  {
    "email": "user@example.com",
    "password": "SecurePass123!",
    "full_name": "John Doe"
  }
  ```

  **Response:** 201 Created
  ```json
  {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "user": {
      "id": "uuid-here",
      "email": "user@example.com",
      "full_name": "John Doe"
    }
  }
  ```

  **Errors:**
  - 400: Email already exists
  - 422: Invalid email format

  **Example:**
  ```bash
  curl -X POST http://localhost:3000/api/auth/register \
    -H "Content-Type: application/json" \
    -d '{"email":"test@example.com","password":"pass123","full_name":"Test"}'
  ```
  ```

- [ ] **Task 4.4: Create Postman collection** (1h)
  **Steps:**
  1. Open Postman
  2. Create new collection "Neo CRM API"
  3. Add all 33 endpoints
  4. Setup environment variables:
     - `base_url`: http://localhost:3000
     - `token`: (auto-set from login)
  5. Export collection JSON
  6. Save to: `backend/postman/Neo_CRM.postman_collection.json`

**End of Day 4 Goal:**
```bash
# Test coverage
cargo tarpaulin
# Expected: 50%+ coverage

# Documentation complete
cat backend/API.md  # Should document all 33 endpoints

# Postman collection ready
ls backend/postman/  # Should have collection JSON
```

---

### 🟣 DAY 5: CI/CD & Code Quality (8h)

#### Morning (4h)
- [ ] **Task 5.1: Setup GitHub Actions** (2h)
  ```bash
  # Create: .github/workflows/backend-tests.yml
  ```
  ```yaml
  name: Backend Tests

  on: [push, pull_request]

  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v3
        - uses: actions-rs/toolchain@v1
          with:
            toolchain: stable
        - name: Run tests
          run: |
            cd backend
            cargo test --all
        - name: Run clippy
          run: cargo clippy -- -D warnings
        - name: Check formatting
          run: cargo fmt --check
  ```

- [ ] **Task 5.2: Fix clippy warnings** (2h)
  ```bash
  cargo clippy --fix
  cargo clippy -- -D warnings
  # Fix all remaining warnings manually
  ```

#### Afternoon (4h)
- [ ] **Task 5.3: Code cleanup** (3h)
  **Remove unused CQRS code:**
  ```bash
  # Backup first
  mkdir -p ../cqrs-backup
  cp -r src/core ../cqrs-backup/
  cp -r src/domains/file_system ../cqrs-backup/

  # Remove
  rm -rf src/core
  rm -rf src/domains/file_system
  rm -rf src/api

  # Update main.rs - remove imports
  # Update routes.rs - remove CQRS routes
  ```

- [ ] **Task 5.4: Format code** (30min)
  ```bash
  cargo fmt
  cargo fmt -- --check
  ```

- [ ] **Task 5.5: Final build check** (30min)
  ```bash
  cargo clean
  cargo build --release
  cargo test --all

  # Expected:
  # - Build: Success
  # - Tests: 60+ passing
  # - Warnings: < 10
  ```

**End of Day 5 Goal:**
```bash
# CI/CD working
git push
# GitHub Actions should pass all checks

# Code quality
cargo clippy -- -D warnings  # 0 errors
cargo fmt -- --check         # Already formatted
cargo test                   # 60+ tests passing
cargo build --warnings=0     # < 10 warnings

# Documentation complete
ls -la backend/
# Should have: API.md, postman/, coverage/
```

---

## ✅ WEEK 1 COMPLETION CHECKLIST

### Testing
- [ ] 60+ tests written and passing
- [ ] Test coverage > 50%
- [ ] All critical paths tested
- [ ] Edge cases covered

### Documentation
- [ ] API.md complete (33 endpoints)
- [ ] Postman collection exported
- [ ] Test utilities documented
- [ ] README updated

### Code Quality
- [ ] Clippy warnings < 10
- [ ] Code formatted (rustfmt)
- [ ] Unused code removed
- [ ] CQRS code cleaned up

### CI/CD
- [ ] GitHub Actions workflow created
- [ ] Tests run on push
- [ ] Clippy check enabled
- [ ] Format check enabled

---

## 📊 SUCCESS METRICS

**Week 1 Target:**
```
Tests Written:        60+ tests
Test Coverage:        > 50%
Build Warnings:       < 10
API Documentation:    100% (33/33 endpoints)
CI/CD Pipeline:       ✅ Working
Code Cleanup:         ✅ CQRS removed
```

**Quality Gates:**
```bash
# All of these must pass:
cargo test --all                 # 60+ tests pass
cargo clippy -- -D warnings      # 0 errors
cargo fmt -- --check             # Already formatted
cargo tarpaulin                  # > 50% coverage
cargo build --release            # Success
```

---

## 🚀 GETTING STARTED NOW

### Option 1: Start Immediately (Recommended)
```bash
# Step 1: Create feature branch
git checkout -b feature/testing-infrastructure

# Step 2: Add test dependencies
cd backend
# Edit Cargo.toml (add dev-dependencies from Day 1)

# Step 3: Build
cargo build

# Step 4: Create first test file
mkdir -p src/models/tests
touch src/models/tests/user_test.rs

# Step 5: Start writing tests!
```

### Option 2: Review & Plan First
```bash
# Read the roadmap
cat BACKEND_ROADMAP.md

# Review current code
cd backend/src
ls -R

# Check current test status
cargo test

# Plan your approach
# Then follow Option 1
```

---

## 🆘 TROUBLESHOOTING

### Issue: Tests won't compile
```bash
# Solution: Check imports
cargo check --tests

# Fix import errors
# Run again
```

### Issue: Test database conflicts
```bash
# Solution: Use in-memory database for tests
// In test_utils.rs
let pool = SqlitePool::connect(":memory:").await?;
```

### Issue: Slow tests
```bash
# Solution: Run in parallel
cargo test -- --test-threads=4

# Or run specific test
cargo test test_name
```

### Issue: Coverage tool fails
```bash
# Solution: Install dependencies
sudo apt-get install pkg-config libssl-dev

# Then retry
cargo install cargo-tarpaulin
```

---

## 📚 RESOURCES

### Testing Guides
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Axum Testing](https://docs.rs/axum/latest/axum/testing/)
- [SQLx Testing](https://github.com/launchbadge/sqlx#testing)

### Example Tests
```bash
# Check our existing tests
cat backend/tests/auth_tests.rs

# Learn from examples
https://github.com/tokio-rs/axum/tree/main/examples
```

### Tools
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Coverage
- [cargo-watch](https://github.com/watchexec/cargo-watch) - Auto-run tests
- [nextest](https://nexte.st/) - Faster test runner

---

## 🎯 NEXT STEPS AFTER WEEK 1

Once Week 1 is complete, proceed to:

**Week 2: Security & Performance**
- Day 6-7: Code cleanup (DONE in Week 1)
- Day 8-9: Security hardening
- Day 10: Performance optimization

See [BACKEND_ROADMAP.md](./BACKEND_ROADMAP.md) for full details.

---

## 💪 MOTIVATION

**Why Testing Matters:**
- 🐛 Catch bugs before production
- 🔒 Prevent regressions
- 📚 Document behavior
- 🚀 Refactor with confidence
- ⚡ Ship faster (really!)

**Quote:**
> "Code without tests is broken by design." - Jacob Kaplan-Moss

---

**Ready to start? Pick a task from Day 1 and go! 🚀**

**Questions?** Check [BACKEND_ROADMAP.md](./BACKEND_ROADMAP.md) for strategy.
