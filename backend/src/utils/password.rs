use bcrypt::{hash as bcrypt_hash, verify as bcrypt_verify, DEFAULT_COST};

pub fn hash(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt_hash(password, DEFAULT_COST)
}

pub fn verify(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt_verify(password, hash)
}
