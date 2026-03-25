-- Async report export jobs queue (queued -> processing -> ready/failed)
CREATE TABLE IF NOT EXISTS report_export_jobs (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    report_type TEXT NOT NULL, -- clients | tasks | users | dashboard
    format TEXT NOT NULL, -- csv | json
    status TEXT NOT NULL DEFAULT 'queued', -- queued | processing | ready | failed
    object_uri TEXT,
    error_message TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_report_export_jobs_user_status_created
ON report_export_jobs(user_id, status, created_at);

CREATE INDEX IF NOT EXISTS idx_report_export_jobs_status
ON report_export_jobs(status);

-- Keep updated_at fresh
CREATE TRIGGER IF NOT EXISTS update_report_export_jobs_updated_at
AFTER UPDATE ON report_export_jobs
FOR EACH ROW
BEGIN
    UPDATE report_export_jobs SET updated_at = datetime('now') WHERE id = NEW.id;
END;

