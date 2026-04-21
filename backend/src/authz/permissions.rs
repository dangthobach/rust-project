//! Stable permission codes — must match seeds in `permissions` table migrations.
//!
//! These are **engine-level** codes: the application logic (data-scope filters,
//! file-policy checks, etc.) branches on specific codes here.  Which roles hold
//! which codes is 100 % dynamic and configured via the admin UI.

// ── System ────────────────────────────────────────────────────────────────────
pub const SYSTEM_SUPERUSER: &str = "system.superuser";

// ── Files ─────────────────────────────────────────────────────────────────────
pub const FILES_LIST_OWN: &str      = "files.list.own";
pub const FILES_LIST_ALL: &str      = "files.list.all";
pub const FILES_SEARCH_OWN: &str    = "files.search.own";
pub const FILES_SEARCH_ALL: &str    = "files.search.all";
pub const FILES_READ_OWN: &str      = "files.read.own";
pub const FILES_READ_ALL: &str      = "files.read.all";
pub const FILES_DOWNLOAD_OWN: &str  = "files.download.own";
pub const FILES_DOWNLOAD_ALL: &str  = "files.download.all";
pub const FILES_UPLOAD: &str        = "files.upload";
pub const FILES_DELETE_OWN: &str    = "files.delete.own";
pub const FILES_DELETE_ANY: &str    = "files.delete.any";

// ── Branch & data scope ───────────────────────────────────────────────────────
pub const BRANCH_DATA_ALL: &str       = "branch.data.all";
pub const BRANCH_MANAGE: &str         = "branch.manage";
pub const RESOURCE_GRANT_MANAGE: &str = "resource.grant.manage";

// ── RBAC management (admin UI) ────────────────────────────────────────────────
pub const ROLE_MANAGE: &str       = "role.manage";
pub const PERMISSION_MANAGE: &str = "permission.manage";

// ── User management (admin UI) ────────────────────────────────────────────────
pub const USER_MANAGE: &str = "user.manage";

// ── Domain entity write operations ────────────────────────────────────────────
pub const CLIENT_WRITE: &str    = "client.write";
pub const TASK_DELETE_ANY: &str = "task.delete.any";
