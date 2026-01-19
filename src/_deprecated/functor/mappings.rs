// Copyright 2025 Cowboy AI, LLC.

//! Object Mappings: Category(Nix) → Category(Infrastructure)
//!
//! This module implements the object mapping component of the functor,
//! converting Nix data structures into Infrastructure domain objects.
//!
//! ## Mappings
//!
//! 1. **NixTopology → InfrastructureAggregate** (primary mapping)
//!    - Maps complete topology with nodes, networks, connections
//!    - Generates all necessary events for aggregate construction
//!
//! 2. **NixFlake → InfrastructureAggregate**
//!    - Maps flake inputs/outputs to infrastructure components
//!
//! 3. **NixPackage → SoftwareConfiguration**
//!    - Maps package metadata to software configuration
//!
//! 4. **NixModule → ComputeResource**
//!    - Maps NixOS module to compute resource with configuration
//!
//! 5. **NixApplication → SoftwareArtifact**
//!    - Maps application to deployed service metadata

use super::{FunctorError, Result};
use crate::infrastructure::*;
use crate::nix::*;
use crate::nix::topology::*;

// ============================================================================
// 1. NixTopology → InfrastructureAggregate (PRIMARY MAPPING)
// ============================================================================

/// Map NixTopology to InfrastructureAggregate
///
/// This is the main functor mapping that converts a complete Nix topology
/// specification into an Infrastructure aggregate.
///
/// ## Process
///
/// 1. Create new Infrastructure aggregate
/// 2. Map all topology nodes to compute resources
/// 3. Map all networks
/// 4. Map all connections
/// 5. Apply all events to build aggregate state
pub fn map_topology_to_infrastructure(topology: &NixTopology) -> Result<InfrastructureAggregate> {
    let infrastructure_id = InfrastructureId::new();
    let mut aggregate = InfrastructureAggregate::new(infrastructure_id);
    let identity = MessageIdentity::new_root();

    // Map all nodes to compute resources
    for (_node_name, node) in &topology.nodes {
        map_topology_node_to_resource(&mut aggregate, node, &identity)?;
    }

    // Map all networks
    for (_network_name, network) in &topology.networks {
        map_topology_network_to_network(&mut aggregate, network, &identity)?;
    }

    // Map all connections
    for connection in &topology.connections {
        map_topology_connection(&mut aggregate, connection, &identity)?;
    }

    Ok(aggregate)
}

/// Map a topology node to a compute resource
fn map_topology_node_to_resource(
    aggregate: &mut InfrastructureAggregate,
    node: &TopologyNode,
    identity: &MessageIdentity,
) -> Result<()> {
    // Convert TopologyNodeType to ComputeType
    let compute_type = match node.node_type {
        TopologyNodeType::PhysicalServer => ComputeType::Physical,
        TopologyNodeType::VirtualMachine => ComputeType::VirtualMachine,
        TopologyNodeType::Container => ComputeType::Container,
        TopologyNodeType::NetworkDevice => ComputeType::Physical, // Network devices as physical
    };

    // Create hostname (use node name if no better option)
    let hostname = Hostname::new(format!("{}.local", node.name))
        .map_err(|e| FunctorError::MappingError(format!("Invalid hostname: {}", e)))?;

    // Parse system architecture
    let system = SystemArchitecture::new(&node.system);

    // Build capabilities from hardware config
    let mut capabilities = ResourceCapabilities::new();
    if let Some(hw) = &node.hardware {
        capabilities.cpu_cores = hw.cpu_cores;
        capabilities.memory_mb = hw.memory_mb;
        capabilities.storage_gb = hw.storage_gb;
        capabilities.metadata = hw.details.clone();
    }

    // Create resource ID from node name
    let resource_id = ResourceId::new(&node.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid resource ID: {}", e)))?;

    // Register compute resource
    let spec = ComputeResourceSpec {
                system_description: None,
        id: resource_id.clone(),
        resource_type: compute_type,
        hostname,
        system,
        capabilities,
    };

    aggregate.handle_register_compute_resource(spec, identity)?;

    // Map interfaces
    for interface in &node.interfaces {
        map_node_interface(aggregate, &resource_id, interface, identity)?;
    }

    Ok(())
}

/// Map a node interface
fn map_node_interface(
    aggregate: &mut InfrastructureAggregate,
    resource_id: &ResourceId,
    interface: &NodeInterface,
    identity: &MessageIdentity,
) -> Result<()> {
    let interface_id = InterfaceId::new(&interface.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid interface ID: {}", e)))?;

    // Parse IP address if available
    let addresses = if let Some(ip_str) = &interface.ip_address {
        if let Ok(ip) = ip_str.parse() {
            vec![ip]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let spec = InterfaceSpec {
        id: interface_id,
        resource_id: resource_id.clone(),
        network_id: interface.network.as_ref().and_then(|n| NetworkId::new(n).ok()),
        addresses,
    };

    aggregate.handle_add_interface(spec, identity)?;
    Ok(())
}

/// Map a topology network
fn map_topology_network_to_network(
    aggregate: &mut InfrastructureAggregate,
    network: &TopologyNetwork,
    identity: &MessageIdentity,
) -> Result<()> {
    let network_id = NetworkId::new(&network.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid network ID: {}", e)))?;

    // Parse CIDR if available
    let cidr_v4 = if let Some(cidr) = &network.cidr_v4 {
        // Parse CIDR notation like "192.168.1.0/24"
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(addr), Ok(prefix)) = (parts[0].parse(), parts[1].parse()) {
                Some(Ipv4Network::new(addr, prefix)
                    .map_err(|e| FunctorError::MappingError(format!("Invalid IPv4 CIDR: {}", e)))?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let cidr_v6 = if let Some(cidr) = &network.cidr_v6 {
        // Parse CIDR notation like "fe80::/64"
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(addr), Ok(prefix)) = (parts[0].parse(), parts[1].parse()) {
                Some(Ipv6Network::new(addr, prefix)
                    .map_err(|e| FunctorError::MappingError(format!("Invalid IPv6 CIDR: {}", e)))?)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let spec = NetworkSpec {
        id: network_id,
        name: network.name.clone(),
        cidr_v4,
        cidr_v6,
    };

    aggregate.handle_define_network(spec, identity)?;
    Ok(())
}

/// Map a topology connection
fn map_topology_connection(
    aggregate: &mut InfrastructureAggregate,
    connection: &TopologyConnection,
    identity: &MessageIdentity,
) -> Result<()> {
    let from_resource = ResourceId::new(&connection.from_node)
        .map_err(|e| FunctorError::MappingError(format!("Invalid from_resource: {}", e)))?;

    let from_interface = InterfaceId::new(&connection.from_interface)
        .map_err(|e| FunctorError::MappingError(format!("Invalid from_interface: {}", e)))?;

    let to_resource = ResourceId::new(&connection.to_node)
        .map_err(|e| FunctorError::MappingError(format!("Invalid to_resource: {}", e)))?;

    let to_interface = InterfaceId::new(&connection.to_interface)
        .map_err(|e| FunctorError::MappingError(format!("Invalid to_interface: {}", e)))?;

    let spec = ConnectionSpec {
        from_resource,
        from_interface,
        to_resource,
        to_interface,
    };

    aggregate.handle_connect_resources(spec, identity)?;
    Ok(())
}

// ============================================================================
// 2. NixFlake → InfrastructureAggregate
// ============================================================================

/// Map NixFlake to InfrastructureAggregate
///
/// Flakes represent complete, reproducible configurations. We map them
/// to a complete Infrastructure aggregate.
pub fn map_flake_to_infrastructure(_flake: &NixFlake) -> Result<InfrastructureAggregate> {
    let infrastructure_id = InfrastructureId::new();
    let aggregate = InfrastructureAggregate::new(infrastructure_id);

    // For now, create empty aggregate
    // Full implementation would parse flake outputs and map them
    Ok(aggregate)
}

// ============================================================================
// 3. NixPackage → SoftwareConfiguration
// ============================================================================

/// Map NixPackage to SoftwareConfiguration
///
/// Packages represent installable software with metadata.
pub fn map_package_to_software_config(package: &NixPackage) -> Result<SoftwareConfiguration> {
    let config_id = ConfigurationId::new();

    // Create software artifact
    let software_id = SoftwareId::new(&package.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid software ID: {}", e)))?;

    let version = if let Some(ver) = &package.version {
        Version::new(ver)
    } else {
        Version::new("0.0.0")
    };

    let artifact = SoftwareArtifact {
        id: software_id.clone(),
        name: package.name.clone(),
        version,
        derivation_path: package.drv_path.clone(),
    };

    // We need a resource_id - use a placeholder for now
    let resource_id = ResourceId::new("placeholder")
        .map_err(|e| FunctorError::MappingError(format!("Invalid resource ID: {}", e)))?;

    let config = SoftwareConfiguration {
        id: config_id,
        resource_id,
        software: artifact,
        configuration_data: serde_json::Value::Object(serde_json::Map::new()),
        dependencies: Vec::new(),
    };

    Ok(config)
}

// ============================================================================
// 4. NixModule → ComputeResource
// ============================================================================

/// Map NixModule to ComputeResource
///
/// NixOS modules define system configuration. We map them to compute resources.
pub fn map_module_to_compute_resource(module: &NixModule) -> Result<ComputeResource> {
    let resource_id = ResourceId::new(&module.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid resource ID: {}", e)))?;

    // Extract system from module config
    let system = SystemArchitecture::new("x86_64-linux");

    let hostname = Hostname::new(format!("{}.local", module.name))
        .map_err(|e| FunctorError::MappingError(format!("Invalid hostname: {}", e)))?;

    let resource = ComputeResource {
                system_description: None,
        id: resource_id,
        resource_type: ComputeType::Physical,
        hostname,
        system,
        capabilities: ResourceCapabilities::new(),
        interfaces: Vec::new(),
        services: Vec::new(),
        guests: Vec::new(),
    };

    Ok(resource)
}

// ============================================================================
// 5. NixApplication → SoftwareArtifact
// ============================================================================

/// Map NixApplication to SoftwareArtifact
///
/// Applications are executable programs.
pub fn map_application_to_software_artifact(app: &NixApplication) -> Result<SoftwareArtifact> {
    let software_id = SoftwareId::new(&app.name)
        .map_err(|e| FunctorError::MappingError(format!("Invalid software ID: {}", e)))?;

    let version = Version::new("1.0.0");

    let artifact = SoftwareArtifact {
        id: software_id,
        name: app.name.clone(),
        version,
        derivation_path: Some(app.program.clone()),
    };

    Ok(artifact)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_empty_topology() {
        let topology = NixTopology::new("test".to_string());
        let result = map_topology_to_infrastructure(&topology);
        assert!(result.is_ok());

        let aggregate = result.unwrap();
        assert_eq!(aggregate.resources.len(), 0);
        assert_eq!(aggregate.networks.len(), 0);
    }

    #[test]
    fn test_map_topology_with_node() {
        let mut topology = NixTopology::new("test".to_string());

        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = map_topology_to_infrastructure(&topology);
        assert!(result.is_ok());

        let aggregate = result.unwrap();
        assert_eq!(aggregate.resources.len(), 1);
    }

    #[test]
    fn test_map_topology_with_network() {
        let mut topology = NixTopology::new("test".to_string());

        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN)
            .with_cidr_v4("192.168.1.0/24".to_string());
        topology.add_network(network);

        let result = map_topology_to_infrastructure(&topology);
        assert!(result.is_ok());

        let aggregate = result.unwrap();
        assert_eq!(aggregate.networks.len(), 1);
    }

    #[test]
    fn test_map_package() {
        let package = NixPackage::new("hello".to_string(), "x86_64-linux".to_string())
            .with_version("2.10".to_string());

        let result = map_package_to_software_config(&package);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.software.name, "hello");
        assert_eq!(format!("{}", config.software.version), "2.10");
    }

    #[test]
    fn test_map_module() {
        let module = NixModule::new("webserver".to_string());

        let result = map_module_to_compute_resource(&module);
        assert!(result.is_ok());

        let resource = result.unwrap();
        assert_eq!(format!("{}", resource.hostname), "webserver.local");
    }

    #[test]
    fn test_map_application() {
        let app = NixApplication::new(
            "myapp".to_string(),
            "/nix/store/abc-myapp/bin/myapp".to_string(),
            "x86_64-linux".to_string(),
        );

        let result = map_application_to_software_artifact(&app);
        assert!(result.is_ok());

        let artifact = result.unwrap();
        assert_eq!(artifact.name, "myapp");
        assert_eq!(artifact.derivation_path, Some("/nix/store/abc-myapp/bin/myapp".to_string()));
    }

    #[test]
    fn test_map_flake() {
        let flake = NixFlake::new(
            "Test flake".to_string(),
            std::path::PathBuf::from("/tmp/flake"),
        );

        let result = map_flake_to_infrastructure(&flake);
        assert!(result.is_ok());
    }
}
