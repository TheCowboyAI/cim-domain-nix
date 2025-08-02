// Copyright 2025 Cowboy AI, LLC.

//! Value objects for network integration

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Represents a network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    /// Unique identifier for the topology
    pub id: uuid::Uuid,
    /// Human-readable name of the topology
    pub name: String,
    /// List of network nodes in this topology
    nodes: Vec<NetworkNode>,
    /// List of connections between nodes
    connections: Vec<super::NetworkConnection>,
}

impl NetworkTopology {
    /// Create a new network topology
    pub fn new(
        id: uuid::Uuid,
        name: String,
        nodes: Vec<NetworkNode>,
        connections: Vec<super::NetworkConnection>,
    ) -> Self {
        Self {
            id,
            name,
            nodes,
            connections,
        }
    }
    
    /// Get all nodes in the topology
    pub fn nodes(&self) -> &[NetworkNode] {
        &self.nodes
    }
    
    /// Get all connections in the topology
    pub fn connections(&self) -> &[super::NetworkConnection] {
        &self.connections
    }
    
    /// Find a node by ID
    pub fn find_node(&self, node_id: uuid::Uuid) -> Option<&NetworkNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }
    
    /// Get all connections for a specific node
    pub fn node_connections(&self, node_id: uuid::Uuid) -> Vec<&super::NetworkConnection> {
        self.connections
            .iter()
            .filter(|c| c.from_node == node_id || c.to_node == node_id)
            .collect()
    }
    
    /// Check if topology has WireGuard connections
    pub fn has_wireguard(&self) -> bool {
        self.connections.iter().any(|c| matches!(c.network_type, super::NetworkType::Wireguard))
    }
    
    /// Get subnet for a specific network type
    pub fn get_subnet_for_type(&self, network_type: &super::NetworkType) -> Option<String> {
        // This would be more sophisticated in a real implementation
        match network_type {
            super::NetworkType::Ethernet => Some("192.168.1.0/24".to_string()),
            super::NetworkType::Wifi => Some("192.168.10.0/24".to_string()),
            super::NetworkType::Wireguard => Some("10.0.0.0/24".to_string()),
            super::NetworkType::Vlan(id) => Some(format!("172.16.{}.0/24", id % 256)),
            _ => None,
        }
    }
}

/// Represents a node in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    /// Unique identifier for the node
    pub id: uuid::Uuid,
    /// Human-readable name of the node
    pub name: String,
    /// Type of network node
    pub node_type: NodeType,
    /// Network interfaces attached to this node
    pub interfaces: Vec<NetworkInterface>,
    /// Services running on this node
    pub services: Vec<String>,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

impl NetworkNode {
    /// Check if node should act as a router
    pub fn is_router(&self) -> bool {
        matches!(self.node_type, NodeType::Router | NodeType::Gateway)
    }
    
    /// Check if node should run DHCP server
    pub fn should_run_dhcp(&self) -> bool {
        self.services.contains(&"dhcp".to_string()) || self.is_router()
    }
    
    /// Check if node should run DNS server
    pub fn should_run_dns(&self) -> bool {
        self.services.contains(&"dns".to_string()) || self.is_router()
    }
    
    /// Get primary interface
    pub fn primary_interface(&self) -> Option<&NetworkInterface> {
        self.interfaces.first()
    }
    
    /// Get interface by name
    pub fn get_interface(&self, name: &str) -> Option<&NetworkInterface> {
        self.interfaces.iter().find(|i| i.name == name)
    }
}

/// Types of network nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Server machine providing network services
    Server,
    /// End-user workstation or desktop
    Workstation,
    /// Network router for packet forwarding
    Router,
    /// Gateway connecting different networks
    Gateway,
    /// Wireless access point
    AccessPoint,
    /// Internet of Things device
    IoTDevice,
    /// Container instance
    Container,
    /// Virtual machine instance
    VirtualMachine,
}

/// Represents a network interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., eth0, wlan0)
    pub name: String,
    /// MAC address of the interface
    pub mac_address: Option<String>,
    /// Type of network interface
    pub interface_type: InterfaceType,
    /// IP addresses assigned to this interface
    pub addresses: Vec<IpAddress>,
    /// Maximum transmission unit size
    pub mtu: Option<u32>,
    /// VLAN ID if this is a VLAN interface
    pub vlan_id: Option<u16>,
    /// Member interfaces if this is a bridge
    pub bridge_members: Vec<String>,
}

impl NetworkInterface {
    /// Check if interface uses DHCP
    pub fn uses_dhcp(&self) -> bool {
        self.addresses.iter().any(|a| a.dhcp)
    }
    
    /// Get static addresses
    pub fn static_addresses(&self) -> Vec<&IpAddress> {
        self.addresses.iter().filter(|a| !a.dhcp).collect()
    }
    
    /// Check if this is a bridge interface
    pub fn is_bridge(&self) -> bool {
        matches!(self.interface_type, InterfaceType::Bridge)
    }
    
    /// Check if this is a VLAN interface
    pub fn is_vlan(&self) -> bool {
        self.vlan_id.is_some()
    }
}

/// Types of network interfaces
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterfaceType {
    /// Physical Ethernet interface
    Ethernet,
    /// Wireless network interface
    Wifi,
    /// Bridge interface combining multiple interfaces
    Bridge,
    /// Virtual LAN interface
    Vlan,
    /// WireGuard VPN interface
    Wireguard,
    /// Loopback interface (lo)
    Loopback,
    /// Virtual network interface
    Virtual,
}

/// IP address configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddress {
    /// IP address (IPv4 or IPv6)
    pub address: String,
    /// Network prefix length (e.g., 24 for /24)
    pub prefix_length: u8,
    /// Whether this address is obtained via DHCP
    pub dhcp: bool,
}

impl IpAddress {
    /// Create a new static IP address
    pub fn new_static(address: String, prefix_length: u8) -> Self {
        Self {
            address,
            prefix_length,
            dhcp: false,
        }
    }
    
    /// Create a DHCP address marker
    pub fn new_dhcp() -> Self {
        Self {
            address: String::new(),
            prefix_length: 0,
            dhcp: true,
        }
    }
    
    /// Get CIDR notation
    pub fn to_cidr(&self) -> Option<String> {
        if self.dhcp {
            None
        } else {
            Some(format!("{}/{}", self.address, self.prefix_length))
        }
    }
}

/// Network service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkService {
    /// Service name (e.g., ssh, http)
    pub name: String,
    /// Port number if applicable
    pub port: Option<u16>,
    /// Network protocol used by the service
    pub protocol: ServiceProtocol,
    /// Whether the service is publicly accessible
    pub public: bool,
}

/// Service protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceProtocol {
    /// TCP protocol only
    Tcp,
    /// UDP protocol only
    Udp,
    /// Both TCP and UDP protocols
    Both,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_topology() {
        let node = NetworkNode {
            id: uuid::Uuid::new_v4(),
            name: "gateway".to_string(),
            node_type: NodeType::Gateway,
            interfaces: vec![],
            services: vec!["dhcp".to_string()],
            metadata: HashMap::new(),
        };
        
        assert!(node.is_router());
        assert!(node.should_run_dhcp());
        assert!(node.should_run_dns());
    }
    
    #[test]
    fn test_ip_address() {
        let static_ip = IpAddress::new_static("192.168.1.1".to_string(), 24);
        assert_eq!(static_ip.to_cidr(), Some("192.168.1.1/24".to_string()));
        
        let dhcp_ip = IpAddress::new_dhcp();
        assert_eq!(dhcp_ip.to_cidr(), None);
    }
}