//! File authorization: combine RBAC codes with ownership (`files.uploaded_by`).

use uuid::Uuid;

use crate::authz::context::AuthContext;
use crate::authz::permissions as perm;
use crate::error::{AppError, AppResult};
use crate::models::File;

#[inline]
fn owner_id_matches(file: &File, user_id: &Uuid) -> bool {
    file.uploaded_by.map(|ub| ub == *user_id).unwrap_or(false)
}

/// `true` = no `uploaded_by` filter; `false` = restrict to current user.
pub fn file_list_scope(ctx: &AuthContext) -> AppResult<bool> {
    if ctx.superuser() || ctx.has(perm::FILES_LIST_ALL) {
        return Ok(true);
    }
    if ctx.has(perm::FILES_LIST_OWN) {
        return Ok(false);
    }
    Err(AppError::Forbidden(
        "Missing permission to list files".to_string(),
    ))
}

/// Same semantics as list, for search endpoint.
pub fn file_search_scope(ctx: &AuthContext) -> AppResult<bool> {
    if ctx.superuser() || ctx.has(perm::FILES_SEARCH_ALL) {
        return Ok(true);
    }
    if ctx.has(perm::FILES_SEARCH_OWN) {
        return Ok(false);
    }
    Err(AppError::Forbidden(
        "Missing permission to search files".to_string(),
    ))
}

pub fn require_file_upload(ctx: &AuthContext) -> AppResult<()> {
    if ctx.superuser() || ctx.has(perm::FILES_UPLOAD) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "Missing permission: files.upload".to_string(),
        ))
    }
}

/// Returns whether caller may read metadata for this row (used before returning JSON).
pub fn can_read_file_meta(ctx: &AuthContext, file: &File) -> bool {
    if ctx.superuser() || ctx.has(perm::FILES_READ_ALL) {
        return true;
    }
    ctx.has(perm::FILES_READ_OWN) && owner_id_matches(file, &ctx.user_id)
}

pub fn can_download_file(ctx: &AuthContext, file: &File) -> bool {
    if ctx.superuser() || ctx.has(perm::FILES_DOWNLOAD_ALL) {
        return true;
    }
    ctx.has(perm::FILES_DOWNLOAD_OWN) && owner_id_matches(file, &ctx.user_id)
}

pub fn can_delete_file(ctx: &AuthContext, file: &File) -> bool {
    if ctx.superuser() || ctx.has(perm::FILES_DELETE_ANY) {
        return true;
    }
    ctx.has(perm::FILES_DELETE_OWN) && owner_id_matches(file, &ctx.user_id)
}
