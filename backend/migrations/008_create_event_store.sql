-- Event store + snapshots + projection checkpoints (PostgreSQL 17)
-- Global ordering uses seq (replaces SQLite rowid).
CREATE TABLE IF NOT EXISTS event_store (
    seq BIGSERIAL PRIMARY KEY,
    id UUID NOT NULL UNIQUE,
    aggregate_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data TEXT NOT NULL,
    version BIGINT NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata TEXT NOT NULL DEFAULT '{}',
    UNIQUE (aggregate_id, version)
);

CREATE INDEX IF NOT EXISTS idx_event_store_aggregate ON event_store(aggregate_id, version);
CREATE INDEX IF NOT EXISTS idx_event_store_type ON event_store(aggregate_type, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at ON event_store(occurred_at);

CREATE TABLE IF NOT EXISTS snapshots (
    aggregate_id UUID PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_data TEXT NOT NULL,
    version BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_snapshots_type ON snapshots(aggregate_type, version);

CREATE TABLE IF NOT EXISTS projection_positions (
    projection_name TEXT PRIMARY KEY,
    position BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
