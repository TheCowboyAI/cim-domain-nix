# NATS Subject Algebra to ECS Mapping

## Overview

The CIM architecture maps NATS subject algebras to Entity Component System (ECS) patterns, where subject subscriptions act as system filters that determine which entities and components are processed. This creates a distributed ECS where systems can run across multiple services based on subject/correlation/causation filtering.

## Core Concepts

### Subject Algebra as System Filters

In traditional ECS:
```
System = Query<Components> → Transform<Entities>
```

In CIM's distributed ECS:
```
System = Subscribe<Subject × Correlation × Causation> → Transform<Entities>
```

Where:
- **Subject** defines the component types and operations
- **Correlation** groups related entity transformations
- **Causation** tracks entity transformation lineage

## Mapping Architecture

```mermaid
graph TB
    subgraph "NATS Subject Space"
        S1[nix.cmd.flake.create]
        S2[nix.event.flake.created]
        S3[nix.cmd.package.build]
        
        C1[Correlation: workflow-123]
        C2[Causation: parent-event-456]
    end
    
    subgraph "ECS Entity Space"
        E1[Entity: Flake-A]
        E2[Entity: Package-B]
        E3[Entity: Config-C]
        
        subgraph "Components"
            CP1[FlakeComponent]
            CP2[PackageComponent]
            CP3[PathComponent]
            CP4[StateComponent]
        end
    end
    
    subgraph "System Filters"
        SF1[FlakeCreationSystem]
        SF2[PackageBuildSystem]
        SF3[ConfigActivationSystem]
    end
    
    S1 -->|filters| SF1
    S2 -->|triggers| SF2
    S3 -->|filters| SF2
    
    SF1 -->|processes| E1
    SF2 -->|processes| E2
    
    E1 --> CP1
    E1 --> CP3
    E2 --> CP2
    E2 --> CP4
    
    C1 -.->|groups| E1
    C1 -.->|groups| E2
    C2 -.->|tracks| E2
    
    %% CIM Standard Styling
    style S1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style S2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style S3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style SF1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style SF2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style SF3 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style E1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style E2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style E3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Subject Algebra as Component Queries

### Traditional ECS Query
```rust
// Local ECS query
fn build_system(
    query: Query<(&FlakeComponent, &mut StateComponent), With<PackageComponent>>
) {
    for (flake, mut state) in query.iter_mut() {
        // Process entities with all required components
    }
}
```

### CIM Distributed Query
```rust
// Distributed ECS via NATS subjects
impl System for PackageBuildSystem {
    fn subscription_filter() -> SubjectFilter {
        SubjectFilter::new()
            .subjects(vec![
                "nix.cmd.package.build",
                "nix.event.flake.updated"
            ])
            .with_component_types(vec![
                ComponentType::Flake,
                ComponentType::Package,
                ComponentType::State
            ])
    }
    
    async fn process(&self, entity: Entity, components: Components) {
        // Process entities matching subject filter
    }
}
```

## Correlation as Entity Grouping

```mermaid
graph LR
    subgraph "Correlation Space"
        CORR[Correlation: deploy-webapp-789]
        
        subgraph "Related Entities"
            E1[Flake Entity]
            E2[Package Entity]
            E3[Module Entity]
            E4[Config Entity]
        end
        
        subgraph "Systems Processing"
            S1[FlakeSystem]
            S2[BuildSystem]
            S3[DeploySystem]
        end
    end
    
    CORR -->|groups| E1
    CORR -->|groups| E2
    CORR -->|groups| E3
    CORR -->|groups| E4
    
    S1 -->|filters by correlation| E1
    S2 -->|filters by correlation| E2
    S3 -->|filters by correlation| E4
    
    %% CIM Standard Styling
    style CORR fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style E1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style E2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style E3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style E4 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style S1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style S2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style S3 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Causation as Entity Lineage

```mermaid
graph TD
    subgraph "Causation Chain"
        C1[Root Command<br/>ID: A1]
        C2[Caused Event<br/>ID: A2<br/>Caused by: A1]
        C3[Caused Command<br/>ID: A3<br/>Caused by: A2]
        C4[Caused Event<br/>ID: A4<br/>Caused by: A3]
    end
    
    subgraph "Entity Evolution"
        E1[Flake Entity v1]
        E2[Flake Entity v2]
        E3[Package Entity v1]
        E4[Config Entity v1]
    end
    
    C1 -->|creates| E1
    C2 -->|updates| E2
    C3 -->|creates| E3
    C4 -->|creates| E4
    
    E1 -.->|evolves to| E2
    E2 -.->|references| E3
    E3 -.->|used by| E4
    
    %% CIM Standard Styling
    style C1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style C3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C4 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## System Registration Pattern

```mermaid
sequenceDiagram
    participant SYS as System
    participant NATS as NATS Broker
    participant REG as System Registry
    participant ENT as Entity Store
    
    SYS->>REG: Register with filters
    Note over REG: Filters:<br/>- Subjects<br/>- Components<br/>- Correlations
    
    REG->>NATS: Subscribe to subjects
    
    NATS->>SYS: Command received
    SYS->>ENT: Query entities
    Note over ENT: Filter by:<br/>- Component types<br/>- Correlation ID<br/>- Causation chain
    
    ENT->>SYS: Matching entities
    SYS->>SYS: Process entities
    SYS->>NATS: Emit events
```

## Component Type Mapping

```mermaid
graph TB
    subgraph "NATS Subjects"
        subgraph "Flake Subjects"
            FS1[nix.cmd.flake.*]
            FS2[nix.event.flake.*]
        end
        
        subgraph "Package Subjects"
            PS1[nix.cmd.package.*]
            PS2[nix.event.package.*]
        end
    end
    
    subgraph "Component Types"
        subgraph "Core Components"
            C1[IdentityComponent]
            C2[StateComponent]
            C3[MetadataComponent]
        end
        
        subgraph "Domain Components"
            C4[FlakeComponent]
            C5[PackageComponent]
            C6[PathComponent]
            C7[DerivationComponent]
        end
    end
    
    subgraph "Entity Archetypes"
        A1[Flake Archetype]
        A2[Package Archetype]
    end
    
    FS1 --> C4
    FS1 --> C6
    PS1 --> C5
    PS1 --> C7
    
    A1 --> C1
    A1 --> C2
    A1 --> C4
    A1 --> C6
    
    A2 --> C1
    A2 --> C2
    A2 --> C5
    A2 --> C7
    
    %% CIM Standard Styling
    style FS1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style FS2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PS1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style PS2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style A1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style A2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Implementation Example

### System Definition
```rust
use cim_domain_nix::ecs::{System, SubjectFilter, ComponentQuery};

pub struct FlakeUpdateSystem {
    subject_filter: SubjectFilter,
    component_query: ComponentQuery,
}

impl FlakeUpdateSystem {
    pub fn new() -> Self {
        Self {
            subject_filter: SubjectFilter::new()
                .subjects(vec![
                    "nix.cmd.flake.update",
                    "nix.event.flake.input_added"
                ])
                .correlation_required(true),
            component_query: ComponentQuery::new()
                .required(vec![
                    ComponentType::Flake,
                    ComponentType::Path,
                    ComponentType::State
                ])
                .optional(vec![
                    ComponentType::GitRepo
                ]),
        }
    }
}

#[async_trait]
impl System for FlakeUpdateSystem {
    async fn process(
        &self,
        entity_id: EntityId,
        components: ComponentBundle,
        context: SystemContext,
    ) -> Result<Vec<Event>> {
        // Extract components
        let flake: &FlakeComponent = components.get()?;
        let path: &PathComponent = components.get()?;
        let mut state: &mut StateComponent = components.get_mut()?;
        
        // Process based on correlation context
        if let Some(correlation) = context.correlation_id {
            // Part of a larger workflow
            state.workflow_id = Some(correlation);
        }
        
        // Update flake
        let events = self.update_flake(flake, path).await?;
        
        // Track causation
        Ok(events.into_iter()
            .map(|e| e.with_causation(context.message_id))
            .collect())
    }
}
```

### Entity Component Composition
```rust
// Define component types
#[derive(Component)]
pub struct FlakeComponent {
    pub flake_id: FlakeId,
    pub description: String,
    pub inputs: HashMap<String, FlakeRef>,
}

#[derive(Component)]
pub struct StateComponent {
    pub status: EntityStatus,
    pub last_modified: DateTime<Utc>,
    pub workflow_id: Option<CorrelationId>,
    pub version: u64,
}

// Entity archetypes
pub fn create_flake_entity(
    world: &mut World,
    flake_id: FlakeId,
    path: PathBuf,
    correlation: CorrelationId,
) -> Entity {
    world.spawn()
        .insert(IdentityComponent::new(flake_id))
        .insert(FlakeComponent::new(flake_id))
        .insert(PathComponent::new(path))
        .insert(StateComponent::new())
        .insert(CorrelationComponent::new(correlation))
        .id()
}
```

### Subscription-Based System Execution
```rust
pub struct SystemScheduler {
    systems: HashMap<SubjectPattern, Box<dyn System>>,
    entity_store: EntityStore,
}

impl SystemScheduler {
    pub async fn handle_message(
        &mut self,
        subject: &str,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<()> {
        // Extract correlation/causation from headers
        let correlation_id = headers.get("X-Correlation-ID")
            .and_then(|v| v.parse().ok());
        let causation_id = headers.get("X-Causation-ID")
            .and_then(|v| v.parse().ok());
        
        // Find matching systems
        for (pattern, system) in &self.systems {
            if pattern.matches(subject) {
                // Query entities matching system requirements
                let entities = self.entity_store
                    .query()
                    .with_components(&system.required_components())
                    .with_correlation(correlation_id)
                    .execute()
                    .await?;
                
                // Process each entity
                for entity in entities {
                    let context = SystemContext {
                        subject: subject.to_string(),
                        correlation_id,
                        causation_id,
                        message_id: Uuid::new_v4(),
                    };
                    
                    let events = system.process(
                        entity.id,
                        entity.components,
                        context,
                    ).await?;
                    
                    // Publish resulting events
                    for event in events {
                        self.publish_event(event).await?;
                    }
                }
            }
        }
        
        Ok(())
    }
}
```

## Benefits of Subject-Based ECS

### 1. Distributed System Execution
Systems can run across multiple services while operating on the same logical entities:

```mermaid
graph LR
    subgraph "Service A"
        S1[FlakeSystem]
        E1[Flake Entities]
    end
    
    subgraph "Service B"
        S2[BuildSystem]
        E2[Package Entities]
    end
    
    subgraph "Service C"
        S3[DeploySystem]
        E3[Config Entities]
    end
    
    subgraph "NATS"
        SUB[Subject Space]
    end
    
    S1 -->|subscribe| SUB
    S2 -->|subscribe| SUB
    S3 -->|subscribe| SUB
    
    SUB -->|filter| S1
    SUB -->|filter| S2
    SUB -->|filter| S3
```

### 2. Dynamic System Composition
Systems can be added/removed at runtime by subscribing/unsubscribing to subjects:

```rust
// Add a new system dynamically
scheduler.register_system(
    "nix.event.flake.created",
    Box::new(FlakeAnalysisSystem::new())
).await?;

// Remove a system
scheduler.unregister_system("nix.cmd.flake.delete").await?;
```

### 3. Correlation-Based Workflows
Complex workflows are naturally expressed through correlation filtering:

```mermaid
graph TD
    subgraph "Workflow: Deploy Application"
        W1[Create Flake]
        W2[Build Packages]
        W3[Create Config]
        W4[Activate Config]
    end
    
    subgraph "Systems"
        S1[FlakeSystem]
        S2[BuildSystem]
        S3[ConfigSystem]
        S4[ActivationSystem]
    end
    
    W1 -->|correlation: X| S1
    W2 -->|correlation: X| S2
    W3 -->|correlation: X| S3
    W4 -->|correlation: X| S4
    
    Note over S1,S4: All systems filter by correlation X
```

### 4. Causation-Based Debugging
Entity state evolution can be traced through causation chains:

```rust
// Trace entity history
async fn trace_entity_lineage(
    entity_id: EntityId,
    store: &EntityStore,
) -> Vec<EntityVersion> {
    let mut history = Vec::new();
    let mut current = store.get_entity(entity_id).await?;
    
    while let Some(causation) = current.causation_id {
        history.push(current.clone());
        current = store.get_entity_by_event(causation).await?;
    }
    
    history.reverse();
    history
}
```

## Advanced Patterns

### Subject Algebra Joins
Combine multiple subject patterns for complex system filters:

```rust
let filter = SubjectFilter::new()
    .union(vec![
        "nix.cmd.flake.*",
        "nix.event.package.built"
    ])
    .intersect_correlation(vec![
        "workflow-*",
        "deploy-*"
    ]);
```

### Component Algebra
Define component requirements using set operations:

```rust
let query = ComponentQuery::new()
    .all_of(vec![FlakeComponent, StateComponent])
    .any_of(vec![GitComponent, PathComponent])
    .none_of(vec![ErrorComponent]);
```

### Temporal Filtering
Filter entities based on temporal relationships:

```rust
let filter = TemporalFilter::new()
    .within_correlation_window(Duration::hours(1))
    .causation_depth(3)
    .after(timestamp);
```

## Conclusion

By mapping NATS subject algebras to ECS patterns, CIM creates a powerful distributed system architecture where:

1. **Subjects** act as system execution filters
2. **Correlation** groups related entity transformations
3. **Causation** tracks entity evolution lineage
4. **Systems** process entities based on subscription filters
5. **Components** are queried across distributed services

This approach enables building complex, distributed applications while maintaining the simplicity and performance benefits of ECS architecture.