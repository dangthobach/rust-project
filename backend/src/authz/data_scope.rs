//! Row-level scope: combine RBAC (`branch.data.all`, `system.superuser`) with
//! branch membership + optional `resource_grants` (fine-grained).

use sqlx::{QueryBuilder, Postgres, PgPool};
use std::collections::BTreeSet;
use std::sync::Arc;

use uuid::Uuid;

use crate::authz::context::AuthContext;
use crate::authz::permissions as perm;
use crate::error::AppError;
use crate::models::{Client, Task};

/// Stable root branch id (must match migration seed).
pub const ROOT_BRANCH_ID: &str = "00000000-0000-0000-0000-0000000000b1";

#[derive(Clone, Debug)]
pub enum DataScope {
    /// No branch filter; superuser / `branch.data.all`.
    Unrestricted,
    /// Branch IDs the user can access (may be empty → only `resource_grants` apply).
    Restricted(Arc<BTreeSet<String>>),
}

impl DataScope {
    pub fn from_auth_context(ctx: &AuthContext) -> Self {
        if ctx.superuser() || ctx.has(perm::BRANCH_DATA_ALL) {
            return DataScope::Unrestricted;
        }
        DataScope::Restricted(Arc::clone(ctx.accessible_branches()))
    }

    /// Whether `branch_id` may be used when creating/updating rows.
    pub fn allows_branch(&self, branch_id: &str) -> bool {
        match self {
            DataScope::Unrestricted => true,
            DataScope::Restricted(ids) => ids.contains(branch_id),
        }
    }
}

/// Append AND (...) for clients list/search queries.
pub fn push_client_scope_filter(
    qb: &mut QueryBuilder<'_, Postgres>,
    scope: &DataScope,
    actor_user_id: &str,
) {
    match scope.clone() {
        DataScope::Unrestricted => {}
        DataScope::Restricted(ids) => {
            qb.push(" AND (");
            if ids.is_empty() {
                qb.push("id IN (SELECT resource_id FROM resource_grants WHERE user_id = ")
                    .push_bind(actor_user_id.to_string())
                    .push(" AND resource_kind = 'client')");
            } else {
                qb.push("(branch_id IS NOT NULL AND branch_id IN (");
                {
                    let mut sep = qb.separated(", ");
                    for id in ids.iter().cloned() {
                        sep.push_bind(id);
                    }
                }
                qb.push(")) OR id IN (SELECT resource_id FROM resource_grants WHERE user_id = ")
                    .push_bind(actor_user_id.to_string())
                    .push(" AND resource_kind = 'client')");
            }
            qb.push(")");
        }
    }
}

pub fn push_task_scope_filter(
    qb: &mut QueryBuilder<'_, Postgres>,
    scope: &DataScope,
    actor_user_id: &str,
) {
    push_task_scope_filter_aliased(qb, scope, actor_user_id, None);
}

/// Like [`push_task_scope_filter`] but for queries that alias `tasks` (e.g. `t`).
pub fn push_task_scope_filter_aliased(
    qb: &mut QueryBuilder<'_, Postgres>,
    scope: &DataScope,
    actor_user_id: &str,
    table_alias: Option<&str>,
) {
    let pfx = table_alias.map(|a| format!("{a}.")).unwrap_or_default();
    match scope.clone() {
        DataScope::Unrestricted => {}
        DataScope::Restricted(ids) => {
            qb.push(" AND (");
            if ids.is_empty() {
                qb.push(&pfx)
                    .push("id IN (SELECT resource_id FROM resource_grants WHERE user_id = ")
                    .push_bind(actor_user_id.to_string())
                    .push(" AND resource_kind = 'task')");
            } else {
                qb.push("(")
                    .push(&pfx)
                    .push("branch_id IS NOT NULL AND ")
                    .push(&pfx)
                    .push("branch_id IN (");
                {
                    let mut sep = qb.separated(", ");
                    for id in ids.iter().cloned() {
                        sep.push_bind(id);
                    }
                }
                qb.push(")) OR ")
                    .push(&pfx)
                    .push("id IN (SELECT resource_id FROM resource_grants WHERE user_id = ")
                    .push_bind(actor_user_id.to_string())
                    .push(" AND resource_kind = 'task')");
            }
            qb.push(")");
        }
    }
}

async fn has_resource_grant(
    pool: &PgPool,
    user_id: &str,
    kind: &str,
    resource_id: &str,
) -> Result<bool, sqlx::Error> {
    let n: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM resource_grants
        WHERE user_id = $1 AND resource_kind = $2 AND resource_id = $3
        "#,
    )
    .bind(user_id)
    .bind(kind)
    .bind(resource_id)
    .fetch_one(pool)
    .await?;
    Ok(n > 0)
}

fn branch_matches(scope: &DataScope, branch_id: Option<&Uuid>) -> bool {
    match scope {
        DataScope::Unrestricted => true,
        DataScope::Restricted(ids) => branch_id
            .map(|b| ids.contains(&b.to_string()))
            .unwrap_or(false),
    }
}

/// Single-row read: hide existence if out of scope (caller maps to 404).
pub async fn can_read_client(
    pool: &PgPool,
    actor_user_id: &str,
    scope: &DataScope,
    client: &Client,
) -> Result<bool, sqlx::Error> {
    match scope {
        DataScope::Unrestricted => Ok(true),
        DataScope::Restricted(ids) => {
            if branch_matches(scope, client.branch_id.as_ref()) {
                return Ok(true);
            }
            if ids.is_empty() {
                return has_resource_grant(pool, actor_user_id, "client", &client.id.to_string())
                    .await;
            }
            has_resource_grant(pool, actor_user_id, "client", &client.id.to_string()).await
        }
    }
}

pub async fn can_read_task(
    pool: &PgPool,
    actor_user_id: &str,
    scope: &DataScope,
    task: &Task,
) -> Result<bool, sqlx::Error> {
    match scope {
        DataScope::Unrestricted => Ok(true),
        DataScope::Restricted(ids) => {
            if branch_matches(scope, task.branch_id.as_ref()) {
                return Ok(true);
            }
            if ids.is_empty() {
                return has_resource_grant(pool, actor_user_id, "task", &task.id.to_string())
                    .await;
            }
            has_resource_grant(pool, actor_user_id, "task", &task.id.to_string()).await
        }
    }
}

/// Resolve branch for new client: explicit, else root (must still pass `allows_branch`).
pub fn resolve_client_branch_id(requested: Option<&String>) -> String {
    requested
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ROOT_BRANCH_ID.to_string())
}

/// Validate create/update target branch against scope.
pub fn ensure_branch_allowed(scope: &DataScope, branch_id: &str) -> Result<(), AppError> {
    if scope.allows_branch(branch_id) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "No access to assign data to this branch".to_string(),
        ))
    }
}

/// Parse UUID strings from JSON payloads; invalid UUIDs return validation error at command layer.
pub fn parse_branch_uuid(s: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(s.trim()).map_err(|_| {
        AppError::ValidationError("branch_id must be a valid UUID".to_string())
    })
}
