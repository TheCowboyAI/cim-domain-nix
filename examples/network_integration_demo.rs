//! Example demonstrating network event integration for automatic NixOS system generation
//!
//! This example shows how to:
//! 1. Process network topology events from nix-network domain
//! 2. Generate NixOS configurations for each node
//! 3. Handle dynamic network changes

use cim_domain_nix::network::{
    NetworkIntegrationService, NetworkTopologyEvent, NetworkNode, NetworkConnection,
    NetworkType, value_objects::{NodeType, NetworkInterface, InterfaceType, IpAddress},
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Network Integration Demo");
    println!("======================\n");
    
    // Create the network integration service
    let mut service = NetworkIntegrationService::new();
    
    // Simulate a network topology event (would come from nix-network domain via NATS)
    let topology_event = create_office_network_topology();
    
    println!("Processing network topology: {}", topology_event.name);
    println!("Nodes: {}", topology_event.nodes.len());
    println!("Connections: {}\n", topology_event.connections.len());
    
    // Process the topology and generate system configurations
    let systems = service.process_topology_event(topology_event).await?;
    
    println!("Generated {} system configurations:\n", systems.len());
    
    // Display each system configuration
    for system in &systems {
        println!("System: {}", system.hostname);
        println!("  Node ID: {}", system.node_id);
        println!("  Network interfaces:");
        for interface in &system.network_config.interfaces {
            println!("    - {}: {}", 
                interface.name, 
                if interface.dhcp { 
                    "DHCP".to_string() 
                } else { 
                    interface.addresses.join(", ") 
                }
            );
        }
        
        println!("  Services:");
        for service in &system.services {
            if service.enabled {
                println!("    - {}", service.name);
            }
        }
        
        if let Some(firewall) = &system.firewall_config {
            println!("  Firewall:");
            if !firewall.allowed_tcp_ports.is_empty() {
                println!("    TCP ports: {:?}", firewall.allowed_tcp_ports);
            }
            if !firewall.allowed_udp_ports.is_empty() {
                println!("    UDP ports: {:?}", firewall.allowed_udp_ports);
            }
        }
        
        println!();
    }
    
    // Generate NixOS module for the gateway
    if let Some(gateway) = systems.iter().find(|s| s.hostname == "office-gateway") {
        println!("Example NixOS module for gateway:");
        println!("--------------------------------");
        let handler = cim_domain_nix::network::handlers::NetworkSystemHandler::new();
        let module = handler.generate_network_module(&gateway.network_config);
        println!("{}", module);
    }
    
    Ok(())
}

/// Create a sample office network topology
fn create_office_network_topology() -> NetworkTopologyEvent {
    let gateway_id = Uuid::new_v4();
    let web_server_id = Uuid::new_v4();
    let db_server_id = Uuid::new_v4();
    let workstation1_id = Uuid::new_v4();
    let workstation2_id = Uuid::new_v4();
    
    let nodes = vec![
        // Gateway/Router
        NetworkNode {
            id: gateway_id,
            name: "office-gateway".to_string(),
            node_type: NodeType::Gateway,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: Some("00:11:22:33:44:55".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()], // Public interface uses DHCP
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "lan0".to_string(),
                    mac_address: Some("00:11:22:33:44:56".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("192.168.1.1".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec!["dhcp".to_string(), "dns".to_string(), "firewall".to_string()],
            metadata: HashMap::new(),
        },
        
        // Web Server
        NetworkNode {
            id: web_server_id,
            name: "web-server".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:57".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("192.168.1.10".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("role".to_string(), "web".to_string());
                metadata
            },
        },
        
        // Database Server
        NetworkNode {
            id: db_server_id,
            name: "db-server".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:58".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("192.168.1.11".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![],
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("role".to_string(), "database".to_string());
                metadata
            },
        },
        
        // Workstations
        NetworkNode {
            id: workstation1_id,
            name: "workstation-01".to_string(),
            node_type: NodeType::Workstation,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:59".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![],
            metadata: HashMap::new(),
        },
        NetworkNode {
            id: workstation2_id,
            name: "workstation-02".to_string(),
            node_type: NodeType::Workstation,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:5A".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec![],
            metadata: HashMap::new(),
        },
    ];
    
    // Define network connections
    let connections = vec![
        // Gateway to servers
        NetworkConnection {
            from_node: gateway_id,
            from_interface: "lan0".to_string(),
            to_node: web_server_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
        NetworkConnection {
            from_node: gateway_id,
            from_interface: "lan0".to_string(),
            to_node: db_server_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
        // Gateway to workstations
        NetworkConnection {
            from_node: gateway_id,
            from_interface: "lan0".to_string(),
            to_node: workstation1_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
        NetworkConnection {
            from_node: gateway_id,
            from_interface: "lan0".to_string(),
            to_node: workstation2_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
    ];
    
    NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "office-network".to_string(),
        nodes,
        connections,
        timestamp: chrono::Utc::now(),
    }
}