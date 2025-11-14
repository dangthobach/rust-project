-- Insert demo admin user (SQLite compatible)
-- Password: admin123 (hashed with bcrypt)
INSERT OR IGNORE INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe', -- admin123
    'Admin User',
    'admin',
    1
);

-- Insert demo manager user
-- Password: manager123
INSERT OR IGNORE INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    'manager@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe',
    'Manager User',
    'manager',
    1
);

-- Insert demo regular user
-- Password: user123
INSERT OR IGNORE INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000003',
    'user@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe',
    'Regular User',
    'user',
    1
);

-- Insert demo clients
INSERT OR IGNORE INTO clients (name, email, phone, company, status, assigned_to)
VALUES
    ('John Doe', 'john.doe@example.com', '+1-555-0101', 'Tech Corp', 'customer', '00000000-0000-0000-0000-000000000002'),
    ('Jane Smith', 'jane.smith@example.com', '+1-555-0102', 'Design Studio', 'active', '00000000-0000-0000-0000-000000000002'),
    ('Bob Johnson', 'bob.johnson@example.com', '+1-555-0103', 'Marketing Inc', 'prospect', '00000000-0000-0000-0000-000000000003');

-- Insert demo tasks (SQLite doesn't support INTERVAL, use datetime functions)
INSERT OR IGNORE INTO tasks (title, description, status, priority, assigned_to, due_date, created_by)
VALUES
    ('Follow up with John Doe', 'Schedule a meeting to discuss new project', 'todo', 'high', '00000000-0000-0000-0000-000000000002', datetime('now', '+3 days'), '00000000-0000-0000-0000-000000000001'),
    ('Prepare proposal for Jane', 'Create design proposal for website redesign', 'in_progress', 'medium', '00000000-0000-0000-0000-000000000003', datetime('now', '+7 days'), '00000000-0000-0000-0000-000000000001'),
    ('Send contract to Bob', 'Finalize and send marketing contract', 'todo', 'urgent', '00000000-0000-0000-0000-000000000002', datetime('now', '+1 day'), '00000000-0000-0000-0000-000000000001');
