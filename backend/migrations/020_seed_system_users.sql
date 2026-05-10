-- System operator accounts (idempotent)
INSERT INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-0000000000c1'::uuid,
    'administrator@system.local',
    '$2b$12$qa6wQ4XVr5t1auhcUaUw.enX5V.h6FStGY24mvcRBL5pbsz/G0/dK',
    'System Administrator',
    'admin',
    TRUE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO users (id, email, password_hash, full_name, role, is_active)
VALUES (
    '00000000-0000-0000-0000-0000000000c2'::uuid,
    'application@system.local',
    '$2b$12$QuBn4SXpok6aLn5J.yCZ8u9ngElROtDMkI7iZdp6Epb5j/M3V2M5e',
    'Application Integration',
    'user',
    TRUE
)
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT '00000000-0000-0000-0000-0000000000c1'::uuid, r.id
FROM roles r WHERE r.slug = 'admin' LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO user_roles (user_id, role_id)
SELECT '00000000-0000-0000-0000-0000000000c2'::uuid, r.id
FROM roles r WHERE r.slug = 'user' LIMIT 1
ON CONFLICT DO NOTHING;

INSERT INTO user_branches (user_id, branch_id)
VALUES
    ('00000000-0000-0000-0000-0000000000c1'::uuid, '00000000-0000-0000-0000-0000000000b1'::uuid),
    ('00000000-0000-0000-0000-0000000000c2'::uuid, '00000000-0000-0000-0000-0000000000b1'::uuid)
ON CONFLICT DO NOTHING;
