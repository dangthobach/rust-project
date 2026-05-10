-- Optional date range filters for async report exports (inclusive bounds on created_at of source rows)
ALTER TABLE report_export_jobs
    ADD COLUMN IF NOT EXISTS start_date DATE,
    ADD COLUMN IF NOT EXISTS end_date DATE;
