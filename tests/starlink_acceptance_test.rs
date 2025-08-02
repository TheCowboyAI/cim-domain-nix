// Copyright 2025 Cowboy AI, LLC.

//! Acceptance test for Starlink network topology
//! 
//! Tests the scenario:
//! - Starlink router (WiFi disabled) -> UDM Pro WAN
//! - UDM Pro LAN -> UniFi 24-port switch  
//! - Switch -> Mac Studio M3 Ultra (Leaf Node)

use cim_domain_nix::{
    network::{
        NetworkIntegrationService,
        NetworkTopologyEvent,
        NetworkNode,
        NetworkInterface,
        NetworkConnection,
        InterfaceChangeEvent,
        InterfaceChangeType,
    },
    value_objects::{
        InterfaceType, 
        IpAddress,
        NodeType,
    },
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_starlink_network_topology() -> anyhow::Result<()> {
    println!("\n=== Starlink Network Topology Acceptance Test ===");
    
    // Create network integration service
    let mut service = NetworkIntegrationService::new();
    
    // Create nodes
    let starlink_id = Uuid::new_v4();
    let udm_id = Uuid::new_v4();
    let switch_id = Uuid::new_v4();
    let mac_id = Uuid::new_v4();
    
    let nodes = vec![
        // Starlink Router
        NetworkNode {
            id: starlink_id,
            name: "starlink-router".to_string(),
            node_type: NodeType::Gateway,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: Some("00:11:22:33:44:55".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "100.64.0.1".to_string(),
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
                            address: "192.168.100.1".to_string(),
                            prefix_length: 24,
                            dhcp: false,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                }
            ],
            services: vec!["starlink".to_string()],
            metadata: {
                let mut m = HashMap::new();
                m.insert("wifi".to_string(), "disabled".to_string());
                m
            },
        },
        // UDM Pro
        NetworkNode {
            id: udm_id,
            name: "udm-pro".to_string(),
            node_type: NodeType::Router,
            interfaces: vec![
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: Some("00:AA:BB:CC:DD:EE".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "192.168.100.2".to_string(),
                            prefix_length: 24,
                            dhcp: true,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "lan0".to_string(),
                    mac_address: Some("00:AA:BB:CC:DD:EF".to_string()),
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
                "nat".to_string(),
            ],
            metadata: HashMap::new(),
        },
        // UniFi Switch
        NetworkNode {
            id: switch_id,
            name: "unifi-switch-24".to_string(),
            node_type: NodeType::Router,
            interfaces: vec![
                NetworkInterface {
                    name: "uplink".to_string(),
                    mac_address: Some("00:22:33:44:55:66".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "10.0.0.2".to_string(),
                            prefix_length: 24,
                            dhcp: true,
                        }
                    ],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                }
            ],
            services: vec!["switching".to_string()],
            metadata: HashMap::new(),
        },
        // Mac Studio (Leaf Node)
        NetworkNode {
            id: mac_id,
            name: "mac-studio-leaf".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "en0".to_string(),
                    mac_address: Some("00:FF:EE:DD:CC:BB".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![
                        IpAddress {
                            address: "10.0.0.100".to_string(),
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
                "nats".to_string(),
                "cim-leaf".to_string(),
            ],
            metadata: {
                let mut m = HashMap::new();
                m.insert("role".to_string(), "leaf-node".to_string());
                m
            },
        },
    ];
    
    // Create connections
    let connections = vec![
        NetworkConnection {
            from_node: starlink_id,
            to_node: udm_id,
            connection_type: "ethernet".to_string(),
        },
        NetworkConnection {
            from_node: udm_id,
            to_node: switch_id,
            connection_type: "ethernet".to_string(),
        },
        NetworkConnection {
            from_node: switch_id,
            to_node: mac_id,
            connection_type: "ethernet".to_string(),
        },
    ];
    
    // Create topology event
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "starlink-homelab".to_string(),
        nodes,
        connections,
        timestamp: chrono::Utc::now(),
    };
    
    println!("1. Processing network topology...");
    let configs = service.process_topology_event(event).await?;
    
    println!("2. Generated {} system configurations", configs.len());
    assert_eq!(configs.len(), 4);
    
    // Verify configurations
    for config in &configs {
        println!("   - {}: {} services configured", 
            config.hostname, 
            config.services.len()
        );
        
        match config.hostname.as_str() {
            "starlink-router" => {
                assert_eq!(config.services.len(), 1);
            }
            "udm-pro" => {
                assert!(config.services.len() >= 4);
                // Should have DHCP, DNS, firewall, NAT
            }
            "unifi-switch-24" => {
                assert_eq!(config.services.len(), 1);
            }
            "mac-studio-leaf" => {
                assert!(config.services.len() >= 2);
                // Should have NATS and CIM leaf services
                let has_nats = config.services.iter()
                    .any(|s| s.name == "nats");
                let has_cim = config.services.iter()
                    .any(|s| s.name == "cim-leaf");
                assert!(has_nats, "Mac Studio should have NATS service");
                assert!(has_cim, "Mac Studio should have CIM leaf service");
            }
            _ => panic!("Unexpected hostname: {}", config.hostname),
        }
    }
    
    // Test interface change
    println!("\n3. Testing WAN failover scenario...");
    let wan_change = InterfaceChangeEvent {
        node_id: udm_id,
        interface: NetworkInterface {
            name: "wan0".to_string(),
            mac_address: Some("00:AA:BB:CC:DD:EE".to_string()),
            interface_type: InterfaceType::Ethernet,
            addresses: vec![], // Lost connection
            mtu: Some(1500),
            vlan_id: None,
            bridge_members: vec![],
        },
        change_type: InterfaceChangeType::Updated,
        timestamp: chrono::Utc::now(),
    };
    
    service.handle_interface_change(wan_change).await?;
    println!("   ✓ Interface change handled successfully");
    
    println!("\n✅ Starlink topology test completed successfully!");
    Ok(())
}

/// Connection between network nodes
#[derive(Debug, Clone)]
struct NetworkConnection {
    from_node: Uuid,
    to_node: Uuid,
    connection_type: String,
}