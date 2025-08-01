# DDD-ECS Isomorphic Mapping Patterns

## Core Isomorphism

This defines the mathematical, structure-preserving functor between Domain-Driven Design (DDD) and Entity Component System (ECS) in our architecture.

```
DDD Category                    ECS Category
────────────────────────────────────────────
Entity/Aggregate       ←→       Entity
Value Object          ←→       Component
Command Handler       ←→       System (Write)
Query Handler         ←→       System (Read)
Domain Event          ←→       Event
Repository            ←→       Query<Components>
Domain Service        ←→       System (Pure)
Policy                ←→       System (Reactive)
```

## Detailed Mappings

### 1. Entities and Aggregates → ECS Entities

**DDD Entities/Aggregates** map to **ECS Entities** with identity and structure preserved:

```rust
// DDD Domain Model
pub struct GraphAggregate {
    pub id: GraphId,           // Identity
    pub nodes: HashMap<NodeId, Node>,
    pub edges: HashMap<EdgeId, Edge>,
}

// ECS Representation
#[derive(Component)]
pub struct GraphEntity {
    pub aggregate_id: GraphId,  // Same identity
}

// Spawning preserves identity
commands.spawn((
    GraphEntity { aggregate_id: graph_id },
    // Value objects as components...
));
```

**Key Principles:**
- One ECS Entity per DDD Aggregate Root
- Child entities within aggregates also become ECS Entities
- Identity is preserved through ID components
- Aggregate boundaries are maintained through entity relationships

### 2. Value Objects → Components

**DDD Value Objects** map directly to **ECS Components**:

```rust
// DDD Value Object
#[derive(Debug, Clone, PartialEq)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ECS Component (same type, just tagged)
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Position3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Usage
commands.spawn((
    NodeEntity { node_id },
    Position3D { x: 0.0, y: 0.0, z: 0.0 },  // Value object as component
    NodeContent { ... },                     // Another value object
));
```

**Key Principles:**
- Value objects ARE components (not just represented by them)
- Immutability is preserved - replace whole component, never mutate
- No identity - compared by value
- Can be freely copied between entities

### 3. Commands → Write Systems

**DDD Command Handlers** map to **ECS Systems** that process commands:

```rust
// DDD Command
pub struct MoveNode {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub new_position: Position3D,
}

// ECS System (Command Handler)
fn handle_move_node_commands(
    mut commands: Commands,
    mut events: EventWriter<NodeEvent>,
    nodes: Query<(Entity, &NodeEntity, &Position3D)>,
    mut move_commands: EventReader<MoveNodeCommand>,
) {
    for cmd in move_commands.read() {
        // Find the entity
        if let Some((entity, node, old_pos)) = nodes.iter()
            .find(|(_, n, _)| n.node_id == cmd.node_id)
        {
            // Remove old position (value object)
            commands.entity(entity).remove::<Position3D>();

            // Add new position (value object)
            commands.entity(entity).insert(cmd.new_position);

            // Emit domain events
            events.send(NodeEvent::NodeRemoved { ... });
            events.send(NodeEvent::NodeAdded { ... });
        }
    }
}
```

### 4. Queries → Read Systems

**DDD Query Handlers** map to **ECS Systems** that read state:

```rust
// DDD Query
pub struct FindNodesInRadius {
    pub center: Position3D,
    pub radius: f32,
}

// ECS System (Query Handler)
fn find_nodes_in_radius(
    query: Query<(&NodeEntity, &Position3D)>,
    requests: EventReader<FindNodesInRadiusRequest>,
    mut results: EventWriter<QueryResult>,
) {
    for request in requests.read() {
        let nodes: Vec<NodeId> = query.iter()
            .filter(|(_, pos)| {
                let distance = calculate_distance(pos, &request.center);
                distance <= request.radius
            })
            .map(|(node, _)| node.node_id)
            .collect();

        results.send(QueryResult::NodesFound { nodes });
    }
}
```

### 5. Domain Events → ECS Events

**DDD Domain Events** map directly to **ECS Events**:

```rust
// DDD Domain Event
#[derive(Debug, Clone)]
pub struct NodeAdded {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Position3D,
}

// ECS Event (same type, implements Event)
#[derive(Event, Debug, Clone)]
pub struct NodeAdded {
    pub graph_id: GraphId,
    pub node_id: NodeId,
    pub position: Position3D,
}

// Event flow preserved
fn process_domain_events(
    mut events: EventReader<NodeAdded>,
    mut commands: Commands,
) {
    for event in events.read() {
        // React to domain events
        commands.spawn((
            NodeEntity { node_id: event.node_id },
            event.position.clone(),
        ));
    }
}
```

### 6. Repositories → Queries

**DDD Repositories** map to **ECS Queries**:

```rust
// DDD Repository Interface
trait NodeRepository {
    fn find_by_id(&self, id: NodeId) -> Option<Node>;
    fn find_by_type(&self, node_type: NodeType) -> Vec<Node>;
}

// ECS Query (Repository Implementation)
fn node_repository_queries(
    nodes: Query<(&NodeEntity, &NodeContent, &Position3D)>,
) {
    // find_by_id equivalent
    let node = nodes.iter()
        .find(|(n, _, _)| n.node_id == target_id);

    // find_by_type equivalent
    let typed_nodes: Vec<_> = nodes.iter()
        .filter(|(_, content, _)| content.node_type == target_type)
        .collect();
}
```

### 7. Domain Services → Pure Systems

**DDD Domain Services** map to **Pure ECS Systems**:

```rust
// DDD Domain Service
pub struct GraphLayoutService {
    pub fn calculate_layout(&self, nodes: &[Node]) -> HashMap<NodeId, Position3D> {
        // Pure calculation
    }
}

// ECS System (Domain Service)
fn calculate_graph_layout(
    nodes: Query<(&NodeEntity, &Position3D)>,
    mut layout_events: EventWriter<LayoutCalculated>,
) {
    // Pure calculation, no side effects
    let positions = calculate_optimal_layout(&nodes);

    // Emit result as event
    layout_events.send(LayoutCalculated { positions });
}
```

### 8. Policies → Reactive Systems

**DDD Policies** map to **Reactive ECS Systems**:

```rust
// DDD Policy
pub struct AutoConnectPolicy {
    pub fn on_node_added(&self, event: &NodeAdded) -> Vec<Command> {
        // Policy logic
    }
}

// ECS System (Policy)
fn auto_connect_policy(
    mut events: EventReader<NodeAdded>,
    mut commands: EventWriter<ConnectEdgeCommand>,
    nodes: Query<(&NodeEntity, &Position3D)>,
) {
    for event in events.read() {
        // Find nearby nodes
        let nearby_nodes = find_nearby_nodes(&nodes, &event.position);

        // Apply policy: auto-connect to nearby nodes
        for target in nearby_nodes {
            commands.send(ConnectEdgeCommand {
                source: event.node_id,
                target: target.node_id,
                relationship: EdgeRelationship::AutoConnected,
            });
        }
    }
}
```

## Resource Usage Rules

### ❌ NEVER Use Resources for Domain State

```rust
// WRONG - Domain state in resources
#[derive(Resource)]
struct GraphState {
    nodes: HashMap<NodeId, Node>,  // This is domain state!
}

// WRONG - Mutable domain operations
fn bad_system(mut graph_state: ResMut<GraphState>) {
    graph_state.nodes.insert(...);  // Direct mutation!
}
```

### ✅ ONLY Use Resources for Read Models

```rust
// CORRECT - Read model/projection
#[derive(Resource)]
struct GraphMetrics {
    node_count: usize,
    edge_count: usize,
    last_updated: Instant,
}

// CORRECT - Read-only cache
#[derive(Resource)]
struct SpatialIndex {
    rtree: RTree<NodeId>,  // Read-optimized structure
}

// CORRECT - Configuration
#[derive(Resource)]
struct GraphConfig {
    max_nodes: usize,
    auto_connect_radius: f32,
}
```

## Event-Driven Communication

### Command → Event Flow

```rust
// 1. Command enters system
#[derive(Event)]
struct CreateNodeCommand {
    graph_id: GraphId,
    position: Position3D,
}

// 2. System processes command
fn process_create_node(
    mut commands: Commands,
    mut events: EventWriter<NodeCreated>,
    mut cmd_reader: EventReader<CreateNodeCommand>,
) {
    for cmd in cmd_reader.read() {
        // Create entity
        let node_id = NodeId::new();
        commands.spawn((
            NodeEntity { node_id },
            cmd.position.clone(),
        ));

        // Emit event
        events.send(NodeCreated {
            graph_id: cmd.graph_id,
            node_id,
            position: cmd.position,
        });
    }
}

// 3. Other systems react to event
fn update_spatial_index(
    mut events: EventReader<NodeCreated>,
    mut index: ResMut<SpatialIndex>,
) {
    for event in events.read() {
        index.insert(event.node_id, event.position);
    }
}
```

## Structure Preservation Properties

### 1. Identity Preservation
- DDD Entity ID ↔ ECS Entity with ID Component
- Bijective mapping maintained

### 2. Composition Preservation
- DDD Aggregate composition ↔ ECS Entity hierarchy
- Parent-child relationships preserved

### 3. Immutability Preservation
- DDD Value Object immutability ↔ Component replacement (not mutation)
- No in-place updates

### 4. Boundary Preservation
- DDD Aggregate boundaries ↔ ECS Entity boundaries
- No cross-aggregate component sharing

### 5. Event Ordering Preservation
- DDD Event sequence ↔ ECS Event order
- Causality maintained

## Implementation Patterns

### Pattern 1: Aggregate to Entity Spawning

```rust
fn spawn_graph_aggregate(
    mut commands: Commands,
    graph: &GraphAggregate,
) {
    // Spawn root entity
    let graph_entity = commands.spawn((
        GraphEntity { aggregate_id: graph.id },
        GraphMetadata::from(&graph.metadata),
    )).id();

    // Spawn child entities
    for (node_id, node) in &graph.nodes {
        commands.spawn((
            NodeEntity { node_id: *node_id },
            ParentGraph(graph_entity),
            node.position.clone(),
            node.content.clone(),
        ));
    }
}
```

### Pattern 2: Value Object Change

```rust
fn change_node_position(
    mut commands: Commands,
    entity: Entity,
    new_position: Position3D,
) {
    // Remove old value object
    commands.entity(entity).remove::<Position3D>();

    // Insert new value object
    commands.entity(entity).insert(new_position);
}
```

### Pattern 3: Query Projection

```rust
fn project_graph_view(
    graphs: Query<&GraphEntity>,
    nodes: Query<(&NodeEntity, &ParentGraph, &Position3D)>,
) -> GraphView {
    // Build read model from ECS state
    let mut view = GraphView::new();

    for graph in graphs.iter() {
        let graph_nodes = nodes.iter()
            .filter(|(_, parent, _)| parent.0 == graph.entity)
            .map(|(node, _, pos)| (node.node_id, pos.clone()))
            .collect();

        view.add_graph(graph.aggregate_id, graph_nodes);
    }

    view
}
```

## Anti-Patterns to Avoid

### ❌ Mutable Components
```rust
// WRONG
fn mutate_position(mut positions: Query<&mut Position3D>) {
    for mut pos in positions.iter_mut() {
        pos.x += 1.0;  // Violates value object immutability!
    }
}
```

### ❌ Cross-Aggregate Queries
```rust
// WRONG
fn cross_aggregate_query(
    nodes: Query<&NodeEntity>,
    edges: Query<&EdgeEntity>,
) {
    // Querying across aggregate boundaries
}
```

### ❌ Domain Logic in Components
```rust
// WRONG
#[derive(Component)]
struct Node {
    fn validate(&self) -> bool { ... }  // Domain logic!
}
```

### ❌ Resources for Domain State
```rust
// WRONG
#[derive(Resource)]
struct DomainState {
    aggregates: HashMap<AggregateId, Aggregate>,
}
```

## Summary

The DDD-ECS isomorphism maintains:
1. **Entities** remain entities (with identity)
2. **Value Objects** become components (immutable)
3. **Commands/Queries** become systems (behavior)
4. **Events** flow between systems (communication)
5. **Resources** only for read models (never domain state)

This mapping preserves the mathematical structure of both paradigms while enabling high-performance, cache-friendly execution through ECS while maintaining DDD's semantic richness and business alignment.