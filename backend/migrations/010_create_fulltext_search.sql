-- PostgreSQL full-text search (replaces SQLite FTS5): generated tsvector + GIN
ALTER TABLE clients ADD COLUMN IF NOT EXISTS search_vector tsvector
GENERATED ALWAYS AS (
  to_tsvector(
    'simple',
    coalesce(name, '') || ' ' || coalesce(email, '') || ' ' || coalesce(company, '') || ' '
    || coalesce(phone, '') || ' ' || coalesce(notes, '')
  )
) STORED;

CREATE INDEX IF NOT EXISTS idx_clients_search_vector ON clients USING GIN (search_vector);

ALTER TABLE tasks ADD COLUMN IF NOT EXISTS search_vector tsvector
GENERATED ALWAYS AS (
  to_tsvector('simple', coalesce(title, '') || ' ' || coalesce(description, ''))
) STORED;

CREATE INDEX IF NOT EXISTS idx_tasks_search_vector ON tasks USING GIN (search_vector);

ALTER TABLE files ADD COLUMN IF NOT EXISTS search_vector tsvector
GENERATED ALWAYS AS (
  to_tsvector(
    'simple',
    coalesce(name, '') || ' ' || coalesce(original_name, '') || ' ' || coalesce(description, '')
  )
) STORED;

CREATE INDEX IF NOT EXISTS idx_files_search_vector ON files USING GIN (search_vector);
