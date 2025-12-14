# Development Guidelines

## Code Structure

### Module Organization
- Keep aggregate logic minimal - only core identity and lifecycle
- Place all additional data in components
- Maintain clear separation between commands, queries, and events
- Use cross_domain module for all inter-domain communication

### Naming Conventions
- **Commands**: `VerbNoun` format (e.g., `CreatePerson`, `AddSkill`)
- **Events**: Past tense `NounVerbed` (e.g., `PersonCreated`, `SkillAdded`)
- **Components**: Descriptive nouns (e.g., `ContactInfo`, `Skills`)
- **Projections**: `NounView` or `NounSummary` (e.g., `PersonView`, `SkillsSummary`)

## ECS Patterns

### Component Design
```rust
// Components should be:
// 1. Self-contained
// 2. Serializable
// 3. Cloneable
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct ContactInfo {
    pub email: Option<String>,
    pub phone: Option<String>,
    // ...
}
```

### System Implementation
```rust
// Systems should:
// 1. Have single responsibility
// 2. Use queries for data access
// 3. Emit events for state changes
pub fn handle_skill_updates(
    mut commands: Commands,
    query: Query<(&PersonId, &Skills), Changed<Skills>>,
    mut events: EventWriter<SkillUpdated>,
) {
    // Implementation
}
```

## Event Sourcing

### Event Design
- Events must be immutable
- Include all necessary data for reconstruction
- Use domain-specific types (not primitives)
- Version events when schema changes

### Event Handling
```rust
// Always handle events idempotently
match event {
    PersonEvent::Created { id, name, timestamp } => {
        // Check if already exists before applying
    }
    // ...
}
```

## Cross-Domain Integration

### Inbound Events
- Subscribe to relevant external events in `cross_domain/handlers`
- Transform external events to internal commands
- Validate external data before processing

### Outbound Events
- Publish domain events that other domains might need
- Use standard event formats
- Include correlation IDs for tracing

## Testing Standards

### Unit Tests
- Test command validation
- Test event application
- Test component behavior
- Use property-based testing for complex logic

### Integration Tests
- Test cross-domain event flows
- Test projection updates
- Test query consistency
- Use test fixtures for common scenarios

## Performance Considerations

### Component Queries
- Use indexed queries for frequently accessed data
- Batch component updates when possible
- Consider using Changed<T> filters for reactive systems

### Event Storage
- Implement event snapshots for aggregates with many events
- Use appropriate batch sizes for event replay
- Consider event archival strategies

## Security

### Data Privacy
- Never log sensitive personal information
- Implement field-level encryption for PII
- Use audit trails for data access

### Validation
- Validate all command inputs
- Sanitize string inputs
- Check authorization before processing commands

## Error Handling

### Command Failures
```rust
// Return descriptive errors
pub enum PersonCommandError {
    NotFound(PersonId),
    InvalidState { id: PersonId, state: LifecycleState },
    ValidationFailed(String),
}
```

### Event Processing
- Log but don't fail on non-critical event processing errors
- Implement retry mechanisms for transient failures
- Use dead letter queues for persistent failures