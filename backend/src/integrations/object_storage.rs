use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use urlencoding::encode;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put_object(&self, key: &str, bytes: &[u8], content_type: &str) -> anyhow::Result<String>;
    async fn presign_get_url(&self, object_uri: &str, expires_seconds: u64) -> anyhow::Result<Option<String>>;
    async fn health_check(&self) -> anyhow::Result<()>;
}

pub struct LocalObjectStorage {
    base_dir: String,
}
impl LocalObjectStorage {
    pub fn new(base_dir: String) -> Self {
        Self { base_dir }
    }
}

#[async_trait]
impl ObjectStorage for LocalObjectStorage {
    async fn put_object(&self, key: &str, bytes: &[u8], _content_type: &str) -> anyhow::Result<String> {
        let path = format!("{}/{}", self.base_dir, key);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut f = fs::File::create(&path).await?;
        f.write_all(bytes).await?;
        Ok(path)
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        fs::create_dir_all(&self.base_dir).await?;
        Ok(())
    }

    async fn presign_get_url(&self, _object_uri: &str, _expires_seconds: u64) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}

pub struct RustfsObjectStorage {
    endpoint: String,
    bucket: String,
    access_key: String,
    secret_key: String,
}
impl RustfsObjectStorage {
    pub fn new(endpoint: String, bucket: String, access_key: String, secret_key: String) -> Self {
        Self {
            endpoint,
            bucket,
            access_key,
            secret_key,
        }
    }
}

#[async_trait]
impl ObjectStorage for RustfsObjectStorage {
    async fn put_object(&self, key: &str, bytes: &[u8], content_type: &str) -> anyhow::Result<String> {
        tracing::info!(
            endpoint = %self.endpoint,
            bucket = %self.bucket,
            key,
            content_type,
            size = bytes.len(),
            "rustfs put object (sdk adapter)"
        );
        // Placeholder for Rustfs SDK upload call.
        Ok(format!("rustfs://{}/{}", self.bucket, key))
    }

    async fn presign_get_url(&self, object_uri: &str, expires_seconds: u64) -> anyhow::Result<Option<String>> {
        let prefix = format!("rustfs://{}/", self.bucket);
        let object_key = object_uri
            .strip_prefix(&prefix)
            .ok_or_else(|| anyhow::anyhow!("invalid rustfs object uri"))?;
        let exp = chrono::Utc::now().timestamp() + i64::try_from(expires_seconds).unwrap_or(900);
        let signing = format!("GET\n{}\n{}\n{}", self.bucket, object_key, exp);
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("hmac init error: {e}"))?;
        mac.update(signing.as_bytes());
        let sig = hex::encode(mac.finalize().into_bytes());

        let url = format!(
            "{}/{}/{}?access_key={}&expires={}&signature={}",
            self.endpoint.trim_end_matches('/'),
            self.bucket,
            encode(object_key),
            encode(&self.access_key),
            exp,
            sig
        );
        Ok(Some(url))
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        if self.endpoint.trim().is_empty()
            || self.bucket.trim().is_empty()
            || self.access_key.trim().is_empty()
            || self.secret_key.trim().is_empty()
        {
            anyhow::bail!("object storage endpoint/bucket not configured");
        }
        Ok(())
    }
}
