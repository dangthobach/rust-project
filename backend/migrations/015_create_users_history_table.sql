-- Aggregate history (explicit audit snapshots)
CREATE TABLE IF NOT EXISTS aggregate_history (
    id UUID PRIMARY KEY NOT NULL,
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    action TEXT NOT NULL,
    old_status TEXT,
    new_status TEXT,
    actor_id TEXT,
    comment TEXT,
    metadata TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_aggregate_history_ref
ON aggregate_history(aggregate_type, aggregate_id);

CREATE INDEX IF NOT EXISTS idx_aggregate_history_action
ON aggregate_history(action);

CREATE INDEX IF NOT EXISTS idx_aggregate_history_created_at
ON aggregate_history(created_at);
