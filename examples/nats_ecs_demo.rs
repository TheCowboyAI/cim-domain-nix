//! Demo of NATS-ECS integration patterns for the Nix domain
//!
//! This example demonstrates how NATS subjects map to ECS systems
//! for distributed entity processing with correlation and causation tracking.

use chrono::{DateTime, Utc};
use cim_domain_nix::nats::{CommandAction, EventAction, NixSubject};
use cim_domain_nix::value_objects::{CausationId, CorrelationId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, Level};
use tracing_subscriber;
use uuid::Uuid;

// ===== ECS Components =====

/// Component trait for ECS
trait Component: Send + Sync + std::fmt::Debug {
    fn component_type(&self) -> ComponentType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ComponentType {
    Identity,
    Flake,
    Package,
    State,
    Path,
    Correlation,
}

/// Identity component - every entity has one
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentityComponent {
    entity_id: Uuid,
    created_at: DateTime<Utc>,
    entity_type: String,
}

impl Component for IdentityComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Identity
    }
}

/// Flake component
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FlakeComponent {
    flake_id: Uuid,
    description: String,
    inputs: HashMap<String, String>,
}

impl Component for FlakeComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Flake
    }
}

/// Package component
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackageComponent {
    package_id: Uuid,
    flake_ref: String,
    attribute: String,
}

impl Component for PackageComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Package
    }
}

/// State component for tracking entity state
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StateComponent {
    status: EntityStatus,
    last_modified: DateTime<Utc>,
    version: u64,
}

impl Component for StateComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::State
    }
}

/// Path component
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PathComponent {
    path: String,
}

impl Component for PathComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Path
    }
}

/// Correlation component for workflow tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CorrelationComponent {
    correlation_id: CorrelationId,
    workflow_name: String,
}

impl Component for CorrelationComponent {
    fn component_type(&self) -> ComponentType {
        ComponentType::Correlation
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum EntityStatus {
    Created,
    Processing,
    Completed,
    Failed,
}

// ===== Entity Store =====

/// Simple in-memory entity store
struct EntityStore {
    entities: RwLock<HashMap<Uuid, Entity>>,
}

#[derive(Debug)]
struct Entity {
    id: Uuid,
    components: HashMap<ComponentType, Box<dyn Component>>,
}

impl EntityStore {
    fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
        }
    }

    async fn create_entity(&self) -> Uuid {
        let id = Uuid::new_v4();
        let entity = Entity {
            id,
            components: HashMap::new(),
        };

        self.entities.write().await.insert(id, entity);
        id
    }

    async fn add_component(&self, entity_id: Uuid, component: Box<dyn Component>) {
        if let Some(entity) = self.entities.write().await.get_mut(&entity_id) {
            entity
                .components
                .insert(component.component_type(), component);
        }
    }

    async fn query_by_components(
        &self,
        required: Vec<ComponentType>,
        correlation: Option<CorrelationId>,
    ) -> Vec<Uuid> {
        let entities = self.entities.read().await;

        entities
            .values()
            .filter(|entity| {
                // Check required components
                let has_required = required.iter().all(|ct| entity.components.contains_key(ct));

                // Check correlation if specified
                let matches_correlation = correlation.map_or(true, |corr| {
                    entity
                        .components
                        .get(&ComponentType::Correlation)
                        .and_then(|c| c.as_any().downcast_ref::<CorrelationComponent>())
                        .map_or(false, |cc| cc.correlation_id == corr)
                });

                has_required && matches_correlation
            })
            .map(|e| e.id)
            .collect()
    }

    async fn get_entity(&self, entity_id: Uuid) -> Option<Entity> {
        let entities = self.entities.read().await;
        entities.get(&entity_id).map(|e| Entity {
            id: e.id,
            components: HashMap::new(), // Components aren't cloneable, so we return empty for now
        })
    }
}

// ===== ECS Systems =====

/// System trait for processing entities
#[async_trait::async_trait]
trait System: Send + Sync {
    /// Get the subject patterns this system subscribes to
    fn subject_patterns(&self) -> Vec<String>;

    /// Get required component types
    fn required_components(&self) -> Vec<ComponentType>;

    /// Process matching entities
    async fn process(
        &self,
        entity: &Entity,
        context: SystemContext,
    ) -> Result<Vec<SystemEvent>, Box<dyn std::error::Error>>;
}

#[derive(Debug, Clone)]
struct SystemContext {
    subject: String,
    correlation_id: Option<CorrelationId>,
    causation_id: Option<CausationId>,
    message_id: Uuid,
}

#[derive(Debug, Clone)]
struct SystemEvent {
    event_type: String,
    entity_id: Uuid,
    data: serde_json::Value,
}

/// Flake creation system
struct FlakeCreationSystem {
    store: Arc<EntityStore>,
}

#[async_trait::async_trait]
impl System for FlakeCreationSystem {
    fn subject_patterns(&self) -> Vec<String> {
        vec![NixSubject::command(CommandAction::CreateFlake).to_string()]
    }

    fn required_components(&self) -> Vec<ComponentType> {
        vec![] // No requirements for creation
    }

    async fn process(
        &self,
        _entity: &Entity,
        context: SystemContext,
    ) -> Result<Vec<SystemEvent>, Box<dyn std::error::Error>> {
        info!(
            "FlakeCreationSystem processing subject: {}",
            context.subject
        );

        // Create new flake entity
        let entity_id = self.store.create_entity().await;

        // Add components
        self.store
            .add_component(
                entity_id,
                Box::new(IdentityComponent {
                    entity_id,
                    created_at: Utc::now(),
                    entity_type: "flake".to_string(),
                }),
            )
            .await;

        self.store
            .add_component(
                entity_id,
                Box::new(FlakeComponent {
                    flake_id: entity_id,
                    description: "Demo flake".to_string(),
                    inputs: HashMap::new(),
                }),
            )
            .await;

        self.store
            .add_component(
                entity_id,
                Box::new(StateComponent {
                    status: EntityStatus::Created,
                    last_modified: Utc::now(),
                    version: 1,
                }),
            )
            .await;

        self.store
            .add_component(
                entity_id,
                Box::new(PathComponent {
                    path: "/tmp/demo-flake".to_string(),
                }),
            )
            .await;

        // Add correlation if present
        if let Some(corr) = context.correlation_id {
            self.store
                .add_component(
                    entity_id,
                    Box::new(CorrelationComponent {
                        correlation_id: corr,
                        workflow_name: "demo-workflow".to_string(),
                    }),
                )
                .await;
        }

        // Return event
        Ok(vec![SystemEvent {
            event_type: "FlakeCreated".to_string(),
            entity_id,
            data: serde_json::json!({
                "flake_id": entity_id,
                "path": "/tmp/demo-flake",
            }),
        }])
    }
}

/// Package build system
struct PackageBuildSystem {
    store: Arc<EntityStore>,
}

#[async_trait::async_trait]
impl System for PackageBuildSystem {
    fn subject_patterns(&self) -> Vec<String> {
        vec![
            NixSubject::command(CommandAction::BuildPackage).to_string(),
            NixSubject::event(EventAction::FlakeCreated).to_string(),
        ]
    }

    fn required_components(&self) -> Vec<ComponentType> {
        vec![ComponentType::Flake, ComponentType::State]
    }

    async fn process(
        &self,
        entity: &Entity,
        context: SystemContext,
    ) -> Result<Vec<SystemEvent>, Box<dyn std::error::Error>> {
        info!("PackageBuildSystem processing entity: {:?}", entity.id);

        // Create package entity
        let package_id = self.store.create_entity().await;

        self.store
            .add_component(
                package_id,
                Box::new(IdentityComponent {
                    entity_id: package_id,
                    created_at: Utc::now(),
                    entity_type: "package".to_string(),
                }),
            )
            .await;

        self.store
            .add_component(
                package_id,
                Box::new(PackageComponent {
                    package_id,
                    flake_ref: format!("flake:{}", entity.id),
                    attribute: "defaultPackage".to_string(),
                }),
            )
            .await;

        self.store
            .add_component(
                package_id,
                Box::new(StateComponent {
                    status: EntityStatus::Processing,
                    last_modified: Utc::now(),
                    version: 1,
                }),
            )
            .await;

        // Inherit correlation
        if let Some(corr_comp) = entity.components.get(&ComponentType::Correlation) {
            if let Some(corr) = corr_comp.as_any().downcast_ref::<CorrelationComponent>() {
                self.store
                    .add_component(package_id, Box::new(corr.clone()))
                    .await;
            }
        }

        Ok(vec![SystemEvent {
            event_type: "PackageBuilt".to_string(),
            entity_id: package_id,
            data: serde_json::json!({
                "package_id": package_id,
                "flake_entity": entity.id,
            }),
        }])
    }
}

// ===== System Scheduler =====

/// Manages system execution based on NATS subjects
struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
    store: Arc<EntityStore>,
}

impl SystemScheduler {
    fn new(store: Arc<EntityStore>) -> Self {
        Self {
            systems: Vec::new(),
            store,
        }
    }

    fn register_system(&mut self, system: Box<dyn System>) {
        info!(
            "Registered system with patterns: {:?}",
            system.subject_patterns()
        );
        self.systems.push(system);
    }

    async fn handle_subject(
        &self,
        subject: &str,
        correlation_id: Option<CorrelationId>,
        causation_id: Option<CausationId>,
    ) -> Vec<SystemEvent> {
        let mut all_events = Vec::new();

        for system in &self.systems {
            // Check if system handles this subject
            if system.subject_patterns().iter().any(|p| p == subject) {
                debug!("System matches subject {}", subject);

                // Query entities with required components
                let entities = self
                    .store
                    .query_by_components(system.required_components(), correlation_id)
                    .await;

                debug!("Found {} matching entities", entities.len());

                // Process each entity
                for entity_id in entities {
                    let context = SystemContext {
                        subject: subject.to_string(),
                        correlation_id,
                        causation_id,
                        message_id: Uuid::new_v4(),
                    };

                    // Get the actual entity
                    if let Some(entity) = self.store.get_entity(entity_id).await {
                        match system.process(&entity, context).await {
                            Ok(events) => all_events.extend(events),
                            Err(e) => eprintln!("System error: {}", e),
                        }
                    }
                }

                // Also process with no entity for creation systems
                if system.required_components().is_empty() {
                    let context = SystemContext {
                        subject: subject.to_string(),
                        correlation_id,
                        causation_id,
                        message_id: Uuid::new_v4(),
                    };

                    let empty_entity = Entity {
                        id: Uuid::nil(),
                        components: HashMap::new(),
                    };

                    match system.process(&empty_entity, context).await {
                        Ok(events) => all_events.extend(events),
                        Err(e) => eprintln!("System error: {}", e),
                    }
                }
            }
        }

        all_events
    }
}

// ===== Trait implementations for downcasting =====

trait ComponentExt {
    fn as_any(&self) -> &dyn std::any::Any;
}

impl ComponentExt for IdentityComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ComponentExt for FlakeComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ComponentExt for PackageComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ComponentExt for StateComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ComponentExt for PathComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ComponentExt for CorrelationComponent {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Make Component object-safe
impl dyn Component {
    fn as_any(&self) -> &dyn std::any::Any {
        // This would need proper implementation
        unimplemented!()
    }
}

// ===== Main Demo =====

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    println!("=== NATS-ECS Integration Demo ===\n");

    // Create entity store
    let store = Arc::new(EntityStore::new());

    // Create system scheduler
    let mut scheduler = SystemScheduler::new(store.clone());

    // Register systems
    scheduler.register_system(Box::new(FlakeCreationSystem {
        store: store.clone(),
    }));

    scheduler.register_system(Box::new(PackageBuildSystem {
        store: store.clone(),
    }));

    // Simulate workflow with correlation
    let workflow_correlation = CorrelationId::new();
    println!(
        "Starting workflow with correlation: {}",
        workflow_correlation
    );

    // Step 1: Create flake command
    println!("\n1. Processing CreateFlake command");
    let create_subject = NixSubject::command(CommandAction::CreateFlake).to_string();
    let events = scheduler
        .handle_subject(&create_subject, Some(workflow_correlation), None)
        .await;

    for event in &events {
        println!(
            "  → Event: {} for entity {}",
            event.event_type, event.entity_id
        );
    }

    // Step 2: Flake created event triggers package build
    println!("\n2. Processing FlakeCreated event");
    let created_subject = NixSubject::event(EventAction::FlakeCreated).to_string();
    let events = scheduler
        .handle_subject(
            &created_subject,
            Some(workflow_correlation),
            Some(CausationId::new()), // Caused by create command
        )
        .await;

    for event in &events {
        println!(
            "  → Event: {} for entity {}",
            event.event_type, event.entity_id
        );
    }

    // Display entity state
    println!("\n3. Final Entity State:");
    let all_entities = store.entities.read().await;
    for (id, entity) in all_entities.iter() {
        println!("\nEntity {}", id);
        for (comp_type, _) in &entity.components {
            println!("  - Component: {:?}", comp_type);
        }
    }

    // Query by correlation
    println!("\n4. Entities in workflow {}:", workflow_correlation);
    let workflow_entities = store
        .query_by_components(vec![ComponentType::Correlation], Some(workflow_correlation))
        .await;

    println!("Found {} entities in workflow", workflow_entities.len());

    println!("\n=== Demo Complete ===");
    println!("\nKey Concepts Demonstrated:");
    println!("- Systems subscribe to NATS subjects");
    println!("- Entities are created with components");
    println!("- Correlation groups related entities");
    println!("- Systems query entities by components");
    println!("- Events flow through the system");

    Ok(())
}
