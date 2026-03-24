use serde::{Deserialize, Serialize};
use crate::error::AppError;

/// Query parameters for pagination
#[derive(Debug, Deserialize, Clone)]
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
    /// Calculate SQL OFFSET from page and limit
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }

    /// Validate pagination parameters
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page < 1 {
            return Err(AppError::ValidationError(
                "Page number must be >= 1".to_string(),
            ));
        }
        if self.limit < 1 {
            return Err(AppError::ValidationError(
                "Limit must be >= 1".to_string(),
            ));
        }
        if self.limit > 100 {
            return Err(AppError::ValidationError(
                "Limit cannot exceed 100".to_string(),
            ));
        }
        Ok(())
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, page: i64, limit: i64, total: i64) -> Self {
        let total_pages = if total > 0 {
            (total as f64 / limit as f64).ceil() as i64
        } else {
            0
        };

        Self {
            data,
            pagination: PaginationMeta {
                page,
                limit,
                total,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let params = PaginationParams {
            page: default_page(),
            limit: default_limit(),
        };
        assert_eq!(params.page, 1);
        assert_eq!(params.limit, 20);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_offset_calculation() {
        let params = PaginationParams { page: 1, limit: 20 };
        assert_eq!(params.offset(), 0);

        let params = PaginationParams { page: 2, limit: 20 };
        assert_eq!(params.offset(), 20);

        let params = PaginationParams { page: 3, limit: 10 };
        assert_eq!(params.offset(), 20);
    }

    #[test]
    fn test_validate_valid_params() {
        let params = PaginationParams { page: 1, limit: 20 };
        assert!(params.validate().is_ok());

        let params = PaginationParams { page: 5, limit: 50 };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_page() {
        let params = PaginationParams { page: 0, limit: 20 };
        assert!(params.validate().is_err());

        let params = PaginationParams { page: -1, limit: 20 };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_limit() {
        let params = PaginationParams { page: 1, limit: 0 };
        assert!(params.validate().is_err());

        let params = PaginationParams { page: 1, limit: 101 };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_paginated_response() {
        let data = vec![1, 2, 3, 4, 5];
        let response = PaginatedResponse::new(data, 1, 5, 15);

        assert_eq!(response.pagination.page, 1);
        assert_eq!(response.pagination.limit, 5);
        assert_eq!(response.pagination.total, 15);
        assert_eq!(response.pagination.total_pages, 3);
        assert!(response.pagination.has_next);
        assert!(!response.pagination.has_prev);
    }

    #[test]
    fn test_pagination_meta_last_page() {
        let data: Vec<i32> = vec![];
        let response = PaginatedResponse::new(data, 3, 5, 15);

        assert!(!response.pagination.has_next);
        assert!(response.pagination.has_prev);
    }
}
