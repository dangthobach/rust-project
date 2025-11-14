-- Event Store for Event Sourcing (SQLite compatible)
CREATE TABLE IF NOT EXISTS event_store (
    id TEXT PRIMARY KEY,
    aggregate_id TEXT NOT NULL,
    aggregate_type TEXT NOT NULL,
    event_type TEXT NOT NULL,
    event_data TEXT NOT NULL,
    version INTEGER NOT NULL,
    occurred_at TEXT NOT NULL DEFAULT (datetime('now')),
    metadata TEXT DEFAULT '{}',
    
    -- Unique constraint: one version per aggregate
    UNIQUE(aggregate_id, version)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_event_store_aggregate ON event_store(aggregate_id, version);
CREATE INDEX IF NOT EXISTS idx_event_store_type ON event_store(aggregate_type, occurred_at);
CREATE INDEX IF NOT EXISTS idx_event_store_occurred_at ON event_store(occurred_at);

-- Snapshots table
CREATE TABLE IF NOT EXISTS snapshots (
    aggregate_id TEXT PRIMARY KEY,
    aggregate_type TEXT NOT NULL,
    aggregate_data TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_snapshots_type ON snapshots(aggregate_type, version);

-- Projection positions (checkpoint for projections)
CREATE TABLE IF NOT EXISTS projection_positions (
    projection_name TEXT PRIMARY KEY,
    position INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
