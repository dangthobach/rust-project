//! Back-compat module: `utils::pagination` re-exports the canonical pagination types.
//!
//! Canonical types live in `core::shared::pagination`. This keeps older modules
//! compiling while enforcing a single on-the-wire response shape.

pub use crate::core::shared::pagination::{PaginatedResponse, Pagination, PaginationParams};
