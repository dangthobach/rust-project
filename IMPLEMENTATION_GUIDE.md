# 📘 Implementation Guide - Neo-Brutalist CRM

## 🎯 Tổng quan

Dự án này đã được setup với structure hoàn chỉnh. Đây là hướng dẫn chi tiết về cách triển khai từng phần.

## 📂 Project Structure

```
rust-system/
├── backend/              # Rust Axum API
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── config.rs    # Configuration
│   │   ├── error.rs     # Error handling
│   │   ├── routes.rs    # Route definitions
│   │   ├── models/      # Data models
│   │   ├── handlers/    # Request handlers
│   │   ├── middleware/  # Auth middleware
│   │   ├── services/    # Business logic
│   │   └── utils/       # Utilities (JWT, password)
│   ├── migrations/      # Database migrations
│   └── Cargo.toml
│
├── frontend/            # Qwik application
│   ├── src/
│   │   ├── root.tsx     # App root
│   │   ├── global.css   # Global styles
│   │   ├── routes/      # Pages
│   │   ├── components/
│   │   │   ├── ui/      # Base UI components
│   │   │   └── crm/     # CRM-specific components
│   │   └── theme/       # Design tokens & utils
│   ├── package.json
│   └── tailwind.config.ts
│
├── wasm-viewer/         # Rust/WASM file viewer
│   ├── src/lib.rs
│   └── Cargo.toml
│
├── README.md            # Project overview
├── SETUP.md            # Setup instructions
├── API.md              # API documentation
└── docker-compose.yml   # Docker config
```

## 🚀 Deployment Steps

### Step 1: Setup Development Environment

```bash
# 1. Cài đặt tools
rustup update
cargo install sqlx-cli wasm-pack

# 2. Setup database
cd backend
sqlx database create
sqlx migrate run

# 3. Start backend
cargo run

# 4. Mở terminal mới - Build WASM
cd ../wasm-viewer
wasm-pack build --target web --release

# 5. Mở terminal mới - Start frontend
cd ../frontend
npm install
npm run dev
```

### Step 2: Build cho Production

```bash
# Backend
cd backend
cargo build --release

# WASM
cd ../wasm-viewer
wasm-pack build --target web --release

# Frontend
cd ../frontend
npm run build
```

### Step 3: Docker Deployment

```bash
# Build và run tất cả services
docker-compose up -d

# Check logs
docker-compose logs -f backend
docker-compose logs -f frontend
```

## 🎨 Design System Usage

### Colors

```tsx
import { colors } from '~/theme/tokens';

// Use in components
<div style={{ backgroundColor: colors.primary.DEFAULT }}>
  Neon Green Background
</div>
```

### Components

```tsx
import { Button, Card, Input } from '~/components/ui';

// Basic button
<Button variant="primary">Click Me</Button>

// Card với content
<Card hoverable>
  <CardHeader>
    <CardTitle>Title</CardTitle>
  </CardHeader>
  <CardContent>Content here</CardContent>
</Card>

// Form input
<Input
  label="Email"
  type="email"
  placeholder="your@email.com"
  error={errors.email}
/>
```

### Utility Classes

```tsx
// Brutal shadows
<div class="card shadow-brutal hover:shadow-brutal-lg">

// Transform on hover
<div class="transform-brutal">

// Asymmetric positioning
<div class="asymmetric-1">
```

## 📝 Common Tasks

### 1. Add New Database Table

```bash
cd backend

# Tạo migration
sqlx migrate add create_new_table

# Edit file trong migrations/
# Viết SQL để tạo table

# Run migration
sqlx migrate run
```

### 2. Add New API Endpoint

```rust
// 1. Tạo model trong src/models/your_model.rs
#[derive(Serialize, Deserialize, FromRow)]
pub struct YourModel {
    pub id: Uuid,
    pub name: String,
}

// 2. Tạo handler trong src/handlers/your_handler.rs
pub async fn list_items(
    Extension(user_id): Extension<Uuid>,
    State((pool, _)): State<(PgPool, Config)>,
) -> AppResult<Json<Vec<YourModel>>> {
    // Implementation
}

// 3. Add route trong src/routes.rs
.route("/api/items", get(your_handler::list_items))
```

### 3. Add New Frontend Page

```bash
cd frontend/src/routes

# Tạo folder và file
mkdir new-page
touch new-page/index.tsx
```

```tsx
// frontend/src/routes/new-page/index.tsx
import { component$ } from '@builder.io/qwik';
import { type DocumentHead } from '@builder.io/qwik-city';

export default component$(() => {
  return (
    <div class="container-brutal py-8">
      <h1>New Page</h1>
    </div>
  );
});

export const head: DocumentHead = {
  title: 'New Page',
};
```

### 4. Add New UI Component

```tsx
// frontend/src/components/ui/YourComponent.tsx
import { component$, Slot } from '@builder.io/qwik';
import { cn } from '~/theme/utils';

export const YourComponent = component$(({ class: className }) => {
  return (
    <div class={cn('card', className)}>
      <Slot />
    </div>
  );
});
```

### 5. Integrate WASM File Viewer

```tsx
import { useVisibleTask$, useSignal } from '@builder.io/qwik';

export default component$(() => {
  const viewerHtml = useSignal('');

  useVisibleTask$(async () => {
    // Dynamic import WASM
    const { default: init, FileViewer, detect_file_type } =
      await import('~/wasm/pkg');

    await init();

    // Detect file type
    const fileType = detect_file_type('document.pdf');

    // Create viewer
    const viewer = new FileViewer(fileType);

    // Load file (fetch from API)
    const response = await fetch('/api/files/123');
    const buffer = await response.arrayBuffer();
    viewer.load_content(new Uint8Array(buffer));

    // Render
    viewerHtml.value = viewer.render();
  });

  return <div dangerouslySetInnerHTML={viewerHtml.value} />;
});
```

## 🔐 Authentication Flow

### 1. Login

```tsx
// Frontend
const handleLogin = async (email: string, password: string) => {
  const response = await fetch('http://localhost:3000/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });

  const data = await response.json();

  // Store token
  localStorage.setItem('token', data.token);

  // Store user
  localStorage.setItem('user', JSON.stringify(data.user));
};
```

### 2. Make Authenticated Request

```tsx
const token = localStorage.getItem('token');

const response = await fetch('http://localhost:3000/api/clients', {
  headers: {
    'Authorization': `Bearer ${token}`,
  },
});

const clients = await response.json();
```

## 📊 Database Queries

### Using SQLx

```rust
// Select all
let clients = sqlx::query_as::<_, Client>(
    "SELECT * FROM clients WHERE status = $1"
)
.bind("active")
.fetch_all(&pool)
.await?;

// Insert
let client = sqlx::query_as::<_, Client>(
    "INSERT INTO clients (name, email) VALUES ($1, $2) RETURNING *"
)
.bind(&name)
.bind(&email)
.fetch_one(&pool)
.await?;

// Update
let updated = sqlx::query_as::<_, Client>(
    "UPDATE clients SET name = $1 WHERE id = $2 RETURNING *"
)
.bind(&new_name)
.bind(id)
.fetch_one(&pool)
.await?;

// Delete
sqlx::query("DELETE FROM clients WHERE id = $1")
    .bind(id)
    .execute(&pool)
    .await?;
```

## 🧪 Testing

### Backend Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        // Test implementation
    }
}
```

### Frontend Tests

```tsx
import { createDOM } from '@builder.io/qwik/testing';
import { test, expect } from 'vitest';
import { Button } from './Button';

test('Button renders correctly', async () => {
  const { screen, render } = await createDOM();
  await render(<Button>Click Me</Button>);

  expect(screen.innerHTML).toContain('Click Me');
});
```

## 🎯 Next Features to Implement

### High Priority
1. ✅ WebSocket for real-time notifications
2. ✅ File upload with multipart
3. ✅ Advanced search & filtering
4. ✅ Export data (CSV, PDF)
5. ✅ Email notifications

### Medium Priority
6. ✅ Dashboard with analytics
7. ✅ Calendar view for tasks
8. ✅ Activity timeline
9. ✅ User roles & permissions
10. ✅ Dark mode toggle

### Low Priority
11. ✅ Mobile app (PWA)
12. ✅ Integrations (Gmail, Slack)
13. ✅ AI-powered insights
14. ✅ Custom fields
15. ✅ API rate limiting

## 📚 Resources

### Documentation
- [Qwik Docs](https://qwik.builder.io/docs/)
- [Axum Docs](https://docs.rs/axum/)
- [SQLx Docs](https://docs.rs/sqlx/)
- [Tailwind CSS](https://tailwindcss.com/)

### Design Inspiration
- [Brutalist Websites](https://brutalistwebsites.com/)
- [Neo-Brutalism UI](https://www.uxdesigninstitute.com/blog/neo-brutalism/)

### Tools
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [Docker](https://docs.docker.com/)
- [PostgreSQL](https://www.postgresql.org/docs/)

## 🆘 Troubleshooting

### Common Issues

**Issue**: SQLx compile error
```bash
# Solution: Regenerate SQLx metadata
cd backend
cargo sqlx prepare
```

**Issue**: WASM không load
```bash
# Solution: Rebuild WASM
cd wasm-viewer
wasm-pack build --target web --release
```

**Issue**: Frontend build error
```bash
# Solution: Clear cache
rm -rf node_modules .turbo dist
npm install
npm run build
```

## 💡 Tips & Best Practices

1. **Code Organization**: Giữ components nhỏ và tái sử dụng
2. **Error Handling**: Luôn handle errors gracefully
3. **Type Safety**: Sử dụng TypeScript/Rust types đầy đủ
4. **Performance**: Lazy load components khi cần
5. **Testing**: Viết tests cho critical features
6. **Documentation**: Comment code phức tạp
7. **Git**: Commit thường xuyên với messages rõ ràng
8. **Security**: Validate input, sanitize output

Happy Building! 🚀✨
