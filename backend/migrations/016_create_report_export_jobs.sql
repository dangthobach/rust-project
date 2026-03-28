CREATE TABLE IF NOT EXISTS report_export_jobs (
    id UUID PRIMARY KEY NOT NULL,
    user_id UUID NOT NULL,
    report_type TEXT NOT NULL,
    format TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'queued',
    object_uri TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_report_export_jobs_user_status_created
ON report_export_jobs(user_id, status, created_at);

CREATE INDEX IF NOT EXISTS idx_report_export_jobs_status
ON report_export_jobs(status);

DROP TRIGGER IF EXISTS update_report_export_jobs_updated_at ON report_export_jobs;
CREATE TRIGGER update_report_export_jobs_updated_at
BEFORE UPDATE ON report_export_jobs
FOR EACH ROW
EXECUTE FUNCTION trg_set_updated_at();
