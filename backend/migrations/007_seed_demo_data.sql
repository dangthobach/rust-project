-- Demo data (PostgreSQL). Password hashes: same bcrypt as legacy SQLite seeds.
INSERT INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    'admin@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe',
    'Admin User',
    'admin',
    TRUE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000002'::uuid,
    'manager@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe',
    'Manager User',
    'manager',
    TRUE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-000000000003'::uuid,
    'user@crm.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyB.KpRD8sPe',
    'Regular User',
    'user',
    TRUE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO clients (id, name, email, phone, company, status, assigned_to)
VALUES
    ('00000000-0000-0000-0000-0000000000d1'::uuid, 'John Doe', 'john.doe@example.com', '+1-555-0101', 'Tech Corp', 'customer', '00000000-0000-0000-0000-000000000002'::uuid),
    ('00000000-0000-0000-0000-0000000000d2'::uuid, 'Jane Smith', 'jane.smith@example.com', '+1-555-0102', 'Design Studio', 'active', '00000000-0000-0000-0000-000000000002'::uuid),
    ('00000000-0000-0000-0000-0000000000d3'::uuid, 'Bob Johnson', 'bob.johnson@example.com', '+1-555-0103', 'Marketing Inc', 'prospect', '00000000-0000-0000-0000-000000000003'::uuid)
ON CONFLICT (id) DO NOTHING;

INSERT INTO tasks (id, title, description, status, priority, assigned_to, client_id, due_date, created_by)
VALUES
    (
        '00000000-0000-0000-0000-0000000000e1'::uuid,
        'Follow up with John Doe',
        'Schedule a meeting to discuss new project',
        'todo',
        'high',
        '00000000-0000-0000-0000-000000000002'::uuid,
        '00000000-0000-0000-0000-0000000000d1'::uuid,
        (NOW() + INTERVAL '3 days'),
        '00000000-0000-0000-0000-000000000001'::uuid
    ),
    (
        '00000000-0000-0000-0000-0000000000e2'::uuid,
        'Prepare proposal for Jane',
        'Create design proposal for website redesign',
        'in_progress',
        'medium',
        '00000000-0000-0000-0000-000000000003'::uuid,
        '00000000-0000-0000-0000-0000000000d2'::uuid,
        (NOW() + INTERVAL '7 days'),
        '00000000-0000-0000-0000-000000000001'::uuid
    ),
    (
        '00000000-0000-0000-0000-0000000000e3'::uuid,
        'Send contract to Bob',
        'Finalize and send marketing contract',
        'todo',
        'urgent',
        '00000000-0000-0000-0000-000000000002'::uuid,
        '00000000-0000-0000-0000-0000000000d3'::uuid,
        (NOW() + INTERVAL '1 day'),
        '00000000-0000-0000-0000-000000000001'::uuid
    )
ON CONFLICT (id) DO NOTHING;
