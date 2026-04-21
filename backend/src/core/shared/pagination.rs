use serde::{Deserialize, Serialize};
use crate::error::AppError;

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub page_size: i64,
}

impl Pagination {
    pub fn new(page: i64, page_size: i64) -> Self {
        Self {
            page: page.max(1),
            page_size: page_size.max(1).min(100), // Max 100 items per page
        }
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.page_size
    }

    pub fn limit(&self) -> i64 {
        self.page_size
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// Query parameters (page + limit) used by most list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

impl PaginationParams {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(AppError::ValidationError("page must be >= 1".to_string()));
        }
        if self.limit < 1 {
            return Err(AppError::ValidationError("limit must be >= 1".to_string()));
        }
        if self.limit > 100 {
            return Err(AppError::ValidationError("limit must be <= 100".to_string()));
        }
        Ok(())
    }

    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }

    pub fn pagination(&self) -> Pagination {
        Pagination::new(self.page, self.limit)
    }
}

/// Query parameters for paged + searchable lists (admin UIs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedSearchParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
    pub search: Option<String>,
}

impl PagedSearchParams {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(AppError::ValidationError("page must be >= 1".to_string()));
        }
        if self.limit < 1 {
            return Err(AppError::ValidationError("limit must be >= 1".to_string()));
        }
        if self.limit > 100 {
            return Err(AppError::ValidationError("limit must be <= 100".to_string()));
        }
        Ok(())
    }

    pub fn pagination(&self) -> Pagination {
        Pagination::new(self.page, self.limit)
    }

    pub fn search_trimmed(&self) -> String {
        self.search.as_deref().unwrap_or("").trim().to_string()
    }
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: Pagination) -> Self {
        let total_pages = (total as f64 / pagination.page_size as f64).ceil() as i64;
        Self {
            items,
            total,
            page: pagination.page,
            page_size: pagination.page_size,
            total_pages,
        }
    }
}

