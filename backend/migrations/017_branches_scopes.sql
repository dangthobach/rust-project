-- Branches (tree), user↔branch, resource grants; row-level branch_id on clients/tasks
CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY NOT NULL,
    parent_id UUID REFERENCES branches(id) ON DELETE RESTRICT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_branches_parent ON branches(parent_id);

CREATE TABLE IF NOT EXISTS user_branches (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, branch_id)
);

CREATE INDEX IF NOT EXISTS idx_user_branches_user ON user_branches(user_id);
CREATE INDEX IF NOT EXISTS idx_user_branches_branch ON user_branches(branch_id);

CREATE TABLE IF NOT EXISTS resource_grants (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource_kind TEXT NOT NULL CHECK (resource_kind IN ('client', 'task')),
    resource_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, resource_kind, resource_id)
);

CREATE INDEX IF NOT EXISTS idx_resource_grants_user_kind ON resource_grants(user_id, resource_kind);

INSERT INTO branches (id, parent_id, name, slug) VALUES
('00000000-0000-0000-0000-0000000000b1'::uuid, NULL, 'Root', 'root')
ON CONFLICT (id) DO NOTHING;

ALTER TABLE clients ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE SET NULL;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS branch_id UUID REFERENCES branches(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_clients_branch_id ON clients(branch_id);
CREATE INDEX IF NOT EXISTS idx_tasks_branch_id ON tasks(branch_id);

UPDATE clients SET branch_id = '00000000-0000-0000-0000-0000000000b1'::uuid WHERE branch_id IS NULL;
UPDATE tasks SET branch_id = '00000000-0000-0000-0000-0000000000b1'::uuid WHERE branch_id IS NULL;

INSERT INTO user_branches (user_id, branch_id)
SELECT u.id, '00000000-0000-0000-0000-0000000000b1'::uuid FROM users u
ON CONFLICT DO NOTHING;

INSERT INTO permissions (code, description) VALUES
('branch.data.all', 'Access data across all branches (bypass branch filter)'),
('branch.manage', 'Manage branch records and user-branch assignments'),
('resource.grant.manage', 'Assign or revoke per-resource grants for users')
ON CONFLICT (code) DO NOTHING;

INSERT INTO role_permissions (role_id, permission_code)
SELECT '00000000-0000-0000-0000-0000000000a1'::uuid, code FROM permissions
WHERE code IN ('branch.data.all', 'branch.manage', 'resource.grant.manage')
ON CONFLICT DO NOTHING;
