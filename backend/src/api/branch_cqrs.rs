//! Branch tree + user↔branch assignment + optional resource grants (fine-grained).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app_state::AppState;
use crate::authz::permissions as perm;
use crate::authz::{invalidate_branch_cache, AuthContext};
use crate::core::shared::pagination::{PagedSearchParams, PaginatedResponse};
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BranchDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBranchPayload {
    pub parent_id: Option<String>,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct GrantResourcePayload {
    pub user_id: String,
    pub resource_kind: String,
    pub resource_id: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BranchWithAssignmentDto {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub assigned: bool,
}

fn require_branch_manage(ctx: &AuthContext) -> Result<(), AppError> {
    if ctx.superuser() || ctx.has(perm::BRANCH_MANAGE) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "branch.manage permission required".to_string(),
        ))
    }
}

fn require_grant_manage(ctx: &AuthContext) -> Result<(), AppError> {
    if ctx.superuser() || ctx.has(perm::RESOURCE_GRANT_MANAGE) {
        Ok(())
    } else {
        Err(AppError::Forbidden(
            "resource.grant.manage permission required".to_string(),
        ))
    }
}

pub async fn list_branches(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<BranchDto>>> {
    require_branch_manage(&ctx)?;
    let rows = sqlx::query_as::<_, BranchDto>(
        "SELECT * FROM branches WHERE is_active = TRUE ORDER BY name ASC",
    )
    .fetch_all(state.pool())
    .await?;
    Ok(Json(rows))
}

pub async fn create_branch(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<CreateBranchPayload>,
) -> AppResult<(StatusCode, Json<BranchDto>)> {
    require_branch_manage(&ctx)?;
    let slug = payload.slug.trim().to_ascii_lowercase();
    if slug.is_empty() {
        return Err(AppError::ValidationError("slug is required".to_string()));
    }
    if let Some(ref pid) = payload.parent_id {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM branches WHERE id = $1 AND is_active = TRUE")
            .bind(Uuid::parse_str(pid).map_err(|_| AppError::ValidationError("parent_id must be UUID".to_string()))?)
            .fetch_one(state.pool())
            .await?;
        if n == 0 {
            return Err(AppError::ValidationError("parent branch not found".to_string()));
        }
    }
    let id = Uuid::new_v4();
    let parent_uuid = payload
        .parent_id
        .as_ref()
        .map(|p| Uuid::parse_str(p))
        .transpose()
        .map_err(|_| AppError::ValidationError("parent_id must be UUID".to_string()))?;
    let row = sqlx::query_as::<_, BranchDto>(
        r#"
        INSERT INTO branches (id, parent_id, name, slug, is_active)
        VALUES ($1, $2, $3, $4, TRUE)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(parent_uuid)
    .bind(&payload.name)
    .bind(&slug)
    .fetch_one(state.pool())
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AppError::Conflict("slug already exists".to_string())
        } else {
            AppError::Database(e)
        }
    })?;
    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn assign_user_branch(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((user_id, branch_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    require_branch_manage(&ctx)?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?;
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM branches WHERE id = $1 AND is_active = TRUE")
        .bind(Uuid::parse_str(&branch_id).map_err(|_| AppError::ValidationError("branch_id must be UUID".to_string()))?)
        .fetch_one(state.pool())
        .await?;
    if n == 0 {
        return Err(AppError::ValidationError("branch not found".to_string()));
    }
    sqlx::query(
        "INSERT INTO user_branches (user_id, branch_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(user_uuid)
    .bind(Uuid::parse_str(&branch_id).map_err(|_| AppError::ValidationError("branch_id must be UUID".to_string()))?)
    .execute(state.pool())
    .await?;
    invalidate_branch_cache(user_uuid);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_user_branch(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((user_id, branch_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    require_branch_manage(&ctx)?;
    let user_uuid = Uuid::parse_str(&user_id)
        .map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?;
    sqlx::query("DELETE FROM user_branches WHERE user_id = $1 AND branch_id = $2")
        .bind(user_uuid)
        .bind(Uuid::parse_str(&branch_id).map_err(|_| AppError::ValidationError("branch_id must be UUID".to_string()))?)
        .execute(state.pool())
        .await?;
    invalidate_branch_cache(user_uuid);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn grant_resource_access(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(payload): Json<GrantResourcePayload>,
) -> AppResult<StatusCode> {
    require_grant_manage(&ctx)?;

    let kind = payload.resource_kind.trim().to_ascii_lowercase();
    if kind.is_empty() {
        return Err(AppError::ValidationError("resource_kind must not be empty".to_string()));
    }
    let user_uuid = Uuid::parse_str(&payload.user_id)
        .map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?;
    let resource_uuid = Uuid::parse_str(&payload.resource_id)
        .map_err(|_| AppError::ValidationError("resource_id must be UUID".to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO resource_grants (user_id, resource_kind, resource_id)
        VALUES ($1, $2, $3)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_uuid)
    .bind(&kind)
    .bind(resource_uuid)
    .execute(state.pool())
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn revoke_resource_access(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path((user_id, kind, resource_id)): Path<(String, String, String)>,
) -> AppResult<StatusCode> {
    require_grant_manage(&ctx)?;
    let kind = kind.trim().to_ascii_lowercase();
    if kind.is_empty() {
        return Err(AppError::ValidationError("resource_kind must not be empty".to_string()));
    }
    sqlx::query(
        "DELETE FROM resource_grants WHERE user_id = $1 AND resource_kind = $2 AND resource_id = $3",
    )
    .bind(Uuid::parse_str(&user_id).map_err(|_| AppError::ValidationError("user_id must be UUID".to_string()))?)
    .bind(&kind)
    .bind(Uuid::parse_str(&resource_id).map_err(|_| AppError::ValidationError("resource_id must be UUID".to_string()))?)
    .execute(state.pool())
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// List branches + assignment status for a given user (admin UI).
/// GET /api/admin/rbac/users/:user_id/branches?page=&limit=&search=
pub async fn list_branches_for_user(
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(params): Query<PagedSearchParams>,
) -> AppResult<Json<PaginatedResponse<BranchWithAssignmentDto>>> {
    require_branch_manage(&ctx)?;
    params.validate()?;

    let pagination = params.pagination();
    let search = params.search_trimmed();

    let total: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM branches b
        WHERE b.is_active = TRUE
          AND ($1 = '' OR b.name ILIKE '%' || $1 || '%' OR b.slug ILIKE '%' || $1 || '%')
        "#,
    )
    .bind(&search)
    .fetch_one(state.pool())
    .await?;

    let items = sqlx::query_as::<_, BranchWithAssignmentDto>(
        r#"
        SELECT
            b.id,
            b.parent_id,
            b.name,
            b.slug,
            EXISTS(
                SELECT 1 FROM user_branches ub
                WHERE ub.user_id = $1::uuid AND ub.branch_id = b.id
            ) AS assigned
        FROM branches b
        WHERE b.is_active = TRUE
          AND ($2 = '' OR b.name ILIKE '%' || $2 || '%' OR b.slug ILIKE '%' || $2 || '%')
        ORDER BY assigned DESC, b.name ASC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&user_id)
    .bind(&search)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(state.pool())
    .await?;

    Ok(Json(PaginatedResponse::new(items, total, pagination)))
}
