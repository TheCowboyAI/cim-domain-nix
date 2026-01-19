// Copyright (c) 2025 - Cowboy AI, Inc.
//! Example: Roundtrip Integration - Write → Read → Verify
//!
//! This example demonstrates the complete roundtrip cycle:
//! 1. Create ComputeResources
//! 2. Write to topology.nix using TopologyWriter
//! 3. Read back using TopologyReader with rnix parser
//! 4. Verify resources match original data
//!
//! Run with: cargo run --example roundtrip_demo

use anyhow::Result;
use cim_domain_nix::adapters::{TopologyReader, TopologyWriter};
use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Roundtrip Integration Demo ===\n");

    // Step 1: Create original resources
    println!("Step 1: Creating compute resources...");
    let mut resources = Vec::new();

    // Router
    let router_hostname = Hostname::new("router01")?;
    let mut router = ComputeResource::new(router_hostname, ResourceType::Router)?;
    router.set_hardware(
        Some("Ubiquiti".to_string()),
        Some("UniFi Dream Machine Pro".to_string()),
        Some("UDM-12345".to_string()),
    );
    router.add_metadata("rack", "network")?;
    router.add_metadata("vlan_support", "true")?;
    resources.push(router.clone());

    // Switch
    let switch_hostname = Hostname::new("switch01")?;
    let mut switch = ComputeResource::new(switch_hostname, ResourceType::Switch)?;
    switch.set_hardware(
        Some("Ubiquiti".to_string()),
        Some("UniFi Switch 24 PoE".to_string()),
        None,
    );
    switch.add_metadata("poe_capable", "true")?;
    switch.add_metadata("rack", "rack01")?;
    resources.push(switch.clone());

    // Server
    let server_hostname = Hostname::new("pve01")?;
    let mut server = ComputeResource::new(server_hostname, ResourceType::PhysicalServer)?;
    server.set_hardware(
        Some("Dell".to_string()),
        Some("PowerEdge R740".to_string()),
        None,
    );
    server.add_metadata("role", "Proxmox VE Host")?;
    server.add_metadata("cluster", "proxmox")?;
    resources.push(server.clone());

    // Camera (Device type)
    let camera_hostname = Hostname::new("camera01")?;
    let mut camera = ComputeResource::new(camera_hostname, ResourceType::Camera)?;
    camera.set_hardware(
        Some("Hikvision".to_string()),
        Some("DS-2CD2xx".to_string()),
        None,
    );
    camera.add_metadata("location", "zone_1")?;
    resources.push(camera.clone());

    println!("  Created {} resources\n", resources.len());

    // Step 2: Write to topology file
    println!("Step 2: Writing to topology.nix...");
    let mut writer = TopologyWriter::with_name("output/roundtrip_topology.nix", "roundtrip-demo");

    for resource in &resources {
        writer.add_node(resource)?;
    }

    let nix_content = writer.generate_topology()?;
    println!("  Generated {} bytes of Nix code\n", nix_content.len());

    // Write to file
    writer.write_to_file().await?;
    println!("  ✅ Written to output/roundtrip_topology.nix\n");

    // Step 3: Read back with rnix parser
    println!("Step 3: Reading back with rnix parser...");
    let reader = TopologyReader::new();
    let read_resources = reader.read_topology_file(Path::new("output/roundtrip_topology.nix")).await?;
    println!("  Read {} resources\n", read_resources.len());

    // Step 4: Verify roundtrip integrity
    println!("Step 4: Verifying roundtrip integrity...\n");

    assert_eq!(resources.len(), read_resources.len(), "Resource count mismatch!");

    // Verify each resource
    for original in &resources {
        let original_name = original.hostname.short_name();

        let found = read_resources.iter()
            .find(|r| r.hostname.short_name() == original_name)
            .expect(&format!("Could not find resource: {}", original_name));

        println!("  Verifying {}:", original_name);

        // Check hostname
        assert_eq!(original.hostname, found.hostname);
        println!("    ✅ Hostname matches");

        // Check resource type
        // Note: Some types may map differently due to functor (e.g., Camera → Device → Appliance)
        println!("    Resource type: {:?} → {:?}", original.resource_type, found.resource_type);

        // Check hardware info if present
        if original.manufacturer.is_some() {
            assert_eq!(original.manufacturer, found.manufacturer);
            assert_eq!(original.model, found.model);
            println!("    ✅ Hardware info matches");
        }

        // Check metadata
        for (key, value) in &original.metadata {
            match found.metadata.get(key) {
                Some(found_value) => {
                    assert_eq!(value, found_value, "Metadata mismatch for key: {}", key);
                }
                None => panic!("Missing metadata key: {}", key),
            }
        }
        if !original.metadata.is_empty() {
            println!("    ✅ Metadata matches ({} keys)", original.metadata.len());
        }

        println!();
    }

    // Summary
    println!("=== Roundtrip Summary ===");
    println!("✅ Write: {} resources → topology.nix", resources.len());
    println!("✅ Read: topology.nix → {} resources (rnix parser)", read_resources.len());
    println!("✅ Verify: All hostnames, hardware info, and metadata match");
    println!("\n⚠️  Note: Camera → Device → Appliance due to many-to-one functor mapping");
    println!("   This is expected behavior (see functor_demo for details)");

    println!("\n🎉 Roundtrip integration successful!");

    Ok(())
}
