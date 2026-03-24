use async_trait::async_trait;

#[async_trait]
pub trait KafkaPublisher: Send + Sync {
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> anyhow::Result<()>;
    async fn health_check(&self) -> anyhow::Result<()>;
}

#[async_trait]
pub trait RabbitMqPublisher: Send + Sync {
    async fn publish(&self, exchange: &str, routing_key: &str, payload: &str) -> anyhow::Result<()>;
    async fn health_check(&self) -> anyhow::Result<()>;
}

pub struct NoopKafkaPublisher;
#[async_trait]
impl KafkaPublisher for NoopKafkaPublisher {
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> anyhow::Result<()> {
        tracing::debug!(topic, key, payload_len = payload.len(), "noop kafka publish");
        Ok(())
    }
    async fn health_check(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct NoopRabbitMqPublisher;
#[async_trait]
impl RabbitMqPublisher for NoopRabbitMqPublisher {
    async fn publish(&self, exchange: &str, routing_key: &str, payload: &str) -> anyhow::Result<()> {
        tracing::debug!(exchange, routing_key, payload_len = payload.len(), "noop rabbitmq publish");
        Ok(())
    }
    async fn health_check(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

pub struct KafkaPublisherAdapter {
    brokers: String,
}
impl KafkaPublisherAdapter {
    pub fn new(brokers: String) -> Self {
        Self { brokers }
    }
}
#[async_trait]
impl KafkaPublisher for KafkaPublisherAdapter {
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> anyhow::Result<()> {
        tracing::info!(topic, key, brokers = %self.brokers, payload_len = payload.len(), "kafka adapter publish");
        Ok(())
    }
    async fn health_check(&self) -> anyhow::Result<()> {
        if self.brokers.trim().is_empty() {
            anyhow::bail!("kafka brokers not configured");
        }
        Ok(())
    }
}

pub struct RabbitMqPublisherAdapter {
    url: String,
}
impl RabbitMqPublisherAdapter {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
#[async_trait]
impl RabbitMqPublisher for RabbitMqPublisherAdapter {
    async fn publish(&self, exchange: &str, routing_key: &str, payload: &str) -> anyhow::Result<()> {
        tracing::info!(exchange, routing_key, url = %self.url, payload_len = payload.len(), "rabbitmq adapter publish");
        Ok(())
    }
    async fn health_check(&self) -> anyhow::Result<()> {
        if self.url.trim().is_empty() {
            anyhow::bail!("rabbitmq url not configured");
        }
        Ok(())
    }
}
