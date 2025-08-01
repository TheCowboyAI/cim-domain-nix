# Nix Domain API Documentation Index

## Overview

The Nix Domain API provides a comprehensive event-driven interface for managing Nix flakes, packages, modules, overlays, and NixOS configurations. All operations follow CIM standards with proper correlation/causation tracking.

## Documentation Structure

### 📖 Core Documentation

1. **[API Documentation](./api.md)** - Complete API reference
   - Core types and MessageIdentity
   - All commands with examples
   - All events with structures
   - Value objects reference
   - Services documentation
   - Parser and analyzer APIs
   - Integration examples

2. **[API Quick Reference](./api-quick-reference.md)** - Quick lookup guide
   - Command/Event flow diagram
   - Message identity patterns
   - Quick reference tables
   - Error types summary

3. **[API Diagrams](./api-diagrams.md)** - Visual API representations
   - Complete API overview
   - Command hierarchy
   - Event hierarchy
   - Value objects structure
   - Service layer architecture
   - Parser and analyzer flow
   - Message correlation flow
   - Complete workflow example
   - Error handling flow
   - Future NATS integration

4. **[API State Diagrams](./api-state-diagrams.md)** - State and lifecycle diagrams
   - Flake lifecycle states
   - Configuration lifecycle
   - Build process state machine
   - Command processing pipeline
   - Parser state flow
   - Analyzer workflow
   - Correlation chain example
   - Service dependencies

### 🔑 Key Concepts

#### Message Identity Pattern
Every command and event includes correlation/causation tracking:
- **Root messages**: `correlation_id = causation_id = message_id`
- **Caused messages**: Inherit `correlation_id`, `causation_id = parent.message_id`

#### Event-Driven Architecture
- Commands trigger domain logic
- Aggregates validate and process
- Events represent state changes
- All events are immutable

#### CQRS Pattern
- Commands modify state through aggregates
- Queries read from projections
- Complete separation of read/write models

### 📚 API Categories

#### Commands
- **Flake Operations**: CreateFlake, UpdateFlake, AddFlakeInput
- **Build Operations**: BuildPackage
- **Module Operations**: CreateModule, CreateOverlay
- **Configuration Operations**: CreateConfiguration, ActivateConfiguration

#### Events
- **Flake Events**: FlakeCreated, FlakeUpdated, FlakeInputAdded
- **Build Events**: PackageBuilt
- **Module Events**: ModuleCreated, OverlayCreated
- **Configuration Events**: ConfigurationCreated, ConfigurationActivated

#### Value Objects
- **Identity**: MessageIdentity, MessageId, CorrelationId, CausationId
- **References**: FlakeRef, AttributePath, StorePath
- **Structures**: Flake, NixModule, Overlay, NixOSConfiguration, Derivation
- **Expressions**: NixExpression, NixValue

#### Services
- **FlakeService**: High-level flake operations
- **BuildService**: Package building orchestration
- **ConfigurationService**: NixOS configuration management

#### Infrastructure
- **Parser**: AST parsing and manipulation
- **Analyzers**: Security, performance, and dead code analysis
- **Formatters**: Integration with nixpkgs-fmt, alejandra, nixfmt

### 🎨 Diagram Color Legend

Following CIM high-contrast styling standards:
- 🔴 **Red (#FF6B6B)**: Commands, Aggregates, Core Components
- 🟦 **Teal (#4ECDC4)**: Events, Storage, Secondary Operations
- 🟡 **Yellow (#FFE66D)**: Services, Queries, Choice Points
- 🟢 **Light Green (#95E1D3)**: Results, Value Objects, Outcomes
- ⬛ **Dark Gray (#2D3436)**: Infrastructure, External Systems

### 📋 Usage Patterns

#### Basic Workflow
1. Create command with MessageIdentity
2. Execute through service or handler
3. Aggregate processes and validates
4. Events are emitted with correlation
5. Projections update read models

#### Example Flow
```
User Action → CreateFlake (root) → FlakeCreated
     ↓
BuildPackage (caused) → PackageBuilt
     ↓
ActivateConfiguration (caused) → ConfigurationActivated
```

### 🔧 Integration Points

#### Current
- Direct API calls
- File system operations
- Nix CLI integration
- Git for flake.lock tracking

#### Future (with NATS)
- Distributed command processing
- Event streaming with JetStream
- Service mesh architecture
- Global event correlation

### 📊 API Metrics

- **Total Commands**: 8
- **Total Events**: 10
- **Value Object Types**: 15+
- **Service Methods**: 12+
- **Analyzer Types**: 4
- **Error Types**: 5

### 🚀 Getting Started

1. Review [API Documentation](./api.md) for detailed reference
2. Check [API Quick Reference](./api-quick-reference.md) for quick lookup
3. Explore [API Diagrams](./api-diagrams.md) for visual understanding
4. Study [API State Diagrams](./api-state-diagrams.md) for lifecycles

### 🔗 Related Documentation

- [Architecture Overview](./architecture/domain-overview.md)
- [Completion Roadmap](./plan/completion-roadmap.md)
- [NATS Integration Plan](./plan/nats-integration-plan.md)
- [CIM Standards](./.claude/)