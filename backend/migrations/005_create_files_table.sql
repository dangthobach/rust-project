-- PostgreSQL 17: files (binary uploads metadata)
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    original_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT,
    file_size BIGINT,
    uploaded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    client_id UUID REFERENCES clients(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    description TEXT,
    thumbnail_path TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_files_client_id ON files(client_id);
CREATE INDEX IF NOT EXISTS idx_files_task_id ON files(task_id);
CREATE INDEX IF NOT EXISTS idx_files_uploaded_by ON files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_files_file_type ON files(file_type);
CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at DESC);
