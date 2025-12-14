# CIM Domain Service - Standard Template Instructions

**These instructions should be copied to `.claude/CLAUDE.md` in ANY cim-domain-* project to establish the standard architecture.**

## Overview

This is a **cim-domain-*** service - a NATS-based event-sourced microservice following pure Category Theory and Functional Reactive Programming principles.

## 🔴 PRIME DIRECTIVE: CT/FRP Compliance

All cim-domain-* projects MUST follow these principles:

### Pure Functional Event Sourcing
- **NO CRUD operations** - Only immutable events
- **Pure functions only** - Command → Events → State
- **Event Sourcing Pattern**: `(State, Command) → (NewState, [Event])`
- **State Reconstruction**: `fold(Events) → State`

### Category Theory Requirements
1. **Functors**: Structure-preserving transformations
2. **Monads**: Compositional operations with context
3. **Natural Transformations**: Between functors
4. **Coalgebras**: State machines as coalgebraic structures

### Functional Reactive Programming
- **100% FRP compliance** - Zero side effects in domain logic
- **Infrastructure at boundaries** - I/O only in infrastructure layer
- **Pure projections** - `(State, Event) → NewState`
- **Immutable data structures** throughout

### MealyStateMachine Pattern
```rust
impl MealyStateMachine for YourAggregate {
    type State = YourState;
    type Input = YourCommand;
    type Output = Vec<YourEvent>;

    fn output(&self, state: Self::State, input: Self::Input) -> Self::Output {
        // Pure function: (State, Command) → [Event]
    }

    fn transition(&self, state: Self::State, input: Self::Input) -> Self::State {
        // Pure function: (State, Command) → NewState
    }
}
```

## Architecture Requirements

### Project Structure
```
cim-domain-{name}/
├── flake.nix                    # Nix build/deployment (REQUIRED)
├── Cargo.toml                   # Rust package config
├── src/
│   ├── lib.rs                   # Library exports
│   ├── bin/
│   │   └── {name}-service.rs   # NATS service binary (REQUIRED)
│   ├── aggregate/               # Domain aggregates
│   │   ├── mod.rs
│   │   └── {name}.rs           # Main aggregate with MealyStateMachine
│   ├── commands/                # Command types
│   │   └── mod.rs
│   ├── events/                  # Event types (immutable)
│   │   └── mod.rs
│   ├── value_objects/           # Value types
│   │   └── mod.rs
│   ├── services/                # Application services (CQRS)
│   │   └── mod.rs
│   ├── queries/                 # Query specifications
│   │   └── mod.rs
│   ├── projections/             # Pure projection functions
│   │   └── mod.rs
│   ├── category_theory/         # CT trait implementations
│   │   └── mod.rs
│   └── infrastructure/          # Infrastructure adapters
│       ├── nats_integration.rs  # NATS/JetStream (REQUIRED)
│       ├── event_store.rs       # Event store trait
│       └── persistence.rs       # Repository pattern
├── deployment/                  # Deployment configurations (REQUIRED)
│   ├── README.md               # Deployment guide
│   ├── CONTAINER_DEPLOYMENT.md # Container scaling guide
│   ├── nix/
│   │   ├── module.nix          # NixOS module
│   │   ├── container.nix       # Container configuration
│   │   └── flake.nix           # Deployment-specific flake
│   └── systemd/
│       ├── {name}.service      # Systemd unit
│       └── install.sh          # Installation script
├── examples/                    # Example usage
│   ├── basic_usage.rs
│   └── nats_integration.rs
└── tests/                       # Integration tests
    └── aggregate_tests.rs
```

### Required Dependencies (Cargo.toml)
```toml
[dependencies]
# CIM Domain Framework (REQUIRED)
cim-domain = { path = "../cim-domain" }

# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.4", features = ["v7", "serde"] }

# Async runtime
async-trait = "0.1"
tokio = { version = "1.32", features = ["full"] }

# NATS (REQUIRED for services)
async-nats = "0.35"
futures = "0.3"
tokio-stream = "0.1"

[dev-dependencies]
tokio-test = "0.4"

[[bin]]
name = "{name}-service"
path = "src/bin/{name}-service.rs"
```

## Standard Flake Template

### flake.nix (Base Template)
```nix
{
  description = "CIM {Name} Domain - Scalable NATS event-sourced service";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    darwin = {
      url = "github:LnL7/nix-darwin";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, darwin, flake-utils, rust-overlay }:
    let
      flakeContext = {
        inherit (self) inputs;
        inherit self;
      };

      systemOutputs = flake-utils.lib.eachDefaultSystem (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };

          rustVersion = pkgs.rust-bin.nightly.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          };

          # Build service binary
          {name}-service = pkgs.rustPlatform.buildRustPackage {
            pname = "cim-domain-{name}";
            version = "0.1.0";
            src = ./.;

            cargoLock = { lockFile = ./Cargo.lock; };
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl ];
            cargoBuildFlags = [ "--bin" "{name}-service" ];
          };

        in {
          packages = {
            default = {name}-service;
            {name}-service = {name}-service;
          };

          apps.default = {
            type = "app";
            program = "${{name}-service}/bin/{name}-service";
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustVersion cargo-edit cargo-watch
              pkg-config openssl nats-server
            ];
          };
        });

    in systemOutputs // {
      # NixOS modules
      nixosModules = {
        default = import ./deployment/nix/module.nix;
        container = import ./deployment/nix/container.nix flakeContext;
      };

      # Proxmox LXC container
      packages.x86_64-linux.{name}-lxc = nixos-generators.nixosGenerate {
        system = "x86_64-linux";
        format = "proxmox-lxc";
        modules = [
          (import ./deployment/nix/container.nix flakeContext)
          {
            services.cim-domain-{name} = {
              enable = true;
              natsUrl = "nats://10.0.0.41:4222";
            };
          }
        ];
      };

      # nix-darwin support
      darwinModules.default = { config, lib, pkgs, ... }:
        with lib;
        let cfg = config.services.cim-domain-{name};
        in {
          options.services.cim-domain-{name} = {
            enable = mkEnableOption "CIM {Name} Domain Service";
            natsUrl = mkOption {
              type = types.str;
              default = "nats://localhost:4222";
            };
          };

          config = mkIf cfg.enable {
            launchd.daemons.cim-domain-{name} = {
              serviceConfig = {
                ProgramArguments = [
                  "${self.packages.${pkgs.system}.{name}-service}/bin/{name}-service"
                ];
                EnvironmentVariables = { NATS_URL = cfg.natsUrl; };
                KeepAlive = true;
                RunAtLoad = true;
              };
            };
          };
        };
    };
}
```

## NATS Service Binary Template

### src/bin/{name}-service.rs
```rust
//! {Name} Domain Service - NATS microservice

use cim_domain_person::{
    infrastructure::{
        nats_integration::{Nats{Name}Store, {Name}CommandHandler},
        persistence::{PersonRepository, InMemorySnapshotStore},
    },
};
use std::sync::Arc;
use std::env;
use tracing::{info, error};
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("Starting {Name} Domain Service");

    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let stream_name = env::var("STREAM_NAME")
        .unwrap_or_else(|_| "{NAME}_EVENTS".to_string());

    info!("Configuration:");
    info!("  NATS URL: {}", nats_url);
    info!("  Stream Name: {}", stream_name);

    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;
    info!("✓ Connected to NATS");

    // Create event store
    let event_store = Arc::new(Nats{Name}Store::new(client.clone(), stream_name).await?);
    info!("✓ Event store ready");

    // Create repository
    let snapshot_store = Arc::new(InMemorySnapshotStore::new());
    let repository = Arc::new({Name}Repository::new(event_store, snapshot_store, 100));
    info!("✓ Repository ready");

    // Create command handler
    let handler = {Name}CommandHandler::new(repository, client.clone());
    info!("✓ Command handler initialized");

    info!("Service ready - listening on {name}.commands.>");
    info!("Press Ctrl+C to shutdown gracefully");

    // Start command handler
    let handler_task = tokio::spawn(async move {
        if let Err(e) = handler.start().await {
            error!("Handler error: {}", e);
        }
    });

    // Wait for shutdown
    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    handler_task.abort();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    info!("{Name} Domain Service stopped");
    Ok(())
}
```

## Container Deployment Template

### deployment/nix/container.nix
```nix
{ inputs, self, ... }@flakeContext:
{ config, lib, pkgs, modulesPath, ... }:

with lib;

let
  cfg = config.services.cim-domain-{name};

  {name}-service = pkgs.rustPlatform.buildRustPackage rec {
    pname = "cim-domain-{name}";
    version = "0.1.0";
    src = ../..;
    cargoLock = { lockFile = ../../Cargo.lock; };
    nativeBuildInputs = with pkgs; [ pkg-config ];
    buildInputs = with pkgs; [ openssl ];
    cargoBuildFlags = [ "--bin" "{name}-service" ];
  };

in {
  imports = [
    (if builtins.pathExists (modulesPath + "/virtualisation/proxmox-lxc.nix")
     then (modulesPath + "/virtualisation/proxmox-lxc.nix")
     else {})
  ];

  options.services.cim-domain-{name} = {
    enable = mkEnableOption "CIM {Name} Domain Service";
    natsUrl = mkOption {
      type = types.str;
      default = "nats://10.0.0.41:4222";
    };
    streamName = mkOption {
      type = types.str;
      default = "{NAME}_EVENTS";
    };
    containerIp = mkOption {
      type = types.nullOr types.str;
      default = null;
    };
    sshKeys = mkOption {
      type = types.listOf types.str;
      default = [];
    };
  };

  config = mkIf cfg.enable {
    boot.isContainer = mkDefault true;
    system.stateVersion = "24.05";

    networking = {
      hostName = "{name}-service";
      enableIPv6 = mkDefault false;
      interfaces.eth0 = mkIf (cfg.containerIp != null) {
        useDHCP = false;
        ipv4.addresses = [{
          address = cfg.containerIp;
          prefixLength = 19;
        }];
      };
    };

    systemd.services.cim-domain-{name} = {
      description = "CIM {Name} Domain Service";
      wantedBy = [ "multi-user.target" ];
      environment = {
        NATS_URL = cfg.natsUrl;
        STREAM_NAME = cfg.streamName;
      };
      serviceConfig = {
        Type = "simple";
        ExecStart = "${{name}-service}/bin/{name}-service";
        Restart = "always";
        DynamicUser = true;
        StateDirectory = "cim-{name}";
      };
    };

    services.openssh.enable = true;
    users.users.root = mkIf (cfg.sshKeys != []) {
      openssh.authorizedKeys.keys = cfg.sshKeys;
    };
  };
}
```

## Implementation Checklist

When creating a new cim-domain-* project, ensure:

### Core Implementation
- [ ] Main aggregate implements `MealyStateMachine`
- [ ] All state changes through pure event application
- [ ] Commands defined as value types
- [ ] Events defined as immutable structs
- [ ] No CRUD operations anywhere
- [ ] Category Theory traits implemented (Functor, Monad)
- [ ] Pure projection functions for read models

### NATS Integration
- [ ] Service binary in `src/bin/{name}-service.rs`
- [ ] NATS event store implementation
- [ ] Command handler subscribing to `{name}.commands.>`
- [ ] Event publishing to `{name}.events.>`
- [ ] JetStream stream configuration
- [ ] Repository pattern with snapshot support

### Deployment
- [ ] `flake.nix` with all outputs (service, LXC, darwin)
- [ ] Container module in `deployment/nix/container.nix`
- [ ] NixOS module in `deployment/nix/module.nix`
- [ ] Systemd service configuration
- [ ] Container deployment documentation
- [ ] README with deployment examples

### Testing
- [ ] Unit tests for pure functions
- [ ] Integration tests with NATS
- [ ] Examples directory with usage samples
- [ ] NATS cluster testing script

### Documentation
- [ ] README.md with architecture overview
- [ ] CHANGELOG.md with semantic versioning
- [ ] API documentation in `doc/`
- [ ] Deployment guides
- [ ] CT/FRP compliance documentation

## UUID v7 Mandate

**ALWAYS use `Uuid::now_v7()` for time-ordered UUIDs:**
```rust
use uuid::Uuid;

// CORRECT
let id = Uuid::now_v7();

// WRONG - Never use
let id = Uuid::new_v4(); // ❌
```

## NATS Subject Patterns

Follow semantic naming:
```
{domain}.commands.{aggregate_id}     # Commands to specific aggregate
{domain}.commands.>                  # All commands
{domain}.events.{aggregate_id}.>     # Events from aggregate
{domain}.events.>                    # All events
```

Example for "order" domain:
```
order.commands.019a5c11-5f94-7532-9c92-5a9db89e0d9a
order.events.019a5c11-5f94-7532-9c92-5a9db89e0d9a.created
order.events.019a5c11-5f94-7532-9c92-5a9db89e0d9a.item_added
```

## Common Pitfalls to Avoid

1. **❌ CRUD Operations**
   ```rust
   // WRONG
   repository.update(&entity);

   // CORRECT
   let events = entity.handle(command)?;
   repository.save_events(entity.id, events)?;
   ```

2. **❌ Mutable State**
   ```rust
   // WRONG
   entity.status = Status::Completed;

   // CORRECT
   let event = EntityCompleted { id, completed_at };
   entity = entity.apply_event(&event)?;
   ```

3. **❌ Side Effects in Domain**
   ```rust
   // WRONG - I/O in domain logic
   fn handle_command(&mut self, cmd: Command) {
       let data = http_client.get("/api/data").await?; // ❌
       self.data = data;
   }

   // CORRECT - Pure function
   fn handle_command(&self, cmd: Command) -> Vec<Event> {
       // Pure computation only
       vec![Event::DataRequested { id: cmd.id }]
   }
   ```

4. **❌ Direct UUID v4**
   ```rust
   // WRONG
   let id = Uuid::new_v4(); // ❌

   // CORRECT
   let id = Uuid::now_v7(); // ✅ Time-ordered
   ```

## Next Steps

After setting up a new cim-domain-* project:

1. **Build the service**:
   ```bash
   nix build .#{name}-service
   ```

2. **Test locally**:
   ```bash
   NATS_URL=nats://10.0.0.41:4222 cargo run --bin {name}-service
   ```

3. **Build container**:
   ```bash
   nix build .#{name}-lxc
   ```

4. **Deploy to Proxmox**:
   ```bash
   scp result/*.tar.xz root@proxmox:/var/lib/vz/template/cache/
   pct create 150 /var/lib/vz/template/cache/*.tar.xz ...
   ```

5. **Scale horizontally**:
   - Create replicas with different IPs
   - All connect to same NATS cluster
   - Load distributed automatically

## Support

This template is maintained as part of the CIM framework. For questions:
- Reference: `cim-domain-person` (complete example)
- Issues: Your domain repo issues page
- Framework: `cim` registry repository
