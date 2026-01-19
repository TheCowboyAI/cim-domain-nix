// Copyright (c) 2025 - Cowboy AI, Inc.
//! Category Theory Functors for Infrastructure ⟷ Nix Mappings
//!
//! This module implements functors (structure-preserving mappings) between
//! the Infrastructure domain category and the Nix data category.
//!
//! ## Functor Theory
//!
//! A functor F: C → D between categories C and D must satisfy:
//!
//! 1. **Object Mapping**: F maps objects in C to objects in D
//! 2. **Morphism Mapping**: F maps morphisms in C to morphisms in D
//! 3. **Identity Preservation**: F(id_X) = id_F(X)
//! 4. **Composition Preservation**: F(g ∘ f) = F(g) ∘ F(f)
//!
//! ## Functors Implemented
//!
//! ### Bidirectional Functors
//!
//! - **ResourceType ⟷ TopologyNodeType**: Maps infrastructure taxonomy to topology types
//!   - F: ResourceType → TopologyNodeType (35 types → 9 types)
//!   - G: TopologyNodeType → ResourceType (9 types → 9 conservative defaults)
//!   - Note: Many-to-one mapping, G(F(x)) ≠ x for specialized devices
//!
//! ### Future Functors (Planned)
//!
//! - **ComputeResource ⟷ TopologyNode**: Maps full resource entities
//! - **NetworkSegment ⟷ TopologyNetwork**: Maps network configurations
//! - **Interface ⟷ TopologyInterface**: Maps network interfaces
//!
//! ## Examples
//!
//! ```rust
//! use cim_domain_nix::functors::resource_type_functor::*;
//! use cim_infrastructure::ResourceType;
//!
//! // Forward mapping
//! let topology_type = map_resource_type_to_topology(ResourceType::Router);
//! assert_eq!(topology_type, TopologyNodeType::Router);
//!
//! // Reverse mapping
//! let resource_type = map_topology_to_resource_type(TopologyNodeType::Router);
//! assert_eq!(resource_type, ResourceType::Router);
//!
//! // Check if roundtrip works
//! assert!(can_roundtrip(ResourceType::Router));  // Direct mappings work
//! assert!(!can_roundtrip(ResourceType::Camera)); // Specialized devices don't
//! ```

pub mod resource_type_functor;

// Re-export for convenience
pub use resource_type_functor::*;
