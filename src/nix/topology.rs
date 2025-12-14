// Copyright 2025 Cowboy AI, LLC.

//! Nix Topology Module
//!
//! This module provides support for parsing and working with nix-topology
//! (https://github.com/oddlama/nix-topology) format.
//!
//! nix-topology is the canonical format for representing infrastructure
//! topology in Nix. It defines nodes, networks, and connections.
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::nix::topology::*;
//!
//! // Parse a topology from a Nix file
//! let topology = NixTopology::from_file("topology.nix")?;
//!
//! // Access nodes and networks
//! for node in topology.nodes.values() {
//!     println!("Node: {}", node.name);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Topology errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum TopologyError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid topology
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),

    /// Missing node
    #[error("Missing node: {0}")]
    MissingNode(String),

    /// Missing network
    #[error("Missing network: {0}")]
    MissingNetwork(String),
}

/// Result type for topology operations
pub type Result<T> = std::result::Result<T, TopologyError>;

// ============================================================================
// NixTopology - Complete infrastructure topology
// ============================================================================

/// Nix Topology
///
/// Represents a complete infrastructure topology as defined by nix-topology.
/// This is the top-level structure that maps to our Infrastructure aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixTopology {
    /// Unique identifier for this topology
    pub id: Uuid,
    /// Topology name
    pub name: String,
    /// Topology nodes (compute resources)
    pub nodes: HashMap<String, TopologyNode>,
    /// Networks
    pub networks: HashMap<String, TopologyNetwork>,
    /// Connections between nodes
    pub connections: Vec<TopologyConnection>,
    /// Source file path
    pub source_path: Option<PathBuf>,
}

impl NixTopology {
    /// Create a new empty topology
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            nodes: HashMap::new(),
            networks: HashMap::new(),
            connections: Vec::new(),
            source_path: None,
        }
    }

    /// Parse topology from a Nix file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let _content = fs::read_to_string(path)
            .map_err(|e| TopologyError::IoError(format!("Failed to read file: {}", e)))?;

        // Placeholder implementation
        // Full implementation will parse nix-topology format
        let mut topology = Self::new("parsed-topology".to_string());
        topology.source_path = Some(path.to_path_buf());
        Ok(topology)
    }

    /// Add a node
    pub fn add_node(&mut self, node: TopologyNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    /// Add a network
    pub fn add_network(&mut self, network: TopologyNetwork) {
        self.networks.insert(network.name.clone(), network);
    }

    /// Add a connection
    pub fn add_connection(&mut self, connection: TopologyConnection) {
        self.connections.push(connection);
    }

    /// Get a node by name
    pub fn get_node(&self, name: &str) -> Option<&TopologyNode> {
        self.nodes.get(name)
    }

    /// Get a network by name
    pub fn get_network(&self, name: &str) -> Option<&TopologyNetwork> {
        self.networks.get(name)
    }

    /// Get all nodes of a specific type
    pub fn nodes_by_type(&self, node_type: &TopologyNodeType) -> Vec<&TopologyNode> {
        self.nodes
            .values()
            .filter(|n| &n.node_type == node_type)
            .collect()
    }

    /// Get all connections involving a specific node
    pub fn connections_for_node(&self, node_name: &str) -> Vec<&TopologyConnection> {
        self.connections
            .iter()
            .filter(|c| c.from_node == node_name || c.to_node == node_name)
            .collect()
    }
}

impl Default for NixTopology {
    fn default() -> Self {
        Self::new("default-topology".to_string())
    }
}

// ============================================================================
// TopologyNode - Compute resource
// ============================================================================

/// Topology Node
///
/// Represents a compute resource in the topology (server, VM, container, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyNode {
    /// Unique identifier
    pub id: Uuid,
    /// Node name (unique within topology)
    pub name: String,
    /// Node type
    pub node_type: TopologyNodeType,
    /// System architecture (e.g., "x86_64-linux")
    pub system: String,
    /// Hardware configuration
    pub hardware: Option<HardwareConfig>,
    /// Network interfaces
    pub interfaces: Vec<NodeInterface>,
    /// Services running on this node
    pub services: Vec<String>,
    /// Parent node (for VMs/containers)
    pub parent: Option<String>,
    /// Child nodes (guests)
    pub children: Vec<String>,
    /// Tags/labels
    pub tags: HashMap<String, String>,
}

impl TopologyNode {
    /// Create a new node
    pub fn new(name: String, node_type: TopologyNodeType, system: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            node_type,
            system,
            hardware: None,
            interfaces: Vec::new(),
            services: Vec::new(),
            parent: None,
            children: Vec::new(),
            tags: HashMap::new(),
        }
    }

    /// Add an interface
    pub fn add_interface(&mut self, interface: NodeInterface) {
        self.interfaces.push(interface);
    }

    /// Add a service
    pub fn add_service(&mut self, service: String) {
        self.services.push(service);
    }

    /// Add a tag
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

/// Node Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TopologyNodeType {
    /// Physical server
    PhysicalServer,
    /// Virtual machine
    VirtualMachine,
    /// Container
    Container,
    /// Network device (router, switch, etc.)
    NetworkDevice,
}

/// Hardware Configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareConfig {
    /// CPU cores
    pub cpu_cores: Option<u32>,
    /// Memory in MB
    pub memory_mb: Option<u64>,
    /// Storage in GB
    pub storage_gb: Option<u64>,
    /// Additional hardware details
    pub details: HashMap<String, String>,
}

impl HardwareConfig {
    /// Create empty hardware config
    pub fn new() -> Self {
        Self {
            cpu_cores: None,
            memory_mb: None,
            storage_gb: None,
            details: HashMap::new(),
        }
    }
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Node Interface
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeInterface {
    /// Interface name (e.g., "eth0")
    pub name: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// Network this interface is connected to
    pub network: Option<String>,
    /// IP address (v4 or v6)
    pub ip_address: Option<String>,
    /// Whether this is the primary interface
    pub primary: bool,
}

impl NodeInterface {
    /// Create a new interface
    pub fn new(name: String) -> Self {
        Self {
            name,
            mac_address: None,
            network: None,
            ip_address: None,
            primary: false,
        }
    }

    /// Set network
    pub fn with_network(mut self, network: String) -> Self {
        self.network = Some(network);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip: String) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Mark as primary
    pub fn as_primary(mut self) -> Self {
        self.primary = true;
        self
    }
}

// ============================================================================
// TopologyNetwork - Network definition
// ============================================================================

/// Topology Network
///
/// Represents a network in the topology (LAN, VLAN, VPN, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyNetwork {
    /// Unique identifier
    pub id: Uuid,
    /// Network name
    pub name: String,
    /// Network type
    pub network_type: NetworkType,
    /// CIDR notation (IPv4)
    pub cidr_v4: Option<String>,
    /// CIDR notation (IPv6)
    pub cidr_v6: Option<String>,
    /// VLAN ID
    pub vlan_id: Option<u16>,
    /// Tags/labels
    pub tags: HashMap<String, String>,
}

impl TopologyNetwork {
    /// Create a new network
    pub fn new(name: String, network_type: NetworkType) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            network_type,
            cidr_v4: None,
            cidr_v6: None,
            vlan_id: None,
            tags: HashMap::new(),
        }
    }

    /// Set IPv4 CIDR
    pub fn with_cidr_v4(mut self, cidr: String) -> Self {
        self.cidr_v4 = Some(cidr);
        self
    }

    /// Set IPv6 CIDR
    pub fn with_cidr_v6(mut self, cidr: String) -> Self {
        self.cidr_v6 = Some(cidr);
        self
    }

    /// Set VLAN ID
    pub fn with_vlan(mut self, vlan_id: u16) -> Self {
        self.vlan_id = Some(vlan_id);
        self
    }

    /// Add a tag
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
}

/// Network Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkType {
    /// Local Area Network
    LAN,
    /// Virtual LAN
    VLAN,
    /// Virtual Private Network
    VPN,
    /// WAN/Internet
    WAN,
    /// Management network
    Management,
}

// ============================================================================
// TopologyConnection - Physical/logical connections
// ============================================================================

/// Topology Connection
///
/// Represents a connection between two nodes (physical cable, virtual link, etc.)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyConnection {
    /// Unique identifier
    pub id: Uuid,
    /// Source node name
    pub from_node: String,
    /// Source interface name
    pub from_interface: String,
    /// Destination node name
    pub to_node: String,
    /// Destination interface name
    pub to_interface: String,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Connection speed (e.g., "1Gbps")
    pub speed: Option<String>,
}

impl TopologyConnection {
    /// Create a new connection
    pub fn new(
        from_node: String,
        from_interface: String,
        to_node: String,
        to_interface: String,
        connection_type: ConnectionType,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            from_node,
            from_interface,
            to_node,
            to_interface,
            connection_type,
            speed: None,
        }
    }

    /// Set connection speed
    pub fn with_speed(mut self, speed: String) -> Self {
        self.speed = Some(speed);
        self
    }
}

/// Connection Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Physical ethernet cable
    Ethernet,
    /// Virtual bridge
    Bridge,
    /// VPN tunnel
    VPN,
    /// Wireless connection
    Wireless,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_creation() {
        let topology = NixTopology::new("test-topology".to_string());
        assert_eq!(topology.name, "test-topology");
        assert!(topology.nodes.is_empty());
        assert!(topology.networks.is_empty());
        assert!(topology.connections.is_empty());
    }

    #[test]
    fn test_topology_add_node() {
        let mut topology = NixTopology::new("test".to_string());
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);
        assert_eq!(topology.nodes.len(), 1);
    }

    #[test]
    fn test_topology_add_network() {
        let mut topology = NixTopology::new("test".to_string());
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN);
        topology.add_network(network);
        assert_eq!(topology.networks.len(), 1);
    }

    #[test]
    fn test_topology_add_connection() {
        let mut topology = NixTopology::new("test".to_string());
        let conn = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        );
        topology.add_connection(conn);
        assert_eq!(topology.connections.len(), 1);
    }

    #[test]
    fn test_node_creation() {
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        assert_eq!(node.name, "server01");
        assert_eq!(node.node_type, TopologyNodeType::PhysicalServer);
        assert!(node.interfaces.is_empty());
    }

    #[test]
    fn test_node_add_interface() {
        let mut node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        let interface = NodeInterface::new("eth0".to_string());
        node.add_interface(interface);
        assert_eq!(node.interfaces.len(), 1);
    }

    #[test]
    fn test_node_add_service() {
        let mut node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        node.add_service("nginx".to_string());
        assert_eq!(node.services.len(), 1);
    }

    #[test]
    fn test_interface_creation() {
        let interface = NodeInterface::new("eth0".to_string());
        assert_eq!(interface.name, "eth0");
        assert!(!interface.primary);
    }

    #[test]
    fn test_interface_with_network() {
        let interface = NodeInterface::new("eth0".to_string())
            .with_network("lan".to_string());
        assert_eq!(interface.network, Some("lan".to_string()));
    }

    #[test]
    fn test_interface_with_ip() {
        let interface = NodeInterface::new("eth0".to_string())
            .with_ip("192.168.1.10".to_string());
        assert_eq!(interface.ip_address, Some("192.168.1.10".to_string()));
    }

    #[test]
    fn test_interface_as_primary() {
        let interface = NodeInterface::new("eth0".to_string()).as_primary();
        assert!(interface.primary);
    }

    #[test]
    fn test_network_creation() {
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN);
        assert_eq!(network.name, "lan");
        assert_eq!(network.network_type, NetworkType::LAN);
    }

    #[test]
    fn test_network_with_cidr_v4() {
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN)
            .with_cidr_v4("192.168.1.0/24".to_string());
        assert_eq!(network.cidr_v4, Some("192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_network_with_vlan() {
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::VLAN)
            .with_vlan(100);
        assert_eq!(network.vlan_id, Some(100));
    }

    #[test]
    fn test_connection_creation() {
        let conn = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        );
        assert_eq!(conn.from_node, "server01");
        assert_eq!(conn.to_node, "server02");
    }

    #[test]
    fn test_connection_with_speed() {
        let conn = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        )
        .with_speed("1Gbps".to_string());
        assert_eq!(conn.speed, Some("1Gbps".to_string()));
    }

    #[test]
    fn test_hardware_config() {
        let hw = HardwareConfig::new();
        assert!(hw.cpu_cores.is_none());
        assert!(hw.memory_mb.is_none());
    }

    #[test]
    fn test_topology_get_node() {
        let mut topology = NixTopology::new("test".to_string());
        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);
        assert!(topology.get_node("server01").is_some());
        assert!(topology.get_node("nonexistent").is_none());
    }

    #[test]
    fn test_topology_nodes_by_type() {
        let mut topology = NixTopology::new("test".to_string());
        topology.add_node(TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        ));
        topology.add_node(TopologyNode::new(
            "vm01".to_string(),
            TopologyNodeType::VirtualMachine,
            "x86_64-linux".to_string(),
        ));

        let servers = topology.nodes_by_type(&TopologyNodeType::PhysicalServer);
        assert_eq!(servers.len(), 1);

        let vms = topology.nodes_by_type(&TopologyNodeType::VirtualMachine);
        assert_eq!(vms.len(), 1);
    }

    // ============================================================================
    // Additional Tests for 90% Coverage
    // ============================================================================

    #[test]
    fn test_topology_default() {
        let topology = NixTopology::default();
        assert_eq!(topology.name, "default-topology");
        assert!(topology.nodes.is_empty());
    }

    #[test]
    fn test_network_with_cidr_v6() {
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN)
            .with_cidr_v6("fe80::/64".to_string());
        assert_eq!(network.cidr_v6, Some("fe80::/64".to_string()));
    }

    #[test]
    fn test_network_add_tag() {
        let mut network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN);
        network.add_tag("environment".to_string(), "production".to_string());
        assert_eq!(network.tags.get("environment"), Some(&"production".to_string()));
    }

    #[test]
    fn test_node_with_hardware() {
        let mut node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        let mut hw = HardwareConfig::new();
        hw.cpu_cores = Some(8);
        hw.memory_mb = Some(16384);
        hw.storage_gb = Some(1000);
        node.hardware = Some(hw);

        assert!(node.hardware.is_some());
        assert_eq!(node.hardware.as_ref().unwrap().cpu_cores, Some(8));
    }

    #[test]
    fn test_hardware_config_details() {
        let mut hw = HardwareConfig::new();
        hw.details.insert("vendor".to_string(), "Dell".to_string());
        hw.details.insert("model".to_string(), "R730".to_string());
        assert_eq!(hw.details.get("vendor"), Some(&"Dell".to_string()));
    }

    #[test]
    fn test_node_type_variants() {
        let phys = TopologyNodeType::PhysicalServer;
        let vm = TopologyNodeType::VirtualMachine;
        let cont = TopologyNodeType::Container;
        let net = TopologyNodeType::NetworkDevice;

        assert_ne!(phys, vm);
        assert_ne!(cont, net);
    }

    #[test]
    fn test_network_type_variants() {
        let lan = NetworkType::LAN;
        let vlan = NetworkType::VLAN;
        let vpn = NetworkType::VPN;
        let wan = NetworkType::WAN;
        let mgmt = NetworkType::Management;

        assert_ne!(lan, vlan);
        assert_ne!(vpn, wan);
        assert_ne!(lan, mgmt);
    }

    #[test]
    fn test_connection_type_variants() {
        let eth = ConnectionType::Ethernet;
        let bridge = ConnectionType::Bridge;
        let vpn = ConnectionType::VPN;
        let wireless = ConnectionType::Wireless;

        assert_ne!(eth, bridge);
        assert_ne!(vpn, wireless);
    }

    #[test]
    fn test_topology_get_network() {
        let mut topology = NixTopology::new("test".to_string());
        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN);
        topology.add_network(network);
        assert!(topology.get_network("lan").is_some());
        assert!(topology.get_network("wan").is_none());
    }

    #[test]
    fn test_topology_connections_for_node() {
        let mut topology = NixTopology::new("test".to_string());
        topology.add_connection(TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        ));
        topology.add_connection(TopologyConnection::new(
            "server01".to_string(),
            "eth1".to_string(),
            "server03".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        ));

        let conns = topology.connections_for_node("server01");
        assert_eq!(conns.len(), 2);

        let conns = topology.connections_for_node("server02");
        assert_eq!(conns.len(), 1);
    }

    #[test]
    fn test_node_tags() {
        let mut node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        node.add_tag("version".to_string(), "1.0".to_string());
        node.add_tag("author".to_string(), "admin".to_string());

        assert_eq!(node.tags.get("version"), Some(&"1.0".to_string()));
        assert_eq!(node.tags.get("author"), Some(&"admin".to_string()));
    }

    #[test]
    fn test_topology_serialization() {
        let mut topology = NixTopology::new("test".to_string());
        topology.add_node(TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        ));

        let serialized = serde_json::to_string(&topology).unwrap();
        assert!(serialized.contains("server01"));
        assert!(serialized.contains("PhysicalServer"));
    }

    #[test]
    fn test_node_children_and_parent() {
        let mut node = TopologyNode::new(
            "hypervisor".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        node.children.push("vm01".to_string());
        node.children.push("vm02".to_string());
        node.parent = Some("cluster".to_string());

        assert_eq!(node.children.len(), 2);
        assert_eq!(node.parent, Some("cluster".to_string()));
    }

    #[test]
    fn test_interface_mac_address() {
        let mut interface = NodeInterface::new("eth0".to_string());
        interface.mac_address = Some("00:11:22:33:44:55".to_string());
        assert_eq!(interface.mac_address, Some("00:11:22:33:44:55".to_string()));
    }

    #[test]
    fn test_connection_interfaces() {
        let conn = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "switch01".to_string(),
            "port1".to_string(),
            ConnectionType::Ethernet,
        );
        assert_eq!(conn.from_interface, "eth0");
        assert_eq!(conn.to_interface, "port1");
    }

    #[test]
    fn test_network_types_hash() {
        use std::collections::HashSet;
        let mut types: HashSet<NetworkType> = HashSet::new();
        types.insert(NetworkType::LAN);
        types.insert(NetworkType::VLAN);
        types.insert(NetworkType::VPN);
        assert_eq!(types.len(), 3);
    }

    #[test]
    fn test_connection_types_hash() {
        use std::collections::HashSet;
        let mut types: HashSet<ConnectionType> = HashSet::new();
        types.insert(ConnectionType::Ethernet);
        types.insert(ConnectionType::Bridge);
        types.insert(ConnectionType::Wireless);
        assert_eq!(types.len(), 3);
    }
}
