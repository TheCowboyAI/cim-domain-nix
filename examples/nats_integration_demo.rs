//! Demo of NATS integration for the Nix domain

use cim_domain_nix::nats::{
    NatsClient, NatsConfig, EventPublisher, CommandSubscriber, HealthService,
    ServiceDiscovery, ServiceInfo, CommandHandler, SubjectMapper,
};
use cim_domain_nix::commands::NixCommand;
use cim_domain_nix::events::{NixDomainEvent, FlakeCreated};
use cim_domain_nix::value_objects::MessageIdentity;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;
use uuid::Uuid;
use chrono::Utc;
use std::path::PathBuf;

/// Demo command handler
struct DemoCommandHandler;

#[async_trait::async_trait]
impl CommandHandler for DemoCommandHandler {
    async fn handle_command(
        &self,
        command: Box<dyn NixCommand>,
    ) -> cim_domain_nix::nats::Result<Vec<Box<dyn NixDomainEvent>>> {
        info!("Handling command: {:?}", command.command_id());
        
        // Create a demo event in response
        let event = FlakeCreated {
            flake_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            identity: MessageIdentity::new_caused_by(command.identity()),
            path: PathBuf::from("/tmp/demo"),
            description: "Demo flake".to_string(),
            template: None,
        };
        
        Ok(vec![Box::new(event) as Box<dyn NixDomainEvent>])
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    println!("=== Nix Domain NATS Integration Demo ===\n");
    
    // Check if NATS is available
    println!("Attempting to connect to NATS...");
    println!("Make sure NATS is running: docker run -p 4222:4222 nats:latest\n");
    
    // Create configuration
    let config = NatsConfig::from_env();
    
    // Connect to NATS
    let client = match NatsClient::connect(config.clone()).await {
        Ok(client) => {
            println!("✓ Connected to NATS successfully!");
            client
        }
        Err(e) => {
            println!("✗ Failed to connect to NATS: {}", e);
            println!("\nTo run this demo, start NATS with:");
            println!("  docker run -p 4222:4222 nats:latest");
            return Ok(());
        }
    };
    
    // Create event publisher
    let publisher = Arc::new(EventPublisher::new(
        client.client().clone(),
        config.subject_prefix.clone(),
    ));
    
    // Create and start health service
    let mut health_service = HealthService::new(
        client.client().clone(),
        config.service.name.clone(),
        config.service.version.clone(),
    );
    health_service.start().await?;
    println!("✓ Health service started");
    
    // Create and start service discovery
    let service_info = ServiceInfo {
        name: config.service.name.clone(),
        version: config.service.version.clone(),
        instance_id: config.service.instance_id.clone(),
        description: config.service.description.clone(),
        capabilities: vec![
            "flake.create".to_string(),
            "package.build".to_string(),
            "config.activate".to_string(),
        ],
        endpoints: SubjectMapper::all_command_subjects()
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    };
    
    let mut discovery = ServiceDiscovery::new(
        client.client().clone(),
        service_info,
    );
    discovery.start().await?;
    println!("✓ Service discovery started");
    
    // Create command handler
    let handler = Arc::new(DemoCommandHandler);
    
    // Create and start command subscriber
    let mut command_subscriber = CommandSubscriber::new(
        client.client().clone(),
        handler,
        publisher.clone(),
    );
    
    // Note: This would actually subscribe, but we'll skip it for the demo
    // command_subscriber.start().await?;
    println!("✓ Command subscriber created (not started in demo)");
    
    // Display available subjects
    println!("\n=== Available NATS Subjects ===");
    
    println!("\nCommand Subjects ({}):", SubjectMapper::all_command_subjects().len());
    for (i, subject) in SubjectMapper::all_command_subjects().iter().enumerate() {
        if i < 5 {
            println!("  - {}", subject);
        }
    }
    println!("  ... and {} more", SubjectMapper::all_command_subjects().len() - 5);
    
    println!("\nEvent Subjects ({}):", SubjectMapper::all_event_subjects().len());
    for (i, subject) in SubjectMapper::all_event_subjects().iter().enumerate() {
        if i < 5 {
            println!("  - {}", subject);
        }
    }
    println!("  ... and {} more", SubjectMapper::all_event_subjects().len() - 5);
    
    println!("\nSpecial Subjects:");
    println!("  - health.{}", config.service.name);
    println!("  - discovery.{}", config.service.name);
    println!("  - discovery.announce");
    
    // Demonstrate publishing a test event
    println!("\n=== Publishing Test Event ===");
    let test_event = FlakeCreated {
        flake_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        identity: MessageIdentity::new_root(),
        path: PathBuf::from("/tmp/test-flake"),
        description: "Test flake created via NATS".to_string(),
        template: Some("minimal".to_string()),
    };
    
    match publisher.publish_event(&test_event).await {
        Ok(_) => println!("✓ Test event published successfully"),
        Err(e) => println!("✗ Failed to publish test event: {}", e),
    }
    
    // Flush to ensure everything is sent
    publisher.flush().await?;
    
    println!("\n=== Demo Complete ===");
    println!("\nThe Nix domain is now ready for distributed operation via NATS!");
    println!("Services can subscribe to command subjects to handle operations.");
    println!("Events are automatically published for other services to consume.");
    
    Ok(())
}