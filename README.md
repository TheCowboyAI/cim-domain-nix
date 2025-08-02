<!-- Copyright 2025 Cowboy AI, LLC. -->

# CIM Domain Nix

[![CI](https://github.com/thecowboyai/cim-domain-nix/actions/workflows/ci.yml/badge.svg)](https://github.com/thecowboyai/cim-domain-nix/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/cim-domain-nix.svg)](https://crates.io/crates/cim-domain-nix)
[![Documentation](https://docs.rs/cim-domain-nix/badge.svg)](https://docs.rs/cim-domain-nix)
[![Test Coverage](https://img.shields.io/codecov/c/github/thecowboyai/cim-domain-nix)](https://codecov.io/gh/thecowboyai/cim-domain-nix)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Domain-Driven Design module for comprehensive Nix ecosystem integration within the CIM (Composable Information Machine) architecture.

## Overview

`cim-domain-nix` provides event-driven Nix package manager integration, enabling:

- **Flake Management**: Create, update, and analyze Nix flakes with full AST parsing
- **NixOS Configuration**: Generate and manage NixOS system configurations
- **Network-to-System**: Automatic NixOS generation from network topology events
- **Code Analysis**: Security, performance, and dead code analysis for Nix files
- **Formatting**: Integration with nixpkgs-fmt, alejandra, and nixfmt
- **Event Sourcing**: Full correlation/causation tracking per CIM standards
- **NATS Integration**: Distributed command processing and event streaming

## Architecture

The module follows Domain-Driven Design principles with:

```
cim-domain-nix/
├── aggregates/          # Domain aggregates (Flake, Module, Configuration)
├── commands/            # CQRS command types with MessageIdentity
├── events/              # Domain events with correlation/causation
├── handlers/            # Command and query handlers
├── services/            # High-level domain services
├── value_objects/       # Immutable domain values
├── analyzers/           # Code analysis tools
├── formatter/           # Code formatting integration
├── parser/              # AST parsing with rnix
├── network/             # Network topology integration
└── nats/                # NATS pub/sub with 46 mapped subjects
```

## Features

### Core Capabilities
- **Flake Operations**: Create, update, build, and analyze Nix flakes
- **AST Manipulation**: Parse and modify Nix expressions while preserving formatting
- **Dependency Analysis**: Track and visualize flake input dependencies
- **Security Scanning**: Detect insecure patterns and vulnerable configurations
- **Performance Analysis**: Identify evaluation bottlenecks and optimization opportunities
- **Dead Code Detection**: Find unused bindings and functions

### Network Integration
Automatically generate NixOS configurations from network topology:
- Hierarchical network support (client→leaf→cluster→super-cluster)
- Service configuration based on node roles
- Dynamic network change handling
- Firewall and routing automation

### Event-Driven Architecture
- 46 NATS subjects for commands, events, and queries
- Correlation/causation tracking for event sourcing
- Distributed command processing
- Service health monitoring

## Usage

### Basic Example

```rust
use cim_domain_nix::commands::CreateFlake;
use cim_domain_nix::handlers::NixCommandHandler;
use cim_domain_nix::value_objects::MessageIdentity;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handler = NixCommandHandler::new();
    
    let cmd = CreateFlake {
        identity: MessageIdentity::new_root(),
        name: "my-project".to_string(),
        path: std::path::PathBuf::from("/tmp/my-project"),
        description: "My awesome Nix project".to_string(),
        template: Some("rust".to_string()),
    };
    
    let events = handler.handle_create_flake(cmd).await?;
    println!("Created flake with {} events", events.len());
    
    Ok(())
}
```

### Network Integration Example

```rust
use cim_domain_nix::network::{
    NetworkIntegrationService, 
    NetworkTopologyEvent,
};

let mut service = NetworkIntegrationService::new();

// Process network topology event (from nix-network domain)
let topology = create_network_topology();
let systems = service.process_topology_event(topology).await?;

// Generated NixOS configurations for each node
for system in systems {
    println!("System: {} with {} services", 
        system.hostname, 
        system.services.len()
    );
}
```

### NATS Integration Example

```rust
use cim_domain_nix::nats::{NatsClient, NixSubject};

let client = NatsClient::connect("nats://localhost:4222").await?;
let publisher = client.create_publisher();

// Publish command
publisher.publish_command(
    NixSubject::Command(CommandSubject::CreateFlake),
    create_flake_cmd,
).await?;

// Subscribe to events
let mut subscriber = client.subscribe("nix.flake.>").await?;
while let Some(msg) = subscriber.next().await {
    println!("Event: {:?}", msg);
}
```

## Development

### Prerequisites
- Rust 1.70+
- Nix 2.3+ (with flakes enabled)
- NATS server (for distributed features)

### Building

```bash
# Using Cargo
cargo build

# Using Nix
nix develop
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with nextest
cargo nextest run

# Run specific test
cargo test test_create_flake
```

### Examples

The repository includes 11+ examples demonstrating various features:

```bash
# Basic flake creation
cargo run --example create_flake

# Network integration demo
cargo run --example network_integration_demo

# NATS integration
cargo run --example nats_integration_demo

# Code analysis
cargo run --example analyzer_demo
```

## Status

**Current Version**: 0.3.0  
**Completion**: 97%

### Completed Features
- ✅ Core DDD architecture
- ✅ NATS integration (46 subjects)
- ✅ Parser with AST manipulation
- ✅ Analysis framework
- ✅ Formatter integration
- ✅ Network integration
- ✅ Git integration

### Remaining Work
- ⏳ Home Manager support (20%)
- ❌ Language Server Protocol
- ❌ Production hardening

See [PROJECT_STATUS.md](./PROJECT_STATUS.md) for detailed progress.

## Documentation

- [API Documentation](https://docs.rs/cim-domain-nix)
- [Architecture Overview](./doc/architecture/domain-overview.md)
- [NATS Integration Guide](./doc/nats-integration.md)
- [Network Integration Guide](./doc/network-integration.md)
- [Development Guide](./CONTRIBUTING.md)

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Copyright

Copyright 2025 Cowboy AI, LLC. All rights reserved.