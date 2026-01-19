// Copyright (c) 2025 - Cowboy AI, Inc.
//! Port/Adapter Layer for nixos-topology Integration
//!
//! This module implements the port/adapter pattern (hexagonal architecture)
//! for bidirectional integration between the Infrastructure domain and
//! nixos-topology configuration files.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │        cim-infrastructure (DOMAIN)                       │
//! │  Domain events, aggregates, value objects                │
//! └─────────────────────────────────────────────────────────┘
//!                            ▲ │
//!                      READ  │ │ WRITE
//!                            │ ▼
//! ┌─────────────────────────────────────────────────────────┐
//! │          ADAPTERS (This Module)                          │
//! │  ┌──────────────────┐        ┌──────────────────┐       │
//! │  │ TopologyReader   │        │  TopologyWriter  │       │
//! │  │ Nix → Events     │        │  Events → Nix    │       │
//! │  └──────────────────┘        └──────────────────┘       │
//! │           ▲                           │                  │
//! │           │ parse                     │ generate         │
//! └───────────┼───────────────────────────┼──────────────────┘
//!             │                           ▼
//! ┌───────────┴───────────────────────────────────────────┐
//! │         nixos-topology (Nix Files)                     │
//! │  topology.nix, nodes/*.nix, networks.nix               │
//! └────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Adapters
//!
//! ### TopologyReader (Nix → Domain)
//!
//! Reads nixos-topology files and generates Infrastructure domain events.
//!
//! ```rust,no_run
//! use cim_domain_nix::adapters::topology_reader::TopologyReader;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let reader = TopologyReader::new();
//! let events = reader.read_topology_file(Path::new("topology.nix")).await?;
//!
//! for event in events {
//!     // Publish to NATS, apply to projections, etc.
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### TopologyWriter (Domain → Nix)
//!
//! Listens to Infrastructure events and updates nixos-topology files.
//!
//! ```rust,no_run
//! use cim_domain_nix::adapters::topology_writer::TopologyWriter;
//! use cim_infrastructure::domain::InfrastructureEvent;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut writer = TopologyWriter::new(Path::new("topology.nix"));
//!
//! // Listen to events and update topology
//! // writer.apply_event(event).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Design Principles
//!
//! 1. **Port/Adapter Pattern**: Clean separation between domain and infrastructure
//! 2. **Bidirectional**: Read from Nix, write to Nix
//! 3. **Event-Driven**: All changes flow through domain events
//! 4. **Functor-Based**: Type mappings use category theory functors
//! 5. **NATS Integration**: Events flow through NATS JetStream

pub mod topology_reader;
pub mod topology_writer;
// pub mod nats_projector;   // TODO

// Re-export for convenience
pub use topology_reader::TopologyReader;
pub use topology_writer::TopologyWriter;
