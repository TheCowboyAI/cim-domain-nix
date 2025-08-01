# Graph Patterns in CIM

## Core Architecture Principles

- **NATS JetStream Event Store**: Primary source of truth with CID-chained events
- **CQRS Pattern**: Separated write model (commands) and read model (queries)
- **Petgraph Storage**: Lightweight graph structure with component deduplication
- **Bevy ECS Visualization**: Real-time rendering through async/sync bridge
- **Decoupled Layers**:
  ```rust
  struct GraphSystem {
      event_store: EventStore,        // NATS JetStream persistence
      write_model: GraphAggregate,    // Command processing
      read_model: GraphReadModel,     // Query optimization
      bevy_bridge: AsyncSyncBridge,   // Real-time updates
  }
  ```

## Core Purpose

Graphs in CIM serve as the primary visualization and interaction layer for:
- **Domain-Driven Workflows**: Visual representation of business processes
- **Conceptual Spaces**: Geometric knowledge representation
- **Event Flows**: Visualization of event-sourced state changes
- **Dog-fooding**: Self-visualization of CIM's own development

## EventStore with CID Chains

### Event Structure
```rust
#[derive(Serialize, Deserialize)]
pub struct DomainEvent {
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub sequence: u64,
    pub timestamp: SystemTime,
    pub event_cid: Option<Cid>,      // Content-addressed identifier
    pub previous_cid: Option<Cid>,   // Chain to previous event
    pub metadata: EventMetadata,
}

// CID calculation ensures integrity
pub fn calculate_event_cid(
    payload: &[u8],
    previous_cid: Option<Cid>,
    aggregate_id: &AggregateId,
    event_type: &str,
    timestamp: SystemTime,
) -> Result<Cid, EventStoreError> {
    // Uses IPLD dag-cbor format
}
```

### EventStream Transactions
```rust
pub struct EventStreamTransaction {
    pub transaction_id: TransactionId,
    pub sequence_range: SequenceRange,
    pub aggregate_id: AggregateId,
    pub events: Vec<DomainEvent>,
    pub metadata: TransactionMetadata,
}

// Atomic event processing
let transaction = event_service.fetch_transaction(
    aggregate_id,
    TransactionOptions {
        replay_policy: ReplayPolicy::FromBeginning,
        max_events: Some(1000),
    },
).await?;
```

## Domain Model

### Graph Aggregate
```rust
#[derive(Debug, Clone)]
pub struct GraphAggregate {
    pub id: GraphId,
    pub name: String,
    pub graph_type: GraphType,
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
    pub conceptual_mapping: ConceptualMapping,
    pub version: u64,
    pub last_event_cid: Option<Cid>,
}

#[derive(Debug, Clone)]
pub enum GraphType {
    WorkflowGraph,      // Business process visualization
    ConceptualGraph,    // Knowledge representation
    EventFlowGraph,     // Event sourcing visualization
    DevelopmentGraph,   // Dog-fooding self-visualization
}

impl GraphAggregate {
    pub fn handle_command(&mut self, cmd: GraphCommand) -> Result<Vec<DomainEvent>> {
        match cmd {
            GraphCommand::AddNode { node_type, position, metadata } => {
                // Validate domain invariants
                self.validate_node_addition(&node_type)?;

                // Create node with conceptual position
                let node_id = NodeId::new();
                let conceptual_point = self.conceptual_mapping
                    .map_to_conceptual_space(&position)?;

                let event = NodeAdded {
                    graph_id: self.id,
                    node_id,
                    node_type,
                    position,
                    conceptual_point,
                    metadata,
                };

                self.apply_event(&event);
                Ok(vec![DomainEvent::NodeAdded(event)])
            }
            // Other commands...
        }
    }
}
```

### Node and Edge Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    // Workflow Nodes
    WorkflowStep { step_type: StepType },
    Decision { criteria: DecisionCriteria },
    Integration { system: String },

    // Conceptual Nodes
    Concept { embedding: ConceptEmbedding },
    Category { region: ConvexRegion },

    // Event Nodes
    Event { event_type: String },
    Aggregate { aggregate_type: String },

    // Development Nodes (Dog-fooding)
    Feature { status: FeatureStatus },
    Task { priority: Priority },
    Milestone { target_date: DateTime<Utc> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeType {
    // Workflow Edges
    Sequence,
    Conditional { condition: String },
    Parallel,

    // Conceptual Edges
    Similarity { strength: f32 },
    Hierarchy { level: u32 },
    Association { relation_type: String },

    // Event Edges
    Triggers,
    Produces,
    ConsumesFrom,

    // Development Edges
    DependsOn,
    Blocks,
    Implements,
}
```

## CQRS Implementation

### Write Model (Commands)
```rust
pub struct GraphAggregate {
    id: GraphId,
    graph: petgraph::Graph<NodeId, EdgeId>,
    nodes: DashMap<NodeId, NodeEntity>,
    component_indices: ComponentIndices,
}

// Command processing
impl GraphAggregate {
    pub async fn handle_command(&mut self, cmd: GraphCommand) -> Result<Vec<DomainEvent>, Error> {
        match cmd {
            GraphCommand::AddNode { node } => {
                // Validate business rules
                // Generate events
                // Update state
            }
        }
    }
}
```

### Read Model (Queries)
```rust
pub struct GraphReadModel {
    node_views: DashMap<NodeId, NodeView>,
    metrics: Arc<RwLock<GraphMetrics>>,
    query_cache: QueryCache,
}

// Optimized queries
impl GraphReadModel {
    pub fn find_nodes_with_component(
        &self,
        component_type: ComponentType,
        criteria: ComponentCriteria,
    ) -> Result<Vec<NodeView>, QueryError> {
        // Use pre-computed indices
        // Return from cache if available
    }
}
```

## Component Deduplication Strategy

### Memory-Optimized Storage
```rust
// Flyweight pattern for 60-80% memory reduction
pub struct ComponentStorage {
    components: HashMap<ComponentId, Arc<Component>>,
    reference_counts: HashMap<ComponentId, usize>,
}

pub struct NodeEntity {
    pub id: NodeId,
    pub component_ids: HashSet<ComponentId>,  // References, not copies
}

// Before: 500+ bytes per node
// After: 64 bytes per node + shared components
```

## Async/Sync Bridge

### Bridge Architecture
```rust
pub struct AsyncSyncBridge {
    // Commands: Bevy (sync) → NATS (async)
    command_tx: crossbeam::channel::Sender<BridgeCommand>,
    command_rx: Arc<Mutex<crossbeam::channel::Receiver<BridgeCommand>>>,

    // Events: NATS (async) → Bevy (sync)
    event_tx: tokio::sync::mpsc::UnboundedSender<BridgeEvent>,
    event_rx: crossbeam::channel::Receiver<BridgeEvent>,
}

// Batched event processing
impl AsyncSyncBridge {
    pub fn receive_events(&self) -> Vec<BridgeEvent> {
        // Batch events for efficiency
        // Respect timeout for low latency
    }
}
```

## Bevy Integration Patterns

### ECS Components
```rust
#[derive(Component)]
pub struct GraphNode {
    node_id: NodeId,
    graph_index: NodeIndex<u32>,
}

#[derive(Component)]
pub struct GraphEdge {
    edge_id: EdgeId,
    source: Entity,
    target: Entity,
}

#[derive(Component)]
pub struct GraphReference {
    aggregate_id: AggregateId,
    last_event_cid: Option<Cid>,
}
```

### Event Processing Systems
```rust
fn poll_graph_events(
    bridge: Res<AsyncSyncBridge>,
    mut commands: Commands,
    mut graph_events: EventWriter<GraphMutationEvent>,
) {
    let events = bridge.receive_events();

    for event in events {
        match event {
            BridgeEvent::NodeAdded { graph_id, node_id, event_cid } => {
                // Spawn entity
                // Update visualization
                graph_events.send(GraphMutationEvent::NodeAdded {
                    graph_id,
                    node_id,
                });
            }
            // Handle other events
        }
    }
}
```

## Conceptual Space Integration

### Semantic Positioning
```rust
pub struct ConceptualMapping {
    space_id: ConceptualSpaceId,
    dimension_mappings: Vec<DimensionMapping>,
    position_calculator: Box<dyn PositionCalculator>,
}

impl ConceptualMapping {
    pub fn map_to_conceptual_space(
        &self,
        visual_position: &Position3D,
    ) -> Result<ConceptualPoint> {
        // Map visual position to semantic dimensions
        let mut coordinates = Vec::new();

        for mapping in &self.dimension_mappings {
            let value = mapping.extract_from_position(visual_position)?;
            coordinates.push(value);
        }

        Ok(ConceptualPoint {
            space_id: self.space_id,
            coordinates,
            confidence: self.calculate_confidence(&coordinates),
        })
    }

    pub fn find_optimal_visual_position(
        &self,
        conceptual_point: &ConceptualPoint,
    ) -> Result<Position3D> {
        // Inverse mapping for layout algorithms
        self.position_calculator.calculate(conceptual_point)
    }
}
```

### Knowledge-Aware Layout
```rust
pub struct SemanticLayoutEngine {
    conceptual_space: ConceptualSpace,
    layout_constraints: LayoutConstraints,
}

impl SemanticLayoutEngine {
    pub fn layout_graph(
        &self,
        nodes: &[Node],
        edges: &[Edge],
    ) -> Result<HashMap<NodeId, Position3D>> {
        // Use conceptual distances for force-directed layout
        let mut positions = HashMap::new();

        // Initialize with conceptual positions
        for node in nodes {
            let initial_pos = self.conceptual_space
                .get_visual_position(&node.conceptual_point)?;
            positions.insert(node.id, initial_pos);
        }

        // Apply force-directed algorithm with semantic weights
        self.apply_semantic_forces(&mut positions, nodes, edges)?;

        Ok(positions)
    }
}
```

## Dog-fooding Implementation

### Self-Visualization
```rust
pub struct DevelopmentGraph {
    pub graph: GraphAggregate,
    pub progress_tracker: ProgressTracker,
    pub milestone_manager: MilestoneManager,
}

impl DevelopmentGraph {
    pub fn visualize_progress(&mut self) -> Result<()> {
        // Create nodes for current features
        for feature in self.progress_tracker.active_features() {
            self.graph.handle_command(GraphCommand::AddNode {
                node_type: NodeType::Feature {
                    status: feature.status,
                },
                position: self.calculate_feature_position(&feature)?,
                metadata: feature.metadata(),
            })?;
        }

        // Connect dependencies
        for (feature, deps) in self.progress_tracker.dependencies() {
            for dep in deps {
                self.graph.handle_command(GraphCommand::ConnectNodes {
                    source: feature.node_id,
                    target: dep.node_id,
                    edge_type: EdgeType::DependsOn,
                })?;
            }
        }

        Ok(())
    }
}
```

## Performance Optimization

### Query Performance
```rust
// Sub-10ms query latency through:
// 1. Pre-computed indices
// 2. LRU cache with TTL
// 3. Parallel query execution

pub struct QueryCache {
    cache: Arc<RwLock<LruCache<QueryKey, QueryResult>>>,
    hit_rate: Arc<AtomicU64>,
}
```

### Memory Efficiency
```rust
// Component deduplication
// - 80% reduction for similar components
// - Reference counting for lifecycle
// - Bloom filters for fast lookups

pub struct ComponentIndices {
    type_index: HashMap<ComponentType, HashSet<NodeId>>,
    category_index: HashMap<String, HashSet<NodeId>>,
    bloom_filter: BloomFilter<ComponentId>,
}
```

### Spatial Indexing
```rust
pub struct SpatialIndex {
    rtree: RTree<GraphNode>,
    conceptual_index: ConceptualIndex,
    edge_index: HashMap<(NodeId, NodeId), EdgeId>,
}

impl SpatialIndex {
    pub fn find_nodes_in_region(
        &self,
        region: &BoundingBox,
    ) -> Vec<NodeId> {
        self.rtree
            .locate_in_envelope(&region.to_envelope())
            .map(|node| node.node_id)
            .collect()
    }

    pub fn find_semantically_similar(
        &self,
        node_id: NodeId,
        threshold: f32,
    ) -> Vec<(NodeId, f32)> {
        self.conceptual_index
            .find_similar(node_id, threshold)
    }
}
```

### Level-of-Detail (LOD)
```rust
#[derive(Component)]
pub struct GraphLOD {
    pub detail_level: DetailLevel,
    pub visible_radius: f32,
}

fn update_graph_lod(
    camera: Query<&Transform, With<Camera>>,
    mut nodes: Query<(&Transform, &mut GraphLOD, &mut Visibility)>,
) {
    let camera_pos = camera.single().translation;

    for (transform, mut lod, mut visibility) in nodes.iter_mut() {
        let distance = camera_pos.distance(transform.translation);

        // Adjust detail level based on distance
        lod.detail_level = match distance {
            d if d < 50.0 => DetailLevel::Full,
            d if d < 200.0 => DetailLevel::Simplified,
            _ => DetailLevel::Hidden,
        };

        // Update visibility
        *visibility = if distance < lod.visible_radius {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
```

## Snapshot Management

### Fast Recovery
```rust
pub struct SnapshotService {
    object_store: NatsObjectStore,
    compression: CompressionType::Zstd,
}

impl SnapshotService {
    pub async fn create_snapshot(&self, graph: &GraphAggregate) -> Result<Cid, Error> {
        // Serialize graph state
        // Compress with zstd
        // Store in NATS Object Store
        // Return CID for retrieval
    }

    pub async fn restore_from_snapshot(&self, cid: Cid) -> Result<GraphAggregate, Error> {
        // Fetch from object store
        // Decompress
        // Reconstruct graph
    }
}
```

## Testing Patterns

### CID Chain Verification
```rust
#[tokio::test]
async fn test_event_chain_integrity() {
    let event1 = store.append_event(...).await?;
    let event2 = store.append_event(...).await?;

    assert_eq!(event2.previous_cid, event1.event_cid);
    assert!(verify_cid_chain(&[event1, event2])?);
}
```

### CQRS Consistency
```rust
#[test]
fn test_eventual_consistency() {
    // Write through command
    write_model.process_command(AddNode { ... })?;

    // Wait for projection
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Read model should reflect change
    let node = read_model.get_node(node_id)?;
    assert!(node.is_some());
}
```

### Graph Invariant Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conceptual_consistency() {
        let mut graph = create_test_graph();

        // Add nodes
        let node1 = add_concept_node(&mut graph, "Machine Learning");
        let node2 = add_concept_node(&mut graph, "Neural Networks");

        // Connect with similarity edge
        connect_nodes(&mut graph, node1, node2, EdgeType::Similarity { strength: 0.8 });

        // Verify conceptual distance matches edge strength
        let distance = graph.conceptual_mapping
            .calculate_distance(&node1, &node2)
            .unwrap();

        assert!((1.0 - distance - 0.8).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_event_sourcing_replay() {
        let event_store = create_test_event_store().await;
        let mut graph = GraphAggregate::new(GraphId::new());

        // Generate events
        let events = vec![
            create_node_added_event(),
            create_edge_added_event(),
        ];

        // Store and replay
        event_store.append_events(events.clone()).await.unwrap();
        let replayed = event_store.get_events(graph.id).await.unwrap();

        // Apply to graph
        for event in replayed {
            graph.apply_event(&event);
        }

        // Verify state
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
    }
}
```

## Configuration

### NATS JetStream
```yaml
jetstream:
  store_dir: "./data/jetstream"
  max_memory_store: 1GB
  max_file_store: 10GB

streams:
  - name: event-store
    subjects: ["event.store.>"]
    retention: limits
    max_age: 365d
```

### Performance Tuning
```rust
pub struct GraphConfig {
    // Deduplication
    pub dedup_threshold: f32,        // 0.8 = 80% similarity
    pub dedup_batch_size: usize,     // 1000 components

    // Caching
    pub cache_size: usize,           // 10_000 entries
    pub cache_ttl: Duration,         // 5 minutes

    // Batching
    pub event_batch_size: usize,     // 100 events
    pub batch_timeout: Duration,     // 10ms

    // Snapshots
    pub snapshot_interval: u64,      // Every 1000 events
    pub snapshot_compression: bool,  // true (zstd)
}
```

## Operational Guidelines

1. **Event Ordering**:
   - Maintain strict sequence per aggregate
   - Use CID chains for integrity verification
   - Handle out-of-order events gracefully

2. **Memory Management**:
   - Monitor component deduplication ratio
   - Set bounded buffers (10K events default)
   - Use snapshots for long-running graphs

3. **Performance Monitoring**:
   ```rust
   pub struct GraphMetrics {
       event_lag_ms: Histogram,
       query_latency_p99: Gauge,
       cache_hit_ratio: Gauge,
       memory_usage_mb: Gauge,
   }
   ```

4. **Error Recovery**:
   - Automatic retry with exponential backoff
   - Circuit breakers for downstream services
   - Snapshot-based recovery for corruption

## Best Practices

1. **Event-First Design**: Always start with domain events when adding graph features
2. **Conceptual Alignment**: Ensure visual layout reflects semantic relationships
3. **Performance Awareness**: Use spatial indices and LOD for large graphs
4. **Dog-food Continuously**: Use the graph to visualize its own development
5. **Test Invariants**: Verify both visual and conceptual consistency

## Common Pitfalls to Avoid

❌ **Direct Graph Manipulation**
```rust
// WRONG - Bypassing domain logic
graph.nodes.insert(node_id, node);
```

✅ **Command-Based Updates**
```rust
// CORRECT - Through aggregate
graph.handle_command(GraphCommand::AddNode { ... })?;
```

❌ **Ignoring Conceptual Space**
```rust
// WRONG - Only visual positioning
let position = Position3D::new(x, y, z);
```

✅ **Semantic Positioning**
```rust
// CORRECT - Map to conceptual space
let conceptual_point = mapping.map_to_conceptual_space(&position)?;
```

❌ **Tight Coupling to Bevy**
```rust
// WRONG - Domain knows about ECS
impl GraphAggregate {
    fn spawn_entity(&self, commands: &mut Commands) { ... }
}
```

✅ **Clean Separation**
```rust
// CORRECT - Events bridge domains
let event = self.create_event();
// Let Bevy systems handle spawning
```