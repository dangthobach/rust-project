use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::PgPool;
use uuid::Uuid;

/// Short TTL cache: avoids one DISTINCT JOIN per request under load; stale up to TTL after role changes.
const CACHE_TTL: Duration = Duration::from_secs(90);

struct CacheEntry {
    perms: Arc<BTreeSet<String>>,
    loaded_at: Instant,
}

static PERMISSION_CACHE: std::sync::LazyLock<DashMap<Uuid, CacheEntry>> =
    std::sync::LazyLock::new(DashMap::new);

/// Single round-trip: all roles × permissions resolved with DISTINCT.
pub async fn load_effective_permissions(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Arc<BTreeSet<String>>, sqlx::Error> {
    let now = Instant::now();
    if let Some(entry) = PERMISSION_CACHE.get(&user_id) {
        if now.duration_since(entry.loaded_at) < CACHE_TTL {
            return Ok(Arc::clone(&entry.perms));
        }
    }

    let rows = sqlx::query_scalar::<_, String>(
        r#"
        SELECT DISTINCT p.code
        FROM user_roles ur
        INNER JOIN roles r
            ON r.id = ur.role_id AND r.is_active = 1
        INNER JOIN role_permissions rp
            ON rp.role_id = r.id
        INNER JOIN permissions p
            ON p.code = rp.permission_code AND p.is_active = 1
        WHERE ur.user_id = $1
        "#,
    )
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    let set: BTreeSet<String> = rows.into_iter().collect();
    let arc = Arc::new(set);
    PERMISSION_CACHE.insert(
        user_id,
        CacheEntry {
            perms: Arc::clone(&arc),
            loaded_at: now,
        },
    );
    Ok(arc)
}

/// Call after mutating roles/permissions for `user_id` (admin APIs).
#[allow(dead_code)]
pub fn invalidate_permission_cache(user_id: Uuid) {
    PERMISSION_CACHE.remove(&user_id);
}

/// Use when role/permission definitions changed and impact many users.
pub fn invalidate_all_permission_cache() {
    PERMISSION_CACHE.clear();
}
