// Copyright 2025 Cowboy AI, LLC.

//! Example demonstrating a real-world network topology with Starlink
//! 
//! This example shows how to model and generate NixOS configurations for:
//! - Starlink router (WiFi disabled) as WAN
//! - Ubiquiti UDM Pro as main router
//! - UniFi 24-port switch for LAN distribution
//! - Mac Studio M3 Ultra as a CIM leaf node

use cim_domain_nix::{
    network::{
        NetworkIntegrationService,
        value_objects::{
            NetworkTopology, NetworkNode, NodeType, NodeTier,
            NetworkInterface, InterfaceType, IpAddress,
        },
        NetworkTopologyEvent,
    },
    commands::CreateFlake,
    handlers::NixCommandHandler,
    services::FlakeService,
    value_objects::MessageIdentity,
};
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌐 Starlink Network Topology Demo");
    println!("==================================\n");
    
    // Create temporary directory for our configurations
    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();
    
    println!("📁 Output directory: {}\n", base_path.display());
    
    // Step 1: Create the network topology
    println!("1️⃣  Creating network topology...");
    let topology = create_starlink_topology();
    print_topology(&topology);
    
    // Step 2: Generate NixOS configurations
    println!("\n2️⃣  Generating NixOS configurations...");
    let mut service = NetworkIntegrationService::new();
    
    let systems = service.process_topology_event(NetworkTopologyEvent {
        topology: topology.clone(),
        correlation_id: MessageIdentity::new_root().correlation_id,
        timestamp: chrono::Utc::now(),
    }).await?;
    
    println!("   Generated {} system configurations", systems.len());
    
    // Step 3: Create flakes for each system
    println!("\n3️⃣  Creating deployment flakes...");
    let handler = NixCommandHandler::new();
    let flake_service = FlakeService::new();
    
    for system in &systems {
        let system_path = base_path.join(&system.hostname);
        std::fs::create_dir_all(&system_path)?;
        
        println!("\n   📦 Creating flake for: {}", system.hostname);
        println!("      Path: {}", system_path.display());
        
        // Create the flake
        let cmd = CreateFlake {
            identity: MessageIdentity::new_root(),
            path: system_path.clone(),
            description: format!("NixOS configuration for {}", system.hostname),
            template: Some("nixos-system".to_string()),
        };
        
        let events = handler.handle_command(Box::new(cmd)).await?;
        println!("      ✓ Created flake ({} events)", events.len());
        
        // Write the actual configuration
        let config_path = system_path.join("configuration.nix");
        let config_content = generate_nixos_config(system);
        std::fs::write(&config_path, config_content)?;
        println!("      ✓ Wrote configuration.nix");
        
        // Write hardware configuration stub
        let hw_config_path = system_path.join("hardware-configuration.nix");
        let hw_config = generate_hardware_config(&system.hostname);
        std::fs::write(&hw_config_path, hw_config)?;
        println!("      ✓ Wrote hardware-configuration.nix");
        
        // Show key configuration details
        println!("      📋 Configuration summary:");
        println!("         - Tier: {:?}", get_node_tier(&system.hostname));
        println!("         - Services: {}", system.services.join(", "));
        println!("         - Primary IP: {}", get_primary_ip(system));
    }
    
    // Step 4: Special configuration for Mac Studio leaf node
    println!("\n4️⃣  Configuring Mac Studio as CIM Leaf Node...");
    let mac_path = base_path.join("mac-studio-leaf");
    
    // Create CIM-specific configuration
    let cim_config = generate_cim_leaf_config();
    std::fs::write(mac_path.join("cim-leaf.nix"), cim_config)?;
    println!("   ✓ Created CIM leaf node configuration");
    
    // Create NATS configuration
    let nats_config = generate_nats_config();
    std::fs::write(mac_path.join("nats.nix"), nats_config)?;
    println!("   ✓ Created NATS configuration");
    
    // Step 5: Generate deployment script
    println!("\n5️⃣  Generating deployment script...");
    let deploy_script = generate_deployment_script(&systems);
    let script_path = base_path.join("deploy.sh");
    std::fs::write(&script_path, deploy_script)?;
    std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    println!("   ✓ Created deploy.sh");
    
    // Summary
    println!("\n✅ Network topology configuration complete!");
    println!("\n📊 Summary:");
    println!("   - Network name: {}", topology.name);
    println!("   - Total nodes: {}", topology.nodes.len());
    println!("   - Configurations generated: {}", systems.len());
    println!("   - Output directory: {}", base_path.display());
    
    println!("\n🚀 Next steps:");
    println!("   1. Review the generated configurations");
    println!("   2. Customize hardware-configuration.nix for each system");
    println!("   3. Run ./deploy.sh to deploy to your systems");
    println!("   4. The Mac Studio will automatically join the CIM cluster");
    
    // Keep the directory for inspection
    let path = base_path.to_path_buf();
    std::mem::forget(temp_dir);
    println!("\n📁 Configurations saved to: {}", path.display());
    
    Ok(())
}

fn create_starlink_topology() -> NetworkTopology {
    let mut nodes = Vec::new();
    
    // Starlink Router
    nodes.push(NetworkNode {
        id: Uuid::new_v4(),
        name: "starlink-router".to_string(),
        node_type: NodeType::Gateway,
        tier: NodeTier::SuperCluster,
        interfaces: vec![
            NetworkInterface {
                name: "wan0".to_string(),
                mac_address: Some("00:11:22:33:44:55".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "100.64.0.1".to_string(),
                    prefix_length: 24,
                    dhcp: false,
                }],
                mtu: Some(1500),
                vlan_id: None,
                bridge_members: vec![],
            },
            NetworkInterface {
                name: "lan0".to_string(),
                mac_address: Some("00:11:22:33:44:56".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "192.168.100.1".to_string(),
                    prefix_length: 24,
                    dhcp: false,
                }],
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
    });
    
    // UDM Pro
    nodes.push(NetworkNode {
        id: Uuid::new_v4(),
        name: "udm-pro".to_string(),
        node_type: NodeType::Router,
        tier: NodeTier::Cluster,
        interfaces: vec![
            NetworkInterface {
                name: "wan0".to_string(),
                mac_address: Some("00:AA:BB:CC:DD:EE".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "192.168.100.2".to_string(),
                    prefix_length: 24,
                    dhcp: true,
                }],
                mtu: Some(1500),
                vlan_id: None,
                bridge_members: vec![],
            },
            NetworkInterface {
                name: "lan0".to_string(),
                mac_address: Some("00:AA:BB:CC:DD:EF".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "10.0.0.1".to_string(),
                    prefix_length: 24,
                    dhcp: false,
                }],
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
            "unifi".to_string(),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "Dream Machine Pro".to_string());
            m.insert("unifi_version".to_string(), "7.5.187".to_string());
            m
        },
    });
    
    // UniFi Switch
    nodes.push(NetworkNode {
        id: Uuid::new_v4(),
        name: "unifi-switch-24".to_string(),
        node_type: NodeType::Router,
        tier: NodeTier::Leaf,
        interfaces: vec![
            NetworkInterface {
                name: "uplink".to_string(),
                mac_address: Some("00:22:33:44:55:66".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "10.0.0.2".to_string(),
                    prefix_length: 24,
                    dhcp: true,
                }],
                mtu: Some(1500),
                vlan_id: None,
                bridge_members: vec![],
            }
        ],
        services: vec!["switching".to_string(), "poe".to_string()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "USW-24-PoE".to_string());
            m.insert("ports".to_string(), "24".to_string());
            m.insert("poe_budget".to_string(), "250W".to_string());
            m
        },
    });
    
    // Mac Studio
    nodes.push(NetworkNode {
        id: Uuid::new_v4(),
        name: "mac-studio-leaf".to_string(),
        node_type: NodeType::Server,
        tier: NodeTier::Leaf,
        interfaces: vec![
            NetworkInterface {
                name: "en0".to_string(),
                mac_address: Some("00:FF:EE:DD:CC:BB".to_string()),
                interface_type: InterfaceType::Ethernet,
                addresses: vec![IpAddress {
                    address: "10.0.0.100".to_string(),
                    prefix_length: 24,
                    dhcp: true,
                }],
                mtu: Some(1500),
                vlan_id: None,
                bridge_members: vec![],
            }
        ],
        services: vec![
            "nats".to_string(),
            "cim-leaf".to_string(),
            "docker".to_string(),
            "k3s".to_string(),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("model".to_string(), "Mac Studio M3 Ultra".to_string());
            m.insert("cpu".to_string(), "24-core CPU".to_string());
            m.insert("gpu".to_string(), "76-core GPU".to_string());
            m.insert("ram".to_string(), "192GB".to_string());
            m.insert("storage".to_string(), "8TB SSD".to_string());
            m.insert("os".to_string(), "nix-darwin".to_string());
            m.insert("role".to_string(), "cim-leaf-node".to_string());
            m
        },
    });
    
    NetworkTopology {
        id: Uuid::new_v4(),
        name: "starlink-homelab".to_string(),
        nodes,
        connections: vec![
            ("starlink-router".to_string(), "udm-pro".to_string()),
            ("udm-pro".to_string(), "unifi-switch-24".to_string()),
            ("unifi-switch-24".to_string(), "mac-studio-leaf".to_string()),
        ],
        metadata: {
            let mut m = HashMap::new();
            m.insert("location".to_string(), "Home Lab".to_string());
            m.insert("purpose".to_string(), "CIM Development Environment".to_string());
            m.insert("wan_type".to_string(), "Starlink Satellite".to_string());
            m
        },
    }
}

fn print_topology(topology: &NetworkTopology) {
    println!("\n📡 Network Topology: {}", topology.name);
    println!("   Nodes:");
    for node in &topology.nodes {
        println!("   - {} ({:?}, Tier: {:?})", node.name, node.node_type, node.tier);
        if let Some(model) = node.metadata.get("model") {
            println!("     Model: {}", model);
        }
    }
    println!("\n   Connections:");
    for (from, to) in &topology.connections {
        println!("   - {} → {}", from, to);
    }
}

fn generate_nixos_config(system: &cim_domain_nix::value_objects::SystemConfiguration) -> String {
    format!(r#"{{ config, pkgs, ... }}:

{{
  imports = [
    ./hardware-configuration.nix
    {}
  ];

  # Basic system configuration
  networking.hostName = "{}";
  
  # Generated network configuration
{}

  # Services configuration
{}

  # System packages
  environment.systemPackages = with pkgs; [
{}
  ];

  # Extra configuration
{}

  # System state version
  system.stateVersion = "24.05";
}}
"#,
        if system.hostname == "mac-studio-leaf" {
            "./cim-leaf.nix\n    ./nats.nix"
        } else {
            ""
        },
        system.hostname,
        indent_string(&system.networking_config, 2),
        generate_services_config(&system.services),
        system.packages.iter()
            .map(|p| format!("    {}", p))
            .collect::<Vec<_>>()
            .join("\n"),
        indent_string(&system.extra_config, 2),
    )
}

fn generate_hardware_config(hostname: &str) -> String {
    match hostname {
        "mac-studio-leaf" => r#"{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [ ];

  # Mac Studio M3 Ultra specific configuration
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # File systems (customize based on actual setup)
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "apfs";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
  };

  # Hardware configuration
  hardware.cpu.apple.updateMicrocode = true;
  hardware.enableRedistributableFirmware = true;
  
  # Networking
  networking.useDHCP = false;
  networking.interfaces.en0.useDHCP = true;

  # Mac-specific
  services.nix-darwin.enable = true;
}
"#,
        _ => r#"{ config, lib, pkgs, modulesPath, ... }:

{
  imports = [ (modulesPath + "/installer/scan/not-detected.nix") ];

  # Boot loader
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  # File systems (customize based on actual hardware)
  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-label/boot";
    fsType = "vfat";
  };

  # Networking
  networking.useDHCP = false;
  
  # Hardware
  hardware.cpu.intel.updateMicrocode = true;
  hardware.enableRedistributableFirmware = true;
}
"#
    }
}

fn generate_cim_leaf_config() -> String {
    r#"{ config, pkgs, ... }:

{
  # CIM Leaf Node Configuration
  services.cim-leaf = {
    enable = true;
    
    # Node identification
    nodeId = "mac-studio-m3-ultra";
    nodeRole = "leaf";
    
    # NATS connection
    natsUrl = "nats://localhost:4222";
    
    # Leaf node settings
    leafNode = {
      # Connect to cluster nodes
      remotes = [
        "nats://cluster-1.example.com:7422"
        "nats://cluster-2.example.com:7422"
      ];
      
      # Authentication
      credentials = "/etc/cim/leaf.creds";
    };
    
    # Services to run on this leaf node
    services = [
      "compute"
      "storage"
      "inference"
      "development"
    ];
    
    # Resource allocation
    resources = {
      cpu = {
        cores = 20; # Reserve 4 cores for system
        type = "apple-m3-ultra";
      };
      
      memory = {
        allocated = "160G"; # Reserve 32G for system
      };
      
      gpu = {
        enabled = true;
        cores = 76;
        type = "apple-m3-ultra";
      };
    };
  };
  
  # Development tools for CIM
  environment.systemPackages = with pkgs; [
    # CIM development
    cim-cli
    cim-dev-tools
    
    # Container runtime
    docker
    docker-compose
    
    # Kubernetes
    kubectl
    k9s
    
    # Development
    rustup
    go
    nodejs
    python3
    
    # Nix tools
    nix-direnv
    nixpkgs-fmt
    nil
  ];
  
  # Enable Docker
  virtualisation.docker = {
    enable = true;
    enableOnBoot = true;
    autoPrune.enable = true;
  };
  
  # Enable k3s for local Kubernetes
  services.k3s = {
    enable = true;
    role = "agent";
    serverAddr = "https://10.0.0.1:6443";
  };
}
"#
}

fn generate_nats_config() -> String {
    r#"{ config, pkgs, ... }:

{
  # NATS Server Configuration for CIM Leaf Node
  services.nats = {
    enable = true;
    
    # Server identification
    serverName = "mac-studio-leaf";
    
    # Ports
    port = 4222;        # Client connections
    monitoringPort = 8222;  # HTTP monitoring
    
    # Clustering disabled for leaf nodes
    cluster = null;
    
    # Leaf node configuration
    leafNode = {
      enable = true;
      port = 7422;
      
      # TLS for leaf connections
      tls = {
        enable = true;
        cert = "/etc/nats/leaf-cert.pem";
        key = "/etc/nats/leaf-key.pem";
        ca = "/etc/nats/ca.pem";
      };
    };
    
    # JetStream for persistence
    jetstream = {
      enable = true;
      storeDir = "/var/lib/nats/jetstream";
      maxMemory = "10G";
      maxStore = "100G";
    };
    
    # Authorization
    authorization = {
      users = [
        {
          user = "cim-leaf";
          password = "$2a$11$..."; # bcrypt hash
          permissions = {
            publish = [
              "cim.leaf.>"
              "cim.events.>"
              "cim.metrics.>"
            ];
            subscribe = [
              "cim.>"
              "_INBOX.>"
            ];
          };
        }
      ];
    };
    
    # Logging
    logging = {
      debug = false;
      trace = false;
      logtime = true;
      connectErrorReports = 3;
      reconnectErrorReports = 1;
    };
    
    # System account for monitoring
    systemAccount = "SYS";
    
    # Accounts
    accounts = {
      SYS = {
        users = [
          { user = "sys"; password = "$2a$11$..."; }
        ];
      };
      CIM = {
        users = [
          { user = "cim-leaf"; }
        ];
        exports = [
          { stream = "cim.events.>"; accounts = ["*"]; }
          { service = "cim.service.>"; }
        ];
      };
    };
  };
  
  # NATS utilities
  environment.systemPackages = with pkgs; [
    natscli
    nats-top
    nats-bench
  ];
  
  # Firewall rules for NATS
  networking.firewall = {
    allowedTCPPorts = [
      4222  # NATS client
      8222  # NATS monitoring
      7422  # Leaf node connections
    ];
  };
}
"#
}

fn generate_deployment_script(systems: &[cim_domain_nix::value_objects::SystemConfiguration]) -> String {
    let mut script = r#"#!/usr/bin/env bash
# Generated deployment script for Starlink network topology

set -euo pipefail

echo "🚀 Deploying NixOS configurations"
echo "================================="

# Configuration
FLAKE_DIR="$(dirname "$0")"

"#.to_string();

    for system in systems {
        script.push_str(&format!(r#"
# Deploy to {}
deploy_{}_system() {{
    echo -e "\n📦 Deploying to {}..."
    
    if [[ "$1" == "--local" ]]; then
        echo "   Building locally..."
        sudo nixos-rebuild switch --flake "$FLAKE_DIR/{}"
    else
        echo "   Building and deploying remotely..."
        nixos-rebuild switch \
            --flake "$FLAKE_DIR/{}" \
            --target-host "root@{}" \
            --build-host "root@{}" \
            --use-remote-sudo
    fi
    
    echo "   ✅ {} deployed successfully!"
}}

"#,
            system.hostname,
            system.hostname.replace('-', "_"),
            system.hostname,
            system.hostname,
            system.hostname,
            get_primary_ip(system),
            get_primary_ip(system),
            system.hostname,
        ));
    }

    script.push_str(r#"
# Main deployment logic
main() {
    local mode="${1:-remote}"
    
    case "$mode" in
        --local)
            echo "🏠 Local deployment mode"
            ;;
        --remote|"")
            echo "🌐 Remote deployment mode"
            ;;
        --help)
            echo "Usage: $0 [--local|--remote|--help]"
            echo "  --local   Build and deploy on local machine"
            echo "  --remote  Build and deploy over SSH (default)"
            exit 0
            ;;
        *)
            echo "Unknown mode: $mode"
            exit 1
            ;;
    esac
    
    # Deploy in order: gateway -> router -> switch -> leaf
"#);

    for system in systems {
        script.push_str(&format!(
            "    deploy_{}_system \"$mode\"\n",
            system.hostname.replace('-', "_")
        ));
    }

    script.push_str(r#"
    echo -e "\n✅ All systems deployed successfully!"
    echo "🎉 Your Starlink-based CIM network is ready!"
}

# Run main function
main "$@"
"#);

    script
}

fn generate_services_config(services: &[String]) -> String {
    services.iter()
        .filter_map(|service| {
            match service.as_str() {
                "dhcp" => Some("  services.dhcpd4.enable = true;\n  services.dhcpd4.interfaces = [ \"lan0\" ];"),
                "dns" => Some("  services.unbound.enable = true;"),
                "firewall" => Some("  networking.firewall.enable = true;"),
                "nat" => Some("  networking.nat = {\n    enable = true;\n    externalInterface = \"wan0\";\n    internalInterfaces = [ \"lan0\" ];\n  };"),
                "wireguard" => Some("  networking.wireguard.enable = true;"),
                "unifi" => Some("  services.unifi.enable = true;\n  services.unifi.openFirewall = true;"),
                _ => None,
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_node_tier(hostname: &str) -> NodeTier {
    match hostname {
        "starlink-router" => NodeTier::SuperCluster,
        "udm-pro" => NodeTier::Cluster,
        "unifi-switch-24" => NodeTier::Leaf,
        "mac-studio-leaf" => NodeTier::Leaf,
        _ => NodeTier::Client,
    }
}

fn get_primary_ip(system: &cim_domain_nix::value_objects::SystemConfiguration) -> &str {
    // Extract primary IP from networking config
    // This is simplified - in real implementation would parse the config
    match system.hostname.as_str() {
        "starlink-router" => "192.168.100.1",
        "udm-pro" => "10.0.0.1",
        "unifi-switch-24" => "10.0.0.2",
        "mac-studio-leaf" => "10.0.0.100",
        _ => "dhcp",
    }
}

fn indent_string(s: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    s.lines()
        .map(|line| {
            if line.is_empty() {
                line.to_string()
            } else {
                format!("{}{}", indent, line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}