use crate::core::domain::Entity;
use crate::core::events::DomainEvent;

/// Aggregate Root - entity chính trong domain
/// Mỗi aggregate có version để optimistic locking
pub trait AggregateRoot: Entity {
    type Event: DomainEvent;

    /// Version cho optimistic locking trong Event Sourcing
    fn version(&self) -> i64;

    /// Lấy uncommitted events (events chưa được persist)
    fn uncommitted_events(&self) -> &[Self::Event];

    /// Clear events sau khi đã persist vào Event Store
    fn mark_events_as_committed(&mut self);

    /// Apply event để rebuild state (event sourcing)
    fn apply(&mut self, event: &Self::Event);

    /// Tạo event mới và apply vào aggregate
    fn raise_event(&mut self, event: Self::Event) {
        self.apply(&event);
        // Store trong uncommitted events
        // Note: Cần implement trong concrete aggregate
    }
}

/// Trait để check xem có nên tạo snapshot không
pub trait Snapshottable: AggregateRoot {
    /// Kiểm tra xem có nên tạo snapshot không
    /// Thường là mỗi N events (ví dụ: mỗi 10 events)
    fn should_take_snapshot(&self) -> bool {
        self.version() % 10 == 0
    }

    /// Snapshot threshold - số events trước khi snapshot
    fn snapshot_threshold(&self) -> i64 {
        10
    }
}

