# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- **Build**: `cargo build`
- **Test**: `cargo test` or `cargo nextest run`
- **Run Single Test**: `cargo test test_name` or `cargo nextest run test_name`
- **Lint**: `cargo clippy`
- **Format Check**: `cargo fmt --check`
- **Format Fix**: `cargo fmt`

### Nix Development (in nix develop shell)
- **Enter Dev Shell**: `nix develop`
- **Build with Nix**: `nix build`
- **Update flake.lock**: `nix flake update`
- **Check flake**: `nix flake check`

### Examples
- **Create Flake**: `cargo run --example create_flake`
- **Template Demo**: `cargo run --example template_demo`
- **Service Demo**: `cargo run --example service_demo`
- **Analyzer Demo**: `cargo run --example analyzer_demo`
- **Formatter Demo**: `cargo run --example formatter_demo`
- **Git Integration**: `cargo run --example git_integration_demo`

## Architecture

This is the Nix Domain module within the CIM (Composable Information Machine) ecosystem. It provides event-driven integration with the Nix ecosystem.

### Core Design Principles
1. **Event-Driven Architecture**: All state changes are events following CIM patterns
2. **CQRS Pattern**: Separate command handlers (`handlers/commands/`) and query handlers (`handlers/queries/`)
3. **Domain Isolation**: No shared state with other domains
4. **No CRUD Operations**: Everything is an event, following CIM conversation model

### Key Components

**Aggregates** (in `aggregates/`):
- `FlakeAggregate`: Manages Nix flake lifecycle and state
- `ModuleAggregate`: Handles Nix module composition
- `OverlayAggregate`: Manages package overlays
- `ConfigurationAggregate`: NixOS/Home Manager configurations

**Value Objects** (in `value_objects/`):
- `FlakeRef`: Immutable references to flakes (github:owner/repo, path:/some/path)
- `AttributePath`: Nix attribute paths (e.g., `pkgs.hello`)
- `StorePath`: Nix store paths (`/nix/store/...`)
- `Derivation`: Package build specifications

**Services** (in `services/`):
- `FlakeService`: High-level flake operations
- `BuildService`: Package building orchestration
- `ConfigurationService`: System configuration management

**Parser** (in `parser/`):
- Full AST parsing using `rnix`
- Expression evaluation and transformation
- Manipulation of Nix expressions while preserving formatting

**Analyzer** (in `analyzer/`):
- `SecurityAnalyzer`: Detects insecure patterns
- `PerformanceAnalyzer`: Identifies optimization opportunities
- `DeadCodeAnalyzer`: Finds unused code

### Event Flow
1. Commands are dispatched through CQRS handlers
2. Handlers validate input and execute domain logic
3. Domain events are emitted (e.g., `FlakeCreated`, `PackageBuilt`)
4. Events include correlation and causation IDs per CIM standards
5. Query handlers provide read models from event streams

### Integration Points
- **cim-domain**: Core CIM types (DomainEvent, Aggregate, etc.)
- **cim-subject**: Event routing and NATS subjects
- **cim-domain-git**: Git operations for flake.lock tracking
- **System Tools**: nixpkgs-fmt, alejandra, nix CLI

## Important Patterns

### Event Sourcing
All events MUST follow CIM event sourcing patterns:
```rust
// Events must have correlation and causation IDs
pub struct FlakeCreated {
    pub flake_id: FlakeId,
    pub name: String,
    pub created_at: DateTime<Utc>,
    // From DomainEvent trait
    pub correlation_id: String,
    pub causation_id: String,
}
```

### NATS Subjects
Follow CIM subject naming conventions:
- Commands: `nix.flake.create`, `nix.package.build`
- Events: `nix.flake.created`, `nix.package.built`
- Queries: `nix.flake.get`, `nix.package.list`

### Error Handling
Use domain-specific error types:
```rust
#[derive(Error, Debug)]
pub enum NixDomainError {
    #[error("Flake not found: {0}")]
    FlakeNotFound(FlakeId),
    // ...
}
```

## Testing Requirements
- **TDD Workflow**: Write tests before implementation
- **Integration Tests**: Test with real Nix tools when available
- **Mock System Tools**: Use mockall for unit tests
- **Example Coverage**: Each major feature has a working example