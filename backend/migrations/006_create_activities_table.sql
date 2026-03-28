-- PostgreSQL 17: activities (audit trail)
CREATE TABLE IF NOT EXISTS activities (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    description TEXT NOT NULL,
    metadata TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_activities_user_id ON activities(user_id);
CREATE INDEX IF NOT EXISTS idx_activities_entity_type ON activities(entity_type);
CREATE INDEX IF NOT EXISTS idx_activities_entity_id ON activities(entity_id);
CREATE INDEX IF NOT EXISTS idx_activities_action ON activities(action);
CREATE INDEX IF NOT EXISTS idx_activities_created_at ON activities(created_at DESC);
