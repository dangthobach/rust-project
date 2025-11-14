use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain Event trait - tất cả events phải implement
pub trait DomainEvent: Clone + Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// Event type name (for deserialization)
    fn event_type(&self) -> &'static str;

    /// Aggregate ID mà event này thuộc về
    fn aggregate_id(&self) -> Uuid;

    /// Thời điểm event xảy ra
    fn occurred_at(&self) -> DateTime<Utc>;

    /// Version của aggregate sau khi apply event này
    fn version(&self) -> i64;
}

/// Event Envelope - wrapper cho event với metadata
#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope<E: DomainEvent> {
    pub id: Uuid,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: E,
    pub version: i64,
    pub occurred_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

// Manual Deserialize implementation to avoid E: Deserialize bounds
impl<'de, E: DomainEvent> Deserialize<'de> for EventEnvelope<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EventEnvelopeHelper {
            id: Uuid,
            aggregate_id: Uuid,
            aggregate_type: String,
            event_type: String,
            event_data: serde_json::Value,
            version: i64,
            occurred_at: DateTime<Utc>,
            metadata: serde_json::Value,
        }

        let helper = EventEnvelopeHelper::deserialize(deserializer)?;
        let event_data = serde_json::from_value(helper.event_data)
            .map_err(serde::de::Error::custom)?;

        Ok(EventEnvelope {
            id: helper.id,
            aggregate_id: helper.aggregate_id,
            aggregate_type: helper.aggregate_type,
            event_type: helper.event_type,
            event_data,
            version: helper.version,
            occurred_at: helper.occurred_at,
            metadata: helper.metadata,
        })
    }
}

impl<E: DomainEvent> EventEnvelope<E> {
    pub fn new(
        aggregate_id: Uuid,
        aggregate_type: String,
        event: E,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            aggregate_type,
            event_type: event.event_type().to_string(),
            event_data: event,
            version: 0, // Will be set by event store
            occurred_at: Utc::now(),
            metadata,
        }
    }
}

/// Base event metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventMetadata {
    pub user_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
    pub causation_id: Option<Uuid>,
    pub additional_data: serde_json::Value,
}

impl EventMetadata {
    pub fn new(user_id: Option<Uuid>) -> Self {
        Self {
            user_id,
            correlation_id: Some(Uuid::new_v4()),
            causation_id: None,
            additional_data: serde_json::json!({}),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "user_id": self.user_id,
            "correlation_id": self.correlation_id,
            "causation_id": self.causation_id,
            "additional_data": self.additional_data,
        })
    }
}

