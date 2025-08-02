// Copyright 2025 Cowboy AI, LLC.

//! High-level services for network domain operations

use super::commands::*;
use super::handlers::{NetworkCommandHandler, NetworkQueryHandler, NetworkTopologyView};
use super::value_objects::*;
use crate::{
    services::ConfigurationService,
    value_objects::{MessageIdentity, NixOSConfiguration},
    Result,
};
use std::collections::HashMap;

/// Service for managing network topologies and generating NixOS configurations
pub struct NetworkTopologyService {
    command_handler: NetworkCommandHandler,
    query_handler: NetworkQueryHandler,
    config_service: ConfigurationService,
}

impl NetworkTopologyService {
    /// Create a new network topology service
    pub fn new() -> Self {
        Self {
            command_handler: NetworkCommandHandler::new(),
            query_handler: NetworkQueryHandler::new(),
            config_service: ConfigurationService::new(),
        }
    }
    
    /// Create a Starlink-based network topology
    pub async fn create_starlink_topology(
        &mut self,
        name: String,
        wan_subnet: String,
        lan_subnet: String,
    ) -> Result<NetworkTopologyView> {
        // Create the topology
        let create_cmd = CreateNetworkTopology {
            identity: MessageIdentity::new_root(),
            name: name.clone(),
            description: "Starlink-based network topology".to_string(),
            metadata: {
                let mut m = HashMap::new();
                m.insert("wan_type".to_string(), "starlink".to_string());
                m.insert("wan_subnet".to_string(), wan_subnet.clone());
                m.insert("lan_subnet".to_string(), lan_subnet.clone());
                m
            },
        };
        
        let events = self.command_handler.handle_create_topology(create_cmd).await?;
        let topology_id = if let Some(event) = events.first() {
            if let Ok(e) = event.as_any().downcast_ref::<super::events::NetworkTopologyCreated>() {
                e.topology_id
            } else {
                return Err(crate::NixDomainError::Other("Failed to get topology ID".to_string()));
            }
        } else {
            return Err(crate::NixDomainError::Other("No events generated".to_string()));
        };
        
        // Add Starlink router
        let add_starlink = AddNodeToTopology {
            identity: MessageIdentity::new_root(),
            topology_id,
            name: "starlink-router".to_string(),
            node_type: NodeType::Gateway,
            tier: NodeTier::SuperCluster,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()], // CGNAT from Starlink
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "lan0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static(
                        format!("{}.1", wan_subnet),
                        24,
                    )],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec!["starlink".to_string()],
            metadata: {
                let mut m = HashMap::new();
                m.insert("model".to_string(), "Starlink Router Gen2".to_string());
                m.insert("wifi".to_string(), "disabled".to_string());
                m
            },
        };
        
        self.command_handler.handle_add_node(add_starlink).await?;
        
        // Return the topology view
        self.query_handler.get_topology(topology_id).await?
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))
    }
    
    /// Add UDM Pro to topology
    pub async fn add_udm_pro(
        &mut self,
        topology_id: NetworkTopologyId,
        wan_ip: String,
        lan_subnet: String,
    ) -> Result<NetworkNodeId> {
        let add_udm = AddNodeToTopology {
            identity: MessageIdentity::new_root(),
            topology_id,
            name: "udm-pro".to_string(),
            node_type: NodeType::Router,
            tier: NodeTier::Cluster,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static(wan_ip, 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "lan0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static(
                        format!("{}.1", lan_subnet),
                        24,
                    )],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![
                "dhcp".to_string(),
                "dns".to_string(),
                "firewall".to_string(),
                "nat".to_string(),
                "unifi".to_string(),
            ],
            metadata: {
                let mut m = HashMap::new();
                m.insert("model".to_string(), "Dream Machine Pro".to_string());
                m
            },
        };
        
        let events = self.command_handler.handle_add_node(add_udm).await?;
        
        // Extract node ID from event
        if let Some(event) = events.first() {
            if let Ok(e) = event.as_any().downcast_ref::<super::events::NodeAddedToTopology>() {
                Ok(e.node_id)
            } else {
                Err(crate::NixDomainError::Other("Failed to get node ID".to_string()))
            }
        } else {
            Err(crate::NixDomainError::Other("No events generated".to_string()))
        }
    }
    
    /// Add Mac Studio as leaf node
    pub async fn add_mac_studio_leaf(
        &mut self,
        topology_id: NetworkTopologyId,
        ip_address: String,
    ) -> Result<NetworkNodeId> {
        let add_mac = AddNodeToTopology {
            identity: MessageIdentity::new_root(),
            topology_id,
            name: "mac-studio-leaf".to_string(),
            node_type: NodeType::Server,
            tier: NodeTier::Leaf,
            interfaces: vec![
                NetworkInterface {
                    name: "en0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static(ip_address, 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![
                "nats".to_string(),
                "cim-leaf".to_string(),
                "docker".to_string(),
            ],
            metadata: {
                let mut m = HashMap::new();
                m.insert("model".to_string(), "Mac Studio M3 Ultra".to_string());
                m.insert("cpu".to_string(), "24-core".to_string());
                m.insert("gpu".to_string(), "76-core".to_string());
                m.insert("ram".to_string(), "192GB".to_string());
                m.insert("role".to_string(), "cim-leaf-node".to_string());
                m
            },
        };
        
        let events = self.command_handler.handle_add_node(add_mac).await?;
        
        // Extract node ID from event
        if let Some(event) = events.first() {
            if let Ok(e) = event.as_any().downcast_ref::<super::events::NodeAddedToTopology>() {
                Ok(e.node_id)
            } else {
                Err(crate::NixDomainError::Other("Failed to get node ID".to_string()))
            }
        } else {
            Err(crate::NixDomainError::Other("No events generated".to_string()))
        }
    }
    
    /// Generate NixOS configurations for all nodes in topology
    pub async fn generate_nixos_configs(
        &self,
        topology_id: NetworkTopologyId,
    ) -> Result<Vec<NixOSConfiguration>> {
        let topology = self.query_handler.get_topology(topology_id).await?
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))?;
        
        let mut configs = Vec::new();
        
        for node in &topology.nodes {
            let config = self.generate_node_config(&node).await?;
            configs.push(config);
        }
        
        Ok(configs)
    }
    
    /// Generate NixOS configuration for a single node
    async fn generate_node_config(&self, node: &super::handlers::NetworkNodeView) -> Result<NixOSConfiguration> {
        let mut config = NixOSConfiguration {
            hostname: node.name.clone(),
            system: "x86_64-linux".to_string(),
            modules: vec![],
            packages: vec![
                "git".to_string(),
                "vim".to_string(),
                "htop".to_string(),
                "tmux".to_string(),
            ],
            services: HashMap::new(),
            networking: HashMap::new(),
            users: HashMap::new(),
            extra_config: String::new(),
        };
        
        // Configure networking
        config.networking.insert("hostName".to_string(), node.name.clone());
        config.networking.insert("domain".to_string(), "local".to_string());
        
        // Configure interfaces
        for (i, iface) in node.interfaces.iter().enumerate() {
            let iface_config = format!(
                "networking.interfaces.{} = {{\n  useDHCP = {};\n",
                iface.name,
                iface.addresses.iter().any(|a| a.dhcp)
            );
            
            config.extra_config.push_str(&iface_config);
            
            // Add static IPs
            for addr in &iface.addresses {
                if !addr.dhcp {
                    config.extra_config.push_str(&format!(
                        "  ipv4.addresses = [ {{ address = \"{}\"; prefixLength = {}; }} ];\n",
                        addr.address, addr.prefix_length
                    ));
                }
            }
            
            config.extra_config.push_str("};\n\n");
        }
        
        // Configure services based on node type and tier
        match node.tier {
            NodeTier::SuperCluster => {
                // Gateway services
                config.services.insert("firewall".to_string(), serde_json::json!({
                    "enable": true,
                    "allowPing": true,
                    "allowedTCPPorts": [22]
                }));
            }
            NodeTier::Cluster => {
                // Cluster services
                config.services.insert("dhcpd4".to_string(), serde_json::json!({
                    "enable": true,
                    "interfaces": ["lan0"],
                    "extraConfig": "option domain-name-servers 1.1.1.1, 8.8.8.8;"
                }));
                
                config.services.insert("unbound".to_string(), serde_json::json!({
                    "enable": true,
                    "forwardAddresses": ["1.1.1.1", "8.8.8.8"]
                }));
                
                config.networking.insert("nat".to_string(), serde_json::json!({
                    "enable": true,
                    "externalInterface": "wan0",
                    "internalInterfaces": ["lan0"]
                }));
            }
            NodeTier::Leaf => {
                // Leaf node services
                if node.services.contains(&"nats".to_string()) {
                    config.services.insert("nats".to_string(), serde_json::json!({
                        "enable": true,
                        "serverName": node.name.clone(),
                        "jetstream": true,
                        "leafNode": {
                            "enable": true,
                            "remotes": ["nats://cluster.local:7422"]
                        }
                    }));
                }
                
                if node.services.contains(&"cim-leaf".to_string()) {
                    config.packages.extend(vec![
                        "cim-cli".to_string(),
                        "cim-leaf".to_string(),
                    ]);
                    
                    config.extra_config.push_str(&format!(r#"
# CIM Leaf Node Configuration
services.cim-leaf = {{
  enable = true;
  nodeId = "{}";
  nodeRole = "leaf";
  natsUrl = "nats://localhost:4222";
}};
"#, node.name));
                }
            }
            NodeTier::Client => {
                // Client configuration
                config.services.insert("openssh".to_string(), serde_json::json!({
                    "enable": true
                }));
            }
        }
        
        Ok(config)
    }
}