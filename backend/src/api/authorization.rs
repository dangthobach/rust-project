//! Aggregated RBAC + branch data for UI (menus, buttons) and client-side hints.

use axum::{extract::State, response::Json, Extension};
use serde::Serialize;
use sqlx::FromRow;

use crate::app_state::AppState;
use crate::authz::AuthContext;
use crate::error::AppResult;

#[derive(Debug, Serialize, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BranchRow {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct MeAuthorizationResponse {
    pub permissions: Vec<String>,
    pub roles: Vec<RoleRow>,
    pub branches: Vec<BranchRow>,
}

pub async fn me_authorization(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<MeAuthorizationResponse>> {
    let pool = state.pool();

    let permissions: Vec<String> = ctx.permission_codes().iter().cloned().collect();

    let roles = sqlx::query_as::<_, RoleRow>(
        r#"
        SELECT r.id, r.slug, r.description
        FROM roles r
        INNER JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1 AND r.is_active = 1
        ORDER BY r.slug ASC
        "#,
    )
    .bind(&actor_id)
    .fetch_all(pool)
    .await?;

    let branches = sqlx::query_as::<_, BranchRow>(
        r#"
        SELECT b.id, b.parent_id, b.name, b.slug
        FROM branches b
        INNER JOIN user_branches ub ON ub.branch_id = b.id
        WHERE ub.user_id = $1 AND b.is_active = 1
        ORDER BY b.name ASC
        "#,
    )
    .bind(&actor_id)
    .fetch_all(pool)
    .await?;

    Ok(Json(MeAuthorizationResponse {
        permissions,
        roles,
        branches,
    }))
}
