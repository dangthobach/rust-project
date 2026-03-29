//! Unified global search (tasks + clients) for header / command palette UX.
use axum::extract::{Extension, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use sqlx::{QueryBuilder, Postgres};
use std::sync::Arc;

use crate::app_state::AppState;
use crate::authz::data_scope::DataScope;
use crate::authz::AuthContext;
use crate::domains::clients::{SearchClientsHandler, SearchClientsQuery};
use crate::error::{AppError, AppResult};
use crate::models::Client;

#[derive(Debug, Deserialize)]
pub struct UnifiedSearchParams {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    8
}

#[derive(Debug, Serialize)]
pub struct UnifiedSearchResponse {
    pub tasks: Vec<TaskSearchHit>,
    pub clients: Vec<ClientSearchHit>,
}

#[derive(Debug, Serialize)]
pub struct TaskSearchHit {
    pub id: String,
    pub title: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ClientSearchHit {
    pub id: String,
    pub name: String,
    pub company: Option<String>,
}

pub async fn unified_search(
    Extension(actor_id): Extension<String>,
    Extension(ctx): Extension<AuthContext>,
    State(state): State<AppState>,
    Query(params): Query<UnifiedSearchParams>,
) -> AppResult<Json<UnifiedSearchResponse>> {
    let q = params.q.trim().to_string();
    if q.len() < 2 {
        return Err(AppError::ValidationError(
            "Search query must be at least 2 characters".to_string(),
        ));
    }

    let limit = params.limit.clamp(1, 25);
    let scope = DataScope::from_auth_context(&ctx);

    let cq = SearchClientsQuery {
        search_term: q.clone(),
        limit: Some(limit),
        data_scope: scope.clone(),
        actor_user_id: actor_id.clone(),
    };
    let ch = Arc::new(SearchClientsHandler::new(state.pool.clone()));
    let clients: Vec<Client> = state
        .query_bus
        .dispatch_with_handler(cq, ch)
        .await?;

    let pool = state.pool();
    let mut qb = QueryBuilder::<Postgres>::new(
        "SELECT t.id, t.title, t.status FROM tasks t WHERE t.search_vector @@ plainto_tsquery('simple', ",
    );
    qb.push_bind(&q);
    qb.push(")");
    crate::authz::data_scope::push_task_scope_filter_aliased(
        &mut qb,
        &scope,
        &actor_id,
        Some("t"),
    );
    qb.push(
        " ORDER BY ts_rank_cd(t.search_vector, plainto_tsquery('simple', ",
    )
    .push_bind(&q)
    .push(")) DESC NULLS LAST, t.created_at DESC LIMIT ")
    .push_bind(limit);

    #[derive(sqlx::FromRow)]
    struct TaskRow {
        id: uuid::Uuid,
        title: String,
        status: String,
    }

    let task_rows: Vec<TaskRow> = qb.build_query_as().fetch_all(pool).await?;

    let tasks = task_rows
        .into_iter()
        .map(|t| TaskSearchHit {
            id: t.id.to_string(),
            title: t.title,
            status: t.status,
        })
        .collect();

    let clients = clients
        .into_iter()
        .map(|c| ClientSearchHit {
            id: c.id.to_string(),
            name: c.name,
            company: c.company,
        })
        .collect();

    Ok(Json(UnifiedSearchResponse { tasks, clients }))
}
