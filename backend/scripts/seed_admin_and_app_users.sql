-- Same as migrations/018_seed_system_users.sql — apply manually if needed:
--   sqlite3 your.db < scripts/seed_admin_and_app_users.sql
-- Prefer: run full migrations (includes 018) via sqlx or backend startup.

PRAGMA foreign_keys = ON;

INSERT OR IGNORE INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-0000000000c1',
    'administrator@system.local',
    '$2b$12$qa6wQ4XVr5t1auhcUaUw.enX5V.h6FStGY24mvcRBL5pbsz/G0/dK',
    'System Administrator',
    'admin',
    1
);

INSERT OR IGNORE INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-0000000000c2',
    'application@system.local',
    '$2b$12$QuBn4SXpok6aLn5J.yCZ8u9ngElROtDMkI7iZdp6Epb5j/M3V2M5e',
    'Application Integration',
    'user',
    1
);

INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT '00000000-0000-0000-0000-0000000000c1', r.id
FROM roles r WHERE r.slug = 'admin' LIMIT 1;

INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT '00000000-0000-0000-0000-0000000000c2', r.id
FROM roles r WHERE r.slug = 'user' LIMIT 1;

INSERT OR IGNORE INTO user_branches (user_id, branch_id)
VALUES
    ('00000000-0000-0000-0000-0000000000c1', '00000000-0000-0000-0000-0000000000b1'),
    ('00000000-0000-0000-0000-0000000000c2', '00000000-0000-0000-0000-0000000000b1');
