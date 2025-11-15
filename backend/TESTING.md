# Backend Testing Documentation

## Overview

This document describes the testing strategy for the Neo CRM backend.

## Testing Approaches

### 1. Integration Testing (Recommended)

Since the backend is a binary crate, integration testing via API calls is the most practical approach.

#### Running Integration Tests

```bash
# Start the backend server
cargo run --release

# In another terminal, run the integration test script
pwsh ./test-api.ps1
```

The test script (`test-api.ps1`) covers:
- ✅ Health check endpoint
- ✅ User registration
- ✅ Login authentication
- ✅ Client CRUD operations
- ✅ Task CRUD operations

### 2. Manual Testing

Follow the [TESTING_GUIDE.md](../TESTING_GUIDE.md) for comprehensive manual testing instructions.

### 3. Unit Testing (Future)

To enable proper unit tests, the backend would need to be restructured as a library crate with a thin binary wrapper:

```
backend/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── main.rs          # Binary entry point (thin wrapper)
│   └── ... (existing modules)
├── tests/
│   ├── integration_tests.rs
│   └── ...
```

This would allow:
- `cargo test` for unit and integration tests
- `use crm_backend::*` in test modules
- Proper test isolation with in-memory databases

## Current Test Coverage

### API Endpoints Tested

**Authentication:**
- [x] POST /api/auth/register
- [x] POST /api/auth/login

**Clients:**
- [x] GET /api/clients
- [x] POST /api/clients
- [ ] GET /api/clients/:id
- [ ] PATCH /api/clients/:id
- [ ] DELETE /api/clients/:id

**Tasks:**
- [x] GET /api/tasks
- [x] POST /api/tasks
- [x] PATCH /api/tasks/:id
- [ ] GET /api/tasks/:id
- [ ] DELETE /api/tasks/:id

**Files:**
- [ ] GET /api/files
- [ ] POST /api/files/upload
- [ ] GET /api/files/:id/download
- [ ] DELETE /api/files/:id

**Notifications:**
- [ ] GET /api/notifications
- [ ] POST /api/notifications/mark-read

## Running Tests

### Prerequisites

1. Backend must be running:
   ```bash
   cd backend
   cargo run --release
   ```

2. Database must be initialized:
   ```bash
   sqlx migrate run
   ```

### Execute Integration Tests

**Windows (PowerShell):**
```pwsh
cd backend
.\test-api.ps1
```

**Expected Output:**
```
=== Neo CRM Backend Integration Tests ===

[ 1/8 ] Checking if backend is running...
✓ Backend is running

[ 2/8 ] Testing user registration...
✓ User registration successful

[ 3/8 ] Testing login...
✓ Login successful. Token received.

[ 4/8 ] Testing client creation...
✓ Client created successfully (ID: xxx-xxx)

[ 5/8 ] Testing list clients...
✓ Retrieved 1 client(s)

[ 6/8 ] Testing task creation...
✓ Task created successfully (ID: yyy-yyy)

[ 7/8 ] Testing task update...
✓ Task updated successfully

[ 8/8 ] Testing list tasks...
✓ Retrieved 1 task(s)

=====================================
   All Integration Tests Complete
=====================================
```

## Test Data

Integration tests create temporary test data:
- Test user with timestamp: `test_YYYYMMDDHHMMSS@example.com`
- Test client: "Test Client Company"
- Test task: "Test Task" with high priority

**Note:** Test data is not automatically cleaned up. You can reset the database:
```bash
rm data/crm.db
sqlx migrate run
```

## Continuous Integration (CI)

For CI/CD pipelines, use:

```yaml
# Example GitHub Actions workflow
- name: Run Backend Tests
  run: |
    cd backend
    cargo build --release
    cargo run --release &
    sleep 5  # Wait for server startup
    pwsh ./test-api.ps1
```

## Future Improvements

1. **Convert to Library Crate:**
   - Restructure as `lib.rs` + `main.rs`
   - Enable proper unit tests
   - Add `cargo test` to CI pipeline

2. **Add Test Coverage:**
   - File upload/download tests
   - Notification tests
   - Error handling tests
   - Authentication edge cases

3. **Mock Database:**
   - Use in-memory SQLite for tests
   - Reset database between tests
   - Faster test execution

4. **Performance Tests:**
   - Load testing with multiple concurrent users
   - API response time benchmarks
   - Database query optimization

5. **Frontend E2E Tests:**
   - Playwright or Cypress tests
   - Full user workflow testing
   - Screenshot comparison

## Resources

- [TESTING_GUIDE.md](../TESTING_GUIDE.md) - Comprehensive testing guide
- [CRITICAL_ISSUES_FIXED.md](../CRITICAL_ISSUES_FIXED.md) - Known issues and fixes
- [SQLx Documentation](https://github.com/launchbadge/sqlx) - Database testing

## Troubleshooting

### Test Script Fails with "Backend not running"

**Solution:** Start the backend:
```bash
cd backend
cargo run --release
```

### Authentication Tests Fail

**Possible causes:**
1. JWT secret mismatch - check `.env` file
2. Password hashing issues - verify bcrypt is working
3. Database not migrated - run `sqlx migrate run`

### Database Locked Error

**Solution:**
1. Stop all backend instances
2. Delete `.db-shm` and `.db-wal` files
3. Restart backend

## Summary

- ✅ Integration tests via API calls (preferred)
- ✅ PowerShell test script provided
- ✅ Manual testing guide available
- ⏳ Unit tests pending (requires restructuring)
- ⏳ E2E tests pending (frontend + backend)

**Current Status:** Integration testing functional and recommended for now.
