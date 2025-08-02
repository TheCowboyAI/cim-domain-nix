// Copyright 2025 Cowboy AI, LLC.

//! Acceptance test for network topology to NixOS configuration generation
//! 
//! Test scenario:
//! - Starlink router (WiFi disabled) -> UDM Pro WAN
//! - UDM Pro LAN -> UniFi 24-port switch
//! - Switch -> Mac Studio M3 Ultra (Leaf Node)

use cim_domain_nix::{
    network::{
        NetworkIntegrationService,
        value_objects::{
            NetworkTopology, NetworkNode, NodeType, NodeTier, 
            NetworkInterface, InterfaceType, IpAddress, NetworkService,
            ServiceProtocol, InterfaceChangeType,
        },
        NetworkTopologyEvent, InterfaceChangeEvent,
    },
    services::ConfigurationService,
    value_objects::MessageIdentity,
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_starlink_udm_mac_topology() -> anyhow::Result<()> {
    println!("\n=== Network Topology Acceptance Test ===");
    println!("Scenario: Starlink -> UDM Pro -> Switch -> Mac Studio");
    
    // Create network integration service
    let mut service = NetworkIntegrationService::new();
    
    // Step 1: Create the network topology
    let topology = create_test_topology();
    println!("\n1. Created network topology with {} nodes", topology.nodes.len());
    
    // Step 2: Process topology event
    let systems = service.process_topology_event(NetworkTopologyEvent {
        topology: topology.clone(),
        correlation_id: MessageIdentity::new_root().correlation_id,
        timestamp: chrono::Utc::now(),
    }).await?;
    
    println!("\n2. Generated {} NixOS configurations", systems.len());
    
    // Step 3: Verify generated configurations
    verify_configurations(&systems)?;
    
    // Step 4: Test network changes (simulate WAN failover)
    println!("\n4. Testing network changes (WAN failover scenario)");
    let wan_change = InterfaceChangeEvent {
        node_id: topology.nodes[1].id, // UDM Pro
        interface_name: "wan0".to_string(),
        change_type: InterfaceChangeType::StatusChange,
        old_config: Some(serde_json::json!({
            "status": "up",
            "carrier": true
        })),
        new_config: serde_json::json!({
            "status": "down",
            "carrier": false
        }),
        correlation_id: MessageIdentity::new_root().correlation_id,
        timestamp: chrono::Utc::now(),
    };
    
    let updated_config = service.process_interface_change(wan_change).await?;
    println!("   ✓ Updated configuration for failover: {}", updated_config.hostname);
    
    // Step 5: Generate flakes for deployment
    println!("\n5. Generating deployment flakes");
    let config_service = ConfigurationService::new();
    
    for system in &systems {
        let flake_path = format!("/tmp/nix-topology-test/{}", system.hostname);
        println!("   Creating flake for {}: {}", system.hostname, flake_path);
        
        // Create flake.nix content
        let flake_content = generate_flake_for_system(system);
        
        // In a real test, we would write this to disk
        // For now, just verify it's generated
        assert!(!flake_content.is_empty());
        println!("   ✓ Generated flake.nix ({} bytes)", flake_content.len());
    }
    
    println!("\n✅ Acceptance test completed successfully!");
    Ok(())
}

fn create_test_topology() -> NetworkTopology {
    let mut nodes = Vec::new();
    
    // 1. Starlink Router (WiFi disabled)
    let starlink = NetworkNode {
        id: Uuid::new_v4(),
        name: "starlink-router".to_string(),
        node_type: NodeType::Gateway,
        tier: NodeTier::SuperCluster, // Internet gateway
        interfaces: vec![
            NetworkInterface {
                name: "wan0".to_string(),
                mac_address: Some("00:11:22:33:44:55".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![
                    IpAddress {
                        address: "100.64.0.1".to_string(), // Starlink CGNAT
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
            m.insert("model".to_string(), "Starlink Router Gen2".to_string());
            m
        },
    };
    nodes.push(starlink);
    
    // 2. Ubiquiti UDM Pro
    let udm_pro = NetworkNode {
        id: Uuid::new_v4(),
        name: "udm-pro".to_string(),
        node_type: NodeType::Router,
        tier: NodeTier::Cluster, // Site router
        interfaces: vec![
            NetworkInterface {
                name: "wan0".to_string(),
                mac_address: Some("00:AA:BB:CC:DD:EE".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![
                    IpAddress {
                        address: "192.168.100.2".to_string(),
                        prefix_length: 24,
                        dhcp: true, // DHCP from Starlink
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
            "wireguard".to_string(),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "UDM-Pro".to_string());
            m.insert("unifi_version".to_string(), "7.5.187".to_string());
            m
        },
    };
    nodes.push(udm_pro);
    
    // 3. UniFi 24-port Switch
    let unifi_switch = NetworkNode {
        id: Uuid::new_v4(),
        name: "unifi-switch-24".to_string(),
        node_type: NodeType::Router, // L2 switch acts as router in our model
        tier: NodeTier::Leaf, // Access layer
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
            },
            // Port 1 - Mac Studio
            NetworkInterface {
                name: "port1".to_string(),
                mac_address: None,
                interface_type: InterfaceType::Bridge,
                addresses: vec![],
                mtu: Some(1500),
                vlan_id: None,
                bridge_members: vec!["uplink".to_string()],
            }
        ],
        services: vec!["switching".to_string()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "USW-24-PoE".to_string());
            m.insert("ports".to_string(), "24".to_string());
            m
        },
    };
    nodes.push(unifi_switch);
    
    // 4. Mac Studio M3 Ultra (Leaf Node)
    let mac_studio = NetworkNode {
        id: Uuid::new_v4(),
        name: "mac-studio-leaf".to_string(),
        node_type: NodeType::Server,
        tier: NodeTier::Leaf, // Leaf node in CIM architecture
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
            "docker".to_string(),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "Mac Studio M3 Ultra".to_string());
            m.insert("os".to_string(), "nix-darwin".to_string());
            m.insert("role".to_string(), "cim-leaf-node".to_string());
            m
        },
    };
    nodes.push(mac_studio);
    
    NetworkTopology {
        id: Uuid::new_v4(),
        name: "starlink-homelab".to_string(),
        nodes,
        connections: vec![
            // Starlink -> UDM Pro
            ("starlink-router".to_string(), "udm-pro".to_string()),
            // UDM Pro -> Switch
            ("udm-pro".to_string(), "unifi-switch-24".to_string()),
            // Switch -> Mac Studio
            ("unifi-switch-24".to_string(), "mac-studio-leaf".to_string()),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("site".to_string(), "homelab".to_string());
            m.insert("purpose".to_string(), "CIM development environment".to_string());
            m
        },
    }
}

fn verify_configurations(systems: &[cim_domain_nix::value_objects::SystemConfiguration]) -> anyhow::Result<()> {
    println!("\n3. Verifying generated configurations:");
    
    // Verify we have configs for all expected systems
    assert_eq!(systems.len(), 4, "Should have 4 system configurations");
    
    // Verify Starlink router config
    let starlink = systems.iter()
        .find(|s| s.hostname == "starlink-router")
        .expect("Should have Starlink config");
    println!("   ✓ Starlink Router:");
    println!("     - Services: {:?}", starlink.services);
    assert!(starlink.networking_config.contains("192.168.100.1"));
    
    // Verify UDM Pro config
    let udm = systems.iter()
        .find(|s| s.hostname == "udm-pro")
        .expect("Should have UDM Pro config");
    println!("   ✓ UDM Pro:");
    println!("     - Services: {:?}", udm.services);
    assert!(udm.services.contains(&"dhcp".to_string()));
    assert!(udm.services.contains(&"nat".to_string()));
    assert!(udm.services.contains(&"firewall".to_string()));
    assert!(udm.networking_config.contains("10.0.0.1"));
    
    // Verify Switch config
    let switch = systems.iter()
        .find(|s| s.hostname == "unifi-switch-24")
        .expect("Should have Switch config");
    println!("   ✓ UniFi Switch:");
    println!("     - Services: {:?}", switch.services);
    
    // Verify Mac Studio config (most important - our leaf node)
    let mac = systems.iter()
        .find(|s| s.hostname == "mac-studio-leaf")
        .expect("Should have Mac Studio config");
    println!("   ✓ Mac Studio (Leaf Node):");
    println!("     - Services: {:?}", mac.services);
    println!("     - Packages: {} total", mac.packages.len());
    assert!(mac.services.contains(&"nats".to_string()));
    assert!(mac.services.contains(&"cim-leaf".to_string()));
    
    // Verify leaf node has proper NATS configuration
    assert!(mac.extra_config.contains("services.nats"), "Leaf node should have NATS config");
    
    Ok(())
}

fn generate_flake_for_system(system: &cim_domain_nix::value_objects::SystemConfiguration) -> String {
    format!(r#"{{
  description = "NixOS configuration for {}";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    {}
  }};

  outputs = {{ self, nixpkgs{} }}: {{
    nixosConfigurations.{} = nixpkgs.lib.nixosSystem {{
      system = "x86_64-linux";
      modules = [
        ./hardware-configuration.nix
        ({{ pkgs, ... }}: {{
          # Generated network configuration
          networking.hostName = "{}";
          
          # Network interfaces
{}
          
          # Services
{}
          
          # System packages
          environment.systemPackages = with pkgs; [
{}
          ];
          
          # Extra configuration
{}
        }})
      ];
    }};
  }};
}}
"#,
        system.hostname,
        if system.hostname == "mac-studio-leaf" { 
            "nix-darwin.url = \"github:LnL7/nix-darwin\";\n    cim.url = \"github:thecowboyai/cim\";" 
        } else { 
            "" 
        },
        if system.hostname == "mac-studio-leaf" { ", nix-darwin, cim" } else { "" },
        system.hostname,
        system.hostname,
        indent_string(&system.networking_config, 10),
        generate_services_config(&system.services),
        system.packages.iter()
            .map(|p| format!("            {}", p))
            .collect::<Vec<_>>()
            .join("\n"),
        indent_string(&system.extra_config, 10),
    )
}

fn generate_services_config(services: &[String]) -> String {
    services.iter()
        .map(|service| match service.as_str() {
            "dhcp" => "          services.dhcpd4.enable = true;",
            "dns" => "          services.unbound.enable = true;",
            "firewall" => "          networking.firewall.enable = true;",
            "nat" => "          networking.nat.enable = true;\n          networking.nat.externalInterface = \"wan0\";\n          networking.nat.internalInterfaces = [ \"lan0\" ];",
            "wireguard" => "          networking.wireguard.enable = true;",
            "nats" => r#"          services.nats = {
            enable = true;
            serverName = "leaf-node";
            jetstream = true;
            port = 4222;
            leafNode = {
              remotes = [ "nats://cluster.example.com:7422" ];
            };
          };"#,
            "cim-leaf" => r#"          services.cim-leaf = {
            enable = true;
            natsUrl = "nats://localhost:4222";
            nodeRole = "leaf";
          };"#,
            _ => "",
        })
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent_string(s: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    s.lines()
        .map(|line| format!("{}{}", indent, line))
        .collect::<Vec<_>>()
        .join("\n")
}