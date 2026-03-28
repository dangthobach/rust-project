-- RBAC: users <-> roles (M2M), roles <-> permissions (M2M)
CREATE TABLE IF NOT EXISTS roles (
    id UUID PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS permissions (
    code TEXT PRIMARY KEY NOT NULL,
    description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS role_permissions (
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_code TEXT NOT NULL REFERENCES permissions(code) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_code)
);

CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

CREATE INDEX IF NOT EXISTS idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX IF NOT EXISTS idx_role_permissions_perm ON role_permissions(permission_code);

INSERT INTO roles (id, slug, description) VALUES
('00000000-0000-0000-0000-0000000000a1'::uuid, 'admin', 'Full system access'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'manager', 'Team lead — broader file access'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'user', 'Standard user — own files only')
ON CONFLICT (id) DO NOTHING;

INSERT INTO permissions (code, description) VALUES
('system.superuser', 'Bypass normal permission checks (use sparingly)'),
('files.list.own', 'List files uploaded by self'),
('files.list.all', 'List all users files'),
('files.search.own', 'Search within own uploads'),
('files.search.all', 'Search across all uploads'),
('files.read.own', 'Read metadata of own files'),
('files.read.all', 'Read metadata of any file'),
('files.download.own', 'Download own files'),
('files.download.all', 'Download any file'),
('files.upload', 'Upload new files'),
('files.delete.own', 'Delete own files'),
('files.delete.any', 'Delete any user file')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_code)
SELECT '00000000-0000-0000-0000-0000000000a1'::uuid, code FROM permissions WHERE code = 'system.superuser'
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_code) VALUES
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.list.all'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.search.all'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.read.all'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.download.all'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.upload'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'files.delete.any')
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role_id, permission_code) VALUES
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.list.own'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.search.own'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.read.own'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.download.own'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.upload'),
('00000000-0000-0000-0000-0000000000a3'::uuid, 'files.delete.own')
ON CONFLICT DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u
INNER JOIN roles r ON lower(r.slug) = lower(u.role)
ON CONFLICT DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT u.id, '00000000-0000-0000-0000-0000000000a3'::uuid
FROM users u
WHERE NOT EXISTS (
    SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id
)
ON CONFLICT DO NOTHING;
