# 🔌 API Documentation

Base URL: `http://localhost:3000`

## Authentication

Tất cả các protected endpoints yêu cầu JWT token trong header:

```
Authorization: Bearer <your-jwt-token>
```

## Endpoints

### 🔐 Authentication

#### Register
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "full_name": "John Doe",
  "role": "user" // optional: "admin", "manager", "user"
}

Response: 201
{
  "token": "eyJhbGc...",
  "user": {
    "id": "uuid",
    "email": "user@example.com",
    "full_name": "John Doe",
    "role": "user",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123"
}

Response: 200
{
  "token": "eyJhbGc...",
  "user": { ... }
}
```

### 👤 Users

#### Get Current User
```http
GET /api/users/me
Authorization: Bearer <token>

Response: 200
{
  "id": "uuid",
  "email": "user@example.com",
  "full_name": "John Doe",
  "role": "user",
  "avatar_url": null,
  "is_active": true,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### Update User
```http
PATCH /api/users/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "full_name": "Jane Doe",
  "avatar_url": "https://example.com/avatar.jpg"
}

Response: 200
{ ... }
```

### 👥 Clients

#### List Clients
```http
GET /api/clients?status=active&page=1&limit=20&search=john
Authorization: Bearer <token>

Query Parameters:
- status: active | inactive | prospect | customer
- assigned_to: uuid
- search: string
- page: number (default: 1)
- limit: number (default: 20)

Response: 200
[
  {
    "id": "uuid",
    "name": "John Doe",
    "email": "john@example.com",
    "phone": "+1-555-0101",
    "company": "Tech Corp",
    "position": "CEO",
    "address": "123 Main St",
    "status": "active",
    "assigned_to": "uuid",
    "notes": "Important client",
    "avatar_url": null,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Create Client
```http
POST /api/clients
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "John Doe",
  "email": "john@example.com",
  "phone": "+1-555-0101",
  "company": "Tech Corp",
  "position": "CEO",
  "address": "123 Main St",
  "status": "active",
  "assigned_to": "uuid",
  "notes": "VIP client"
}

Response: 200
{ ... }
```

#### Get Client
```http
GET /api/clients/:id
Authorization: Bearer <token>

Response: 200
{ ... }
```

#### Update Client
```http
PATCH /api/clients/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "customer",
  "notes": "Updated notes"
}

Response: 200
{ ... }
```

#### Delete Client
```http
DELETE /api/clients/:id
Authorization: Bearer <token>

Response: 200
{
  "message": "Client deleted successfully"
}
```

### ✅ Tasks

#### List Tasks
```http
GET /api/tasks?status=todo&priority=high&assigned_to=uuid
Authorization: Bearer <token>

Query Parameters:
- status: todo | in_progress | done | cancelled
- priority: low | medium | high | urgent
- assigned_to: uuid
- client_id: uuid
- page: number
- limit: number

Response: 200
[
  {
    "id": "uuid",
    "title": "Follow up with client",
    "description": "Schedule meeting",
    "status": "todo",
    "priority": "high",
    "assigned_to": "uuid",
    "client_id": "uuid",
    "due_date": "2024-01-15T00:00:00Z",
    "completed_at": null,
    "created_by": "uuid",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Create Task
```http
POST /api/tasks
Authorization: Bearer <token>
Content-Type: application/json

{
  "title": "Follow up with client",
  "description": "Schedule meeting",
  "status": "todo",
  "priority": "high",
  "assigned_to": "uuid",
  "client_id": "uuid",
  "due_date": "2024-01-15T00:00:00Z"
}

Response: 200
{ ... }
```

#### Update Task
```http
PATCH /api/tasks/:id
Authorization: Bearer <token>
Content-Type: application/json

{
  "status": "done"
}

Response: 200
{ ... }
```

#### Delete Task
```http
DELETE /api/tasks/:id
Authorization: Bearer <token>

Response: 200
{
  "message": "Task deleted successfully"
}
```

### 🔔 Notifications

#### List Notifications
```http
GET /api/notifications
Authorization: Bearer <token>

Response: 200
[
  {
    "id": "uuid",
    "user_id": "uuid",
    "type": "task_assigned",
    "title": "New Task Assigned",
    "message": "You have been assigned a new task",
    "link": "/tasks/uuid",
    "is_read": false,
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Mark Notifications as Read
```http
POST /api/notifications/mark-read
Authorization: Bearer <token>
Content-Type: application/json

{
  "notification_ids": ["uuid1", "uuid2"]
}

Response: 200
{
  "message": "Notifications marked as read"
}
```

#### Delete Notification
```http
DELETE /api/notifications/:id
Authorization: Bearer <token>

Response: 200
{
  "message": "Notification deleted"
}
```

### 📁 Files

#### List Files
```http
GET /api/files?client_id=uuid&task_id=uuid
Authorization: Bearer <token>

Response: 200
[
  {
    "id": "uuid",
    "name": "document.pdf",
    "original_name": "document.pdf",
    "file_path": "/uploads/uuid-document.pdf",
    "file_type": "application/pdf",
    "file_size": 1024000,
    "uploaded_by": "uuid",
    "client_id": "uuid",
    "task_id": null,
    "description": "Contract",
    "thumbnail_path": null,
    "created_at": "2024-01-01T00:00:00Z"
  }
]
```

#### Upload File
```http
POST /api/files/upload
Authorization: Bearer <token>
Content-Type: multipart/form-data

(Multipart form data with file)

Response: 200
{ ... }
```

#### Get File
```http
GET /api/files/:id
Authorization: Bearer <token>

Response: 200
{ ... }
```

#### Download File
```http
GET /api/files/:id/download
Authorization: Bearer <token>

Response: 200
(File binary data)
```

#### Delete File
```http
DELETE /api/files/:id
Authorization: Bearer <token>

Response: 200
{
  "message": "File deleted successfully"
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "Invalid input data"
}
```

### 401 Unauthorized
```json
{
  "error": "Invalid credentials"
}
```

### 404 Not Found
```json
{
  "error": "Resource not found"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```

## Rate Limiting

Currently no rate limiting implemented. Will be added in future versions.

## WebSocket (Coming Soon)

Real-time notifications via WebSocket:

```javascript
const ws = new WebSocket('ws://localhost:3000/ws/notifications');
ws.onmessage = (event) => {
  const notification = JSON.parse(event.data);
  console.log('New notification:', notification);
};
```
