// Copyright 2025 Cowboy AI, LLC.

//! Network integration for building NixOS systems from network events
//!
//! This module handles events from nix-network domain to automatically
//! generate and configure NixOS systems based on network topology.

pub mod events;
pub mod handlers;
pub mod builders;
pub mod value_objects;

pub use events::{NetworkEventHandler, NetworkEventType};
pub use handlers::NetworkSystemHandler;
pub use builders::{NetworkSystemBuilder, SystemBuildConfig};
pub use value_objects::{NetworkNode, NetworkInterface, NetworkTopology};

use crate::Result;

/// Integration service for network-based system building
pub struct NetworkIntegrationService {
    /// Handler for processing network domain events
    event_handler: NetworkEventHandler,
    /// Handler for managing system configurations
    system_handler: NetworkSystemHandler,
    /// Builder for generating NixOS configurations
    builder: NetworkSystemBuilder,
}

impl NetworkIntegrationService {
    /// Create a new network integration service
    pub fn new() -> Self {
        Self {
            event_handler: NetworkEventHandler::new(),
            system_handler: NetworkSystemHandler::new(),
            builder: NetworkSystemBuilder::new(),
        }
    }

    /// Process a network topology event and build corresponding systems
    pub async fn process_topology_event(
        &mut self,
        event: NetworkTopologyEvent,
    ) -> Result<Vec<SystemConfiguration>> {
        // Extract network topology from event
        let topology = self.event_handler.extract_topology(&event)?;
        
        // Generate system configurations for each node
        let mut configurations = Vec::new();
        
        for node in topology.nodes() {
            let config = self.builder.build_system_for_node(node, &topology).await?;
            configurations.push(config);
        }
        
        Ok(configurations)
    }

    /// Handle network interface change events
    pub async fn handle_interface_change(
        &mut self,
        event: InterfaceChangeEvent,
    ) -> Result<()> {
        // Update affected system configurations
        let affected_systems = self.system_handler.find_affected_systems(&event)?;
        
        for system_id in affected_systems {
            self.system_handler.update_network_config(system_id, &event).await?;
        }
        
        Ok(())
    }
}

/// Events that can be received from nix-network domain
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// A new network topology has been created
    TopologyCreated(NetworkTopologyEvent),
    /// An existing network topology has been updated
    TopologyUpdated(NetworkTopologyEvent),
    /// A network interface has been added to a node
    InterfaceAdded(InterfaceChangeEvent),
    /// A network interface has been removed from a node
    InterfaceRemoved(InterfaceChangeEvent),
    /// A network interface configuration has been updated
    InterfaceUpdated(InterfaceChangeEvent),
    /// A network route has been added
    RouteAdded(RouteChangeEvent),
    /// A network route has been removed
    RouteRemoved(RouteChangeEvent),
    /// A firewall rule has been added
    FirewallRuleAdded(FirewallRuleEvent),
    /// A firewall rule has been removed
    FirewallRuleRemoved(FirewallRuleEvent),
}

/// Network topology event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkTopologyEvent {
    /// Unique identifier for this topology
    pub topology_id: uuid::Uuid,
    /// Human-readable name for the topology
    pub name: String,
    /// List of nodes in the network topology
    pub nodes: Vec<NetworkNode>,
    /// Connections between nodes in the topology
    pub connections: Vec<NetworkConnection>,
    /// When this event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Interface change event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterfaceChangeEvent {
    /// ID of the node where the interface change occurred
    pub node_id: uuid::Uuid,
    /// The network interface that changed
    pub interface: NetworkInterface,
    /// Type of change that occurred
    pub change_type: InterfaceChangeType,
    /// When this event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of interface changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InterfaceChangeType {
    /// Interface was newly added
    Added,
    /// Interface was removed
    Removed,
    /// Interface configuration was updated
    Updated,
}

/// Route change event
#[derive(Debug, Clone)]
pub struct RouteChangeEvent {
    /// ID of the node where the route change occurred
    pub node_id: uuid::Uuid,
    /// The network route that changed
    pub route: NetworkRoute,
    /// Type of change that occurred
    pub change_type: RouteChangeType,
    /// When this event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of route changes
#[derive(Debug, Clone)]
pub enum RouteChangeType {
    /// Route was added to routing table
    Added,
    /// Route was removed from routing table
    Removed,
}

/// Network route definition
#[derive(Debug, Clone)]
pub struct NetworkRoute {
    /// Destination network (CIDR notation)
    pub destination: String,
    /// Gateway IP address for the route
    pub gateway: Option<String>,
    /// Network interface to use for this route
    pub interface: String,
    /// Route metric for priority
    pub metric: Option<u32>,
}

/// Firewall rule event
#[derive(Debug, Clone)]
pub struct FirewallRuleEvent {
    /// ID of the node where the firewall rule change occurred
    pub node_id: uuid::Uuid,
    /// The firewall rule that changed
    pub rule: FirewallRule,
    /// Type of change that occurred
    pub change_type: FirewallChangeType,
    /// When this event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of firewall changes
#[derive(Debug, Clone)]
pub enum FirewallChangeType {
    /// Firewall rule was added
    Added,
    /// Firewall rule was removed
    Removed,
}

/// Firewall rule definition
#[derive(Debug, Clone)]
pub struct FirewallRule {
    /// Name identifier for the rule
    pub name: String,
    /// Traffic direction this rule applies to
    pub direction: FirewallDirection,
    /// Network protocol (tcp, udp, icmp, etc.)
    pub protocol: Option<String>,
    /// Source IP address or network
    pub source: Option<String>,
    /// Destination IP address or network
    pub destination: Option<String>,
    /// Port numbers this rule applies to
    pub ports: Option<Vec<u16>>,
    /// Action to take when rule matches
    pub action: FirewallAction,
}

/// Firewall rule direction
#[derive(Debug, Clone)]
pub enum FirewallDirection {
    /// Rule applies to incoming traffic
    Inbound,
    /// Rule applies to outgoing traffic
    Outbound,
}

/// Firewall rule action
#[derive(Debug, Clone)]
pub enum FirewallAction {
    /// Allow matching traffic
    Allow,
    /// Deny matching traffic
    Deny,
}

/// Network connection between nodes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConnection {
    /// Source node ID
    pub from_node: uuid::Uuid,
    /// Interface name on source node
    pub from_interface: String,
    /// Destination node ID
    pub to_node: uuid::Uuid,
    /// Interface name on destination node
    pub to_interface: String,
    /// Type of network connection
    pub network_type: NetworkType,
}

/// Types of networks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkType {
    /// Standard Ethernet connection
    Ethernet,
    /// Wireless WiFi connection
    Wifi,
    /// Bridge network interface
    Bridge,
    /// VLAN with specified tag ID
    Vlan(u16),
    /// WireGuard VPN connection
    Wireguard,
    /// Other network type with description
    Other(String),
}

/// Generated system configuration
#[derive(Debug, Clone)]
pub struct SystemConfiguration {
    /// Unique identifier for this system configuration
    pub system_id: uuid::Uuid,
    /// ID of the network node this configuration is for
    pub node_id: uuid::Uuid,
    /// Hostname for the system
    pub hostname: String,
    /// Network configuration for the system
    pub network_config: NetworkConfig,
    /// Services to enable on the system
    pub services: Vec<ServiceConfig>,
    /// Optional firewall configuration
    pub firewall_config: Option<FirewallConfig>,
}

/// Network configuration for a system
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Network interface configurations
    pub interfaces: Vec<InterfaceConfig>,
    /// Static route configurations
    pub routes: Vec<RouteConfig>,
    /// DNS resolver configuration
    pub dns: Option<DnsConfig>,
}

/// Interface configuration
#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    /// Interface name (e.g., eth0, wlan0)
    pub name: String,
    /// MAC address for the interface
    pub mac_address: Option<String>,
    /// Static IP addresses in CIDR notation
    pub addresses: Vec<String>,
    /// Whether to use DHCP for configuration
    pub dhcp: bool,
    /// Maximum transmission unit size
    pub mtu: Option<u32>,
}

/// Route configuration
#[derive(Debug, Clone)]
pub struct RouteConfig {
    /// Destination network in CIDR notation
    pub destination: String,
    /// Gateway IP address
    pub gateway: Option<String>,
    /// Interface to use for this route
    pub interface: String,
}

/// DNS configuration
#[derive(Debug, Clone)]
pub struct DnsConfig {
    /// DNS server IP addresses
    pub nameservers: Vec<String>,
    /// DNS search domains
    pub search_domains: Vec<String>,
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Service name (e.g., nginx, postgresql)
    pub name: String,
    /// Whether the service should be enabled
    pub enabled: bool,
    /// Service-specific configuration settings
    pub settings: serde_json::Value,
}

/// Firewall configuration
#[derive(Debug, Clone)]
pub struct FirewallConfig {
    /// Whether to enable the firewall
    pub enable: bool,
    /// TCP ports to allow through firewall
    pub allowed_tcp_ports: Vec<u16>,
    /// UDP ports to allow through firewall
    pub allowed_udp_ports: Vec<u16>,
    /// Custom firewall rules
    pub rules: Vec<FirewallRule>,
}

impl Default for NetworkIntegrationService {
    fn default() -> Self {
        Self::new()
    }
}