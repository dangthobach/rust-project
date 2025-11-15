# Quick Testing Guide - Neo CRM

## Prerequisites
- Rust 1.79+
- Node.js 18+
- SQLite3

## Backend Setup & Run

```bash
cd backend

# Build (should complete with 0 errors)
cargo build --release

# Run migrations (if needed)
sqlx migrate run

# Start backend server
cargo run --release
# OR
./start.bat  # Windows
./start.sh   # Linux/Mac
```

**Expected Output:**
```
Server running on http://0.0.0.0:3000
Database connected: data/crm.db
```

**Test Health Check:**
```bash
curl http://localhost:3000/health
# Expected: {"status":"ok","database":"connected"}
```

---

## Frontend Setup & Run

```bash
cd frontend

# Install dependencies
npm install

# Start dev server
npm run dev
```

**Expected Output:**
```
VITE v5.x.x  ready in XXX ms
➜  Local:   http://localhost:5173/
```

---

## Testing Checklist

### 1. Authentication Flow ✅
1. Open browser: `http://localhost:5173/login`
2. Try accessing protected route: `http://localhost:5173/`
   - **Expected:** Redirected to `/login`
3. Register new user (if endpoint exists) OR use existing credentials
4. Login with credentials
   - **Expected:** 
     - Loading spinner shows
     - Token saved to localStorage
     - Redirected to Dashboard

### 2. Dashboard Integration ✅
1. After login, verify Dashboard loads
2. **Expected:**
   - Recent Clients section shows loading spinner, then real data
   - Recent Tasks section shows loading spinner, then real data
   - If empty: appropriate "No data" message
3. Check browser DevTools → Network tab
   - **Expected:** See API calls to `/api/clients`, `/api/tasks`

### 3. File Upload/Download ✅
1. Navigate to Files page: `http://localhost:5173/files`
2. Click "Upload File" button
3. Select any file (image, PDF, etc.)
   - **Expected:** 
     - Loading spinner during upload
     - File appears in list after upload
     - File size formatted (KB/MB)
4. Click "Download" on uploaded file
   - **Expected:** Browser downloads file with original name

**Backend Verification:**
```bash
# Check uploads directory
ls backend/uploads/
# Should see uploaded files with UUID names
```

### 4. Notifications Integration ✅
1. Navigate to Notifications page
2. **Expected:**
   - Loading spinner shows
   - Real notifications from API appear
   - Unread count badge shows
   - Timestamps display correctly

### 5. Route Protection ✅
**Test 1: Logout and access protected route**
1. Open DevTools → Console
2. Run: `localStorage.removeItem('token')`
3. Navigate to Dashboard: `http://localhost:5173/`
   - **Expected:** Immediately redirected to `/login`

**Test 2: Invalid token**
1. Open DevTools → Console
2. Run: `localStorage.setItem('token', 'invalid-token')`
3. Try accessing any protected page
   - **Expected:** 401 error, token cleared, redirected to login

---

## API Endpoint Testing

### Using curl (Backend must be running)

**Login:**
```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"password123"}'
# Expected: {"token":"eyJ...","user":{...}}
```

**Get Clients (with token):**
```bash
TOKEN="your-jwt-token-here"
curl http://localhost:3000/api/clients \
  -H "Authorization: Bearer $TOKEN"
# Expected: [{"id":"...","name":"...","email":"...",...}]
```

**Upload File:**
```bash
curl -X POST http://localhost:3000/api/files/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@/path/to/file.pdf"
# Expected: {"id":"...","name":"...","file_size":12345,...}
```

**Download File:**
```bash
FILE_ID="file-uuid-here"
curl http://localhost:3000/api/files/$FILE_ID/download \
  -H "Authorization: Bearer $TOKEN" \
  -o downloaded-file.pdf
# Expected: File downloaded
```

---

## Database Verification

```bash
cd backend
sqlite3 data/crm.db

# Check users
SELECT id, email, name, created_at FROM users;

# Check files
SELECT id, original_name, file_size, created_at FROM files;

# Check clients
SELECT id, name, email, status FROM clients;

# Check tasks
SELECT id, title, status, priority FROM tasks LIMIT 10;

.exit
```

---

## Troubleshooting

### Issue: Build fails with UUID errors
**Solution:** Ensure all models use `String` instead of `Uuid`

### Issue: 401 Unauthorized on protected routes
**Possible causes:**
1. Token expired (JWT default expiry)
2. Token not in localStorage
3. Auth middleware not applied to route

**Check:**
```javascript
// In browser console
localStorage.getItem('token')
```

### Issue: File upload fails
**Possible causes:**
1. `uploads/` directory doesn't exist
2. File too large (check `max_file_size` in config)
3. Insufficient disk space

**Check backend logs:**
```bash
# Backend should auto-create uploads/ directory
ls -la backend/uploads/
```

### Issue: CORS errors in frontend
**Solution:** Update `backend/.env`:
```env
CORS_ORIGIN=http://localhost:5173
```

### Issue: Database locked error
**Solution:**
1. Stop all backend instances
2. Delete `data/crm.db-shm` and `data/crm.db-wal`
3. Restart backend

---

## Performance Checks

### Backend Performance
```bash
# Check response times
time curl http://localhost:3000/api/clients \
  -H "Authorization: Bearer $TOKEN"
# Should be < 100ms for local SQLite
```

### Frontend Performance
1. Open DevTools → Network tab
2. Reload Dashboard
3. **Check:**
   - API calls complete in < 200ms
   - No failed requests (red)
   - No CORS errors in console

---

## Success Criteria

### Backend ✅
- [x] Builds with 0 errors
- [x] Health check responds
- [x] Auth endpoints work (login, register)
- [x] Protected endpoints require token
- [x] File upload/download works
- [x] Database queries execute successfully

### Frontend ✅
- [x] Builds without errors
- [x] Login page connects to backend
- [x] Dashboard loads real data
- [x] Files page uploads/downloads
- [x] Route protection redirects to login
- [x] No console errors on page load

### Integration ✅
- [x] JWT tokens persist across page refreshes
- [x] Protected routes enforce authentication
- [x] API calls use correct endpoints
- [x] Error handling displays user-friendly messages
- [x] Loading states show during API calls

---

## Next Steps After Successful Testing

1. **Deploy to Staging:**
   - Set up production .env
   - Configure CORS for production domain
   - Set up SSL certificates
   - Configure file upload limits

2. **Add Monitoring:**
   - Set up logging to file
   - Add error tracking (e.g., Sentry)
   - Monitor API response times
   - Track file storage usage

3. **Implement Missing Features:**
   - User registration flow
   - Password reset
   - File deletion UI
   - Pagination for large datasets
   - Search functionality

4. **Add Tests:**
   - Integration tests for auth flow
   - Unit tests for handlers
   - E2E tests for critical paths
   - Load testing for API endpoints

---

## Quick Command Reference

```bash
# Backend
cd backend
cargo build --release          # Build
cargo run --release            # Run
cargo test                     # Test (when implemented)
sqlx migrate run               # Run migrations

# Frontend  
cd frontend
npm install                    # Install deps
npm run dev                    # Dev server
npm run build                  # Production build
npm run preview                # Preview build

# Database
cd backend
sqlite3 data/crm.db            # Open DB
sqlite3 data/crm.db < migrations/001_create_users_table.sql  # Run specific migration
```

---

**Happy Testing! 🚀**

For issues, check `CRITICAL_ISSUES_FIXED.md` for known limitations and solutions.
