-- Create FTS5 virtual tables for full-text search

-- Clients search
CREATE VIRTUAL TABLE IF NOT EXISTS clients_fts USING fts5(
    id UNINDEXED,
    name,
    email,
    company,
    phone,
    notes,
    tokenize = 'porter unicode61'
);

-- Populate clients FTS from existing data
INSERT INTO clients_fts (id, name, email, company, phone, notes)
SELECT id, name, COALESCE(email, ''), COALESCE(company, ''), COALESCE(phone, ''), COALESCE(notes, '')
FROM clients;

-- Create trigger to keep FTS in sync with clients table
CREATE TRIGGER clients_fts_insert AFTER INSERT ON clients BEGIN
    INSERT INTO clients_fts (id, name, email, company, phone, notes)
    VALUES (
        new.id,
        new.name,
        COALESCE(new.email, ''),
        COALESCE(new.company, ''),
        COALESCE(new.phone, ''),
        COALESCE(new.notes, '')
    );
END;

CREATE TRIGGER clients_fts_update AFTER UPDATE ON clients BEGIN
    UPDATE clients_fts SET
        name = new.name,
        email = COALESCE(new.email, ''),
        company = COALESCE(new.company, ''),
        phone = COALESCE(new.phone, ''),
        notes = COALESCE(new.notes, '')
    WHERE id = new.id;
END;

CREATE TRIGGER clients_fts_delete AFTER DELETE ON clients BEGIN
    DELETE FROM clients_fts WHERE id = old.id;
END;

-- Tasks search
CREATE VIRTUAL TABLE IF NOT EXISTS tasks_fts USING fts5(
    id UNINDEXED,
    title,
    description,
    tokenize = 'porter unicode61'
);

-- Populate tasks FTS from existing data
INSERT INTO tasks_fts (id, title, description)
SELECT id, title, COALESCE(description, '')
FROM tasks;

-- Create trigger to keep FTS in sync with tasks table
CREATE TRIGGER tasks_fts_insert AFTER INSERT ON tasks BEGIN
    INSERT INTO tasks_fts (id, title, description)
    VALUES (new.id, new.title, COALESCE(new.description, ''));
END;

CREATE TRIGGER tasks_fts_update AFTER UPDATE ON tasks BEGIN
    UPDATE tasks_fts SET
        title = new.title,
        description = COALESCE(new.description, '')
    WHERE id = new.id;
END;

CREATE TRIGGER tasks_fts_delete AFTER DELETE ON tasks BEGIN
    DELETE FROM tasks_fts WHERE id = old.id;
END;

-- Files search
CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
    id UNINDEXED,
    name,
    original_name,
    description,
    tokenize = 'porter unicode61'
);

-- Populate files FTS from existing data
INSERT INTO files_fts (id, name, original_name, description)
SELECT id, name, original_name, COALESCE(description, '')
FROM files;

-- Create trigger to keep FTS in sync with files table
CREATE TRIGGER files_fts_insert AFTER INSERT ON files BEGIN
    INSERT INTO files_fts (id, name, original_name, description)
    VALUES (new.id, new.name, new.original_name, COALESCE(new.description, ''));
END;

CREATE TRIGGER files_fts_update AFTER UPDATE ON files BEGIN
    UPDATE files_fts SET
        name = new.name,
        original_name = new.original_name,
        description = COALESCE(new.description, '')
    WHERE id = new.id;
END;

CREATE TRIGGER files_fts_delete AFTER DELETE ON files BEGIN
    DELETE FROM files_fts WHERE id = old.id;
END;
