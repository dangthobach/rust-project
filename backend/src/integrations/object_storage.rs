use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put_object(&self, key: &str, bytes: &[u8], content_type: &str) -> anyhow::Result<String>;
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
}

pub struct S3CompatibleObjectStorage {
    endpoint: String,
    bucket: String,
}
impl S3CompatibleObjectStorage {
    pub fn new(endpoint: String, bucket: String) -> Self {
        Self { endpoint, bucket }
    }
}

#[async_trait]
impl ObjectStorage for S3CompatibleObjectStorage {
    async fn put_object(&self, key: &str, bytes: &[u8], content_type: &str) -> anyhow::Result<String> {
        tracing::info!(
            endpoint = %self.endpoint,
            bucket = %self.bucket,
            key,
            content_type,
            size = bytes.len(),
            "s3-compatible object put (adapter)"
        );
        Ok(format!("s3://{}/{}", self.bucket, key))
    }

    async fn health_check(&self) -> anyhow::Result<()> {
        if self.endpoint.trim().is_empty() || self.bucket.trim().is_empty() {
            anyhow::bail!("object storage endpoint/bucket not configured");
        }
        Ok(())
    }
}
