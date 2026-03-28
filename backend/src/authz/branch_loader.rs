//! Resolves effective branch IDs: user's direct assignments plus all descendant branches (tree).
//! Cached per user with short TTL to limit DB load under high concurrency.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::PgPool;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(90);

struct CacheEntry {
    ids: Arc<BTreeSet<String>>,
    loaded_at: Instant,
}

static BRANCH_CACHE: std::sync::LazyLock<DashMap<Uuid, CacheEntry>> =
    std::sync::LazyLock::new(DashMap::new);

/// All branch IDs reachable from `user_branches` (including nested children).
/// Single query; assumes no cycles in `branches` (enforced on write).
pub async fn load_accessible_branch_ids(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Arc<BTreeSet<String>>, sqlx::Error> {
    let now = Instant::now();
    if let Some(entry) = BRANCH_CACHE.get(&user_id) {
        if now.duration_since(entry.loaded_at) < CACHE_TTL {
            return Ok(Arc::clone(&entry.ids));
        }
    }

    let uid = user_id.to_string();
    let rows = sqlx::query_scalar::<_, String>(
        r#"
        WITH RECURSIVE seeds AS (
            SELECT b.id
            FROM branches b
            INNER JOIN user_branches ub
                ON ub.branch_id = b.id AND ub.user_id = $1
            WHERE b.is_active = 1
        ),
        subtree AS (
            SELECT id FROM seeds
            UNION ALL
            SELECT b.id
            FROM branches b
            INNER JOIN subtree s ON b.parent_id = s.id
            WHERE b.is_active = 1
        )
        SELECT DISTINCT id FROM subtree
        "#,
    )
    .bind(&uid)
    .fetch_all(pool)
    .await?;

    let set: BTreeSet<String> = rows.into_iter().collect();
    let arc = Arc::new(set);
    BRANCH_CACHE.insert(
        user_id,
        CacheEntry {
            ids: Arc::clone(&arc),
            loaded_at: now,
        },
    );
    Ok(arc)
}

pub fn invalidate_branch_cache(user_id: Uuid) {
    BRANCH_CACHE.remove(&user_id);
}

pub fn invalidate_all_branch_cache() {
    BRANCH_CACHE.clear();
}
