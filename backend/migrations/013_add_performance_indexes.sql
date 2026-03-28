-- Extra indexes (PostgreSQL: case-insensitive via lower(), not COLLATE NOCASE)
CREATE INDEX IF NOT EXISTS idx_tasks_created_at_desc ON tasks(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date_status ON tasks(due_date, status);
CREATE INDEX IF NOT EXISTS idx_tasks_status_priority_assigned ON tasks(status, priority, assigned_to);

CREATE INDEX IF NOT EXISTS idx_clients_created_at_desc ON clients(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_clients_name_lower ON clients (lower(name));
CREATE INDEX IF NOT EXISTS idx_clients_status_assigned_to ON clients(status, assigned_to);
CREATE INDEX IF NOT EXISTS idx_clients_email_lower ON clients (lower(email));

CREATE INDEX IF NOT EXISTS idx_files_uploaded_by_created_at ON files(uploaded_by, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_files_file_type_perf ON files(file_type);
CREATE INDEX IF NOT EXISTS idx_files_created_at_desc ON files(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_notifications_user_created_desc ON notifications(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_user_is_read_created ON notifications(user_id, is_read, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_activities_entity_type_id ON activities(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_activities_user_created_desc ON activities(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_activities_created_at_desc ON activities(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_users_role_active ON users(role, is_active);
CREATE INDEX IF NOT EXISTS idx_users_created_at_desc ON users(created_at DESC);

ANALYZE users;
ANALYZE clients;
ANALYZE tasks;
ANALYZE files;
ANALYZE notifications;
ANALYZE activities;
