use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id as String (UUID text)
    pub exp: i64,
    pub iat: i64,
}

fn get_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key_change_in_production".to_string())
}

pub fn generate_token(user_id: &str) -> AppResult<String> {
    let secret = get_secret();
    let now = Utc::now();
    let expiration = now + Duration::days(1);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("Failed to generate token: {}", e)))
}

pub fn verify_token(token: &str) -> AppResult<String> {
    let secret = get_secret();

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    Ok(token_data.claims.sub)
}
