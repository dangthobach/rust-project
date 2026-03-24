-- RBAC: users <-> roles (M2M), roles <-> permissions (M2M)
-- Permission codes are stable API contracts (e.g. files.download.own).

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS roles (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS permissions (
    code TEXT PRIMARY KEY NOT NULL,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_code)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id TEXT NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_perm ON role_permissions(permission_code);

INSERT OR IGNORE INTO roles (id, slug, description) VALUES
('00000000-0000-0000-0000-0000000000a1', 'admin', 'Full system access'),
('00000000-0000-0000-0000-0000000000a2', 'manager', 'Team lead — broader file access'),
('00000000-0000-0000-0000-0000000000a3', 'user', 'Standard user — own files only');

INSERT OR IGNORE INTO permissions (code, description) VALUES
('system.superuser', 'Bypass normal permission checks (use sparingly)'),
('files.list.own', 'List files uploaded by self'),
('files.list.all', 'List all users files'),
('files.search.own', 'Search (FTS) within own uploads'),
('files.search.all', 'Search (FTS) across all uploads'),
('files.read.own', 'Read metadata of own files'),
('files.read.all', 'Read metadata of any file'),
('files.download.own', 'Download own files'),
('files.download.all', 'Download any file'),
('files.upload', 'Upload new files'),
('files.delete.own', 'Delete own files'),
('files.delete.any', 'Delete any user file');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code)
SELECT '00000000-0000-0000-0000-0000000000a1', code FROM permissions WHERE code = 'system.superuser';

INSERT OR IGNORE INTO role_permissions (role_id, permission_code) VALUES
('00000000-0000-0000-0000-0000000000a2', 'files.list.all'),
('00000000-0000-0000-0000-0000000000a2', 'files.search.all'),
('00000000-0000-0000-0000-0000000000a2', 'files.read.all'),
('00000000-0000-0000-0000-0000000000a2', 'files.download.all'),
('00000000-0000-0000-0000-0000000000a2', 'files.upload'),
('00000000-0000-0000-0000-0000000000a2', 'files.delete.any');

INSERT OR IGNORE INTO role_permissions (role_id, permission_code) VALUES
('00000000-0000-0000-0000-0000000000a3', 'files.list.own'),
('00000000-0000-0000-0000-0000000000a3', 'files.search.own'),
('00000000-0000-0000-0000-0000000000a3', 'files.read.own'),
('00000000-0000-0000-0000-0000000000a3', 'files.download.own'),
('00000000-0000-0000-0000-0000000000a3', 'files.upload'),
('00000000-0000-0000-0000-0000000000a3', 'files.delete.own');

INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u
INNER JOIN roles r ON LOWER(r.slug) = LOWER(u.role);

INSERT OR IGNORE INTO user_roles (user_id, role_id)
SELECT u.id, '00000000-0000-0000-0000-0000000000a3'
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id
);
