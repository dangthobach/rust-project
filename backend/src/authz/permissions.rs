//! Stable permission codes (must match `permissions` table seeds / migrations).

pub const SYSTEM_SUPERUSER: &str = "system.superuser";

pub const FILES_LIST_OWN: &str = "files.list.own";
pub const FILES_LIST_ALL: &str = "files.list.all";
pub const FILES_SEARCH_OWN: &str = "files.search.own";
pub const FILES_SEARCH_ALL: &str = "files.search.all";
pub const FILES_READ_OWN: &str = "files.read.own";
pub const FILES_READ_ALL: &str = "files.read.all";
pub const FILES_DOWNLOAD_OWN: &str = "files.download.own";
pub const FILES_DOWNLOAD_ALL: &str = "files.download.all";
pub const FILES_UPLOAD: &str = "files.upload";
pub const FILES_DELETE_OWN: &str = "files.delete.own";
pub const FILES_DELETE_ANY: &str = "files.delete.any";
