//! Cached loader for the `system_settings` table.
//!
//! Settings change rarely; the 5-minute TTL keeps DB load negligible while
//! ensuring changes propagate quickly after an admin update.
//! Call [`invalidate_settings_cache`] immediately after any write to the table.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use sqlx::PgPool;

const CACHE_TTL: Duration = Duration::from_secs(300);

// ── Domain model ─────────────────────────────────────────────────────────────

/// Runtime-configurable system defaults; all fields have safe compile-time fallbacks
/// so the application stays functional even before the table is populated.
#[derive(Debug, Clone)]
pub struct SystemSettings {
    /// Role slugs assigned to every new self-registered user.
    /// Kept non-empty: if the DB value is empty/malformed, the default is used.
    pub default_role_slugs: Vec<String>,

    /// Branch UUID assigned to every new user.
    /// Must be the string representation of a valid UUID.
    pub default_branch_id: String,

    /// Gate for the `/auth/register` public endpoint.
    pub registration_enabled: bool,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            default_role_slugs: vec!["user".to_string()],
            default_branch_id: "00000000-0000-0000-0000-0000000000b1".to_string(),
            registration_enabled: true,
        }
    }
}

// ── Cache ─────────────────────────────────────────────────────────────────────

struct CacheEntry {
    settings: Arc<SystemSettings>,
    loaded_at: Instant,
}

// System settings are global (not per-user), so we use `()` as the key.
static SETTINGS_CACHE: std::sync::LazyLock<DashMap<(), CacheEntry>> =
    std::sync::LazyLock::new(DashMap::new);

// ── Public API ────────────────────────────────────────────────────────────────

/// Return cached settings or fetch fresh from DB.
///
/// * DB errors propagate — callers must handle them.
/// * Missing or unrecognised keys are silently skipped; the field retains its
///   [`Default`] value, keeping the system operational.
pub async fn load_system_settings(pool: &PgPool) -> Result<Arc<SystemSettings>, sqlx::Error> {
    let now = Instant::now();
    if let Some(entry) = SETTINGS_CACHE.get(&()) {
        if now.duration_since(entry.loaded_at) < CACHE_TTL {
            return Ok(Arc::clone(&entry.settings));
        }
    }

    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT key, value FROM system_settings")
            .fetch_all(pool)
            .await?;

    let mut s = SystemSettings::default();

    for (key, value) in rows {
        match key.as_str() {
            "default_role_slugs" => {
                match serde_json::from_str::<Vec<String>>(&value) {
                    Ok(slugs) if !slugs.is_empty() => s.default_role_slugs = slugs,
                    Ok(_) => {
                        tracing::warn!("system_settings.default_role_slugs is an empty array; keeping default");
                    }
                    Err(e) => {
                        tracing::warn!("system_settings.default_role_slugs is not valid JSON: {e}; keeping default");
                    }
                }
            }
            "default_branch_id" => {
                let v = value.trim().to_string();
                if !v.is_empty() {
                    s.default_branch_id = v;
                } else {
                    tracing::warn!("system_settings.default_branch_id is blank; keeping default");
                }
            }
            "registration_enabled" => {
                s.registration_enabled = value.trim().eq_ignore_ascii_case("true");
            }
            _ => {} // forward-compatible: unknown keys are silently ignored
        }
    }

    let arc = Arc::new(s);
    SETTINGS_CACHE.insert(
        (),
        CacheEntry {
            settings: Arc::clone(&arc),
            loaded_at: now,
        },
    );
    Ok(arc)
}

/// Invalidate the settings cache.
/// Must be called immediately after any write to `system_settings`.
pub fn invalidate_settings_cache() {
    SETTINGS_CACHE.clear();
}
