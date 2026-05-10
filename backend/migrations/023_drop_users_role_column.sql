-- Safety backfill: assign default 'user' role to any accounts with no role assignment.
-- Users created by 007/020 were already backfilled in 014; this covers edge cases.
INSERT INTO user_roles (user_id, role_id)
SELECT u.id, r.id
FROM users u
JOIN roles r ON lower(r.slug) = lower(u.role)
WHERE NOT EXISTS (SELECT 1 FROM user_roles ur WHERE ur.user_id = u.id)
ON CONFLICT DO NOTHING;

-- Remove the legacy role label column (source of truth is user_roles M:N table)
DROP INDEX IF EXISTS idx_users_role;
ALTER TABLE users DROP COLUMN IF EXISTS role;
