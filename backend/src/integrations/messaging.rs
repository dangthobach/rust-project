use async_trait::async_trait;
use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
#[cfg(not(windows))]
use rdkafka::producer::{FutureProducer, FutureRecord};
#[cfg(not(windows))]
use rdkafka::ClientConfig;
#[cfg(not(windows))]
use tokio::time::Duration;

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
    #[cfg(not(windows))]
    producer: FutureProducer,
}
impl KafkaPublisherAdapter {
    pub fn new(brokers: String) -> anyhow::Result<Self> {
        #[cfg(windows)]
        {
            // rdkafka-sys build requires unix-like toolchain on Windows runners.
            // Keep API compatible and fallback to log-only mode.
            Ok(Self { brokers })
        }
        #[cfg(not(windows))]
        {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| anyhow::anyhow!("kafka producer create error: {e}"))?;
        Ok(Self { brokers, producer })
        }
    }
}
#[async_trait]
impl KafkaPublisher for KafkaPublisherAdapter {
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> anyhow::Result<()> {
        #[cfg(not(windows))]
        {
        self.producer
            .send(
                FutureRecord::to(topic).key(key).payload(payload),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| anyhow::anyhow!("kafka publish error: {e}"))?;
        }
        #[cfg(windows)]
        {
            tracing::warn!(
                topic,
                key,
                brokers = %self.brokers,
                payload_len = payload.len(),
                "kafka sdk fallback mode on windows"
            );
        }
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
        let conn = Connection::connect(&self.url, ConnectionProperties::default()).await?;
        let ch = conn.create_channel().await?;
        ch.exchange_declare(
            exchange.into(),
            ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
        ch.basic_publish(
            exchange.into(),
            routing_key.into(),
            BasicPublishOptions::default(),
            payload.as_bytes(),
            BasicProperties::default().with_content_type("application/json".into()),
        )
        .await?
        .await?;
        Ok(())
    }
    async fn health_check(&self) -> anyhow::Result<()> {
        if self.url.trim().is_empty() {
            anyhow::bail!("rabbitmq url not configured");
        }
        Ok(())
    }
}
