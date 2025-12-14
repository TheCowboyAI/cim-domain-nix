# CIM Domain Nix: Comprehensive Architecture Design

## Executive Summary

**cim-domain-nix** is a Category Theory functor that maps between two categories:
- **Source**: The Category of Nix (language and data structures)
- **Target**: The Category of Infrastructure (compute, network, software, policies)

**Key Insight**: Nix is our **data storage format**, not something we manage. Infrastructure is our **domain model**, event-sourced and stored in NATS JetStream + Neo4j. We use **nix-topology** as the canonical Nix representation of infrastructure, and **NetBox** as an alternate read model for queries.

---

## Part 1: Category Theory Foundation

### The Two Categories

#### Category of Nix (Source Category)

**Objects** (7 fundamental types):

1. **Attribute Sets (Attrsets)**
   ```nix
   { x = 1; y = 2; nested = { z = 3; }; }
   ```
   - Basic: `{ ... }`
   - Recursive: `rec { x = "foo"; y = x + "bar"; }`
   - With inheritance: `{ inherit x y; ... }`

2. **Derivations**
   ```nix
   derivation {
     name = "hello";
     system = "x86_64-linux";
     builder = /bin/bash;
     args = [ "-c" "echo hello > $out" ];
   }
   ```
   - Required: `name`, `system`, `builder`
   - Optional: `args`, `outputs`, environment variables
   - Result: Store path + derivation file

3. **Packages** (Special Derivations)
   ```nix
   stdenv.mkDerivation {
     pname = "mypackage";
     version = "1.0.0";
     src = ./src;
     buildInputs = [ pkg1 pkg2 ];
   }
   ```
   - Extends derivation with stdenv
   - Standard phases: unpack, patch, configure, build, install
   - Composition via overlays and overrides

4. **Modules**
   ```nix
   { config, pkgs, ... }:
   {
     options = { /* option declarations */ };
     config = { /* option definitions */ };
     imports = [ ./other-module.nix ];
   }
   ```
   - Three-part structure: options, config, imports
   - Compose via `imports` list
   - Evaluated in module system with merging

5. **Overlays**
   ```nix
   self: super: {
     mypackage = super.mypackage.override { enableFeature = true; };
   }
   ```
   - Function: `final: prev: { ... }`
   - Modifies package set
   - Composable via function composition

6. **Flakes**
   ```nix
   {
     description = "My flake";

     inputs = {
       nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
       flake-utils.url = "github:numtide/flake-utils";
     };

     outputs = { self, nixpkgs, ... }: {
       packages.x86_64-linux.default = /* derivation */;
       nixosConfigurations.myhost = /* configuration */;
       devShells.x86_64-linux.default = /* shell */;
     };
   }
   ```
   - Top-level composition unit
   - Locked dependencies (flake.lock)
   - Standard output schema

7. **Applications**
   ```nix
   {
     type = "app";
     program = "${pkgs.hello}/bin/hello";
   }
   ```
   - Executable specification
   - Used in `nix run`

**Morphisms in Category of Nix**:
- Function application: `f x`
- Composition: `import`, `callPackage`
- Merging: `//` operator, module system merge
- Inheritance: `inherit`, `rec`
- Overrides: `override`, `overrideAttrs`

**Category Laws**:
- Identity: `x // {} = x`
- Associativity: `(a // b) // c = a // (b // c)` (right-biased)
- Composition: `import` chains preserve structure

---

#### Category of Infrastructure (Target Category)

**Objects** (4 fundamental types):

1. **Compute Resources**
   ```rust
   pub struct ComputeResource {
       pub id: ResourceId,
       pub resource_type: ComputeType,  // Physical, VM, Container
       pub hostname: String,
       pub system: SystemArchitecture,  // x86_64-linux, aarch64-darwin
       pub capabilities: ResourceCapabilities,
       pub interfaces: Vec<NetworkInterface>,
       pub services: Vec<ServiceId>,
       pub guests: Vec<ResourceId>,  // VMs, containers
   }
   ```

2. **Network Topology**
   ```rust
   pub struct NetworkTopology {
       pub id: TopologyId,
       pub networks: Vec<Network>,
       pub connections: Vec<PhysicalConnection>,
   }

   pub struct Network {
       pub id: NetworkId,
       pub name: String,
       pub cidr_v4: Option<Ipv4Network>,
       pub cidr_v6: Option<Ipv6Network>,
   }

   pub struct PhysicalConnection {
       pub from: (ResourceId, InterfaceId),
       pub to: (ResourceId, InterfaceId),
   }
   ```

3. **Software Configurations**
   ```rust
   pub struct SoftwareConfiguration {
       pub id: ConfigurationId,
       pub resource_id: ResourceId,
       pub software: SoftwareArtifact,
       pub configuration: ConfigurationData,
       pub dependencies: Vec<SoftwareId>,
   }

   pub struct SoftwareArtifact {
       pub id: SoftwareId,
       pub name: String,
       pub version: Version,
       pub derivation: DerivationSpec,
   }
   ```

4. **Policy Rules**
   ```rust
   pub struct PolicyRule {
       pub id: PolicyId,
       pub name: String,
       pub scope: PolicyScope,  // Resource, Network, Global
       pub rules: Vec<Rule>,
   }
   ```

**Morphisms in Category of Infrastructure**:
- Deployment: `ComputeResource → SoftwareConfiguration`
- Connection: `NetworkInterface → NetworkInterface`
- Dependency: `Software → Software`
- Composition: `Resource + Resource → Cluster`
- Policy Application: `PolicyRule → Resource`

**Category Laws**:
- Identity: Null deployment preserves resource state
- Associativity: Connection transitivity
- Composition: Software dependency chains

---

### The Functor: F : Nix → Infrastructure

**Object Mapping** (F on objects):

| Nix Object | Infrastructure Object | Mapping Logic |
|------------|----------------------|---------------|
| **Flake** | `ComputeCluster` | Flake inputs → Resource dependencies<br>Flake outputs → Deployed configurations |
| **Derivation** | `SoftwareArtifact` | Derivation spec → Build specification<br>Store path → Artifact location |
| **Package** | `SoftwareArtifact` + `BuildSpec` | Extended derivation with phases<br>Dependencies → Software graph |
| **Module** | `ConfigurationTemplate` | Module options → Configuration schema<br>Module config → Configuration instance |
| **Overlay** | `ConfigurationOverride` | Overlay function → Policy modification<br>Package overrides → Config changes |
| **Application** | `DeployedService` | App program → Service executable<br>App type → Service type |
| **Attrset** | `ConfigurationScope` | Nested structure preserved<br>Keys → Configuration namespace |

**Morphism Mapping** (F on arrows):

| Nix Morphism | Infrastructure Morphism | Structure Preservation |
|--------------|------------------------|------------------------|
| `import` | Configuration composition | F(import a b) = compose(F(a), F(b)) |
| `//` (merge) | Configuration merge | F(a // b) = merge(F(a), F(b)) |
| `override` | Policy application | F(override p x) = apply(F(p), F(x)) |
| Function call | Dependency injection | F(f x) = inject(F(f), F(x)) |
| Module merge | Config aggregation | F(merge mods) = aggregate(F(mods)) |

**Functor Laws**:

1. **Identity Preservation**: `F(id) = id`
   - Empty attrset `{}` maps to empty configuration
   - Identity function maps to null transformation

2. **Composition Preservation**: `F(g ∘ f) = F(g) ∘ F(f)`
   - Import chains: `F(import (import x))` = `compose(F(import), F(import))(F(x))`
   - Module composition: `F(merge [a b c])` = `merge([F(a), F(b), F(c)])`

3. **Structure Preservation**:
   - Nesting: `F({ a = { b = x; }; })` preserves hierarchy
   - Lists: `F([a, b, c])` = `[F(a), F(b), F(c)]`
   - Dependencies: If `a` depends on `b` in Nix, `F(a)` depends on `F(b)` in Infrastructure

---

### Inverse Functor: F⁻¹ : Infrastructure → Nix

**Purpose**: Project Infrastructure domain state back to Nix files (nix-topology format)

**Object Mapping** (F⁻¹ on objects):

| Infrastructure Object | Nix Object | Projection Logic |
|----------------------|------------|------------------|
| `ComputeResource` | nix-topology node | Resource → node definition<br>Interfaces → interface config |
| `Network` | nix-topology network | Network → CIDR + assignments |
| `PhysicalConnection` | `physicalConnections` | Connection → interface link |
| `SoftwareArtifact` | Package derivation | Artifact → package spec |
| `SoftwareConfiguration` | Module config | Config → NixOS module |
| `PolicyRule` | Module options | Policy → option definitions |

**Example Projection**:

```nix
# F⁻¹(ComputeResource) → nix-topology node
{
  topology.nodes.server01 = {
    deviceType = "server";
    hardware.info = "Dell PowerEdge R640";

    interfaces.eth0 = {
      network = "lan";
      physicalConnections = [{
        node = "switch01";
        interface = "port1";
      }];
    };

    services.nginx.enable = true;
  };

  topology.networks.lan = {
    name = "Production LAN";
    cidrv4 = "10.0.1.0/24";
  };
}
```

---

## Part 2: Data Specifications

### Nix Language Specification (Complete)

#### Primitive Types

```rust
pub enum NixValue {
    String(String),           // "hello"
    Integer(i64),             // 123
    Float(f64),               // 3.14
    Bool(bool),               // true, false
    Null,                     // null
    Path(PathBuf),            // /etc/config, ./relative
    LookupPath(String),       // <nixpkgs>
}
```

#### Compound Types

```rust
pub struct AttrSet {
    pub recursive: bool,
    pub attrs: HashMap<String, NixExpression>,
}

pub struct List {
    pub elements: Vec<NixExpression>,
}
```

#### Operators

```rust
pub enum BinaryOp {
    // Arithmetic
    Add,       // +
    Sub,       // -
    Mul,       // *
    Div,       // /

    // Comparison
    Eq,        // ==
    Neq,       // !=
    Lt,        // <
    Lte,       // <=
    Gt,        // >
    Gte,       // >=

    // Logical
    And,       // &&
    Or,        // ||
    Implies,   // ->

    // String/Path
    Concat,    // +

    // Attrset
    Merge,     // //
    Update,    // //
}

pub enum UnaryOp {
    Not,       // !
    Negate,    // -
}
```

#### Language Constructs

```rust
pub enum NixExpression {
    // Values
    Value(NixValue),
    AttrSet(AttrSet),
    List(List),

    // Variables
    Ident(String),
    Select { expr: Box<NixExpression>, attr: String, default: Option<Box<NixExpression>> },

    // Functions
    Lambda { param: Parameter, body: Box<NixExpression> },
    Apply { func: Box<NixExpression>, arg: Box<NixExpression> },

    // Conditionals
    If { cond: Box<NixExpression>, then_expr: Box<NixExpression>, else_expr: Box<NixExpression> },
    Assert { cond: Box<NixExpression>, body: Box<NixExpression> },

    // Bindings
    Let { bindings: Vec<Binding>, body: Box<NixExpression> },
    With { env: Box<NixExpression>, body: Box<NixExpression> },

    // Operations
    BinaryOp { op: BinaryOp, left: Box<NixExpression>, right: Box<NixExpression> },
    UnaryOp { op: UnaryOp, expr: Box<NixExpression> },

    // Special
    Inherit { from: Option<Box<NixExpression>>, attrs: Vec<String> },
}

pub enum Parameter {
    Ident(String),
    Pattern { formals: Vec<Formal>, ellipsis: bool, at: Option<String> },
}

pub struct Formal {
    pub name: String,
    pub default: Option<NixExpression>,
}
```

### nix-topology Schema (Target Format)

```rust
pub struct NixTopology {
    pub nodes: HashMap<NodeId, TopologyNode>,
    pub networks: HashMap<NetworkId, TopologyNetwork>,
}

pub struct TopologyNode {
    pub name: String,
    pub device_type: DeviceType,  // "server", "switch", "router", etc.
    pub hardware_info: Option<String>,
    pub interfaces: HashMap<InterfaceId, TopologyInterface>,
    pub services: HashMap<ServiceName, ServiceConfig>,
    pub guests: Vec<NodeId>,  // VMs, containers
}

pub struct TopologyInterface {
    pub name: String,
    pub network: NetworkId,
    pub addresses: Vec<IpAddr>,
    pub physical_connections: Vec<PhysicalConnection>,
}

pub struct TopologyNetwork {
    pub name: String,
    pub cidr_v4: Option<Ipv4Network>,
    pub cidr_v6: Option<Ipv6Network>,
}

pub struct PhysicalConnection {
    pub node: NodeId,
    pub interface: InterfaceId,
}
```

### Infrastructure Domain Model (Event-Sourced)

```rust
// Aggregate Root
pub struct InfrastructureAggregate {
    pub id: InfrastructureId,
    pub version: u64,
    pub resources: HashMap<ResourceId, ComputeResource>,
    pub topology: NetworkTopology,
    pub configurations: HashMap<ConfigurationId, SoftwareConfiguration>,
    pub policies: HashMap<PolicyId, PolicyRule>,
    pub uncommitted_events: Vec<InfrastructureEvent>,
}

// Commands
pub enum InfrastructureCommand {
    RegisterComputeResource {
        identity: MessageIdentity,
        resource: ComputeResourceSpec,
    },

    DefineNetworkTopology {
        identity: MessageIdentity,
        topology: NetworkTopologySpec,
    },

    ConfigureSoftware {
        identity: MessageIdentity,
        resource_id: ResourceId,
        configuration: SoftwareConfigurationSpec,
    },

    ApplyPolicy {
        identity: MessageIdentity,
        policy: PolicyRuleSpec,
    },

    ConnectResources {
        identity: MessageIdentity,
        connection: ConnectionSpec,
    },
}

// Events
pub enum InfrastructureEvent {
    ComputeResourceRegistered {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        resource_id: ResourceId,
        resource: ComputeResource,
    },

    NetworkTopologyDefined {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        topology_id: TopologyId,
        networks: Vec<Network>,
        connections: Vec<PhysicalConnection>,
    },

    SoftwareConfigured {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        configuration_id: ConfigurationId,
        resource_id: ResourceId,
        software: SoftwareArtifact,
        configuration: ConfigurationData,
    },

    PolicyApplied {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        policy_id: PolicyId,
        scope: PolicyScope,
        rules: Vec<Rule>,
    },

    ResourcesConnected {
        event_id: Uuid,
        correlation_id: Uuid,
        causation_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
        connection: PhysicalConnection,
    },
}
```

---

## Part 3: Architecture Design

### Ports & Adapters (Hexagonal Architecture)

```
┌──────────────────────────────────────────────────────────────┐
│                  Infrastructure Domain (Core)                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  InfrastructureAggregate                               │  │
│  │  - Compute Resources                                   │  │
│  │  - Network Topology                                    │  │
│  │  - Software Configurations                             │  │
│  │  - Policy Rules                                        │  │
│  │                                                         │  │
│  │  Event-Sourced State                                   │  │
│  │  (NATS JetStream + Neo4j Graph)                       │  │
│  └────────────────────────────────────────────────────────┘  │
└────────────────┬───────────────────────────┬──────────────────┘
                 │                           │
          ┌──────▼─────┐              ┌──────▼─────┐
          │ Input Port │              │Output Port │
          │ (Commands) │              │ (Events)   │
          └──────┬─────┘              └──────┬─────┘
                 │                           │
     ┌───────────▼──────────┐    ┌───────────▼──────────┐
     │  NixInputAdapter     │    │  NixOutputAdapter    │
     │  ┌────────────────┐  │    │  ┌────────────────┐  │
     │  │ NixParser      │  │    │  │ NixProjector   │  │
     │  │ (rnix-parser)  │  │    │  │ (nix-topology) │  │
     │  └────────────────┘  │    │  └────────────────┘  │
     │  ┌────────────────┐  │    │  ┌────────────────┐  │
     │  │ NixFunctor     │  │    │  │ NixWriter      │  │
     │  │ F: Nix → Infra │  │    │  │ (to Git repo)  │  │
     │  └────────────────┘  │    │  └────────────────┘  │
     └───────────┬──────────┘    └───────────┬──────────┘
                 │                           │
     ┌───────────▼───────────────────────────▼──────────┐
     │        Git Repository (Nix Files)                 │
     │  ┌─────────────────────────────────────────────┐ │
     │  │ flake.nix                                    │ │
     │  │ topology.nix (nix-topology format)          │ │
     │  │ modules/*.nix                                │ │
     │  │ packages/*.nix                               │ │
     │  │ overlays/*.nix                               │ │
     │  └─────────────────────────────────────────────┘ │
     └──────────────────────────────────────────────────┘

     ┌─────────────────────────────────────────────────┐
     │  Alternate Read Model: NetBox (HTTP/GraphQL)    │
     │  ┌───────────────────────────────────────────┐  │
     │  │ NetBoxProjection (Event Listener)         │  │
     │  │ - Listens to Infrastructure Events        │  │
     │  │ - Projects to NetBox DCIM model           │  │
     │  │ - Provides query interface                │  │
     │  └───────────────────────────────────────────┘  │
     └─────────────────────────────────────────────────┘
```

### Component Responsibilities

#### 1. Infrastructure Domain (Core)

**Purpose**: Event-sourced domain model for infrastructure

**Responsibilities**:
- Define compute resources, networks, software, policies
- Handle domain commands via aggregate
- Emit domain events for all state changes
- Enforce business invariants
- No knowledge of Nix or any external format

**Key Files**:
- `src/infrastructure/aggregate.rs` - Aggregate root
- `src/infrastructure/commands.rs` - Domain commands
- `src/infrastructure/events.rs` - Domain events
- `src/infrastructure/value_objects.rs` - VOs (ResourceId, NetworkId, etc.)

---

#### 2. Nix Objects (Data Representations)

**Purpose**: Represent Nix language constructs as Rust types

**Responsibilities**:
- Model Nix AST types (attrsets, derivations, modules, flakes, etc.)
- Preserve Nix semantics and structure
- No domain logic - pure data structures
- Compatible with rnix-parser output

**Key Files**:
- `src/nix_objects/attrset.rs`
- `src/nix_objects/derivation.rs`
- `src/nix_objects/package.rs`
- `src/nix_objects/module.rs`
- `src/nix_objects/overlay.rs`
- `src/nix_objects/flake.rs`
- `src/nix_objects/application.rs`

---

#### 3. Functor (Category Theory Mapping)

**Purpose**: Bidirectional structure-preserving mapping

**Responsibilities**:
- F: Map Nix objects → Infrastructure objects
- F⁻¹: Map Infrastructure objects → Nix objects (nix-topology)
- Preserve category structure (composition, identity)
- Enforce functor laws
- Pure functions - no side effects

**Key Files**:
- `src/functor/mod.rs` - Functor trait
- `src/functor/nix_to_infra.rs` - F: Nix → Infrastructure
- `src/functor/infra_to_nix.rs` - F⁻¹: Infrastructure → Nix
- `src/functor/morphisms.rs` - Morphism mappings

**Example**:
```rust
pub trait Functor<A, B> {
    fn map_object(&self, obj: A) -> Result<B>;
    fn map_morphism(&self, f: Morphism<A>) -> Result<Morphism<B>>;
}

pub struct NixToInfraFunctor;

impl Functor<ParsedFlake, ComputeCluster> for NixToInfraFunctor {
    fn map_object(&self, flake: ParsedFlake) -> Result<ComputeCluster> {
        // Map flake.inputs → Resource dependencies
        // Map flake.outputs → Deployed configurations
        // Preserve structure
    }
}
```

---

#### 4. Input Adapter (Nix → Infrastructure)

**Purpose**: Read Nix files from Git, parse, and generate Infrastructure commands

**Responsibilities**:
- Read Nix files from Git repository
- Parse using rnix-parser
- Apply functor F: Nix → Infrastructure
- Generate Infrastructure commands
- Emit commands to NATS

**Key Files**:
- `src/adapters/input/nix_parser.rs` - Parse Nix files
- `src/adapters/input/nix_reader.rs` - Read from Git
- `src/adapters/input/command_generator.rs` - Generate commands

**Flow**:
```
Git repo/flake.nix
  → NixParser.parse()
  → ParsedFlake
  → NixFunctor.map()
  → ComputeCluster
  → CommandGenerator.generate()
  → InfrastructureCommand::RegisterComputeResource
  → NATS publish
```

---

#### 5. Output Adapter (Infrastructure → Nix)

**Purpose**: Listen to Infrastructure events, project to nix-topology, write to Git

**Responsibilities**:
- Listen to Infrastructure events (NATS subscription)
- Apply functor F⁻¹: Infrastructure → Nix
- Generate nix-topology format
- Write Nix files to Git repository
- Commit and push changes

**Key Files**:
- `src/adapters/output/nix_projector.rs` - Project to nix-topology
- `src/adapters/output/nix_writer.rs` - Write Nix files
- `src/adapters/output/git_writer.rs` - Git operations
- `src/adapters/output/event_listener.rs` - NATS event subscription

**Flow**:
```
InfrastructureEvent::ComputeResourceRegistered (NATS)
  → EventListener.handle()
  → NixProjector.project()
  → NixTopology
  → NixWriter.generate_files()
  → topology.nix
  → GitWriter.commit_and_push()
  → Git repository updated
```

---

#### 6. NetBox Projection (Alternate Read Model)

**Purpose**: Project Infrastructure events to NetBox for queries and visualization

**Responsibilities**:
- Subscribe to Infrastructure events
- Map to NetBox DCIM model (devices, interfaces, cables, IP addresses)
- Update NetBox via REST API
- Provide alternate query interface

**Key Files**:
- `src/projections/netbox/mod.rs` - NetBox projection
- `src/projections/netbox/mapper.rs` - Infrastructure → NetBox mapping
- `src/projections/netbox/client.rs` - NetBox API client

**Flow**:
```
InfrastructureEvent (NATS)
  → NetBoxProjection.handle()
  → Map to NetBox model
  → NetBox API call (create device, interface, cable, etc.)
  → NetBox UI reflects Infrastructure state
```

---

## Part 4: Implementation Plan

### Phase 1: Infrastructure Domain Core (Week 1)

**Goal**: Event-sourced Infrastructure domain, completely independent of Nix

**Tasks**:
1. Define Infrastructure aggregate
2. Define commands (RegisterComputeResource, DefineNetworkTopology, etc.)
3. Define events (with correlation/causation IDs)
4. Define value objects (ResourceId, NetworkId, etc.)
5. Implement command handlers in aggregate
6. Implement event application (apply_event)
7. Write unit tests for aggregate logic

**Deliverables**:
- `src/infrastructure/` module
- 100+ tests for domain logic
- Zero Nix dependencies

**Success Criteria**:
- ✅ Aggregate handles all commands
- ✅ Events emitted for all state changes
- ✅ Business invariants enforced
- ✅ Compiles and tests pass
- ✅ No external dependencies

---

### Phase 2: Nix Objects Representation (Week 2)

**Goal**: Rust types for all 7 Nix object types

**Tasks**:
1. Define AttrSet, List, Value types
2. Define Derivation structure
3. Define Package structure (extends Derivation)
4. Define Module structure (options, config, imports)
5. Define Overlay structure (function type)
6. Define Flake structure (inputs, outputs, lock)
7. Define Application structure
8. Integrate rnix-parser for parsing
9. Write parsers for each type

**Deliverables**:
- `src/nix_objects/` module
- Parsers for all Nix constructs
- 80+ tests for parsing

**Success Criteria**:
- ✅ Parse flake.nix successfully
- ✅ Parse nix-topology format
- ✅ Parse derivations, packages, modules
- ✅ Preserve AST structure
- ✅ Round-trip parsing (parse → serialize → parse)

---

### Phase 3: Category Theory Functor (Week 3)

**Goal**: Bidirectional structure-preserving mappings

**Tasks**:
1. Define Functor trait
2. Implement F: Nix → Infrastructure
   - Map Flake → ComputeCluster
   - Map Derivation → SoftwareArtifact
   - Map Module → ConfigurationTemplate
   - Map Overlay → ConfigurationOverride
3. Implement F⁻¹: Infrastructure → Nix
   - Map ComputeResource → nix-topology node
   - Map Network → nix-topology network
   - Map Connection → physicalConnection
4. Implement morphism mappings
5. Verify functor laws (identity, composition)
6. Write property-based tests

**Deliverables**:
- `src/functor/` module
- Bidirectional mappings
- 100+ tests including property tests

**Success Criteria**:
- ✅ F preserves structure
- ✅ F⁻¹ ∘ F ≈ id (round-trip)
- ✅ Functor laws verified
- ✅ Composition preserved

---

### Phase 4: Input/Output Adapters (Week 4)

**Goal**: Read from Git, write to Git

**Tasks**:
1. **Input Adapter**:
   - Implement Git reader (libgit2)
   - Implement Nix parser integration
   - Implement command generator
   - Wire to NATS publisher
2. **Output Adapter**:
   - Implement NATS event listener
   - Implement nix-topology projector
   - Implement Nix file writer
   - Implement Git committer
3. Write integration tests

**Deliverables**:
- `src/adapters/input/` module
- `src/adapters/output/` module
- End-to-end adapter tests

**Success Criteria**:
- ✅ Read flake.nix from Git
- ✅ Generate Infrastructure commands
- ✅ Listen to Infrastructure events
- ✅ Write nix-topology files
- ✅ Commit to Git successfully

---

### Phase 5: NATS Integration & Projections (Week 5)

**Goal**: Complete event-driven architecture

**Tasks**:
1. NATS command handler
2. NATS event publisher
3. NATS event subscriber
4. Neo4j graph projection
5. NetBox projection (alternate read model)
6. Health monitoring
7. End-to-end integration tests

**Deliverables**:
- `src/nats/` module
- `src/projections/netbox/` module
- Complete event streaming
- 50+ integration tests

**Success Criteria**:
- ✅ Commands flow: Git → NATS → Aggregate
- ✅ Events flow: Aggregate → NATS → Projections
- ✅ Neo4j graph updated
- ✅ NetBox synchronized
- ✅ nix-topology files written

---

## Part 5: File Structure

```
cim-domain-nix/
├── src/
│   ├── lib.rs                      # Public API
│   │
│   ├── infrastructure/             # DOMAIN CORE (Hexagon Center)
│   │   ├── mod.rs
│   │   ├── aggregate.rs            # InfrastructureAggregate
│   │   ├── commands.rs             # Domain commands
│   │   ├── events.rs               # Domain events
│   │   ├── value_objects.rs        # ResourceId, NetworkId, etc.
│   │   └── tests.rs
│   │
│   ├── nix_objects/                # Nix Data Structures
│   │   ├── mod.rs
│   │   ├── attrset.rs              # Attribute sets
│   │   ├── derivation.rs           # Derivations
│   │   ├── package.rs              # Packages
│   │   ├── module.rs               # Modules
│   │   ├── overlay.rs              # Overlays
│   │   ├── flake.rs                # Flakes
│   │   ├── application.rs          # Applications
│   │   └── tests.rs
│   │
│   ├── functor/                    # Category Theory Functor
│   │   ├── mod.rs
│   │   ├── nix_to_infra.rs        # F: Nix → Infrastructure
│   │   ├── infra_to_nix.rs        # F⁻¹: Infrastructure → Nix
│   │   ├── morphisms.rs            # Morphism mappings
│   │   └── tests.rs
│   │
│   ├── adapters/                   # Ports & Adapters
│   │   ├── input/                  # Input Port: Nix → Commands
│   │   │   ├── mod.rs
│   │   │   ├── nix_parser.rs      # Parse Nix files (rnix)
│   │   │   ├── nix_reader.rs      # Read from Git
│   │   │   ├── command_generator.rs # Generate commands
│   │   │   └── tests.rs
│   │   │
│   │   └── output/                 # Output Port: Events → Nix
│   │       ├── mod.rs
│   │       ├── nix_projector.rs   # Project to nix-topology
│   │       ├── nix_writer.rs      # Write Nix files
│   │       ├── git_writer.rs      # Git operations
│   │       ├── event_listener.rs  # NATS subscription
│   │       └── tests.rs
│   │
│   ├── nats/                       # NATS Integration
│   │   ├── mod.rs
│   │   ├── command_handler.rs     # Handle Infrastructure commands
│   │   ├── event_publisher.rs     # Publish Infrastructure events
│   │   ├── config.rs
│   │   ├── subject.rs             # NATS subjects
│   │   └── tests.rs
│   │
│   ├── projections/                # Read Models
│   │   ├── neo4j/                 # Graph projection
│   │   │   ├── mod.rs
│   │   │   ├── projector.rs
│   │   │   └── tests.rs
│   │   │
│   │   └── netbox/                # NetBox projection
│   │       ├── mod.rs
│   │       ├── mapper.rs          # Infrastructure → NetBox
│   │       ├── client.rs          # NetBox API
│   │       └── tests.rs
│   │
│   └── client/                     # Client SDK
│       ├── mod.rs
│       └── tests.rs
│
├── tests/                          # Integration tests
│   ├── end_to_end_test.rs
│   ├── functor_laws_test.rs
│   └── nix_roundtrip_test.rs
│
├── examples/
│   ├── parse_flake.rs
│   ├── infrastructure_from_nix.rs
│   ├── nix_from_infrastructure.rs
│   └── netbox_sync.rs
│
├── docs/
│   ├── ARCHITECTURE.md
│   ├── CATEGORY_THEORY.md
│   ├── NIX_SPECIFICATION.md
│   └── API.md
│
├── Cargo.toml
├── flake.nix                       # Nix development environment
├── COMPREHENSIVE_DESIGN.md         # This document
└── README.md
```

---

## Part 6: Key Design Decisions

### 1. Why Category Theory?

**Problem**: How to maintain structural consistency between two different representations (Nix files and Infrastructure domain)?

**Solution**: Category theory provides a mathematically rigorous framework for structure-preserving transformations.

**Benefits**:
- **Composition preservation**: Nested structures map correctly
- **Identity preservation**: Empty operations stay empty
- **Bidirectionality**: Can go both ways reliably
- **Verifiability**: Functor laws can be tested

---

### 2. Why Event Sourcing?

**Problem**: How to maintain Infrastructure state with full auditability and projections to multiple formats?

**Solution**: Event sourcing with NATS JetStream as event store.

**Benefits**:
- **Audit trail**: Every Infrastructure change is recorded
- **Multiple projections**: Nix files, Neo4j, NetBox all stay in sync
- **Time travel**: Can reconstruct state at any point
- **Distributed**: NATS enables multi-node processing

---

### 3. Why Ports & Adapters?

**Problem**: How to keep domain logic independent of external formats (Nix, NetBox, Git)?

**Solution**: Hexagonal architecture with clean boundaries.

**Benefits**:
- **Testability**: Domain logic tests without I/O
- **Flexibility**: Swap adapters (different Git hosting, different projections)
- **Clarity**: Clear separation of concerns
- **Maintainability**: Changes to formats don't affect domain

---

### 4. Why nix-topology?

**Problem**: How to represent Infrastructure in Nix format?

**Solution**: Use nix-topology as the canonical Nix representation.

**Benefits**:
- **Standard format**: Already used in Nix community
- **Visualization**: Built-in diagram generation
- **NixOS integration**: Works with NixOS module system
- **Proven**: Battle-tested in production

---

### 5. Why NetBox as Read Model?

**Problem**: How to provide human-friendly queries on Infrastructure?

**Solution**: Project Infrastructure events to NetBox.

**Benefits**:
- **UI**: Web interface for browsing infrastructure
- **DCIM**: Full data center infrastructure management features
- **API**: GraphQL/REST for queries
- **Standard**: Industry-standard tool

---

## Part 7: Success Criteria

### Functional Requirements

- ✅ Parse flake.nix and generate Infrastructure commands
- ✅ Handle Infrastructure commands via aggregate
- ✅ Emit Infrastructure events for all state changes
- ✅ Project Infrastructure state to nix-topology format
- ✅ Write nix-topology files to Git
- ✅ Project Infrastructure state to NetBox
- ✅ Round-trip: Nix → Infrastructure → Nix preserves structure

### Non-Functional Requirements

- ✅ Zero direct Nix command execution
- ✅ Domain completely independent of Nix
- ✅ Functor laws verified with property tests
- ✅ 90%+ test coverage
- ✅ All tests passing
- ✅ Zero compilation warnings
- ✅ Full documentation (rustdoc)

### Category Theory Requirements

- ✅ Identity morphism preserved: `F(id) = id`
- ✅ Composition preserved: `F(g ∘ f) = F(g) ∘ F(f)`
- ✅ Structure preserved: Nesting, dependencies, relationships
- ✅ Bidirectional: `F⁻¹ ∘ F ≈ id` (up to isomorphism)

---

## Part 8: Example Scenarios

### Scenario 1: Parse Flake, Create Infrastructure

**Input**: Git repository with `flake.nix`

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }: {
    nixosConfigurations.server01 = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ config, pkgs, ... }: {
          networking.hostName = "server01";
          services.nginx.enable = true;
        })
      ];
    };
  };
}
```

**Flow**:
1. NixInputAdapter reads `flake.nix` from Git
2. NixParser parses to `ParsedFlake`
3. NixToInfraFunctor maps to `ComputeCluster`
4. CommandGenerator generates `RegisterComputeResource` command
5. Command published to NATS
6. InfrastructureAggregate handles command
7. `ComputeResourceRegistered` event emitted
8. Event stored in NATS JetStream
9. Neo4j projection updated
10. NetBox projection updated

**Result**: Infrastructure domain now contains server01 with nginx service

---

### Scenario 2: Update Infrastructure, Write Nix

**Trigger**: User sends `ConfigureSoftware` command via API

```rust
InfrastructureCommand::ConfigureSoftware {
    identity: MessageIdentity::new_root(),
    resource_id: ResourceId::new("server01"),
    configuration: SoftwareConfigurationSpec {
        software: SoftwareId::new("postgresql"),
        version: Version::parse("15.0").unwrap(),
        configuration: json!({
            "port": 5432,
            "max_connections": 100,
        }),
    },
}
```

**Flow**:
1. Command received via NATS
2. InfrastructureAggregate handles command
3. `SoftwareConfigured` event emitted
4. NixOutputAdapter listens to event
5. InfrastructureToNixFunctor projects to nix-topology
6. NixWriter generates updated `topology.nix`:

```nix
{
  topology.nodes.server01 = {
    services.postgresql = {
      enable = true;
      port = 5432;
      settings.max_connections = 100;
    };
  };
}
```

7. GitWriter commits and pushes to Git
8. NetBox projection also updated

**Result**: Nix files and NetBox reflect new PostgreSQL service

---

### Scenario 3: Define Network Topology

**Input**: Infrastructure command to create network

```rust
InfrastructureCommand::DefineNetworkTopology {
    identity: MessageIdentity::new_root(),
    topology: NetworkTopologySpec {
        networks: vec![
            NetworkSpec {
                id: NetworkId::new("production-lan"),
                name: "Production LAN".into(),
                cidr_v4: Some("10.0.1.0/24".parse().unwrap()),
            },
        ],
        connections: vec![
            ConnectionSpec {
                from: (ResourceId::new("server01"), InterfaceId::new("eth0")),
                to: (ResourceId::new("switch01"), InterfaceId::new("port1")),
            },
        ],
    },
}
```

**Flow**:
1. Command handled by aggregate
2. `NetworkTopologyDefined` event emitted
3. NixOutputAdapter projects to nix-topology:

```nix
{
  topology.networks.production-lan = {
    name = "Production LAN";
    cidrv4 = "10.0.1.0/24";
  };

  topology.nodes.server01.interfaces.eth0 = {
    network = "production-lan";
    physicalConnections = [{
      node = "switch01";
      interface = "port1";
    }];
  };
}
```

4. Files written to Git
5. NetBox projection creates:
   - IP Prefix: 10.0.1.0/24
   - Device: server01
   - Interface: eth0
   - Cable: server01:eth0 ↔ switch01:port1

**Result**: Network topology in all three representations (Events, Nix, NetBox)

---

## Part 9: Dependencies

### Rust Crates

```toml
[dependencies]
# Core
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.11", features = ["v7"] }
chrono = "0.4"
url = "2.5"

# Nix Parsing
rnix = "0.11"  # Nix parser

# Git Integration
git2 = "0.19"

# NATS
async-nats = "0.37"

# Neo4j
neo4rs = "0.8"

# NetBox
reqwest = { version = "0.12", features = ["json"] }

# Error Handling
thiserror = "2.0"
anyhow = "1.0"

# Testing
proptest = "1.5"  # Property-based testing for functor laws

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## Part 10: Next Steps

1. **Review and approve this design**
2. **Create project skeleton**:
   - Directory structure
   - Cargo.toml
   - flake.nix for development
3. **Phase 1: Infrastructure Domain** (Week 1)
4. **Phase 2: Nix Objects** (Week 2)
5. **Phase 3: Functor** (Week 3)
6. **Phase 4: Adapters** (Week 4)
7. **Phase 5: Integration** (Week 5)

---

## Conclusion

This design provides a mathematically rigorous, event-sourced architecture for mapping between Nix configurations and Infrastructure domain model. By using Category Theory functors and Ports & Adapters pattern, we achieve:

- **Separation**: Domain independent of data formats
- **Consistency**: Structure-preserving transformations
- **Flexibility**: Multiple projections (Nix, NetBox)
- **Auditability**: Event-sourced state
- **Testability**: Property-based tests for functor laws

The key insight is: **Nix is data storage, Infrastructure is the domain model**.
