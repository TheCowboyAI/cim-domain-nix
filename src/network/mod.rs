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
    event_handler: NetworkEventHandler,
    system_handler: NetworkSystemHandler,
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
    TopologyCreated(NetworkTopologyEvent),
    TopologyUpdated(NetworkTopologyEvent),
    InterfaceAdded(InterfaceChangeEvent),
    InterfaceRemoved(InterfaceChangeEvent),
    InterfaceUpdated(InterfaceChangeEvent),
    RouteAdded(RouteChangeEvent),
    RouteRemoved(RouteChangeEvent),
    FirewallRuleAdded(FirewallRuleEvent),
    FirewallRuleRemoved(FirewallRuleEvent),
}

/// Network topology event data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkTopologyEvent {
    pub topology_id: uuid::Uuid,
    pub name: String,
    pub nodes: Vec<NetworkNode>,
    pub connections: Vec<NetworkConnection>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Interface change event
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterfaceChangeEvent {
    pub node_id: uuid::Uuid,
    pub interface: NetworkInterface,
    pub change_type: InterfaceChangeType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of interface changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InterfaceChangeType {
    Added,
    Removed,
    Updated,
}

/// Route change event
#[derive(Debug, Clone)]
pub struct RouteChangeEvent {
    pub node_id: uuid::Uuid,
    pub route: NetworkRoute,
    pub change_type: RouteChangeType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of route changes
#[derive(Debug, Clone)]
pub enum RouteChangeType {
    Added,
    Removed,
}

/// Network route definition
#[derive(Debug, Clone)]
pub struct NetworkRoute {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: String,
    pub metric: Option<u32>,
}

/// Firewall rule event
#[derive(Debug, Clone)]
pub struct FirewallRuleEvent {
    pub node_id: uuid::Uuid,
    pub rule: FirewallRule,
    pub change_type: FirewallChangeType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Types of firewall changes
#[derive(Debug, Clone)]
pub enum FirewallChangeType {
    Added,
    Removed,
}

/// Firewall rule definition
#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub name: String,
    pub direction: FirewallDirection,
    pub protocol: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub ports: Option<Vec<u16>>,
    pub action: FirewallAction,
}

/// Firewall rule direction
#[derive(Debug, Clone)]
pub enum FirewallDirection {
    Inbound,
    Outbound,
}

/// Firewall rule action
#[derive(Debug, Clone)]
pub enum FirewallAction {
    Allow,
    Deny,
}

/// Network connection between nodes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkConnection {
    pub from_node: uuid::Uuid,
    pub from_interface: String,
    pub to_node: uuid::Uuid,
    pub to_interface: String,
    pub network_type: NetworkType,
}

/// Types of networks
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkType {
    Ethernet,
    Wifi,
    Bridge,
    Vlan(u16),
    Wireguard,
    Other(String),
}

/// Generated system configuration
#[derive(Debug, Clone)]
pub struct SystemConfiguration {
    pub system_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub hostname: String,
    pub network_config: NetworkConfig,
    pub services: Vec<ServiceConfig>,
    pub firewall_config: Option<FirewallConfig>,
}

/// Network configuration for a system
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub interfaces: Vec<InterfaceConfig>,
    pub routes: Vec<RouteConfig>,
    pub dns: Option<DnsConfig>,
}

/// Interface configuration
#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    pub name: String,
    pub mac_address: Option<String>,
    pub addresses: Vec<String>,
    pub dhcp: bool,
    pub mtu: Option<u32>,
}

/// Route configuration
#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: String,
}

/// DNS configuration
#[derive(Debug, Clone)]
pub struct DnsConfig {
    pub nameservers: Vec<String>,
    pub search_domains: Vec<String>,
}

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
}

/// Firewall configuration
#[derive(Debug, Clone)]
pub struct FirewallConfig {
    pub enable: bool,
    pub allowed_tcp_ports: Vec<u16>,
    pub allowed_udp_ports: Vec<u16>,
    pub rules: Vec<FirewallRule>,
}

impl Default for NetworkIntegrationService {
    fn default() -> Self {
        Self::new()
    }
}