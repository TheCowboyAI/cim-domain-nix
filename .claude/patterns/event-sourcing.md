# Event Sourcing Patterns

## Event Persistence

Persistence is optional - just implement the `Persistable` trait where needed:

```rust
pub trait Persistable {
    fn persist(&self, store: &dyn EventStore) -> Result<Cid>;
}

// Only business-critical events implement Persistable
impl Persistable for OrderPlaced { /* ... */ }
impl Persistable for PaymentProcessed { /* ... */ }

// UI/technical events don't implement it
struct CacheInvalidated { /* ... */ } // No persistence
```

## Event Correlation and Causation Requirements

### MANDATORY: All Events Must Include Correlation/Causation IDs

Every event, command, and query in the system MUST include correlation and causation IDs:

```rust
pub struct MessageIdentity {
    pub message_id: MessageId,        // Unique for this message
    pub correlation_id: CorrelationId, // Groups related messages (REQUIRED)
    pub causation_id: CausationId,    // What caused this message (REQUIRED)
}

// Timestamp is separate metadata, NOT part of the correlation algebra
pub struct EventMetadata {
    pub identity: MessageIdentity,
    pub timestamp: SystemTime,
    pub actor: Option<ActorId>,
}
```

### Correlation/Causation Rules:
1. **Root Messages**: `MessageId = CorrelationId = CausationId` (self-correlation)
2. **Caused Messages**: Inherit `CorrelationId` from parent, `CausationId = parent.MessageId`
3. **All NATS Messages**: Must include correlation headers (X-Correlation-ID, X-Causation-ID)
4. **Event Streams**: Must validate correlation chains and detect cycles

### Implementation Pattern:
```rust
// ALWAYS use MessageFactory for creating messages
let root_cmd = MessageFactory::create_root(CreateOrder { ... });
let caused_event = MessageFactory::create_caused_by(OrderCreated { ... }, &root_cmd);

// NEVER create messages directly
let bad = Event { correlation_id: Uuid::new_v4(), ... }; // ❌ WRONG
```

## Event Design Principles

### 1. Event Structure with CID Chains

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    // Identity
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub sequence: u64,

    // CID Chain for integrity
    pub event_cid: Cid,
    pub previous_cid: Option<Cid>,

    // Payload
    pub event_type: String,
    pub payload: serde_json::Value,

    // Metadata
    pub timestamp: SystemTime,
    pub correlation_id: CorrelationId,  // REQUIRED, not optional
    pub causation_id: CausationId,      // REQUIRED, not optional
    pub actor: Option<ActorId>,
}
```

### 2. Event Naming Conventions

Follow past-tense naming that reflects business outcomes:

✅ **Good Event Names**
- `NodeAdded`
- `EdgeConnected`
- `GraphPublished`
- `ConceptSpaceCalculated`

❌ **Bad Event Names**
- `AddNode` (command, not event)
- `NodeData` (not descriptive)
- `UpdateGraph` (too generic)

### 3. Event Payload Design

```rust
// Events should be self-contained and immutable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAdded {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub initial_position: Position3D,
    pub components: Vec<ComponentData>, // Snapshot, not references
    pub metadata: HashMap<String, Value>,
}

// For large data, use CID references
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeDatasetProcessed {
    pub dataset_id: DatasetId,
    pub result_cid: Cid, // Points to NATS Object Store
    pub summary: ProcessingSummary,
}
```

## Event Store Integration

### 1. NATS JetStream Configuration

```rust
// Stream per aggregate type
pub async fn create_event_stream(js: &jetstream::Context) -> Result<Stream> {
    js.create_stream(StreamConfig {
        name: "EVENTS".to_string(),
        subjects: vec![
            "events.graph.>".to_string(),
            "events.node.>".to_string(),
            "events.edge.>".to_string(),
        ],
        retention: RetentionPolicy::Limits,
        storage: StorageType::File,
        max_age: Duration::from_days(365),
        duplicate_window: Duration::from_secs(120),
        ..Default::default()
    }).await
}
```

### 2. Event Publishing Pattern

```rust
impl EventStore {
    pub async fn append_events(
        &self,
        aggregate_id: AggregateId,
        events: Vec<DomainEvent>,
        expected_version: Option<u64>,
    ) -> Result<()> {
        // Optimistic concurrency control
        if let Some(version) = expected_version {
            self.verify_version(aggregate_id, version).await?;
        }

        // Calculate CID chain
        let mut previous_cid = self.get_latest_cid(aggregate_id).await?;

        for event in events {
            // Calculate event CID
            let event_cid = calculate_cid(&event, previous_cid)?;

            // Publish to NATS
            let subject = format!("events.{}.{}",
                event.aggregate_type(),
                aggregate_id
            );

            self.jetstream
                .publish(subject, event.to_bytes()?)
                .await?;

            previous_cid = Some(event_cid);
        }

        Ok(())
    }
}
```

## Event Sourcing Patterns

### 1. Aggregate Loading

```rust
pub async fn load_aggregate<A: Aggregate>(
    event_store: &EventStore,
    aggregate_id: AggregateId,
) -> Result<A> {
    let events = event_store
        .get_events(aggregate_id)
        .await?;

    let mut aggregate = A::default();

    for event in events {
        // Verify CID chain
        verify_cid_chain(&event, &aggregate.last_cid)?;

        // Apply event
        aggregate.apply_event(event)?;
    }

    Ok(aggregate)
}
```

### 2. Snapshot Strategy

```rust
// Snapshot every N events or time period
pub struct SnapshotPolicy {
    pub event_threshold: u64,  // e.g., every 100 events
    pub time_threshold: Duration, // e.g., every hour
}

impl EventStore {
    pub async fn maybe_snapshot<A: Aggregate>(
        &self,
        aggregate: &A,
        policy: &SnapshotPolicy,
    ) -> Result<()> {
        if should_snapshot(aggregate, policy) {
            let snapshot = aggregate.to_snapshot()?;
            let snapshot_cid = self.object_store
                .put_object(snapshot)
                .await?;

            self.store_snapshot_reference(
                aggregate.id(),
                snapshot_cid,
                aggregate.version(),
            ).await?;
        }
        Ok(())
    }
}
```

### 3. Event Replay and Projection

```rust
pub trait Projection: Send + Sync {
    type Event;

    async fn handle_event(&mut self, event: Self::Event) -> Result<()>;

    async fn get_checkpoint(&self) -> Option<EventSequence>;

    async fn save_checkpoint(&mut self, sequence: EventSequence) -> Result<()>;
}

// Replay events to rebuild projections
pub async fn replay_projection<P: Projection>(
    event_store: &EventStore,
    projection: &mut P,
    from_sequence: Option<EventSequence>,
) -> Result<()> {
    let start = from_sequence
        .or(projection.get_checkpoint().await)
        .unwrap_or(EventSequence(0));

    let mut consumer = event_store
        .consumer_from_sequence(start)
        .await?;

    while let Some(event) = consumer.next().await? {
        projection.handle_event(event).await?;

        // Checkpoint periodically
        if event.sequence % 100 == 0 {
            projection.save_checkpoint(event.sequence).await?;
        }
    }

    Ok(())
}
```

## Best Practices

### 1. Event Versioning

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum NodeEvent {
    #[serde(rename = "1.0")]
    V1(NodeEventV1),

    #[serde(rename = "2.0")]
    V2(NodeEventV2),
}

// Handle multiple versions
impl From<NodeEvent> for DomainEvent {
    fn from(event: NodeEvent) -> Self {
        match event {
            NodeEvent::V1(e) => e.into(), // Convert old format
            NodeEvent::V2(e) => e.into(),
        }
    }
}
```

### 2. Idempotency

```rust
// Use event IDs for idempotency
impl EventStore {
    pub async fn append_event_idempotent(
        &self,
        event: DomainEvent,
    ) -> Result<()> {
        // NATS deduplication window handles this
        let headers = Headers::new()
            .add("Nats-Msg-Id", event.event_id.to_string());

        self.jetstream
            .publish_with_headers(subject, headers, payload)
            .await
    }
}
```

### 3. Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum EventSourcingError {
    #[error("CID chain broken at sequence {0}")]
    BrokenCidChain(u64),

    #[error("Concurrent modification: expected version {expected}, got {actual}")]
    ConcurrentModification { expected: u64, actual: u64 },

    #[error("Event replay failed: {0}")]
    ReplayError(String),
}
```

## Testing Event Sourcing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_cid_chain() {
        let mut store = InMemoryEventStore::new();

        let event1 = create_test_event(1);
        let event2 = create_test_event(2);

        store.append_events(vec![event1.clone()]).await.unwrap();
        store.append_events(vec![event2.clone()]).await.unwrap();

        let events = store.get_all_events().await.unwrap();

        assert_eq!(events[1].previous_cid, Some(events[0].event_cid));
        assert!(verify_cid_chain(&events).is_ok());
    }
}
```

## Common Pitfalls to Avoid

❌ **Mutable Events**
```rust
// WRONG - Events must be immutable
event.timestamp = SystemTime::now();
```

❌ **Business Logic in Event Handlers**
```rust
// WRONG - Apply should only update state
fn apply_event(&mut self, event: Event) {
    if self.validate_business_rule() { // NO!
        self.state = event.new_state;
    }
}
```

❌ **Storing Entity References**
```rust
// WRONG - Store data, not references
struct NodeAdded {
    node: Arc<Node>, // Will break on replay!
}
```

✅ **Correct Patterns**
- Events are immutable records of what happened
- Business logic in command handlers only
- Events contain data snapshots, not references
- CID chains ensure integrity
- Projections handle read model updates