-- Create files table (SQLite compatible)
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    original_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    file_type TEXT, -- MIME type
    file_size INTEGER, -- in bytes
    uploaded_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    client_id TEXT REFERENCES clients(id) ON DELETE CASCADE,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    description TEXT,
    thumbnail_path TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Create indexes for file queries
CREATE INDEX IF NOT EXISTS idx_files_client_id ON files(client_id);
CREATE INDEX IF NOT EXISTS idx_files_task_id ON files(task_id);
CREATE INDEX IF NOT EXISTS idx_files_uploaded_by ON files(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_files_file_type ON files(file_type);
CREATE INDEX IF NOT EXISTS idx_files_created_at ON files(created_at DESC);
