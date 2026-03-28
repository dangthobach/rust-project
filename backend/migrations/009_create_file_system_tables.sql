-- Read model: file_views + folder_tree + file_permissions (PostgreSQL 17)
CREATE TABLE IF NOT EXISTS file_views (
    id UUID PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    parent_id UUID,
    size BIGINT DEFAULT 0,
    mime_type TEXT,
    owner_id UUID NOT NULL,
    permissions TEXT DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL,
    created_by UUID,
    updated_at TIMESTAMPTZ NOT NULL,
    updated_by UUID,
    deleted_at TIMESTAMPTZ,
    deleted_by UUID,
    item_type TEXT NOT NULL CHECK (item_type IN ('file', 'folder'))
);

CREATE INDEX IF NOT EXISTS idx_file_views_path ON file_views(path);
CREATE INDEX IF NOT EXISTS idx_file_views_parent ON file_views(parent_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_file_views_owner ON file_views(owner_id);
CREATE INDEX IF NOT EXISTS idx_file_views_type ON file_views(item_type);
CREATE INDEX IF NOT EXISTS idx_file_views_deleted ON file_views(deleted_at) WHERE deleted_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS folder_tree (
    ancestor_id UUID NOT NULL,
    descendant_id UUID NOT NULL,
    depth INTEGER NOT NULL,
    PRIMARY KEY (ancestor_id, descendant_id),
    FOREIGN KEY (ancestor_id) REFERENCES file_views(id) ON DELETE CASCADE,
    FOREIGN KEY (descendant_id) REFERENCES file_views(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_folder_tree_descendant ON folder_tree(descendant_id);
CREATE INDEX IF NOT EXISTS idx_folder_tree_ancestor ON folder_tree(ancestor_id);

CREATE TABLE IF NOT EXISTS file_permissions (
    file_id UUID NOT NULL,
    subject_type TEXT NOT NULL CHECK (subject_type IN ('user', 'group', 'everyone')),
    subject_id UUID,
    permission TEXT NOT NULL CHECK (permission IN ('read', 'write', 'delete', 'share', 'admin')),
    inherited BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (file_id) REFERENCES file_views(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS uq_file_permissions_unique
ON file_permissions (file_id, subject_type, subject_id, permission) NULLS NOT DISTINCT;

CREATE INDEX IF NOT EXISTS idx_file_permissions_file ON file_permissions(file_id);
CREATE INDEX IF NOT EXISTS idx_file_permissions_subject ON file_permissions(subject_type, subject_id);
