//! Integration tests for network event handling and system building

use cim_domain_nix::network::{
    NetworkIntegrationService, NetworkTopologyEvent, NetworkNode, NetworkConnection,
    NetworkType, InterfaceChangeEvent, InterfaceChangeType,
    value_objects::{NodeType, NetworkInterface, InterfaceType, IpAddress},
};
use cim_domain_nix::value_objects::MessageIdentity;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_network_topology_to_systems() {
    let mut service = NetworkIntegrationService::new();
    
    // Create a network topology with router, server, and workstation
    let router_id = Uuid::new_v4();
    let server_id = Uuid::new_v4();
    let workstation_id = Uuid::new_v4();
    
    let nodes = vec![
        NetworkNode {
            id: router_id,
            name: "gateway".to_string(),
            node_type: NodeType::Gateway,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:55".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("192.168.1.1".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "wan0".to_string(),
                    mac_address: Some("00:11:22:33:44:56".to_string()),
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec!["dhcp".to_string(), "dns".to_string()],
            metadata: HashMap::new(),
        },
        NetworkNode {
            id: server_id,
            name: "webserver".to_string(),
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
        NetworkNode {
            id: workstation_id,
            name: "workstation01".to_string(),
            node_type: NodeType::Workstation,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: Some("00:11:22:33:44:58".to_string()),
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
    
    let connections = vec![
        NetworkConnection {
            from_node: router_id,
            from_interface: "eth0".to_string(),
            to_node: server_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
        NetworkConnection {
            from_node: router_id,
            from_interface: "eth0".to_string(),
            to_node: workstation_id,
            to_interface: "eth0".to_string(),
            network_type: NetworkType::Ethernet,
        },
    ];
    
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "office-network".to_string(),
        nodes,
        connections,
        timestamp: chrono::Utc::now(),
    };
    
    // Process the topology event
    let systems = service.process_topology_event(event).await.unwrap();
    
    // Verify we got 3 system configurations
    assert_eq!(systems.len(), 3);
    
    // Verify gateway configuration
    let gateway_config = systems.iter().find(|s| s.hostname == "gateway").unwrap();
    assert_eq!(gateway_config.network_config.interfaces.len(), 2);
    assert!(gateway_config.services.iter().any(|s| s.name == "dhcpd4"));
    assert!(gateway_config.services.iter().any(|s| s.name == "unbound"));
    assert!(gateway_config.services.iter().any(|s| s.name == "nat"));
    assert!(gateway_config.firewall_config.is_some());
    
    // Verify server configuration
    let server_config = systems.iter().find(|s| s.hostname == "webserver").unwrap();
    assert_eq!(server_config.network_config.interfaces.len(), 1);
    assert!(server_config.services.iter().any(|s| s.name == "nginx"));
    assert!(server_config.network_config.routes.len() > 0);
    
    // Verify workstation configuration
    let workstation_config = systems.iter().find(|s| s.hostname == "workstation01").unwrap();
    assert!(workstation_config.network_config.interfaces[0].dhcp);
}

#[tokio::test]
async fn test_interface_change_handling() {
    let mut service = NetworkIntegrationService::new();
    
    // First create a simple topology
    let node_id = Uuid::new_v4();
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "test-network".to_string(),
        nodes: vec![
            NetworkNode {
                id: node_id,
                name: "testserver".to_string(),
                node_type: NodeType::Server,
                interfaces: vec![
                    NetworkInterface {
                        name: "eth0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Ethernet,
                        addresses: vec![IpAddress::new_static("192.168.1.20".to_string(), 24)],
                        mtu: Some(1500),
                        vlan_id: None,
                        bridge_members: vec![],
                    },
                ],
                services: vec![],
                metadata: HashMap::new(),
            },
        ],
        connections: vec![],
        timestamp: chrono::Utc::now(),
    };
    
    let systems = service.process_topology_event(event).await.unwrap();
    assert_eq!(systems.len(), 1);
    
    // Now add a new interface
    let new_interface = NetworkInterface {
        name: "eth1".to_string(),
        mac_address: None,
        interface_type: InterfaceType::Ethernet,
        addresses: vec![IpAddress::new_static("10.0.0.1".to_string(), 24)],
        mtu: Some(1500),
        vlan_id: None,
        bridge_members: vec![],
    };
    
    let interface_event = InterfaceChangeEvent {
        node_id,
        interface: new_interface,
        change_type: InterfaceChangeType::Added,
        timestamp: chrono::Utc::now(),
    };
    
    service.handle_interface_change(interface_event).await.unwrap();
}

#[tokio::test]
async fn test_wireguard_network_topology() {
    let mut service = NetworkIntegrationService::new();
    
    let vpn_server_id = Uuid::new_v4();
    let vpn_client_id = Uuid::new_v4();
    
    let nodes = vec![
        NetworkNode {
            id: vpn_server_id,
            name: "vpn-server".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("203.0.113.1".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "wg0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Wireguard,
                    addresses: vec![IpAddress::new_static("10.0.0.1".to_string(), 24)],
                    mtu: Some(1420),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec!["wireguard".to_string()],
            metadata: HashMap::new(),
        },
        NetworkNode {
            id: vpn_client_id,
            name: "vpn-client".to_string(),
            node_type: NodeType::Workstation,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_dhcp()],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "wg0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Wireguard,
                    addresses: vec![IpAddress::new_static("10.0.0.2".to_string(), 24)],
                    mtu: Some(1420),
                    vlan_id: None,
                    bridge_members: vec![],
                },
            ],
            services: vec!["wireguard".to_string()],
            metadata: HashMap::new(),
        },
    ];
    
    let connections = vec![
        NetworkConnection {
            from_node: vpn_server_id,
            from_interface: "wg0".to_string(),
            to_node: vpn_client_id,
            to_interface: "wg0".to_string(),
            network_type: NetworkType::Wireguard,
        },
    ];
    
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "vpn-network".to_string(),
        nodes,
        connections,
        timestamp: chrono::Utc::now(),
    };
    
    let systems = service.process_topology_event(event).await.unwrap();
    
    assert_eq!(systems.len(), 2);
    
    // Verify VPN server has WireGuard service
    let vpn_server = systems.iter().find(|s| s.hostname == "vpn-server").unwrap();
    assert!(vpn_server.services.iter().any(|s| s.name == "wireguard"));
    
    // Verify firewall allows WireGuard port
    let firewall = vpn_server.firewall_config.as_ref().unwrap();
    assert!(firewall.allowed_udp_ports.contains(&51820));
}

#[tokio::test]
async fn test_vlan_network_topology() {
    let mut service = NetworkIntegrationService::new();
    
    let router_id = Uuid::new_v4();
    let server_id = Uuid::new_v4();
    
    let nodes = vec![
        NetworkNode {
            id: router_id,
            name: "vlan-router".to_string(),
            node_type: NodeType::Router,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Ethernet,
                    addresses: vec![IpAddress::new_static("192.168.1.1".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: None,
                    bridge_members: vec![],
                },
                NetworkInterface {
                    name: "eth0.10".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Vlan,
                    addresses: vec![IpAddress::new_static("172.16.10.1".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: Some(10),
                    bridge_members: vec![],
                },
            ],
            services: vec!["dhcp".to_string()],
            metadata: HashMap::new(),
        },
        NetworkNode {
            id: server_id,
            name: "vlan-server".to_string(),
            node_type: NodeType::Server,
            interfaces: vec![
                NetworkInterface {
                    name: "eth0.10".to_string(),
                    mac_address: None,
                    interface_type: InterfaceType::Vlan,
                    addresses: vec![IpAddress::new_static("172.16.10.10".to_string(), 24)],
                    mtu: Some(1500),
                    vlan_id: Some(10),
                    bridge_members: vec![],
                },
            ],
            services: vec![],
            metadata: HashMap::new(),
        },
    ];
    
    let connections = vec![
        NetworkConnection {
            from_node: router_id,
            from_interface: "eth0.10".to_string(),
            to_node: server_id,
            to_interface: "eth0.10".to_string(),
            network_type: NetworkType::Vlan(10),
        },
    ];
    
    let event = NetworkTopologyEvent {
        topology_id: Uuid::new_v4(),
        name: "vlan-network".to_string(),
        nodes,
        connections,
        timestamp: chrono::Utc::now(),
    };
    
    let systems = service.process_topology_event(event).await.unwrap();
    
    assert_eq!(systems.len(), 2);
    
    // Verify VLAN interfaces are configured
    let router = systems.iter().find(|s| s.hostname == "vlan-router").unwrap();
    assert_eq!(router.network_config.interfaces.len(), 2);
    
    let vlan_interface = router.network_config.interfaces
        .iter()
        .find(|i| i.name == "eth0.10")
        .unwrap();
    assert!(vlan_interface.addresses.contains(&"172.16.10.1/24".to_string()));
}