// Copyright 2025 Cowboy AI, LLC.

//! Projections: Category(Infrastructure) → Category(Nix)
//!
//! This module implements the reverse mappings (projections) that convert
//! Infrastructure domain objects back into Nix data structures.
//!
//! These projections are used to write Infrastructure state to Nix files,
//! enabling Nix as a persistent storage format for Infrastructure configuration.
//!
//! ## Projections
//!
//! 1. **InfrastructureAggregate → NixTopology** (primary projection)
//!    - Converts complete aggregate to nix-topology format
//!
//! 2. **SoftwareConfiguration → NixPackage**
//!    - Projects software config back to package definition
//!
//! 3. **ComputeResource → TopologyNode**
//!    - Projects resource to topology node
//!
//! ## Round-Trip Property
//!
//! Ideally, projections should satisfy: `project(map(x)) ≈ x`
//! (up to semantic equivalence)

use super::Result;
use crate::infrastructure::*;
use crate::nix::*;
use crate::nix::topology::*;

// ============================================================================
// 1. InfrastructureAggregate → NixTopology (PRIMARY PROJECTION)
// ============================================================================

/// Project InfrastructureAggregate to NixTopology
///
/// This is the main projection that converts Infrastructure state back
/// to nix-topology format for persistence.
pub fn project_infrastructure_to_topology(
    infrastructure: &InfrastructureAggregate,
) -> Result<NixTopology> {
    let mut topology = NixTopology::new(format!("infrastructure-{}", infrastructure.id));

    // Project all compute resources to topology nodes
    for (_resource_id, resource) in &infrastructure.resources {
        let node = project_resource_to_topology_node(resource, infrastructure)?;
        topology.add_node(node);
    }

    // Project all networks
    for (_network_id, network) in &infrastructure.networks {
        let topo_network = project_network_to_topology_network(network)?;
        topology.add_network(topo_network);
    }

    // Project all connections
    for connection in &infrastructure.connections {
        let topo_conn = project_connection_to_topology_connection(connection)?;
        topology.add_connection(topo_conn);
    }

    Ok(topology)
}

/// Project ComputeResource to TopologyNode
fn project_resource_to_topology_node(
    resource: &ComputeResource,
    infrastructure: &InfrastructureAggregate,
) -> Result<TopologyNode> {
    // Convert ComputeType to TopologyNodeType
    let node_type = match resource.resource_type {
        ComputeType::Physical => TopologyNodeType::PhysicalServer,
        ComputeType::VirtualMachine => TopologyNodeType::VirtualMachine,
        ComputeType::Container => TopologyNodeType::Container,
    };

    let mut node = TopologyNode::new(
        format!("{}", resource.id),
        node_type,
        format!("{}", resource.system),
    );

    // Project capabilities to hardware config
    if resource.capabilities.cpu_cores.is_some()
        || resource.capabilities.memory_mb.is_some()
        || resource.capabilities.storage_gb.is_some()
    {
        let mut hw = HardwareConfig::new();
        hw.cpu_cores = resource.capabilities.cpu_cores;
        hw.memory_mb = resource.capabilities.memory_mb;
        hw.storage_gb = resource.capabilities.storage_gb;
        hw.details = resource.capabilities.metadata.clone();
        node.hardware = Some(hw);
    }

    // Project interfaces
    for interface_id in &resource.interfaces {
        if let Some(interface) = infrastructure.interfaces.get(interface_id) {
            let node_interface = project_interface_to_node_interface(interface)?;
            node.add_interface(node_interface);
        }
    }

    // Add services
    for service_id in &resource.services {
        node.add_service(format!("{}", service_id));
    }

    Ok(node)
}

/// Project NetworkInterface to NodeInterface
fn project_interface_to_node_interface(interface: &NetworkInterface) -> Result<NodeInterface> {
    let node_interface = NodeInterface::new(format!("{}", interface.id));

    // Note: network, MAC address, and IP address would need additional lookup
    // For now, just use the interface ID as name

    Ok(node_interface)
}

/// Project Network to TopologyNetwork
fn project_network_to_topology_network(network: &Network) -> Result<TopologyNetwork> {
    let mut topo_network = TopologyNetwork::new(
        format!("{}", network.id),
        NetworkType::LAN, // Default, could be inferred from network properties
    );

    topo_network.name = network.name.clone();

    // Project CIDR ranges
    if let Some(cidr_v4) = &network.cidr_v4 {
        topo_network.cidr_v4 = Some(format!("{}/{}", cidr_v4.address, cidr_v4.prefix_len));
    }

    if let Some(cidr_v6) = &network.cidr_v6 {
        topo_network.cidr_v6 = Some(format!("{}/{}", cidr_v6.address, cidr_v6.prefix_len));
    }

    Ok(topo_network)
}

/// Project PhysicalConnection to TopologyConnection
fn project_connection_to_topology_connection(
    connection: &PhysicalConnection,
) -> Result<TopologyConnection> {
    let topo_conn = TopologyConnection::new(
        format!("{}", connection.from_resource),
        format!("{}", connection.from_interface),
        format!("{}", connection.to_resource),
        format!("{}", connection.to_interface),
        ConnectionType::Ethernet, // Default
    );

    Ok(topo_conn)
}

// ============================================================================
// 2. SoftwareConfiguration → NixPackage
// ============================================================================

/// Project SoftwareConfiguration to NixPackage
pub fn project_software_config_to_package(config: &SoftwareConfiguration) -> Result<NixPackage> {
    let package = NixPackage::new(
        config.software.name.clone(),
        "x86_64-linux".to_string(), // Default system
    )
    .with_version(format!("{}", config.software.version))
    .with_description(format!("Software: {}", config.software.name));

    Ok(package)
}

// ============================================================================
// 3. ComputeResource → TopologyNode (standalone)
// ============================================================================

/// Project standalone ComputeResource to TopologyNode
///
/// This is a simplified version that doesn't require the full aggregate context.
pub fn project_resource_to_node(resource: &ComputeResource) -> Result<TopologyNode> {
    let node_type = match resource.resource_type {
        ComputeType::Physical => TopologyNodeType::PhysicalServer,
        ComputeType::VirtualMachine => TopologyNodeType::VirtualMachine,
        ComputeType::Container => TopologyNodeType::Container,
    };

    let mut node = TopologyNode::new(
        format!("{}", resource.id),
        node_type,
        format!("{}", resource.system),
    );

    // Project capabilities
    if resource.capabilities.cpu_cores.is_some()
        || resource.capabilities.memory_mb.is_some()
        || resource.capabilities.storage_gb.is_some()
    {
        let mut hw = HardwareConfig::new();
        hw.cpu_cores = resource.capabilities.cpu_cores;
        hw.memory_mb = resource.capabilities.memory_mb;
        hw.storage_gb = resource.capabilities.storage_gb;
        hw.details = resource.capabilities.metadata.clone();
        node.hardware = Some(hw);
    }

    Ok(node)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_empty_infrastructure() {
        let infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
        let result = project_infrastructure_to_topology(&infrastructure);
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.nodes.len(), 0);
        assert_eq!(topology.networks.len(), 0);
    }

    #[test]
    fn test_project_infrastructure_with_resource() {
        let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        let spec = ComputeResourceSpec {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            system_description: None,
            capabilities: ResourceCapabilities::new(),
        };

        infrastructure
            .handle_register_compute_resource(spec, &identity)
            .unwrap();

        let result = project_infrastructure_to_topology(&infrastructure);
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.nodes.len(), 1);
    }

    #[test]
    fn test_project_infrastructure_with_network() {
        let mut infrastructure = InfrastructureAggregate::new(InfrastructureId::new());
        let identity = MessageIdentity::new_root();

        let spec = NetworkSpec {
            id: NetworkId::new("lan").unwrap(),
            name: "LAN".to_string(),
            cidr_v4: Some(Ipv4Network::new(std::net::Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()),
            cidr_v6: None,
        };

        infrastructure
            .handle_define_network(spec, &identity)
            .unwrap();

        let result = project_infrastructure_to_topology(&infrastructure);
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.networks.len(), 1);
    }

    #[test]
    fn test_project_software_config() {
        let software = SoftwareArtifact {
            id: SoftwareId::new("nginx").unwrap(),
            name: "nginx".to_string(),
            version: Version::new("1.20.0"),
            derivation_path: None,
        };

        let config = SoftwareConfiguration {
            id: ConfigurationId::new(),
            resource_id: ResourceId::new("server01").unwrap(),
            software,
            configuration_data: serde_json::Value::Object(serde_json::Map::new()),
            dependencies: Vec::new(),
        };

        let result = project_software_config_to_package(&config);
        assert!(result.is_ok());

        let package = result.unwrap();
        assert_eq!(package.name, "nginx");
        assert_eq!(package.version, Some("1.20.0".to_string()));
    }

    #[test]
    fn test_project_resource_to_node() {
        let resource = ComputeResource {
            id: ResourceId::new("server01").unwrap(),
            resource_type: ComputeType::Physical,
            hostname: Hostname::new("server01.local").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            system_description: None,
            capabilities: ResourceCapabilities::new(),
            interfaces: Vec::new(),
            services: Vec::new(),
            guests: Vec::new(),
        };

        let result = project_resource_to_node(&resource);
        assert!(result.is_ok());

        let node = result.unwrap();
        assert_eq!(node.name, "server01");
        assert_eq!(node.node_type, TopologyNodeType::PhysicalServer);
    }

    #[test]
    fn test_round_trip_resource() {
        // Create original node
        let original_node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );

        // Map to Infrastructure
        let mut topology = NixTopology::new("test".to_string());
        topology.add_node(original_node.clone());

        let infrastructure = crate::functor::mappings::map_topology_to_infrastructure(&topology)
            .unwrap();

        // Project back to Topology
        let projected_topology = project_infrastructure_to_topology(&infrastructure).unwrap();

        // Verify round-trip
        assert_eq!(projected_topology.nodes.len(), 1);
        assert!(projected_topology.nodes.contains_key("server01"));
    }
}
