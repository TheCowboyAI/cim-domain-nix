# Nix Domain API Documentation

## Overview

The Nix domain API provides event-driven commands, queries, and events for managing Nix flakes, packages, modules, overlays, and NixOS configurations within the CIM ecosystem. All operations follow CQRS patterns with proper correlation/causation tracking.

## Core Types

### MessageIdentity

All commands and events in the Nix domain include message identity for correlation and causation tracking:

```rust
use cim_domain_nix::value_objects::{MessageIdentity, MessageFactory};

// Create a root command (self-correlated)
let identity = MessageFactory::create_root_identity();

// Create a caused message (inherits correlation)
let caused_identity = MessageFactory::create_caused_identity(&parent_identity);
```

**Fields:**
- `message_id`: Unique identifier for this message
- `correlation_id`: Groups related messages together
- `causation_id`: Indicates what caused this message

## Commands

### CreateFlake

Creates a new Nix flake with a template.

```rust
use cim_domain_nix::commands::CreateFlake;
use cim_domain_nix::value_objects::MessageFactory;

let command = CreateFlake {
    identity: MessageFactory::create_root_identity(),
    path: PathBuf::from("/path/to/flake"),
    description: "My awesome flake".to_string(),
    template: Some("github:nixos/templates#rust".to_string()),
};
```

**Fields:**
- `identity`: Message identity for correlation/causation
- `path`: Path where the flake will be created
- `description`: Human-readable description
- `template`: Optional template to use (e.g., "github:nixos/templates#rust")

**Events Emitted:**
- `FlakeCreated`

### UpdateFlake

Updates an existing flake (typically `nix flake update`).

```rust
use cim_domain_nix::commands::UpdateFlake;

let command = UpdateFlake {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    path: PathBuf::from("/path/to/flake"),
};
```

**Fields:**
- `identity`: Message identity for correlation/causation
- `path`: Path to the flake to update

**Events Emitted:**
- `FlakeUpdated`

### AddFlakeInput

Adds an input to a flake.

```rust
use cim_domain_nix::commands::AddFlakeInput;

let command = AddFlakeInput {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    path: PathBuf::from("/path/to/flake"),
    name: "nixpkgs".to_string(),
    url: "github:NixOS/nixpkgs/nixos-unstable".to_string(),
};
```

**Fields:**
- `identity`: Message identity for correlation/causation
- `path`: Path to the flake
- `name`: Name of the input
- `url`: URL of the input flake

**Events Emitted:**
- `FlakeInputAdded`

### BuildPackage

Builds a package from a flake.

```rust
use cim_domain_nix::commands::BuildPackage;
use cim_domain_nix::value_objects::AttributePath;

let command = BuildPackage {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    flake_ref: "github:owner/repo".to_string(),
    attribute: AttributePath::from_str("packages.x86_64-linux.default"),
    output_path: None,
};
```

**Fields:**
- `identity`: Message identity for correlation/causation
- `flake_ref`: Flake reference to build from
- `attribute`: Attribute path to the package
- `output_path`: Optional output path override

**Events Emitted:**
- `PackageBuilt`

### CreateModule

Creates a new Nix module.

```rust
use cim_domain_nix::commands::CreateModule;
use cim_domain_nix::value_objects::NixModule;

let module = NixModule {
    id: Uuid::new_v4(),
    name: "my-service".to_string(),
    options: HashMap::new(),
    config: serde_json::json!({}),
    imports: vec![],
};

let command = CreateModule {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    name: "my-service".to_string(),
    module,
};
```

**Events Emitted:**
- `ModuleCreated`

### CreateOverlay

Creates a new package overlay.

```rust
use cim_domain_nix::commands::CreateOverlay;
use cim_domain_nix::value_objects::Overlay;

let overlay = Overlay {
    id: Uuid::new_v4(),
    name: "my-overlay".to_string(),
    overrides: HashMap::new(),
    additions: HashMap::new(),
};

let command = CreateOverlay {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    name: "my-overlay".to_string(),
    overlay,
};
```

**Events Emitted:**
- `OverlayCreated`

### CreateConfiguration

Creates a new NixOS configuration.

```rust
use cim_domain_nix::commands::CreateConfiguration;
use cim_domain_nix::value_objects::NixOSConfiguration;

let config = NixOSConfiguration {
    id: Uuid::new_v4(),
    name: "my-system".to_string(),
    system: "x86_64-linux".to_string(),
    path: PathBuf::from("/etc/nixos/configuration.nix"),
    hostname: "my-machine".to_string(),
    modules: vec![],
    overlays: vec![],
    packages: vec![],
    specialisations: HashMap::new(),
};

let command = CreateConfiguration {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    name: "my-system".to_string(),
    configuration: config,
};
```

**Events Emitted:**
- `ConfigurationCreated`

### ActivateConfiguration

Activates a NixOS configuration.

```rust
use cim_domain_nix::commands::ActivateConfiguration;
use cim_domain_nix::events::ActivationType;

let command = ActivateConfiguration {
    identity: MessageFactory::create_caused_identity(&parent_identity),
    name: "my-system".to_string(),
    activation_type: ActivationType::Switch,
};
```

**Activation Types:**
- `Switch`: Switch to configuration immediately
- `Boot`: Set as boot configuration
- `Test`: Test configuration without making permanent

**Events Emitted:**
- `ConfigurationActivated`

## Events

All events implement the `NixDomainEvent` trait and include correlation/causation IDs:

### FlakeCreated

```rust
pub struct FlakeCreated {
    pub flake_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub identity: MessageIdentity,
    pub path: PathBuf,
    pub description: String,
    pub template: Option<String>,
}
```

### FlakeUpdated

```rust
pub struct FlakeUpdated {
    pub flake_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub identity: MessageIdentity,
    pub path: PathBuf,
}
```

### FlakeInputAdded

```rust
pub struct FlakeInputAdded {
    pub flake_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub identity: MessageIdentity,
    pub path: PathBuf,
    pub input_name: String,
    pub input_url: String,
}
```

### PackageBuilt

```rust
pub struct PackageBuilt {
    pub package_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub identity: MessageIdentity,
    pub flake_ref: String,
    pub attribute: AttributePath,
    pub output_path: PathBuf,
    pub build_time: Duration,
}
```

### Event Factory

Use `NixEventFactory` for creating events with proper correlation:

```rust
use cim_domain_nix::events::NixEventFactory;

// Create a root event (for initial actions)
let event = NixEventFactory::create_flake_created_root(
    flake_id,
    path,
    description,
    template,
);

// Create a caused event (for subsequent actions)
let event = NixEventFactory::create_flake_updated(
    flake_id,
    path,
    &parent_identity,
);
```

## Value Objects

### FlakeRef

Reference to a Nix flake.

```rust
use cim_domain_nix::value_objects::FlakeRef;

let flake = FlakeRef::new("github:NixOS/nixpkgs")
    .with_revision("nixos-23.11")
    .with_subflake("lib");

assert_eq!(flake.to_string(), "github:NixOS/nixpkgs/nixos-23.11#lib");
```

### AttributePath

Nix attribute path (e.g., "packages.x86_64-linux.hello").

```rust
use cim_domain_nix::value_objects::AttributePath;

let path = AttributePath::from_str("packages.x86_64-linux.hello");
let segments = path.segments; // ["packages", "x86_64-linux", "hello"]
```

### StorePath

Parsed Nix store path.

```rust
use cim_domain_nix::value_objects::StorePath;

let path = StorePath::parse("/nix/store/abc123-hello-1.0")?;
assert_eq!(path.hash, "abc123");
assert_eq!(path.name, "hello-1.0");
```

## Services

### FlakeService

High-level operations for flakes.

```rust
use cim_domain_nix::services::FlakeService;

let service = FlakeService::new();

// Create a flake
service.create_flake(path, description, template).await?;

// Update flake inputs
service.update_flake(path).await?;

// Add an input
service.add_input(path, name, url).await?;
```

### BuildService

Package building operations.

```rust
use cim_domain_nix::services::BuildService;

let service = BuildService::new();

// Build a package
let result = service.build_package(flake_ref, attribute).await?;
println!("Built: {}", result.output_path.display());
```

### ConfigurationService

NixOS configuration management.

```rust
use cim_domain_nix::services::ConfigurationService;

let service = ConfigurationService::new();

// Create configuration
service.create_configuration(name, config).await?;

// Activate configuration
service.activate_configuration(name, ActivationType::Switch).await?;
```

## Parser API

### Parsing Nix Expressions

```rust
use cim_domain_nix::parser::{parse_nix_file, parse_nix_expr};

// Parse a file
let ast = parse_nix_file("flake.nix").await?;

// Parse an expression
let expr = parse_nix_expr("{ pkgs }: pkgs.hello")?;
```

### AST Manipulation

```rust
use cim_domain_nix::parser::manipulator::AstManipulator;

let manipulator = AstManipulator::new(ast);

// Add an attribute
manipulator.add_attribute("programs.git.enable", "true")?;

// Modify a value
manipulator.modify_value("description", "\"Updated description\"")?;

// Get the modified expression
let updated = manipulator.to_string();
```

## Analyzer API

### Security Analysis

```rust
use cim_domain_nix::analyzer::SecurityAnalyzer;

let analyzer = SecurityAnalyzer::new();
let issues = analyzer.analyze_file("configuration.nix").await?;

for issue in issues {
    println!("{}: {}", issue.severity, issue.message);
}
```

### Performance Analysis

```rust
use cim_domain_nix::analyzer::PerformanceAnalyzer;

let analyzer = PerformanceAnalyzer::new();
let suggestions = analyzer.analyze_flake("flake.nix").await?;
```

## Error Handling

```rust
use cim_domain_nix::NixDomainError;

#[derive(Error, Debug)]
pub enum NixDomainError {
    #[error("Flake not found at {0}")]
    FlakeNotFound(PathBuf),
    
    #[error("Invalid flake reference: {0}")]
    InvalidFlakeRef(String),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Nix command failed: {0}")]
    NixCommandFailed(String),
}
```

## Integration Examples

### Complete Workflow Example

```rust
use cim_domain_nix::{
    commands::{CreateFlake, BuildPackage},
    events::NixEventFactory,
    services::{FlakeService, BuildService},
    value_objects::{MessageFactory, AttributePath},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start with a root command
    let create_cmd = CreateFlake {
        identity: MessageFactory::create_root_identity(),
        path: PathBuf::from("./my-project"),
        description: "My Rust project".to_string(),
        template: Some("github:nixos/templates#rust".to_string()),
    };
    
    // Execute through service (which emits events)
    let flake_service = FlakeService::new();
    flake_service.handle_command(create_cmd).await?;
    
    // Build the package (caused by the creation)
    let build_cmd = BuildPackage {
        identity: MessageFactory::create_caused_identity(&create_cmd.identity),
        flake_ref: "./my-project".to_string(),
        attribute: AttributePath::from_str("packages.x86_64-linux.default"),
        output_path: None,
    };
    
    let build_service = BuildService::new();
    let result = build_service.handle_command(build_cmd).await?;
    
    println!("Package built at: {:?}", result.output_path);
    
    Ok(())
}
```

### NATS Integration (Future)

```rust
// Coming soon with NATS integration
use cim_domain_nix::nats::{NatsClient, EventPublisher};

let client = NatsClient::connect(config).await?;
let publisher = EventPublisher::new(client);

// Events will be automatically published
publisher.publish_event(&event).await?;
```

## Performance Considerations

- Parser operations are CPU-intensive; use async for I/O
- Build operations can be long-running; implement timeouts
- AST manipulation preserves formatting where possible
- Analyzers run in parallel using Rayon

## Security Considerations

- All file paths are validated before operations
- Nix expressions are parsed safely without evaluation
- Build operations run in Nix sandbox
- Sensitive data in configurations should be handled via secrets management