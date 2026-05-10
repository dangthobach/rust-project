use axum::{extract::State, response::Json, Extension};
use serde::Serialize;

use crate::{app_state::AppState, authz::AuthContext, error::AppResult};

// ── DB row ─────────────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct MenuRow {
    key: String,
    parent_key: Option<String>,
    label: String,
    path: Option<String>,
    icon: Option<String>,
    sort_order: i32,
}

// ── Response DTO ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MenuNode {
    pub key: String,
    pub label: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub children: Vec<MenuNode>,
}

// ── Handler ────────────────────────────────────────────────────────────────────

/// Returns the menu tree visible to the current user.
///
/// Logic:
///   - superuser (dev principal or system.superuser role) → all active menus
///   - regular user → menus where required_permission IS NULL
///                    OR the user holds that permission
///
/// GET /api/menus/my-menus
pub async fn get_my_menus(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<MenuNode>>> {
    let rows: Vec<MenuRow> = if ctx.superuser() {
        // Dev principal / system.superuser → return every active menu
        sqlx::query_as(
            r#"
            SELECT key, parent_key, label, path, icon, sort_order
            FROM menus
            WHERE is_active = true
            ORDER BY COALESCE(parent_key, key) ASC, sort_order ASC, key ASC
            "#,
        )
        .fetch_all(state.pool())
        .await?
    } else {
        // Normal user: show menus matching their effective permission set
        let perm_codes: Vec<String> = ctx.permission_codes().iter().cloned().collect();
        sqlx::query_as(
            r#"
            SELECT key, parent_key, label, path, icon, sort_order
            FROM menus
            WHERE is_active = true
              AND (required_permission IS NULL OR required_permission = ANY($1))
            ORDER BY COALESCE(parent_key, key) ASC, sort_order ASC, key ASC
            "#,
        )
        .bind(&perm_codes)
        .fetch_all(state.pool())
        .await?
    };

    Ok(Json(build_tree(&rows, None)))
}

// ── Tree builder ───────────────────────────────────────────────────────────────

fn build_tree(all: &[MenuRow], parent: Option<&str>) -> Vec<MenuNode> {
    let mut nodes: Vec<MenuNode> = all
        .iter()
        .filter(|r| r.parent_key.as_deref() == parent)
        .map(|r| MenuNode {
            key: r.key.clone(),
            label: r.label.clone(),
            path: r.path.clone(),
            icon: r.icon.clone(),
            sort_order: r.sort_order,
            children: build_tree(all, Some(&r.key)),
        })
        .collect();
    nodes.sort_by_key(|n| n.sort_order);
    nodes
}
