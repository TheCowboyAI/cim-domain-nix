# CIM Domain Nix: Architecture Redesign

## Current Problems

The current implementation treats Nix as something to **manage** and **integrate with**:
- ❌ "Create flakes", "Update configurations", "Build packages"
- ❌ Commands like `CreateFlake`, `UpdateModule`, `BuildPackage`
- ❌ Trying to run Nix operations (evaluation, building, formatting)
- ❌ "Managing" NixOS configurations and Home Manager
- ❌ Treating Nix as a tool we control

**This is fundamentally wrong.**

## Correct Architecture: Category Theory Functor

### Core Concept

```
Category of Nix  ──Functor F──>  Category of Infrastructure
  (Data Layer)                     (Domain Model)
```

**Nix is the data storage format. Infrastructure is the domain model.**

### The Two Categories

#### Source Category: Nix
Objects in this category:
1. **Attribute Sets** (attrsets)
2. **Packages**
3. **Derivations**
4. **Overlays**
5. **Modules**
6. **Applications**
7. **Flakes**

Morphisms: Nix language transformations (composition, inheritance, overrides)

#### Target Category: Infrastructure
Objects in this category:
- **Compute Resources** (servers, VMs, containers)
- **Network Topology** (nodes, connections, routes)
- **Software Configurations** (services, applications, policies)
- **Policy Rules** (security, access control, compliance)

Morphisms: Infrastructure relationships (deployment, dependency, composition)

### Ports & Adapters Pattern

```
┌─────────────────────────────────────────────┐
│      Infrastructure Domain (Core)            │
│  - Compute Resources                         │
│  - Network Topology                          │
│  - Software Configurations                   │
│  - Policy Rules                              │
│                                              │
│  Stored as: Event-Sourced Graph              │
│  (NATS JetStream + Neo4j)                    │
└──────────────┬───────────────┬───────────────┘
               │               │
        ┌──────▼─────┐  ┌─────▼──────┐
        │ Input Port │  │ Output Port│
        └──────┬─────┘  └─────┬──────┘
               │               │
    ┌──────────▼─────┐  ┌─────▼──────────┐
    │ NixReader      │  │ NixWriter      │
    │ Adapter        │  │ Adapter        │
    └──────┬─────────┘  └─────┬──────────┘
           │                   │
    ┌──────▼───────────────────▼──────┐
    │   Nix Files (Git Repositories)  │
    │   - flake.nix                    │
    │   - *.nix modules                │
    │   - configurations               │
    └──────────────────────────────────┘
```

### Functor Mapping: F(Nix) → Infrastructure

| Nix Object | Infrastructure Object | Description |
|------------|----------------------|-------------|
| **Flake** | ComputeCluster | Complete system definition → Cluster of resources |
| **Package** | SoftwareArtifact | Derivation → Deployable software |
| **Module** | ConfigurationTemplate | Nix module → Reusable config pattern |
| **Overlay** | ConfigurationOverride | Overlay → Policy/config modification |
| **Derivation** | BuildSpecification | Build instructions → Software spec |
| **Application** | DeployedService | Configured app → Running service |
| **Attrset** | ConfigurationScope | Attribute set → Configuration namespace |

### Event Sourcing Architecture

#### Infrastructure Domain Events
```rust
pub enum InfrastructureEvent {
    // Compute Resources
    ComputeResourceRegistered {
        resource_id: ResourceId,
        resource_type: ComputeType,  // Server, VM, Container
        capabilities: ResourceCapabilities,
    },

    // Software Configurations
    SoftwareConfigured {
        resource_id: ResourceId,
        software_id: SoftwareId,
        configuration: ConfigurationData,
    },

    // Network Topology
    NetworkTopologyDefined {
        topology_id: TopologyId,
        nodes: Vec<NetworkNode>,
        connections: Vec<Connection>,
    },

    // Policy Rules
    PolicyApplied {
        policy_id: PolicyId,
        target: ResourceId,
        rules: PolicyRules,
    },
}
```

#### Infrastructure Domain Commands
```rust
pub enum InfrastructureCommand {
    RegisterComputeResource { /* ... */ },
    ConfigureSoftware { /* ... */ },
    DefineNetworkTopology { /* ... */ },
    ApplyPolicy { /* ... */ },
}
```

### Input Adapter: Nix → Infrastructure

**Purpose**: Read Nix files from Git, parse them, generate Infrastructure domain events

```rust
pub struct NixInputAdapter {
    parser: NixParser,
    functor: NixToInfrastructureFunctor,
}

impl NixInputAdapter {
    /// Read a flake.nix file and generate Infrastructure events
    pub async fn parse_flake(&self, path: &Path) -> Result<Vec<InfrastructureEvent>> {
        // 1. Parse flake.nix using rnix
        let ast = self.parser.parse_file(path)?;

        // 2. Apply functor F: Nix → Infrastructure
        let infrastructure_graph = self.functor.map_flake(ast)?;

        // 3. Generate domain events
        let events = self.generate_events(infrastructure_graph)?;

        Ok(events)
    }
}
```

### Output Adapter: Infrastructure → Nix

**Purpose**: Project Infrastructure state into Nix files, write to Git

```rust
pub struct NixOutputAdapter {
    projector: InfrastructureToNixProjector,
    writer: NixFileWriter,
}

impl NixOutputAdapter {
    /// Project Infrastructure state to Nix files
    pub async fn project_infrastructure(&self, state: &InfrastructureState) -> Result<NixFiles> {
        // 1. Apply inverse functor F⁻¹: Infrastructure → Nix
        let nix_representation = self.projector.project(state)?;

        // 2. Generate Nix file structure
        let files = self.generate_nix_files(nix_representation)?;

        // 3. Write to filesystem (Git repo)
        self.writer.write_files(files)?;

        Ok(files)
    }
}
```

### Category Theory Functor Implementation

```rust
/// Functor F: Category(Nix) → Category(Infrastructure)
pub struct NixToInfrastructureFunctor {
    // Object mappings
    object_map: HashMap<NixObjectType, InfrastructureObjectType>,

    // Morphism mappings (preserves structure)
    morphism_map: HashMap<NixRelation, InfrastructureRelation>,
}

impl NixToInfrastructureFunctor {
    /// Map Nix Flake → Infrastructure ComputeCluster
    pub fn map_flake(&self, flake: ParsedFlake) -> Result<ComputeCluster> {
        // Preserve category structure
        let cluster = ComputeCluster {
            id: ClusterId::new(),
            inputs: self.map_inputs(flake.inputs)?,
            outputs: self.map_outputs(flake.outputs)?,
            systems: self.map_systems(flake.systems)?,
        };

        Ok(cluster)
    }

    /// Map Nix Package → Infrastructure SoftwareArtifact
    pub fn map_package(&self, pkg: ParsedPackage) -> Result<SoftwareArtifact> {
        // ...
    }

    /// Map Nix Module → Infrastructure ConfigurationTemplate
    pub fn map_module(&self, module: ParsedModule) -> Result<ConfigurationTemplate> {
        // ...
    }
}
```

### Complete Flow Example

#### Input Flow: Reading Nix → Infrastructure

```
1. Git Repo: flake.nix
   ↓
2. NixInputAdapter.parse_flake()
   ↓
3. NixParser → ParsedFlake (AST)
   ↓
4. NixToInfrastructureFunctor.map_flake()
   ↓
5. ComputeCluster (Infrastructure Object)
   ↓
6. InfrastructureCommand::RegisterComputeResource
   ↓
7. Infrastructure Aggregate handles command
   ↓
8. InfrastructureEvent::ComputeResourceRegistered
   ↓
9. Event stored in NATS JetStream
   ↓
10. Neo4j Projection updated
```

#### Output Flow: Infrastructure → Writing Nix

```
1. Infrastructure state change
   ↓
2. InfrastructureEvent::SoftwareConfigured
   ↓
3. NixOutputAdapter listens to event
   ↓
4. InfrastructureToNixProjector.project()
   ↓
5. Generate Nix representation
   ↓
6. NixFileWriter.write_files()
   ↓
7. Updated flake.nix written to Git
   ↓
8. Git commit + push
```

## Required Restructuring

### Delete Current Structure
```
❌ src/commands/       # Wrong: Commands to "create" Nix things
❌ src/handlers/       # Wrong: Handlers that execute Nix operations
❌ src/domains/network/    # Wrong: Trying to "generate" NixOS configs
❌ src/domains/home_manager/  # Wrong: Trying to "manage" Home Manager
❌ src/formatter/      # Wrong: Running Nix formatters
❌ src/analyzer/       # Wrong: Analyzing Nix code directly
```

### New Structure
```
✅ src/infrastructure/
   ├── aggregate.rs        # Infrastructure Aggregate
   ├── commands.rs         # Infrastructure Commands
   ├── events.rs           # Infrastructure Events
   └── value_objects.rs    # Infrastructure VOs

✅ src/functor/
   ├── mod.rs              # Functor trait
   ├── nix_to_infra.rs     # F: Nix → Infrastructure
   └── infra_to_nix.rs     # F⁻¹: Infrastructure → Nix

✅ src/adapters/
   ├── input/
   │   ├── nix_parser.rs   # Parse Nix files (using rnix)
   │   └── nix_reader.rs   # Read Nix from Git
   └── output/
       ├── nix_projector.rs # Project Infrastructure → Nix
       └── nix_writer.rs    # Write Nix to Git

✅ src/nix_objects/
   ├── flake.rs            # Nix Flake representation
   ├── package.rs          # Nix Package representation
   ├── module.rs           # Nix Module representation
   ├── derivation.rs       # Nix Derivation representation
   ├── overlay.rs          # Nix Overlay representation
   ├── application.rs      # Nix Application representation
   └── attrset.rs          # Nix Attrset representation
```

## Key Principles

1. **Nix is Data, Not Operations**
   - We don't "create" or "build" Nix things
   - We **read** Nix files as input data
   - We **write** Nix files as output projections

2. **Infrastructure is the Domain**
   - All business logic lives in Infrastructure domain
   - Infrastructure state is event-sourced
   - Infrastructure is the source of truth

3. **Functor Preserves Structure**
   - Category theory functor maintains relationships
   - Composition in Nix → Composition in Infrastructure
   - Identity morphisms preserved

4. **Ports & Adapters**
   - Clean separation between domain and data format
   - Adapters handle I/O concerns
   - Domain remains pure

5. **Event Sourcing**
   - All Infrastructure changes are events
   - Nix files are **projections** of Infrastructure state
   - NATS JetStream + Neo4j store the truth

## Implementation Priority

### Phase 1: Core Domain (Week 1)
- [ ] Infrastructure Aggregate
- [ ] Infrastructure Commands
- [ ] Infrastructure Events
- [ ] Infrastructure Value Objects

### Phase 2: Nix Objects (Week 2)
- [ ] Nix object representations (7 types)
- [ ] Nix parser integration (rnix)

### Phase 3: Functor (Week 3)
- [ ] NixToInfrastructureFunctor
- [ ] InfrastructureToNixFunctor
- [ ] Object mappings
- [ ] Morphism mappings

### Phase 4: Adapters (Week 4)
- [ ] NixInputAdapter
- [ ] NixOutputAdapter
- [ ] Git integration
- [ ] File I/O

### Phase 5: Integration (Week 5)
- [ ] NATS event streaming
- [ ] Neo4j projections
- [ ] End-to-end tests
- [ ] Documentation

## Success Criteria

✅ Infrastructure domain is completely independent of Nix
✅ Nix files are read as input via adapters
✅ Nix files are written as output via projections
✅ Category theory functor preserves structure
✅ All Infrastructure state is event-sourced
✅ No direct Nix command execution
✅ Clean ports & adapters architecture
