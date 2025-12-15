// Copyright 2025 Cowboy AI, LLC.

//! CIM Domain Nix: Category Theory Functor for Infrastructure
//!
//! This crate implements a Category Theory functor that maps between:
//! - **Source Category**: Nix (language and data structures)
//! - **Target Category**: Infrastructure (compute, network, software, policies)
//!
//! ## Architecture
//!
//! ```text
//! Category(Nix) ──Functor F──> Category(Infrastructure)
//!   (Data Layer)                  (Domain Model)
//! ```
//!
//! **Key Principle**: Nix is our data storage format. Infrastructure is our domain model.
//!
//! ## Phase 1: Infrastructure Domain Core ✅
//!
//! Event-sourced domain model for infrastructure, completely independent of Nix.
//!
//! ### Example
//!
//! ```rust
//! use cim_domain_nix::infrastructure::*;
//!
//! // Create aggregate
//! let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
//!
//! // Register a compute resource
//! let identity = MessageIdentity::new_root();
//! let spec = ComputeResourceSpec {
//!     id: ResourceId::new("server01").unwrap(),
//!     resource_type: ComputeType::Physical,
//!     hostname: Hostname::new("server01.example.com").unwrap(),
//!     system: SystemArchitecture::x86_64_linux(),
//!     system_description: None,
//!     capabilities: ResourceCapabilities::new(),
//! };
//!
//! infrastructure.handle_register_compute_resource(spec, &identity).unwrap();
//!
//! // Get uncommitted events
//! let events = infrastructure.take_uncommitted_events();
//! assert_eq!(events.len(), 1);
//! ```
//!
//! ## Phase 2: Nix Objects Representation ✅
//!
//! Rust representations of Nix language constructs, ready for functor mapping.
//!
//! ### Example
//!
//! ```rust
//! use cim_domain_nix::nix::*;
//!
//! // Parse a Nix string
//! let parser = NixParser::new();
//! let ast = parser.parse_str("{ x = 1; y = 2; }").unwrap();
//!
//! // Work with Nix values
//! let mut attrs = NixAttrset::new();
//! attrs.insert("name".to_string(), NixValue::String(NixString::new("hello")));
//!
//! // Parse topology
//! let topology = NixTopology::new("my-infrastructure".to_string());
//! ```
//!
//! ## Phase 3: Category Theory Functor ✅
//!
//! Structure-preserving mappings between Nix and Infrastructure categories.
//!
//! ### Example
//!
//! ```rust
//! use cim_domain_nix::functor::*;
//! use cim_domain_nix::nix::*;
//! use cim_domain_nix::infrastructure::*;
//!
//! // Create functor
//! let functor = NixInfrastructureFunctor::new();
//!
//! // Map Nix topology to Infrastructure
//! let topology = NixTopology::new("my-infra".to_string());
//! let infrastructure = functor.map_topology(&topology).unwrap();
//!
//! // Project back to Nix (for persistence)
//! let projected = functor.project_topology(&infrastructure).unwrap();
//!
//! // Verify functor laws
//! use cim_domain_nix::functor::laws::*;
//! verify_identity_for_topology(&topology).unwrap();
//! verify_composition_for_topology(&topology).unwrap();
//! ```
//!
//! ## Phase 4: Input/Output Adapters 🔨
//!
//! I/O layer for reading and writing Nix files.
//!
//! ### Example
//!
//! ```rust
//! use cim_domain_nix::io::*;
//! use cim_domain_nix::nix::topology::*;
//!
//! // Read a topology from file
//! // let topology = read_topology("topology.nix").unwrap();
//!
//! // Create and write a topology
//! let mut topology = NixTopology::new("my-infra".to_string());
//! let node = TopologyNode::new(
//!     "server01".to_string(),
//!     TopologyNodeType::PhysicalServer,
//!     "x86_64-linux".to_string(),
//! );
//! topology.add_node(node);
//!
//! // Write to file
//! // write_topology(&topology, "output.nix").unwrap();
//!
//! // Validate before writing
//! let writer = TopologyWriter::new();
//! let nix_content = writer.write_string(&topology).unwrap();
//! assert!(nix_content.contains("server01"));
//! ```
//!
//! ## Planned Phases
//!
//! - **Phase 1**: Infrastructure Domain Core ✅ (Complete)
//! - **Phase 2**: Nix Objects Representation ✅ (Complete)
//! - **Phase 3**: Category Theory Functor ✅ (Complete)
//! - **Phase 4**: Input/Output Adapters 🔨 (In Progress)
//! - **Phase 5**: NATS Integration & Projections
//!
//! ## Design Principles
//!
//! 1. **Nix is Data, Not Operations** - We read/write Nix files, not execute them
//! 2. **Infrastructure is the Domain** - All business logic, event-sourced
//! 3. **Functor Preserves Structure** - Category Theory mapping
//! 4. **Ports & Adapters** - Clean separation
//! 5. **Event Sourcing** - Nix files are projections of Infrastructure state

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

// Phase 1: Infrastructure Domain Core
pub mod infrastructure;

// Phase 2: Nix Objects Representation
pub mod nix;

// Phase 3: Category Theory Functor
pub mod functor;

// Phase 4: Input/Output Adapters
pub mod io;

// Re-export for convenience
pub use infrastructure::*;
