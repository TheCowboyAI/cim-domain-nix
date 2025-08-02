// Copyright 2025 Cowboy AI, LLC.

//! Value objects for the network domain

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a network topology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkTopologyId(pub Uuid);

impl NetworkTopologyId {
    /// Create a new topology ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for NetworkTopologyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a network node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkNodeId(pub Uuid);

impl NetworkNodeId {
    /// Create a new node ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for NetworkNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node tier in the CIM hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeTier {
    /// Client nodes - endpoints that consume services
    Client,
    /// Leaf nodes - first level of service providers
    Leaf,
    /// Cluster nodes - coordinate multiple leaf nodes
    Cluster,
    /// Super-cluster nodes - coordinate multiple clusters
    SuperCluster,
}

impl NodeTier {
    /// Get the numeric level of this tier (higher = more authority)
    pub fn level(&self) -> u8 {
        match self {
            NodeTier::Client => 0,
            NodeTier::Leaf => 1,
            NodeTier::Cluster => 2,
            NodeTier::SuperCluster => 3,
        }
    }
    
    /// Check if this tier can provide services to another tier
    pub fn can_serve(&self, other: &NodeTier) -> bool {
        self.level() > other.level()
    }
}

/// Type of network node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Network gateway (internet connection)
    Gateway,
    /// Router/firewall
    Router,
    /// Server providing services
    Server,
    /// User workstation
    Workstation,
    /// Wireless access point
    AccessPoint,
    /// IoT device
    IoTDevice,
    /// Virtual machine
    VirtualMachine,
}

/// Network interface configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., eth0, wlan0)
    pub name: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// Interface type
    pub interface_type: InterfaceType,
    /// IP addresses assigned
    pub addresses: Vec<IpAddress>,
    /// MTU size
    pub mtu: Option<u32>,
    /// VLAN ID if tagged
    pub vlan_id: Option<u16>,
    /// Bridge members if this is a bridge
    pub bridge_members: Vec<String>,
}

/// Type of network interface
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceType {
    /// Physical ethernet
    Ethernet,
    /// WiFi interface
    Wifi,
    /// Bridge interface
    Bridge,
    /// VLAN interface
    Vlan,
    /// WireGuard tunnel
    Wireguard,
    /// Loopback
    Loopback,
    /// Virtual interface
    Virtual,
}

/// IP address configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpAddress {
    /// IP address (v4 or v6)
    pub address: String,
    /// Network prefix length
    pub prefix_length: u8,
    /// Whether this was obtained via DHCP
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
    
    /// Create a DHCP address placeholder
    pub fn new_dhcp() -> Self {
        Self {
            address: "dhcp".to_string(),
            prefix_length: 0,
            dhcp: true,
        }
    }
    
    /// Get CIDR notation
    pub fn cidr(&self) -> String {
        if self.dhcp {
            "dhcp".to_string()
        } else {
            format!("{}/{}", self.address, self.prefix_length)
        }
    }
}

/// Network service configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkService {
    /// Service name
    pub name: String,
    /// Port number
    pub port: Option<u16>,
    /// Protocol
    pub protocol: ServiceProtocol,
    /// Whether service is publicly accessible
    pub public: bool,
}

/// Network protocol
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceProtocol {
    Tcp,
    Udp,
    Both,
}

/// Network connection between nodes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConnection {
    /// Source node ID
    pub from_node: NetworkNodeId,
    /// Destination node ID
    pub to_node: NetworkNodeId,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Connection properties
    pub properties: ConnectionProperties,
}

/// Type of network connection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Physical ethernet cable
    Ethernet,
    /// WiFi connection
    Wifi,
    /// VPN tunnel
    Vpn,
    /// Virtual connection
    Virtual,
}

/// Connection properties
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionProperties {
    /// Bandwidth in Mbps
    pub bandwidth: Option<u32>,
    /// Latency in ms
    pub latency: Option<u32>,
    /// Whether connection is redundant
    pub redundant: bool,
    /// VLAN tags for this connection
    pub vlan_tags: Vec<u16>,
}