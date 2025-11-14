-- Read Model: File Views (denormalized for queries) - SQLite compatible
CREATE TABLE IF NOT EXISTS file_views (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,  -- Materialized path: /folder1/folder2/
    parent_id TEXT,
    size INTEGER DEFAULT 0,
    mime_type TEXT,
    owner_id TEXT NOT NULL,
    permissions TEXT DEFAULT '[]',
    
    -- Audit
    created_at TEXT NOT NULL,
    created_by TEXT,
    updated_at TEXT NOT NULL,
    updated_by TEXT,
    
    -- Soft Delete
    deleted_at TEXT,
    deleted_by TEXT,
    
    -- Type: 'file' or 'folder'
    item_type TEXT NOT NULL CHECK (item_type IN ('file', 'folder'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_file_views_path ON file_views(path);
CREATE INDEX IF NOT EXISTS idx_file_views_parent ON file_views(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_file_views_owner ON file_views(owner_id);
CREATE INDEX IF NOT EXISTS idx_file_views_type ON file_views(item_type);
CREATE INDEX IF NOT EXISTS idx_file_views_deleted ON file_views(deleted_at) WHERE deleted_at IS NOT NULL;

-- Folder tree (closure table for fast tree queries)
CREATE TABLE IF NOT EXISTS folder_tree (
    ancestor_id TEXT NOT NULL,
    descendant_id TEXT NOT NULL,
    depth INTEGER NOT NULL,
    
    PRIMARY KEY (ancestor_id, descendant_id),
    FOREIGN KEY (ancestor_id) REFERENCES file_views(id) ON DELETE CASCADE,
    FOREIGN KEY (descendant_id) REFERENCES file_views(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_folder_tree_descendant ON folder_tree(descendant_id);
CREATE INDEX IF NOT EXISTS idx_folder_tree_ancestor ON folder_tree(ancestor_id);

-- File permissions (denormalized for fast ACL checks)
CREATE TABLE IF NOT EXISTS file_permissions (
    file_id TEXT NOT NULL,
    subject_type TEXT NOT NULL CHECK (subject_type IN ('user', 'group', 'everyone')),
    subject_id TEXT,  -- NULL for 'everyone'
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write', 'delete', 'share', 'admin')),
    inherited INTEGER DEFAULT 0,
    
    PRIMARY KEY (file_id, subject_type, subject_id, permission),
    FOREIGN KEY (file_id) REFERENCES file_views(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_file_permissions_file ON file_permissions(file_id);
CREATE INDEX IF NOT EXISTS idx_file_permissions_subject ON file_permissions(subject_type, subject_id);
