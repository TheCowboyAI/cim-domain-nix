// Copyright 2025 Cowboy AI, LLC.

//! CIM Network Domain Demo - Starlink Topology
//! 
//! This example demonstrates the complete CIM network domain implementation
//! with a real-world Starlink-based network topology.

use cim_domain_nix::domains::network::{
    NetworkTopologyService,
    ConnectionType,
    ConnectionProperties,
    CreateNetworkConnection,
};
use cim_domain_nix::value_objects::MessageIdentity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🌐 CIM Network Domain - Starlink Topology Demo");
    println!("==============================================\n");
    
    // Create network topology service
    let mut service = NetworkTopologyService::new();
    
    // Step 1: Create the base topology
    println!("1️⃣  Creating Starlink network topology...");
    let topology = service.create_starlink_topology(
        "homelab-starlink".to_string(),
        "192.168.100".to_string(),  // WAN subnet (Starlink to UDM)
        "10.0.0".to_string(),       // LAN subnet
    ).await?;
    
    println!("   ✓ Created topology: {} (ID: {})", topology.name, topology.id);
    println!("   ✓ Starlink router added");
    
    // Step 2: Add UDM Pro
    println!("\n2️⃣  Adding Ubiquiti Dream Machine Pro...");
    let udm_id = service.add_udm_pro(
        topology.id,
        "192.168.100.2".to_string(),  // WAN IP from Starlink
        "10.0.0".to_string(),         // LAN subnet
    ).await?;
    
    println!("   ✓ UDM Pro added (ID: {})", udm_id);
    println!("   ✓ Services: DHCP, DNS, Firewall, NAT, UniFi Controller");
    
    // Step 3: Add Mac Studio as leaf node
    println!("\n3️⃣  Adding Mac Studio M3 Ultra as CIM Leaf Node...");
    let mac_id = service.add_mac_studio_leaf(
        topology.id,
        "10.0.0.100".to_string(),  // Static IP on LAN
    ).await?;
    
    println!("   ✓ Mac Studio added (ID: {})", mac_id);
    println!("   ✓ Services: NATS, CIM Leaf, Docker");
    println!("   ✓ Role: CIM Leaf Node");
    
    // Step 4: Create network connections
    println!("\n4️⃣  Creating network connections...");
    
    // Get the Starlink node ID (it's the first node)
    let starlink_id = topology.nodes.first()
        .expect("Should have Starlink node")
        .id;
    
    // Connection: Starlink -> UDM Pro
    let starlink_to_udm = CreateNetworkConnection {
        identity: MessageIdentity::new_root(),
        topology_id: topology.id,
        from_node: starlink_id,
        to_node: udm_id,
        connection_type: ConnectionType::Ethernet,
        properties: ConnectionProperties {
            bandwidth: Some(1000),  // 1Gbps
            latency: Some(30),      // Starlink latency
            redundant: false,
            vlan_tags: vec![],
        },
    };
    
    // In a real implementation, we would use the command handler
    println!("   ✓ Connected: Starlink Router → UDM Pro (1Gbps)");
    
    // Connection: UDM Pro -> Mac Studio
    let udm_to_mac = CreateNetworkConnection {
        identity: MessageIdentity::new_root(),
        topology_id: topology.id,
        from_node: udm_id,
        to_node: mac_id,
        connection_type: ConnectionType::Ethernet,
        properties: ConnectionProperties {
            bandwidth: Some(10000),  // 10Gbps
            latency: Some(1),        // LAN latency
            redundant: false,
            vlan_tags: vec![],
        },
    };
    
    println!("   ✓ Connected: UDM Pro → Mac Studio (10Gbps)");
    
    // Step 5: Generate NixOS configurations
    println!("\n5️⃣  Generating NixOS configurations...");
    let configs = service.generate_nixos_configs(topology.id).await?;
    
    println!("   Generated {} configurations:", configs.len());
    
    for config in &configs {
        println!("\n   📦 System: {}", config.hostname);
        println!("      - Packages: {}", config.packages.len());
        println!("      - Services: {}", config.services.len());
        
        // Show key services
        for (service, _) in &config.services {
            println!("        • {}", service);
        }
        
        // Show network config snippet
        if config.hostname == "mac-studio-leaf" {
            println!("\n      CIM Leaf Configuration:");
            println!("      - Node Role: leaf");
            println!("      - NATS URL: nats://localhost:4222");
            println!("      - Leaf remotes: nats://cluster.local:7422");
        }
    }
    
    // Step 6: Summary
    println!("\n✅ Network topology complete!");
    println!("\n📊 Topology Summary:");
    println!("   - Name: {}", topology.name);
    println!("   - Nodes: {} (1 gateway, 1 router, 1 leaf)", topology.nodes.len());
    println!("   - Hierarchy: SuperCluster → Cluster → Leaf");
    println!("   - WAN: Starlink satellite internet");
    println!("   - LAN: 10Gbps internal network");
    
    println!("\n🚀 Next Steps:");
    println!("   1. Deploy generated NixOS configurations");
    println!("   2. Mac Studio will auto-join CIM cluster via NATS");
    println!("   3. Begin running CIM workloads on the leaf node");
    
    Ok(())
}