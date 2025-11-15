use bcrypt::{hash as bcrypt_hash, verify as bcrypt_verify, DEFAULT_COST};
use crate::error::AppError;

pub fn hash(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt_hash(password, DEFAULT_COST)
}

pub fn verify(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt_verify(password, hash)
}

// Aliases for better naming
pub fn hash_password(password: &str) -> Result<String, AppError> {
    hash(password).map_err(|e| AppError::InternalServerError(format!("Password hashing failed: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    verify(password, hash).map_err(|e| AppError::InternalServerError(format!("Password verification failed: {}", e)))
}
