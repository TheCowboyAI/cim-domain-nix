// Copyright (c) 2025 - Cowboy AI, Inc.
//! Example: Generate nixos-topology from Infrastructure Resources
//!
//! This example demonstrates how to use TopologyWriter to generate
//! a nixos-topology configuration from Infrastructure domain resources.
//!
//! Run with: cargo run --example generate_topology

use cim_domain_nix::adapters::topology_writer::TopologyWriter;
use cim_infrastructure::{ComputeResource, Hostname, ResourceType};

fn main() -> anyhow::Result<()> {
    println!("=== Generate nixos-topology Example ===\n");

    // Create topology writer
    let mut writer = TopologyWriter::with_name("output/topology.nix", "homelab");

    // Add a router
    println!("Adding router...");
    let router_hostname = Hostname::new("router01")?;
    let mut router = ComputeResource::new(router_hostname, ResourceType::Router)?;
    router.set_hardware(
        Some("Ubiquiti".to_string()),
        Some("UniFi Dream Machine Pro".to_string()),
        Some("UDM-12345".to_string()),
    );
    router.add_metadata("rack", "network")?;
    router.add_metadata("vlan_support", "true")?;
    writer.add_node(&router)?;

    // Add switches
    println!("Adding switches...");
    for i in 1..=3 {
        let hostname = Hostname::new(&format!("switch{:02}", i))?;
        let mut switch = ComputeResource::new(hostname, ResourceType::Switch)?;
        switch.set_hardware(
            Some("Ubiquiti".to_string()),
            Some("UniFi Switch 24 PoE".to_string()),
            None,
        );
        switch.add_metadata("rack", &format!("rack{:02}", i))?;
        switch.add_metadata("poe_capable", "true")?;
        writer.add_node(&switch)?;
    }

    // Add physical servers
    println!("Adding servers...");
    let server_types = vec![
        ("pve01", "Proxmox VE Host"),
        ("pve02", "Proxmox VE Host"),
        ("pve03", "Proxmox VE Host"),
    ];

    for (name, description) in server_types {
        let hostname = Hostname::new(name)?;
        let mut server = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;
        server.set_hardware(
            Some("Dell".to_string()),
            Some("PowerEdge R740".to_string()),
            None,
        );
        server.add_metadata("role", description)?;
        server.add_metadata("cluster", "proxmox")?;
        writer.add_node(&server)?;
    }

    // Add security camera
    println!("Adding security cameras...");
    for i in 1..=5 {
        let hostname = Hostname::new(&format!("camera{:02}", i))?;
        let mut camera = ComputeResource::new(hostname, ResourceType::Camera)?;
        camera.set_hardware(Some("Hikvision".to_string()), Some("DS-2CD2xx".to_string()), None);
        camera.add_metadata("location", &format!("zone_{}", (i - 1) / 2 + 1))?;
        camera.add_metadata("recording", "continuous")?;
        writer.add_node(&camera)?;
    }

    // Add KVM and monitors
    println!("Adding KVM and monitors...");
    let kvm_hostname = Hostname::new("kvm01")?;
    let mut kvm = ComputeResource::new(kvm_hostname, ResourceType::KVM)?;
    kvm.add_metadata("ports", "8")?;
    kvm.add_metadata("rack", "operations")?;
    writer.add_node(&kvm)?;

    for i in 1..=2 {
        let hostname = Hostname::new(&format!("monitor{:02}", i))?;
        let mut monitor = ComputeResource::new(hostname, ResourceType::Monitor)?;
        monitor.add_metadata("size", "24_inch")?;
        monitor.add_metadata("rack", "operations")?;
        writer.add_node(&monitor)?;
    }

    // Generate and display topology
    println!("\n=== Generated Topology ===\n");
    let nix_code = writer.generate_topology()?;
    println!("{}", nix_code);

    println!("\n=== Summary ===");
    println!("Total nodes: {}", writer.node_count());
    println!("Output file: output/topology.nix");

    println!("\nTo write to file, use:");
    println!("  writer.write_to_file().await?;");
    println!("\n(Note: Requires tokio runtime for async write)");

    Ok(())
}
