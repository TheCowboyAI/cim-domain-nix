# NATS-Based Event-Driven Refactoring Plan for CIM Domain Person

## Executive Summary

This plan outlines the refactoring of `cim-domain-person` to align with the pure event-driven architecture demonstrated in `cim-graph`. The refactoring will transform the current hybrid approach into a NATS-centric, event-sourced system with clear command/event boundaries and immutable event streams.

## Current State Analysis

### Current Architecture
- **Pattern**: ECS (Entity Component System) with Bevy
- **State Management**: Mixed approach with direct mutations and event sourcing
- **Messaging**: Basic NATS integration for cross-domain events
- **Storage**: Traditional event store with database persistence
- **API**: Command/Query pattern but not purely event-driven

### Key Differences from CIM-Graph
1. **Event Flow**: Current system allows direct state mutations; cim-graph enforces event-only changes
2. **NATS Integration**: Limited to cross-domain messaging; cim-graph uses NATS as primary event store
3. **State Machines**: Not currently used; cim-graph uses them for all command validation
4. **Content Addressing**: Not implemented; cim-graph uses IPLD for event storage
5. **Policy Engine**: No automatic event generation; cim-graph has policy-driven behaviors

## Target Architecture

### Core Principles
1. **Pure Event Sourcing**: All state changes MUST go through events
2. **NATS JetStream**: Primary event persistence and streaming mechanism
3. **State Machines**: Validate commands and enforce business rules
4. **Content-Addressed Events**: Use IPLD for immutable event storage
5. **Policy-Driven Behaviors**: Automatic event generation based on rules

### Event Flow
```
Command → State Machine → Event → IPLD → NATS JetStream → Projections
                            ↓
                        Event Store
```

## Refactoring Phases

### Phase 1: Foundation (2-3 weeks)
**Goal**: Establish NATS JetStream infrastructure and event patterns

1. **NATS JetStream Setup**
   ```rust
   // New infrastructure module
   pub struct PersonEventStore {
       jetstream: JetStreamContext,
       ipld_store: IpldStore,
   }
   ```

2. **Event Subject Hierarchy**
   ```
   cim.person.lifecycle.*     // Created, Activated, Suspended, Archived
   cim.person.identity.*      // NameUpdated, IdentityLinked
   cim.person.components.*    // ContactAdded, SkillUpdated, etc.
   cim.person.relationships.* // LocationAssigned, EmploymentAdded
   ```

3. **Content-Addressed Event Storage**
   - Implement IPLD integration for event persistence
   - Store event CIDs in NATS messages
   - Enable event verification and audit trails

### Phase 2: State Machine Integration (2-3 weeks)
**Goal**: Replace command handlers with state machines

1. **Person Lifecycle State Machine**
   ```rust
   pub enum PersonState {
       Draft,
       Active,
       Suspended,
       Archived,
   }
   
   pub struct PersonStateMachine {
       transitions: HashMap<(PersonState, CommandType), PersonState>,
   }
   ```

2. **Component State Machines**
   - ContactInfoStateMachine
   - SkillsStateMachine
   - PreferencesStateMachine
   - RelationshipStateMachine

3. **Validation Rules**
   - Move all validation logic into state machines
   - Enforce invariants through state transitions
   - Generate appropriate error events for invalid transitions

### Phase 3: Pure Event-Driven API (3-4 weeks)
**Goal**: Transform all APIs to pure event-driven model

1. **Command Processing Pipeline**
   ```rust
   pub async fn process_command(cmd: PersonCommand) -> Result<Vec<Event>> {
       // 1. Load current state from event stream
       let state = load_person_state(&cmd.person_id).await?;
       
       // 2. Validate command via state machine
       let events = state_machine.validate_and_process(state, cmd)?;
       
       // 3. Store events in IPLD
       let cids = ipld_store.store_events(&events).await?;
       
       // 4. Publish to NATS with CIDs
       jetstream.publish_events(&events, &cids).await?;
       
       Ok(events)
   }
   ```

2. **Event Subscription Model**
   ```rust
   pub async fn subscribe_to_person_events() -> EventStream {
       jetstream.subscribe("cim.person.>")
           .await
           .map(|msg| deserialize_event(msg))
   }
   ```

3. **Projection Updates**
   - Convert all projections to event-driven updates
   - Remove direct database writes
   - Implement catch-up subscription for rebuilding

### Phase 4: Policy Engine (2-3 weeks)
**Goal**: Implement automatic event generation based on policies

1. **Policy Framework**
   ```rust
   pub trait PersonPolicy {
       fn evaluate(&self, event: &PersonEvent) -> Vec<PersonCommand>;
   }
   ```

2. **Example Policies**
   - Auto-archive inactive persons after 1 year
   - Generate skill recommendations based on activity
   - Cascade relationship updates across domains

3. **Policy Execution Engine**
   ```rust
   pub struct PolicyEngine {
       policies: Vec<Box<dyn PersonPolicy>>,
       command_bus: CommandBus,
   }
   ```

### Phase 5: Migration and Compatibility (2-3 weeks)
**Goal**: Ensure smooth transition from current system

1. **Data Migration**
   - Export existing events to IPLD
   - Replay events through new system
   - Verify projection consistency

2. **API Compatibility Layer**
   - Temporary adapters for existing APIs
   - Gradual deprecation strategy
   - Client migration guide

3. **Cross-Domain Integration Updates**
   - Update event formats for compatibility
   - Implement event transformation layers
   - Test with other CIM domains

## Implementation Details

### New Module Structure
```
src/
├── domain/           # Pure domain logic
│   ├── events/       # Event definitions
│   ├── commands/     # Command definitions
│   └── state/        # State machines
├── infrastructure/   # Technical concerns
│   ├── nats/         # NATS JetStream integration
│   ├── ipld/         # Content-addressed storage
│   └── projections/  # Read model updates
├── application/      # Use cases
│   ├── handlers/     # Command processors
│   ├── policies/     # Business rules
│   └── queries/      # Read operations
└── adapters/         # External interfaces
    ├── http/         # REST API
    ├── grpc/         # gRPC service
    └── graphql/      # GraphQL endpoint
```

### Key Dependencies Updates
```toml
[dependencies]
async-nats = "0.35"
nats-jetstream = "0.35"
ipld = "0.16"
state_machine_future = "0.2"
```

### Testing Strategy
1. **Unit Tests**: State machines and domain logic
2. **Integration Tests**: NATS event flows
3. **End-to-End Tests**: Complete command/event cycles
4. **Migration Tests**: Data consistency verification

## Benefits of Refactoring

1. **Pure Event Sourcing**: Complete audit trail and time travel capabilities
2. **Scalability**: NATS JetStream handles high-throughput event streams
3. **Decoupling**: Clear boundaries between domains
4. **Flexibility**: Policy-driven behaviors enable dynamic business rules
5. **Consistency**: State machines ensure valid state transitions
6. **Resilience**: Event replay enables system recovery

## Risks and Mitigation

### Risk 1: Breaking Changes
- **Mitigation**: Compatibility layer and phased migration

### Risk 2: Performance Impact
- **Mitigation**: Benchmark critical paths and optimize projections

### Risk 3: Complexity Increase
- **Mitigation**: Comprehensive documentation and examples

### Risk 4: Integration Issues
- **Mitigation**: Extensive cross-domain testing

## Success Metrics

1. **All state changes via events**: 100% event-sourced
2. **NATS message throughput**: >10,000 events/second
3. **Projection lag**: <100ms for 95th percentile
4. **Zero data loss**: Event replay produces identical state
5. **API compatibility**: Existing clients continue working

## Timeline Summary

- **Phase 1**: Weeks 1-3 - Foundation
- **Phase 2**: Weeks 4-6 - State Machines
- **Phase 3**: Weeks 7-10 - Pure Event API
- **Phase 4**: Weeks 11-13 - Policy Engine
- **Phase 5**: Weeks 14-16 - Migration

**Total Duration**: 16 weeks (4 months)

## Next Steps

1. Review and approve refactoring plan
2. Set up development environment with NATS JetStream
3. Create proof-of-concept for Phase 1
4. Establish testing infrastructure
5. Begin incremental implementation