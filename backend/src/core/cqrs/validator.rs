use validator::Validate;

/// Validation result
pub type ValidationResult = Result<(), validator::ValidationErrors>;

/// Validate command/query
pub fn validate<T: Validate>(item: &T) -> ValidationResult {
    item.validate()
}

