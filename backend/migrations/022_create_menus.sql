-- ============================================================================
-- 022: Dynamic menu system
--
-- Design decisions:
--   • required_permission → NULL   = any authenticated user
--   • required_permission → code   = user must hold that permission
--   • parent_key → NULL            = top-level item (root)
--   • path → NULL                  = section header (non-clickable group)
--   • icon                         = emoji or icon-key string (FE resolves)
--   • sort_order                   = lower = higher in list
-- ============================================================================

CREATE TABLE IF NOT EXISTS menus (
    id                   UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    key                  VARCHAR(100) NOT NULL UNIQUE,
    parent_key           VARCHAR(100) REFERENCES menus(key) ON DELETE SET NULL,
    label                VARCHAR(200) NOT NULL,
    path                 VARCHAR(500),
    icon                 VARCHAR(100),
    sort_order           INTEGER      NOT NULL DEFAULT 0,
    is_active            BOOLEAN      NOT NULL DEFAULT true,
    -- NULL → visible to all authenticated users
    -- <code> → visible only if user holds this permission
    required_permission  VARCHAR(100) REFERENCES permissions(code) ON DELETE SET NULL,
    created_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_menus_parent_key   ON menus(parent_key);
CREATE INDEX IF NOT EXISTS idx_menus_sort_order   ON menus(sort_order);
CREATE INDEX IF NOT EXISTS idx_menus_required_perm ON menus(required_permission);

-- ── Seed: default menu items ──────────────────────────────────────────────────
-- Main navigation (no permission required — any authenticated user)
INSERT INTO menus (key, parent_key, label, path, icon, sort_order, required_permission) VALUES
    ('dashboard',       NULL,    'Dashboard',      '/',                'dashboard',  10,  NULL),
    ('clients',         NULL,    'Clients',         '/clients',         'clients',    20,  NULL),
    ('tasks',           NULL,    'Tasks',           '/tasks',           'tasks',      30,  NULL),
    ('reports',         NULL,    'Reports',         '/reports',         'reports',    40,  NULL),
    ('notifications',   NULL,    'Alerts',          '/notifications',   'bell',       50,  NULL),
    ('files',           NULL,    'Files',           '/files',           'files',      60,  NULL)
ON CONFLICT (key) DO NOTHING;

-- Admin section header (non-clickable group, requires any admin permission)
INSERT INTO menus (key, parent_key, label, path, icon, sort_order, required_permission) VALUES
    ('admin',           NULL,    'Admin',           NULL,               'shield',     100, 'role.manage')
ON CONFLICT (key) DO NOTHING;

-- Admin sub-items
INSERT INTO menus (key, parent_key, label, path, icon, sort_order, required_permission) VALUES
    ('admin.users',       'admin', 'Users',          '/users',                    'users',   10, 'user.manage'),
    ('admin.rbac.roles',  'admin', 'Roles',          '/admin/rbac/roles',         'shield',  20, 'role.manage'),
    ('admin.rbac.perms',  'admin', 'Permissions',    '/admin/rbac/permissions',   'key',     30, 'permission.manage'),
    ('admin.rbac.matrix', 'admin', 'Ma trận',        '/admin/rbac/matrix',        'grid',    40, 'role.manage'),
    ('admin.user-roles',  'admin', 'User-Role',      '/admin/rbac/user-roles',    'user-cog',50, 'user.manage')
ON CONFLICT (key) DO NOTHING;
