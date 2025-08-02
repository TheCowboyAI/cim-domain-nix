// Copyright 2025 Cowboy AI, LLC.

//! Simple example demonstrating network topology processing

use cim_domain_nix::network::{
    NetworkIntegrationService,
    NetworkTopologyEvent,
    NetworkNode,
    NetworkInterface,
};
use cim_domain_nix::value_objects::{
    InterfaceType,
    IpAddress,
    NodeType,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌐 Simple Network Topology Demo");
    println!("================================\n");
    
    // Create network integration service
    let mut service = NetworkIntegrationService::new();
    
    // Create a simple network with router and server
    let router_id = Uuid::new_v4();
    let server_id = Uuid::new_v4();
    
    let nodes = vec![
        // Router
        NetworkNode {
            id: router_id,
            name: "main-router".to_string(),
            node_type: NodeType::Router,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: Some("00:11:22:33:44:55".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "192.168.1.1".to_string(),
                            prefix_length: 24,
                            dhcp: false,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "lan0".to_string(),
                    mac_address: Some("00:11:22:33:44:56".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "10.0.0.1".to_string(),
                            prefix_length: 24,
                            dhcp: false,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                }
            ],
            services: vec![
                "dhcp".to_string(),
                "dns".to_string(),
                "firewall".to_string(),
            ],
            metadata: HashMap::new(),
        },
        // Server
        NetworkNode {
            id: server_id,
            name: "app-server".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:AA:BB:CC:DD:EE".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "10.0.0.10".to_string(),
                            prefix_length: 24,
                            dhcp: true,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                }
            ],
            services: vec![
                "nginx".to_string(),
                "postgresql".to_string(),
            ],
            metadata: HashMap::new(),
        },
    ];
    
    // Create simple connection list
    let connections = vec![(router_id, server_id, "ethernet")];
    
    // Create topology event
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "simple-network".to_string(),
        nodes,
        connections: connections.into_iter()
            .map(|(from, to, conn_type)| serde_json::json!({
                "from_node": from,
                "to_node": to,
                "connection_type": conn_type
            }))
            .collect::<Vec<_>>(),
        timestamp: chrono::Utc::now(),
    };
    
    println!("1. Processing network topology...");
    let configs = service.process_topology_event(event).await?;
    
    println!("\n2. Generated {} system configurations:", configs.len());
    
    for config in &configs {
        println!("\n   System: {}", config.hostname);
        println!("   - Node ID: {}", config.node_id);
        println!("   - Services: {}", config.services.len());
        
        // Show network configuration snippet
        if let Some(net_config) = &config.network_config.interfaces.first() {
            println!("   - Primary interface: {}", net_config.name);
        }
        
        // Show services
        for service in &config.services {
            println!("     • {} (enabled: {})", service.name, service.enabled);
        }
    }
    
    println!("\n✅ Network topology processed successfully!");
    
    Ok(())
}