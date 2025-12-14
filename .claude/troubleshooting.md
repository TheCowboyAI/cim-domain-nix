# Troubleshooting Guide

## Common Issues and Solutions

### Component Registration Errors

**Problem**: Component not found when querying
```
Error: Component type 'ContactInfo' not registered
```

**Solution**:
1. Ensure component is registered in the Bevy app:
```rust
// In src/infrastructure/app.rs
app.register_type::<ContactInfo>();
```

2. Check component derives:
```rust
#[derive(Component, Reflect, Serialize, Deserialize)]
pub struct ContactInfo { /* ... */ }
```

### Event Sourcing Issues

**Problem**: Events not being applied to aggregate
```
Person state doesn't reflect recent changes
```

**Solution**:
1. Verify event handler registration:
```rust
// Check event processor setup
event_processor.register_handler(person_event_handler);
```

2. Ensure events are being stored:
```rust
// Add logging to event store
event_store.append(&person_id, event).await
    .map_err(|e| error!("Failed to store event: {}", e))?;
```

3. Check event replay on aggregate load:
```rust
// Verify aggregate rebuilds from events
let events = event_store.get_events(&person_id).await?;
let person = Person::from_events(events)?;
```

### Cross-Domain Communication

**Problem**: External events not triggering updates
```
Organization events not updating person employment
```

**Solution**:
1. Verify NATS subscription:
```bash
# Check active subscriptions
nats sub "organization.events.*" --server=nats://localhost:4222
```

2. Check event handler mapping:
```rust
// In cross_domain/handlers/organization.rs
match event {
    OrganizationEvent::EmployeeAdded { .. } => {
        // Ensure handler logic exists
    }
}
```

3. Verify event deserialization:
```rust
// Add debug logging
debug!("Received external event: {:?}", event);
```

### Query Performance

**Problem**: Slow person queries with multiple components
```
GetPersonWithComponents taking >1s
```

**Solution**:
1. Use component queries efficiently:
```rust
// Bad: Multiple individual queries
let contact = query_contact(&person_id).await?;
let skills = query_skills(&person_id).await?;
let prefs = query_preferences(&person_id).await?;

// Good: Batch query
let components = query_person_components(&person_id).await?;
```

2. Implement caching for frequently accessed data:
```rust
// Add caching layer
let cache_key = format!("person_view:{}", person_id);
if let Some(cached) = cache.get(&cache_key).await? {
    return Ok(cached);
}
```

3. Use projections for complex queries:
```rust
// Instead of computing on demand, use pre-built projection
let summary = get_person_summary_projection(&person_id).await?;
```

### Test Failures

**Problem**: Integration tests failing intermittently
```
test cross_domain::test_location_assignment ... FAILED
```

**Solution**:
1. Add proper test synchronization:
```rust
// Wait for async operations
tokio::time::sleep(Duration::from_millis(100)).await;

// Or use explicit synchronization
test_env.wait_for_event::<PersonLocationAssigned>().await;
```

2. Use test fixtures consistently:
```rust
// Create standard test data
let person = test_fixtures::create_test_person();
let location = test_fixtures::create_test_location();
```

3. Clean up test state:
```rust
// Ensure clean state between tests
#[tokio::test]
async fn test_something() {
    let env = TestEnvironment::new().await; // Fresh environment
    // test code
    // TestEnvironment drops and cleans up
}
```

### Debugging Techniques

#### Enable Verbose Logging
```bash
# Set log level
export RUST_LOG=cim_domain_person=debug,cim_domain=trace

# Or in code
env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
```

#### Trace Event Flow
```rust
// Add correlation IDs to track event flow
let correlation_id = Uuid::new_v4();
info!("Processing command {} with correlation {}", command_type, correlation_id);
```

#### Inspect Event Store
```rust
// Debug helper to dump all events for a person
pub async fn debug_person_events(person_id: &PersonId) {
    let events = event_store.get_events(person_id).await.unwrap();
    for (index, event) in events.iter().enumerate() {
        println!("Event {}: {:?}", index, event);
    }
}
```

#### Component State Inspector
```rust
// Debug system to log component states
pub fn debug_person_components(
    query: Query<(&PersonId, Option<&ContactInfo>, Option<&Skills>)>,
) {
    for (person_id, contact, skills) in query.iter() {
        debug!("Person {:?}: contact={:?}, skills={:?}", 
               person_id, contact.is_some(), skills.is_some());
    }
}
```

## Performance Profiling

### Identify Bottlenecks
```rust
// Use timing macros
let start = Instant::now();
let result = expensive_operation().await;
debug!("Operation took: {:?}", start.elapsed());
```

### Memory Usage
```rust
// Monitor component counts
pub fn monitor_components(query: Query<Entity, With<PersonId>>) {
    let count = query.iter().count();
    if count > 10000 {
        warn!("High number of person entities: {}", count);
    }
}
```

## Getting Help

1. Check existing tests for usage examples
2. Review documentation in `docs/` folder
3. Enable debug logging for specific modules
4. Use the test environment for isolated reproduction
5. Check cross-domain event flows with NATS monitoring