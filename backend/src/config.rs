use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub host: String,
    pub port: u16,
    pub cors_origin: String,
    pub max_file_size: usize,
    pub upload_dir: String,
    pub redis_url: String,
    pub kafka_brokers: String,
    pub rabbitmq_url: String,
    pub object_storage_provider: String,
    pub object_storage_endpoint: String,
    pub object_storage_bucket: String,
    pub object_storage_access_key: String,
    pub object_storage_secret_key: String,
    pub default_tenant_id: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:./data/crm.db".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiration: std::env::var("JWT_EXPIRATION")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()?,
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            cors_origin: std::env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            max_file_size: std::env::var("MAX_FILE_SIZE")
                .unwrap_or_else(|_| "10485760".to_string())
                .parse()?,
            upload_dir: std::env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            kafka_brokers: std::env::var("KAFKA_BROKERS")
                .unwrap_or_else(|_| "".to_string()),
            rabbitmq_url: std::env::var("RABBITMQ_URL")
                .unwrap_or_else(|_| "".to_string()),
            object_storage_provider: std::env::var("OBJECT_STORAGE_PROVIDER")
                .unwrap_or_else(|_| "local".to_string()),
            object_storage_endpoint: std::env::var("OBJECT_STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "".to_string()),
            object_storage_bucket: std::env::var("OBJECT_STORAGE_BUCKET")
                .unwrap_or_else(|_| "crm-objects".to_string()),
            object_storage_access_key: std::env::var("OBJECT_STORAGE_ACCESS_KEY")
                .unwrap_or_else(|_| "".to_string()),
            object_storage_secret_key: std::env::var("OBJECT_STORAGE_SECRET_KEY")
                .unwrap_or_else(|_| "".to_string()),
            default_tenant_id: std::env::var("DEFAULT_TENANT_ID")
                .unwrap_or_else(|_| "public".to_string()),
        };

        Ok(config)
    }
}
