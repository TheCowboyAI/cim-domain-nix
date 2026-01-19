// Copyright 2025 Cowboy AI, LLC.

//! Functor Laws Verification
//!
//! This module implements verification of the two functor laws:
//!
//! 1. **Identity Preservation**: F(id_X) = id_F(X)
//!    - The functor maps identity morphisms to identity morphisms
//!
//! 2. **Composition Preservation**: F(g ∘ f) = F(g) ∘ F(f)
//!    - The functor preserves composition of morphisms
//!
//! These laws ensure that our functor is structure-preserving and maintains
//! the categorical properties of the source and target categories.

use super::{FunctorError, Result};
use crate::infrastructure::*;
use crate::nix::*;
use crate::nix::topology::*;

// ============================================================================
// Identity Law Verification
// ============================================================================

/// Verify identity preservation: F(id_X) = id_F(X)
///
/// For a topology T, verify that:
/// 1. Mapping T and immediately projecting back gives something equivalent to T
/// 2. The identity transformation on T is preserved
pub fn verify_identity_for_topology(topology: &NixTopology) -> Result<()> {
    // Map to Infrastructure
    let infrastructure = crate::functor::mappings::map_topology_to_infrastructure(topology)?;

    // Project back
    let projected = crate::functor::projections::project_infrastructure_to_topology(&infrastructure)?;

    // Verify structural equivalence (same number of nodes, networks, connections)
    if topology.nodes.len() != projected.nodes.len() {
        return Err(FunctorError::LawViolation(format!(
            "Identity law violated: node count mismatch ({} != {})",
            topology.nodes.len(),
            projected.nodes.len()
        )));
    }

    if topology.networks.len() != projected.networks.len() {
        return Err(FunctorError::LawViolation(format!(
            "Identity law violated: network count mismatch ({} != {})",
            topology.networks.len(),
            projected.networks.len()
        )));
    }

    if topology.connections.len() != projected.connections.len() {
        return Err(FunctorError::LawViolation(format!(
            "Identity law violated: connection count mismatch ({} != {})",
            topology.connections.len(),
            projected.connections.len()
        )));
    }

    // Verify node names are preserved
    for (node_name, _) in &topology.nodes {
        if !projected.nodes.contains_key(node_name) {
            return Err(FunctorError::LawViolation(format!(
                "Identity law violated: node '{}' not preserved",
                node_name
            )));
        }
    }

    // Verify network names are preserved
    for (network_name, _) in &topology.networks {
        if !projected.networks.contains_key(network_name) {
            return Err(FunctorError::LawViolation(format!(
                "Identity law violated: network '{}' not preserved",
                network_name
            )));
        }
    }

    Ok(())
}

/// Verify identity for a single resource
pub fn verify_identity_for_resource(node: &TopologyNode) -> Result<()> {
    // Create minimal topology with single node
    let mut topology = NixTopology::new("test".to_string());
    topology.add_node(node.clone());

    // Map and project
    let infrastructure = crate::functor::mappings::map_topology_to_infrastructure(&topology)?;
    let projected = crate::functor::projections::project_infrastructure_to_topology(&infrastructure)?;

    // Verify node is preserved
    if !projected.nodes.contains_key(&node.name) {
        return Err(FunctorError::LawViolation(format!(
            "Identity law violated: node '{}' not preserved",
            node.name
        )));
    }

    let projected_node = projected.nodes.get(&node.name).unwrap();

    // Verify key properties preserved
    if projected_node.system != node.system {
        return Err(FunctorError::LawViolation(format!(
            "Identity law violated: system mismatch"
        )));
    }

    Ok(())
}

// ============================================================================
// Composition Law Verification
// ============================================================================

/// Verify composition preservation: F(g ∘ f) = F(g) ∘ F(f)
///
/// This is harder to verify directly since we need Nix→Nix morphisms.
/// Instead, we verify that sequential operations preserve structure.
pub fn verify_composition_for_topology(
    topology: &NixTopology,
) -> Result<()> {
    // Operation 1: Map to Infrastructure
    let infrastructure1 = crate::functor::mappings::map_topology_to_infrastructure(topology)?;

    // Operation 2: Project back to Topology
    let topology2 = crate::functor::projections::project_infrastructure_to_topology(&infrastructure1)?;

    // Operation 3: Map again to Infrastructure
    let infrastructure2 = crate::functor::mappings::map_topology_to_infrastructure(&topology2)?;

    // Verify that infrastructure1 and infrastructure2 have same structure
    if infrastructure1.resources.len() != infrastructure2.resources.len() {
        return Err(FunctorError::LawViolation(format!(
            "Composition law violated: resource count mismatch"
        )));
    }

    if infrastructure1.networks.len() != infrastructure2.networks.len() {
        return Err(FunctorError::LawViolation(format!(
            "Composition law violated: network count mismatch"
        )));
    }

    Ok(())
}

/// Verify composition for adding a node (sequence of operations)
pub fn verify_composition_for_add_node(
    base_topology: &NixTopology,
    new_node: &TopologyNode,
) -> Result<()> {
    // Path 1: Add node in Nix, then map
    let mut topology1 = base_topology.clone();
    topology1.add_node(new_node.clone());
    let infrastructure1 = crate::functor::mappings::map_topology_to_infrastructure(&topology1)?;

    // Path 2: Map first, then add resource
    let mut infrastructure2 = crate::functor::mappings::map_topology_to_infrastructure(base_topology)?;
    let identity = MessageIdentity::new_root();

    // Add the resource to infrastructure2 (simulating the functor mapping of the add operation)
    let resource_id = ResourceId::new(&new_node.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid resource ID: {}", e)))?;

    let spec = ComputeResourceSpec {
                system_description: None,
        id: resource_id,
        resource_type: match new_node.node_type {
            TopologyNodeType::PhysicalServer => ComputeType::Physical,
            TopologyNodeType::VirtualMachine => ComputeType::VirtualMachine,
            TopologyNodeType::Container => ComputeType::Container,
            TopologyNodeType::NetworkDevice => ComputeType::Physical,
        },
        hostname: Hostname::new(format!("{}.local", new_node.name))
            .map_err(|e| FunctorError::MappingError(format!("Invalid hostname: {}", e)))?,
        system: SystemArchitecture::new(&new_node.system),
        capabilities: ResourceCapabilities::new(),
    };

    infrastructure2.handle_register_compute_resource(spec, &identity)?;

    // Both paths should result in same number of resources
    if infrastructure1.resources.len() != infrastructure2.resources.len() {
        return Err(FunctorError::LawViolation(format!(
            "Composition law violated: different resource counts ({} vs {})",
            infrastructure1.resources.len(),
            infrastructure2.resources.len()
        )));
    }

    Ok(())
}

// ============================================================================
// Round-Trip Verification (Special Case)
// ============================================================================

/// Verify round-trip property: project(map(x)) ≈ x
///
/// This is not strictly a functor law, but it's important for data integrity.
pub fn verify_round_trip_topology(topology: &NixTopology) -> Result<()> {
    // Map to Infrastructure and back
    let infrastructure = crate::functor::mappings::map_topology_to_infrastructure(topology)?;
    let projected = crate::functor::projections::project_infrastructure_to_topology(&infrastructure)?;

    // Verify structure is preserved (up to renaming/reordering)
    if topology.nodes.len() != projected.nodes.len() {
        return Err(FunctorError::LawViolation(format!(
            "Round-trip failed: node count mismatch"
        )));
    }

    if topology.networks.len() != projected.networks.len() {
        return Err(FunctorError::LawViolation(format!(
            "Round-trip failed: network count mismatch"
        )));
    }

    if topology.connections.len() != projected.connections.len() {
        return Err(FunctorError::LawViolation(format!(
            "Round-trip failed: connection count mismatch"
        )));
    }

    Ok(())
}

/// Verify round-trip for a single package
pub fn verify_round_trip_package(package: &NixPackage) -> Result<()> {
    // Map to SoftwareConfiguration and back
    let config = crate::functor::mappings::map_package_to_software_config(package)?;
    let projected = crate::functor::projections::project_software_config_to_package(&config)?;

    // Verify key properties preserved
    if package.name != projected.name {
        return Err(FunctorError::LawViolation(format!(
            "Round-trip failed: package name mismatch"
        )));
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_empty_topology() {
        let topology = NixTopology::new("test".to_string());
        let result = verify_identity_for_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identity_topology_with_node() {
        let mut topology = NixTopology::new("test".to_string());
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = verify_identity_for_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identity_single_resource() {
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );

        let result = verify_identity_for_resource(&node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_composition_empty_topology() {
        let topology = NixTopology::new("test".to_string());
        let result = verify_composition_for_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_composition_topology_with_node() {
        let mut topology = NixTopology::new("test".to_string());
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = verify_composition_for_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_composition_add_node() {
        let base_topology = NixTopology::new("test".to_string());
        let new_node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );

        let result = verify_composition_for_add_node(&base_topology, &new_node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_round_trip_empty_topology() {
        let topology = NixTopology::new("test".to_string());
        let result = verify_round_trip_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_round_trip_topology_with_node() {
        let mut topology = NixTopology::new("test".to_string());
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = verify_round_trip_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_round_trip_package() {
        let package = NixPackage::new("nginx".to_string(), "x86_64-linux".to_string())
            .with_version("1.20.0".to_string());

        let result = verify_round_trip_package(&package);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identity_preserves_system() {
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "aarch64-linux".to_string(),
        );

        let result = verify_identity_for_resource(&node);
        assert!(result.is_ok());
    }

    #[test]
    fn test_identity_topology_with_network() {
        let mut topology = NixTopology::new("test".to_string());
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN)
            .with_cidr_v4("192.168.1.0/24".to_string());
        topology.add_network(network);

        let result = verify_identity_for_topology(&topology);
        assert!(result.is_ok());
    }
}
