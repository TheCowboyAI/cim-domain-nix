# Common Development Tasks

## Adding a New Component Type

### 1. Define the Component
```rust
// In src/components/mod.rs
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct NewComponent {
    pub field1: String,
    pub field2: Option<i32>,
    // Add fields as needed
}
```

### 2. Create Commands
```rust
// In src/commands/mod.rs
#[derive(Debug, Clone)]
pub struct AddNewComponent {
    pub person_id: PersonId,
    pub component: NewComponent,
}

#[derive(Debug, Clone)]
pub struct UpdateNewComponent {
    pub person_id: PersonId,
    pub component: NewComponent,
}
```

### 3. Add Events
```rust
// In src/events/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonEvent {
    // ... existing events
    NewComponentAdded {
        person_id: PersonId,
        component: NewComponent,
        timestamp: DateTime<Utc>,
    },
    NewComponentUpdated {
        person_id: PersonId,
        component: NewComponent,
        timestamp: DateTime<Utc>,
    },
}
```

### 4. Implement Command Handler
```rust
// In src/commands/handlers/new_component.rs
pub async fn handle_add_new_component(
    command: AddNewComponent,
    person_repo: &PersonRepository,
    event_store: &EventStore,
) -> Result<()> {
    // Validate person exists
    let person = person_repo.get(&command.person_id)
        .ok_or(PersonError::NotFound)?;
    
    // Create event
    let event = PersonEvent::NewComponentAdded {
        person_id: command.person_id,
        component: command.component,
        timestamp: Utc::now(),
    };
    
    // Store event
    event_store.append(&command.person_id, event).await?;
    
    Ok(())
}
```

### 5. Update Systems
```rust
// In src/services/systems/component_sync.rs
pub fn sync_new_component(
    mut commands: Commands,
    events: EventReader<PersonEvent>,
    query: Query<Entity, With<PersonId>>,
) {
    for event in events.iter() {
        match event {
            PersonEvent::NewComponentAdded { person_id, component, .. } => {
                // Find entity and add component
                if let Some(entity) = find_person_entity(&query, person_id) {
                    commands.entity(entity).insert(component.clone());
                }
            }
            // Handle other events
        }
    }
}
```

### 6. Add Tests
```rust
// In tests/components/new_component_test.rs
#[tokio::test]
async fn test_add_new_component() {
    let test_env = TestEnvironment::new().await;
    
    // Create person
    let person_id = PersonId::new();
    test_env.create_person(person_id, "Test User").await;
    
    // Add component
    let component = NewComponent {
        field1: "value".to_string(),
        field2: Some(42),
    };
    
    let command = AddNewComponent { person_id, component };
    test_env.send_command(command).await.unwrap();
    
    // Verify component added
    let person_view = test_env.get_person_view(person_id).await.unwrap();
    assert!(person_view.has_component::<NewComponent>());
}
```

## Implementing a New Projection

### 1. Define Projection Structure
```rust
// In src/projections/new_projection.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewProjection {
    pub person_id: PersonId,
    pub computed_field: String,
    pub aggregated_data: Vec<DataPoint>,
    pub last_updated: DateTime<Utc>,
}
```

### 2. Create Projection Handler
```rust
// In src/projections/handlers/new_projection.rs
pub struct NewProjectionHandler {
    storage: ProjectionStorage<NewProjection>,
}

impl EventHandler for NewProjectionHandler {
    async fn handle(&mut self, event: PersonEvent) -> Result<()> {
        match event {
            PersonEvent::Created { id, name, timestamp } => {
                let projection = NewProjection {
                    person_id: id,
                    computed_field: compute_initial_value(&name),
                    aggregated_data: vec![],
                    last_updated: timestamp,
                };
                self.storage.save(&id, projection).await?;
            }
            PersonEvent::SomeRelevantEvent { person_id, data, timestamp } => {
                if let Some(mut projection) = self.storage.get(&person_id).await? {
                    projection.aggregated_data.push(process_data(data));
                    projection.last_updated = timestamp;
                    self.storage.save(&person_id, projection).await?;
                }
            }
            // Handle other relevant events
        }
        Ok(())
    }
}
```

### 3. Add Query Support
```rust
// In src/queries/projections.rs
pub struct GetNewProjection {
    pub person_id: PersonId,
}

pub async fn handle_get_new_projection(
    query: GetNewProjection,
    projection_store: &ProjectionStore,
) -> Result<Option<NewProjection>> {
    projection_store.get::<NewProjection>(&query.person_id).await
}
```

### 4. Register in Application
```rust
// In src/infrastructure/app_state.rs
// Add to projection handlers initialization
let new_projection_handler = NewProjectionHandler::new(storage.clone());
event_processor.register_handler(Box::new(new_projection_handler));
```

## Adding Cross-Domain Integration

### 1. Define External Event Handler
```rust
// In src/cross_domain/handlers/new_domain.rs
pub struct NewDomainEventHandler {
    command_bus: CommandBus,
}

impl ExternalEventHandler for NewDomainEventHandler {
    async fn handle(&self, event: ExternalEvent) -> Result<()> {
        match event {
            ExternalEvent::NewDomain(NewDomainEvent::SomethingHappened { 
                external_id, 
                data 
            }) => {
                // Transform to internal command
                let command = UpdatePersonFromExternal {
                    person_id: map_external_to_person_id(external_id)?,
                    data: transform_data(data),
                };
                self.command_bus.send(command).await?;
            }
            _ => {} // Ignore other events
        }
        Ok(())
    }
}
```

### 2. Configure Event Subscriptions
```rust
// In src/infrastructure/messaging/subscriptions.rs
pub async fn setup_subscriptions(nats_client: &NatsClient) -> Result<()> {
    // Subscribe to new domain events
    nats_client.subscribe("new_domain.events.*", |event| {
        // Route to handler
    }).await?;
    
    Ok(())
}
```

### 3. Add Integration Tests
```rust
// In tests/cross_domain/new_domain_test.rs
#[tokio::test]
async fn test_new_domain_integration() {
    let test_env = TestEnvironment::new().await;
    
    // Simulate external event
    let external_event = NewDomainEvent::SomethingHappened {
        external_id: "ext123".to_string(),
        data: TestData::default(),
    };
    
    test_env.publish_external_event(external_event).await;
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify person updated
    let person_id = map_external_to_person_id("ext123").unwrap();
    let person = test_env.get_person(person_id).await.unwrap();
    assert_eq!(person.some_field, expected_value);
}
```

## Writing Tests

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_creation() {
        let component = NewComponent {
            field1: "test".to_string(),
            field2: None,
        };
        
        assert_eq!(component.field1, "test");
        assert!(component.field2.is_none());
    }
    
    #[test]
    fn test_validation() {
        let result = validate_component(&NewComponent {
            field1: "".to_string(), // Invalid
            field2: Some(-1), // Invalid
        });
        
        assert!(result.is_err());
    }
}
```

### Integration Test Template
```rust
#[tokio::test]
async fn test_complete_flow() {
    let env = TestEnvironment::new().await;
    
    // Setup
    let person_id = env.create_test_person().await;
    
    // Execute
    env.send_command(TestCommand { person_id }).await.unwrap();
    
    // Verify
    let events = env.get_person_events(person_id).await;
    assert!(events.iter().any(|e| matches!(e, PersonEvent::TestEventOccurred { .. })));
    
    // Cleanup handled by TestEnvironment
}
```