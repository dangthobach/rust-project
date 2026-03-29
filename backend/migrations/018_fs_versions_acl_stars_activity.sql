-- File system extensions: versions, stars, and activity helpers (PostgreSQL 17)

-- Link fs file_views to binary storage (legacy uploads table `files`)
ALTER TABLE file_views
    ADD COLUMN IF NOT EXISTS storage_file_id UUID,
    ADD COLUMN IF NOT EXISTS current_version_id UUID;

CREATE INDEX IF NOT EXISTS idx_file_views_storage_file_id ON file_views(storage_file_id);

-- Versions table: immutable records; current version enforced via partial unique index
CREATE TABLE IF NOT EXISTS fs_file_versions (
    id UUID PRIMARY KEY NOT NULL,
    file_id UUID NOT NULL REFERENCES file_views(id) ON DELETE CASCADE,
    storage_file_id UUID NOT NULL REFERENCES files(id) ON DELETE RESTRICT,
    version_no INTEGER NOT NULL,
    is_current BOOLEAN NOT NULL DEFAULT FALSE,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_fs_file_versions_file_version
ON fs_file_versions(file_id, version_no);

CREATE UNIQUE INDEX IF NOT EXISTS uq_fs_file_versions_current
ON fs_file_versions(file_id)
WHERE is_current = TRUE;

CREATE INDEX IF NOT EXISTS idx_fs_file_versions_file_created_at
ON fs_file_versions(file_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_fs_file_versions_storage_file
ON fs_file_versions(storage_file_id);

-- Stars (per-user pin/star)
CREATE TABLE IF NOT EXISTS fs_file_stars (
    file_id UUID NOT NULL REFERENCES file_views(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (file_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_fs_file_stars_user ON fs_file_stars(user_id, created_at DESC);

-- Activity materialized view is optional; for now we'll query `aggregate_history`.
-- Keep a lightweight index for common file activity lookups.
CREATE INDEX IF NOT EXISTS idx_aggregate_history_file_ref_created_at
ON aggregate_history(aggregate_type, aggregate_id, created_at DESC);

