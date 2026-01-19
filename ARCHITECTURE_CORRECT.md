# CIM Domain Nix - Correct Architecture

**Status**: Refactoring to Port/Adapter Pattern
**Date**: 2026-01-18
**Issue**: Previous code built against old infrastructure domain with different types

## What This Module Is

**cim-domain-nix is a PORT/ADAPTER**, not a domain itself.

```
┌─────────────────────────────────────────────────────────┐
│             cim-infrastructure (DOMAIN)                  │
│  Event-sourced domain model with value objects:         │
│  - ComputeResource (entity)                              │
│  - ResourceType (enum with 35 types)                     │
│  - Hostname (value object with DNS validation)           │
│  - IpAddressWithCidr, MacAddress, VlanId, Mtu           │
│  - InfrastructureEvents                                  │
└─────────────────────────────────────────────────────────┘
                            ▲
                            │ imports
                            │
┌─────────────────────────────────────────────────────────┐
│          cim-domain-nix (PORT/ADAPTER)                   │
│  Bidirectional mapping between domain and Nix:           │
│                                                          │
│  READ:  nixos-topology → Infrastructure Events          │
│  WRITE: Infrastructure Events → nixos-topology files    │
│                                                          │
│  Functors (Category Theory):                            │
│  - F: ResourceType → TopologyNodeType                   │
│  - G: ComputeResource → TopologyNode                    │
│  - H: NetworkSegment → TopologyNetwork                  │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼ reads/writes
┌─────────────────────────────────────────────────────────┐
│         nixos-topology (Nix configuration)               │
│  Files: topology.nix, networks.nix, nodes/*.nix         │
└─────────────────────────────────────────────────────────┘
```

## Current State vs Target State

### Current (Broken)
- ❌ Built against old `InfrastructureAggregate` that doesn't exist
- ❌ References `ComputeType` enum that was replaced with `ResourceType`
- ❌ Expects old domain structure with different field names
- ❌ 131 compilation errors due to type mismatches

### Target (Correct)
- ✅ Import from `cim-infrastructure` (fixed in Cargo.toml)
- ✅ Work with NEW domain model: `ComputeResource`, `ResourceType`, value objects
- ✅ Bidirectional adapters: Read topology → Emit events, Listen events → Write topology
- ✅ Clean port/adapter separation

## Architecture Layers

### Layer 1: Domain (cim-infrastructure)
**What it provides:**
```rust
pub struct ComputeResource {
    pub id: AggregateId,
    pub hostname: Hostname,
    pub resource_type: ResourceType,  // Router, Switch, PhysicalServer, etc.
    pub organization_id: Option<AggregateId>,
    pub location_id: Option<AggregateId>,
    pub owner_id: Option<AggregateId>,
    // ... hardware details, metadata
}

pub enum ResourceType {
    PhysicalServer, VirtualMachine, Router, Switch,
    Camera, KVM, Monitor, // ... 32 more types
}

pub enum InfrastructureEvent {
    ComputeRegistered { /* ... */ },
    NetworkDefined { /* ... */ },
    InterfaceAdded { /* ... */ },
    IPAssigned { /* ... */ },
}
```

### Layer 2: Adapter (cim-domain-nix)
**What it does:**

#### A. Topology Reader (Nix → Domain Events)
```rust
pub struct TopologyReader {
    // Read nixos-topology files
}

impl TopologyReader {
    /// Read topology.nix and generate Infrastructure events
    pub fn read_topology(&self, path: &Path) -> Result<Vec<InfrastructureEvent>> {
        // 1. Parse topology.nix using rnix
        // 2. Extract nodes, networks, connections
        // 3. Map to ResourceType using functor
        // 4. Generate ComputeRegistered events
        // 5. Generate NetworkDefined events
        // 6. Generate InterfaceAdded events
    }
}
```

#### B. Topology Writer (Domain Events → Nix Files)
```rust
pub struct TopologyWriter {
    // Write nixos-topology files
}

impl TopologyWriter {
    /// Listen to Infrastructure events, update topology files
    pub fn project_event(&mut self, event: &InfrastructureEvent) -> Result<()> {
        match event {
            InfrastructureEvent::ComputeRegistered { resource, .. } => {
                self.add_topology_node(resource)?;
            }
            InfrastructureEvent::NetworkDefined { network, .. } => {
                self.add_topology_network(network)?;
            }
            // ...
        }
    }
}
```

#### C. Functors (Type Mappings)
```rust
pub mod functors {
    use cim_infrastructure::{ResourceType, ComputeResource};
    use nixos_topology::{TopologyNodeType, TopologyNode};

    /// Functor F: ResourceType → TopologyNodeType
    pub fn map_resource_type(rt: ResourceType) -> TopologyNodeType {
        match rt {
            ResourceType::PhysicalServer => TopologyNodeType::PhysicalServer,
            ResourceType::VirtualMachine => TopologyNodeType::VirtualMachine,
            ResourceType::Router => TopologyNodeType::Router,
            ResourceType::Switch => TopologyNodeType::Switch,
            // Map all 35 types
            _ => TopologyNodeType::Device,
        }
    }

    /// Functor G: ComputeResource → TopologyNode
    pub fn map_compute_resource(resource: &ComputeResource) -> Result<TopologyNode> {
        TopologyNode {
            name: resource.hostname.short_name().to_string(),
            node_type: map_resource_type(resource.resource_type),
            system: detect_system_arch(resource),
            hardware: map_hardware_config(resource),
            // ...
        }
    }
}
```

### Layer 3: External (nixos-topology)
**What we integrate with:**
- `topology.nix` - Main topology definition
- `nodes/*.nix` - Individual node configurations
- `networks.nix` - Network segment definitions
- `connections.nix` - Physical/logical connections

## Integration Points

### 1. Discovery Flow (Nix → Infrastructure)
```
topology.nix files
    │
    ▼ (rnix parser)
TopologyReader
    │
    ▼ (functors)
InfrastructureEvents
    │
    ▼ (NATS publish)
Event Store (JetStream)
    │
    ▼ (projections)
NetBox, Monitoring, etc.
```

### 2. Management Flow (Infrastructure → Nix)
```
User registers device via API
    │
    ▼
ComputeRegistered event
    │
    ▼ (NATS stream)
TopologyWriter (consumer)
    │
    ▼ (functors)
Update topology.nix
    │
    ▼
Git commit + push
```

## File Structure (New)

```
cim-domain-nix/
├── src/
│   ├── lib.rs                    # Main entry, documentation
│   ├── adapters/                 # Port/Adapter pattern
│   │   ├── mod.rs
│   │   ├── topology_reader.rs   # Nix → Events
│   │   ├── topology_writer.rs   # Events → Nix
│   │   └── nats_projector.rs    # NATS consumer
│   ├── functors/                 # Category theory mappings
│   │   ├── mod.rs
│   │   ├── resource_type.rs     # ResourceType ⟷ TopologyNodeType
│   │   ├── compute_resource.rs  # ComputeResource ⟷ TopologyNode
│   │   ├── network_segment.rs   # NetworkSegment ⟷ TopologyNetwork
│   │   └── laws.rs              # Verify functor laws
│   └── nix/                      # Nix AST manipulation
│       ├── mod.rs
│       ├── parser.rs            # Parse .nix files
│       ├── writer.rs            # Write .nix files
│       └── topology.rs          # Topology-specific structures
├── examples/
│   ├── read_topology.rs         # Read existing topology
│   ├── write_topology.rs        # Generate topology from events
│   └── roundtrip.rs             # Verify read→write roundtrip
└── tests/
    ├── functor_laws.rs          # Verify functors obey category laws
    └── integration.rs           # End-to-end tests
```

## Migration Plan

### Phase 1: Fix Dependencies ✅
- [x] Fix Cargo.toml: `cim-infrastructure` path
- [x] Fix src/infrastructure.rs: `cim_infrastructure` import
- [x] Add nixos-topology to flake.nix

### Phase 2: Create New Adapters (Next)
- [ ] Create `adapters/topology_reader.rs`
- [ ] Create `adapters/topology_writer.rs`
- [ ] Create `functors/resource_type.rs`
- [ ] Create `functors/compute_resource.rs`

### Phase 3: Deprecate Old Code
- [ ] Move old functor code to `_deprecated/`
- [ ] Add deprecation warnings
- [ ] Update README with migration guide

### Phase 4: Integration
- [ ] Create NATS projector service
- [ ] Test roundtrip: Read topology → Events → Write topology
- [ ] Verify functor laws hold

## Key Principles

1. **Nix is Data, Not Logic** - We parse and generate Nix files, we don't execute them
2. **Infrastructure is the Domain** - All business logic lives in cim-infrastructure
3. **Port/Adapter Pattern** - Clean separation between domain and Nix format
4. **Category Theory Functors** - Structure-preserving mappings with laws
5. **Event-Driven** - Topology updates driven by Infrastructure events

## Type Mappings

### ResourceType → TopologyNodeType
```rust
ResourceType::PhysicalServer    → TopologyNodeType::PhysicalServer
ResourceType::VirtualMachine    → TopologyNodeType::VirtualMachine
ResourceType::ContainerHost     → TopologyNodeType::Container
ResourceType::Router            → TopologyNodeType::Router
ResourceType::Switch            → TopologyNodeType::Switch
ResourceType::Firewall          → TopologyNodeType::Firewall
ResourceType::Camera            → TopologyNodeType::Device
ResourceType::KVM               → TopologyNodeType::Device
ResourceType::Monitor           → TopologyNodeType::Device
// ... 26 more mappings
```

### ComputeResource Fields → TopologyNode
```rust
ComputeResource {
    hostname: Hostname           → name: String
    resource_type: ResourceType  → node_type: TopologyNodeType
    location_id: AggregateId     → location: String (resolve via cim-domain-location)
    manufacturer: Option<String> → hardware.vendor: String
    model: Option<String>        → hardware.model: String
    metadata: HashMap            → tags: HashMap
}
```

## Next Steps

1. Create new adapter files (topology_reader.rs, topology_writer.rs)
2. Implement functors for current domain model
3. Write tests verifying functor laws
4. Create example showing roundtrip: Nix → Events → Nix
5. Document migration from old code

## References

- **cim-infrastructure**: `/git/thecowboyai/cim-infrastructure`
- **nixos-topology**: https://github.com/oddlama/nixos-topology
- **Domain Value Objects**: `/git/thecowboyai/cim-infrastructure/docs/DOMAIN_VALUE_OBJECTS.md`
