-- Runtime-configurable system defaults; all values editable via admin UI without code changes.
CREATE TABLE IF NOT EXISTS system_settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    description TEXT,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ── New permission codes for fully-dynamic RBAC management ──────────────────
INSERT INTO permissions (code, description) VALUES
('role.manage',       'Create, update, and delete roles'),
('permission.manage', 'Create, update, and delete permission codes'),
('user.manage',       'Create, update, and delete users via admin panel'),
('client.write',      'Create, update, and delete clients'),
('task.delete.any',   'Delete any task regardless of ownership or scope')
ON CONFLICT (code) DO NOTHING;

-- Grant all new management permissions to the admin role
INSERT INTO role_permissions (role_id, permission_code)
SELECT '00000000-0000-0000-0000-0000000000a1'::uuid, code
FROM permissions
WHERE code IN ('role.manage', 'permission.manage', 'user.manage', 'client.write', 'task.delete.any')
ON CONFLICT DO NOTHING;

-- Grant write permissions to the manager role
INSERT INTO role_permissions (role_id, permission_code) VALUES
('00000000-0000-0000-0000-0000000000a2'::uuid, 'client.write'),
('00000000-0000-0000-0000-0000000000a2'::uuid, 'task.delete.any')
ON CONFLICT DO NOTHING;

-- ── System defaults (all values configurable from admin UI) ─────────────────
INSERT INTO system_settings (key, value, description) VALUES
(
    'default_role_slugs',
    '["user"]',
    'JSON array of role slugs auto-assigned to new self-registered users. Example: ["user","viewer"]'
),
(
    'default_branch_id',
    '00000000-0000-0000-0000-0000000000b1',
    'UUID of the branch automatically assigned to every new user. Must be an existing active branch.'
),
(
    'registration_enabled',
    'true',
    'Set to "false" to disable public self-service registration.'
)
ON CONFLICT (key) DO NOTHING;

-- ── Lift hard-coded CHECK constraint on resource_kind ───────────────────────
-- The previous constraint locked resource_kind to ('client','task').
-- Removing it lets admins extend grants to future entity types without schema changes.
ALTER TABLE resource_grants
    DROP CONSTRAINT IF EXISTS resource_grants_resource_kind_check;

-- A non-empty constraint is still enforced at application level.
ALTER TABLE resource_grants
    ADD CONSTRAINT resource_grants_resource_kind_nonempty
    CHECK (resource_kind <> '');
