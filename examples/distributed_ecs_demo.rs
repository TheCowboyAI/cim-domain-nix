//! Advanced demo of distributed ECS patterns with NATS
//!
//! This example shows how multiple services can coordinate through
//! NATS subjects to implement a distributed Entity Component System.

use cim_domain_nix::commands::NixCommand;
use cim_domain_nix::nats::{
    CommandAction, CommandHandler, CommandSubscriber, EventAction, EventHandler, EventPublisher,
    NatsClient, NatsConfig, NixSubject, SubjectMapper,
};
// use cim_domain_nix::events::NixDomainEvent;
use async_nats::header::HeaderMap;
use cim_domain_nix::value_objects::{CorrelationId, MessageIdentity};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn, Level};
use uuid::Uuid;

// ===== Distributed Entity Registry =====

/// Registry that tracks entities across distributed services
#[derive(Clone)]
struct DistributedEntityRegistry {
    client: async_nats::Client,
    local_cache: Arc<RwLock<std::collections::HashMap<Uuid, EntityMetadata>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EntityMetadata {
    entity_id: Uuid,
    entity_type: String,
    owning_service: String,
    correlation_id: Option<CorrelationId>,
    components: Vec<String>,
    last_updated: chrono::DateTime<chrono::Utc>,
}

impl DistributedEntityRegistry {
    async fn new(client: async_nats::Client) -> Self {
        let registry = Self {
            client,
            local_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        };

        // Subscribe to entity updates
        let mut sub = registry
            .client
            .subscribe("entity.registry.>")
            .await
            .unwrap();
        let cache = registry.local_cache.clone();

        tokio::spawn(async move {
            while let Some(msg) = sub.next().await {
                if let Ok(metadata) = serde_json::from_slice::<EntityMetadata>(&msg.payload) {
                    cache.write().await.insert(metadata.entity_id, metadata);
                }
            }
        });

        registry
    }

    async fn register_entity(
        &self,
        metadata: EntityMetadata,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Store locally
        self.local_cache
            .write()
            .await
            .insert(metadata.entity_id, metadata.clone());

        // Broadcast to other services
        let subject = format!("entity.registry.{}.registered", metadata.entity_type);
        self.client
            .publish(subject, serde_json::to_vec(&metadata)?.into())
            .await?;

        Ok(())
    }

    async fn query_by_correlation(&self, correlation_id: CorrelationId) -> Vec<EntityMetadata> {
        self.local_cache
            .read()
            .await
            .values()
            .filter(|m| m.correlation_id == Some(correlation_id))
            .cloned()
            .collect()
    }
}

// ===== Distributed System Coordinator =====

/// Coordinates system execution across services
struct DistributedSystemCoordinator {
    service_id: String,
    client: async_nats::Client,
    registry: DistributedEntityRegistry,
}

impl DistributedSystemCoordinator {
    async fn new(service_id: String, client: async_nats::Client) -> Self {
        let registry = DistributedEntityRegistry::new(client.clone()).await;

        Self {
            service_id,
            client,
            registry,
        }
    }

    /// Claim ownership of entity processing
    async fn claim_entity_processing(
        &self,
        entity_id: Uuid,
        subject: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let claim_subject = format!("system.claim.{}", entity_id);
        let claim = serde_json::json!({
            "service_id": self.service_id,
            "entity_id": entity_id,
            "subject": subject,
            "timestamp": chrono::Utc::now(),
        });

        // Try to claim with request/reply
        match self
            .client
            .request(claim_subject, serde_json::to_vec(&claim)?.into())
            .await
        {
            Ok(response) => {
                let claimed: bool = serde_json::from_slice(&response.payload)?;
                Ok(claimed)
            }
            Err(_) => {
                // No other service contested, we own it
                Ok(true)
            }
        }
    }

    /// Distribute work across services
    async fn distribute_work(
        &self,
        subject: &str,
        entities: Vec<Uuid>,
    ) -> std::collections::HashMap<Uuid, String> {
        let mut assignments = std::collections::HashMap::new();

        for entity_id in entities {
            if let Ok(claimed) = self.claim_entity_processing(entity_id, subject).await {
                if claimed {
                    assignments.insert(entity_id, self.service_id.clone());
                    info!("Service {} claimed entity {}", self.service_id, entity_id);
                }
            }
        }

        assignments
    }
}

// ===== Workflow Orchestrator =====

/// Orchestrates complex workflows across entities
#[derive(Clone)]
struct WorkflowOrchestrator {
    client: async_nats::Client,
    workflows: Arc<RwLock<std::collections::HashMap<CorrelationId, WorkflowState>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowState {
    workflow_id: CorrelationId,
    name: String,
    steps: Vec<WorkflowStep>,
    current_step: usize,
    entities: Vec<Uuid>,
    started_at: chrono::DateTime<chrono::Utc>,
    status: WorkflowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkflowStep {
    name: String,
    subject: String,
    required_components: Vec<String>,
    timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl WorkflowOrchestrator {
    fn new(client: async_nats::Client) -> Self {
        Self {
            client,
            workflows: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    async fn start_workflow(
        &self,
        name: String,
        steps: Vec<WorkflowStep>,
    ) -> Result<CorrelationId, Box<dyn std::error::Error>> {
        let workflow_id = CorrelationId::new();
        let workflow = WorkflowState {
            workflow_id,
            name: name.clone(),
            steps,
            current_step: 0,
            entities: Vec::new(),
            started_at: chrono::Utc::now(),
            status: WorkflowStatus::Running,
        };

        self.workflows
            .write()
            .await
            .insert(workflow_id, workflow.clone());

        // Broadcast workflow start
        self.client
            .publish("workflow.started", serde_json::to_vec(&workflow)?.into())
            .await?;

        info!("Started workflow '{}' with ID {}", name, workflow_id);

        // Start first step
        self.execute_current_step(workflow_id).await?;

        Ok(workflow_id)
    }

    async fn execute_current_step(
        &self,
        workflow_id: CorrelationId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workflows = self.workflows.read().await;
        if let Some(workflow) = workflows.get(&workflow_id) {
            if workflow.current_step < workflow.steps.len() {
                let step = &workflow.steps[workflow.current_step];

                // Publish command with correlation
                let mut headers = HeaderMap::new();
                headers.insert("X-Correlation-ID", workflow_id.to_string().as_str());
                headers.insert("X-Workflow-Name", workflow.name.as_str());
                headers.insert("X-Workflow-Step", step.name.as_str());

                self.client
                    .publish_with_headers(
                        step.subject.clone(),
                        headers,
                        vec![].into(), // Actual command would go here
                    )
                    .await?;

                info!(
                    "Executed workflow step '{}' with subject {}",
                    step.name, step.subject
                );
            }
        }

        Ok(())
    }

    async fn advance_workflow(
        &self,
        workflow_id: CorrelationId,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(&workflow_id) {
            workflow.current_step += 1;

            if workflow.current_step >= workflow.steps.len() {
                workflow.status = WorkflowStatus::Completed;
                info!("Workflow {} completed", workflow_id);
            } else {
                drop(workflows); // Release lock before async call
                self.execute_current_step(workflow_id).await?;
            }
        }

        Ok(())
    }
}

// ===== Service Mesh Integration =====

/// Represents a service in the mesh
struct MeshService {
    service_id: String,
    client: async_nats::Client,
    coordinator: DistributedSystemCoordinator,
    orchestrator: WorkflowOrchestrator,
    publisher: Arc<EventPublisher>,
}

impl MeshService {
    async fn new(service_id: String, nats_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = async_nats::connect(nats_url).await?;
        let coordinator =
            DistributedSystemCoordinator::new(service_id.clone(), client.clone()).await;
        let orchestrator = WorkflowOrchestrator::new(client.clone());
        let publisher = Arc::new(EventPublisher::new(client.clone(), "nix".to_string()));

        Ok(Self {
            service_id,
            client,
            coordinator,
            orchestrator,
            publisher,
        })
    }

    async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting mesh service: {}", self.service_id);

        // Subscribe to workflow events
        let mut workflow_sub = self.client.subscribe("workflow.>").await?;
        let orchestrator = self.orchestrator.clone();

        tokio::spawn(async move {
            while let Some(msg) = workflow_sub.next().await {
                if msg.subject.ends_with(".completed") {
                    // Extract workflow ID and advance
                    if let Some(header) =
                        msg.headers.as_ref().and_then(|h| h.get("X-Correlation-ID"))
                    {
                        if let Ok(workflow_id) = header.as_str().parse::<Uuid>() {
                            let _ = orchestrator
                                .advance_workflow(CorrelationId(workflow_id))
                                .await;
                        }
                    }
                }
            }
        });

        // Subscribe to health checks
        let service_id = self.service_id.clone();
        let mut health_sub = self
            .client
            .subscribe(format!("health.{}", service_id))
            .await?;
        let client = self.client.clone();

        tokio::spawn(async move {
            while let Some(msg) = health_sub.next().await {
                let health = serde_json::json!({
                    "service_id": service_id,
                    "status": "healthy",
                    "timestamp": chrono::Utc::now(),
                });

                if let Some(reply) = msg.reply {
                    let _ = client
                        .publish(reply, serde_json::to_vec(&health).unwrap().into())
                        .await;
                }
            }
        });

        Ok(())
    }
}

// ===== Demo Scenarios =====

async fn demo_distributed_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Distributed Workflow Demo ===");

    // Create three services
    let mut service_a = MeshService::new("service-a".to_string(), "nats://localhost:4222").await?;
    let mut service_b = MeshService::new("service-b".to_string(), "nats://localhost:4222").await?;
    let mut service_c = MeshService::new("service-c".to_string(), "nats://localhost:4222").await?;

    // Start all services
    service_a.start().await?;
    service_b.start().await?;
    service_c.start().await?;

    // Define a multi-step workflow
    let workflow_steps = vec![
        WorkflowStep {
            name: "Create Flake".to_string(),
            subject: NixSubject::command(CommandAction::CreateFlake).to_string(),
            required_components: vec![],
            timeout: Duration::from_secs(30),
        },
        WorkflowStep {
            name: "Build Package".to_string(),
            subject: NixSubject::command(CommandAction::BuildPackage).to_string(),
            required_components: vec!["FlakeComponent".to_string()],
            timeout: Duration::from_secs(300),
        },
        WorkflowStep {
            name: "Create Configuration".to_string(),
            subject: NixSubject::command(CommandAction::CreateConfiguration).to_string(),
            required_components: vec!["PackageComponent".to_string()],
            timeout: Duration::from_secs(60),
        },
    ];

    // Start workflow on service A
    let workflow_id = service_a
        .orchestrator
        .start_workflow("Deploy Application".to_string(), workflow_steps)
        .await?;

    println!("Started workflow: {}", workflow_id);

    // Simulate workflow progression
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Query entities in workflow across all services
    println!("\nQuerying entities in workflow across services:");

    for service in [&service_a, &service_b, &service_c] {
        let entities = service
            .coordinator
            .registry
            .query_by_correlation(workflow_id)
            .await;
        println!(
            "  Service {}: {} entities",
            service.service_id,
            entities.len()
        );
    }

    Ok(())
}

async fn demo_entity_migration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Entity Migration Demo ===");

    // This would demonstrate how entities can migrate between services
    // based on load balancing or failure scenarios

    println!("Entity migration demo (simplified)");
    println!("- Service A creates entity");
    println!("- Service A becomes overloaded");
    println!("- Entity migrates to Service B");
    println!("- Service B continues processing");

    Ok(())
}

// ===== Main Demo =====

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    println!("=== Distributed ECS with NATS Demo ===");
    println!("\nThis demo requires NATS to be running:");
    println!("  docker run -p 4222:4222 nats:latest\n");

    // Try to connect to NATS
    match async_nats::connect("nats://localhost:4222").await {
        Ok(_) => {
            println!("✓ Connected to NATS\n");

            // Run demos
            if let Err(e) = demo_distributed_workflow().await {
                println!("Workflow demo error: {}", e);
            }

            if let Err(e) = demo_entity_migration().await {
                println!("Migration demo error: {}", e);
            }
        }
        Err(e) => {
            println!("✗ Could not connect to NATS: {}", e);
            println!("\nThis demo shows how distributed ECS would work:");
            println!("- Multiple services coordinate through NATS");
            println!("- Entities are distributed across services");
            println!("- Workflows orchestrate multi-step processes");
            println!("- Services claim entity ownership dynamically");
            println!("- Entity state is synchronized via events");
        }
    }

    println!("\n=== Key Concepts Demonstrated ===");
    println!("1. Distributed Entity Registry");
    println!("2. System Coordination across services");
    println!("3. Workflow Orchestration");
    println!("4. Entity ownership and migration");
    println!("5. Service mesh patterns");

    Ok(())
}
