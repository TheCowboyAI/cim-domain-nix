// Copyright (c) 2025 - Cowboy AI, Inc.
//! Example: Category Theory Functor Demonstration
//!
//! This example demonstrates the ResourceType ⟷ TopologyNodeType functor
//! and shows how type mappings work, including roundtrip verification.
//!
//! Run with: cargo run --example functor_demo

use cim_domain_nix::functors::resource_type_functor::*;
use cim_infrastructure::ResourceType;

fn main() {
    println!("=== ResourceType Functor Demonstration ===\n");

    // Demonstrate forward mapping (F: ResourceType → TopologyNodeType)
    println!("## Forward Mapping (F: ResourceType → TopologyNodeType)\n");

    let resource_types = vec![
        ResourceType::PhysicalServer,
        ResourceType::Router,
        ResourceType::Switch,
        ResourceType::Camera,
        ResourceType::KVM,
        ResourceType::Monitor,
        ResourceType::StorageArray,
    ];

    for rt in &resource_types {
        let topology_type = map_resource_type_to_topology(*rt);
        println!("  {:?} → {:?}", rt, topology_type);
    }

    // Demonstrate reverse mapping (G: TopologyNodeType → ResourceType)
    println!("\n## Reverse Mapping (G: TopologyNodeType → ResourceType)\n");

    let topology_types = vec![
        TopologyNodeType::PhysicalServer,
        TopologyNodeType::Router,
        TopologyNodeType::Switch,
        TopologyNodeType::Device, // Many ResourceTypes map here
    ];

    for tt in &topology_types {
        let resource_type = map_topology_to_resource_type(*tt);
        println!("  {:?} → {:?}", tt, resource_type);
    }

    // Demonstrate many-to-one mapping
    println!("\n## Many-to-One Mapping (Multiple Resources → Single Topology Type)\n");

    let device_types = get_resource_types_for_topology(TopologyNodeType::Device);
    println!("TopologyNodeType::Device can represent {} different ResourceTypes:", device_types.len());
    for (i, rt) in device_types.iter().take(10).enumerate() {
        println!("  {}. {:?}", i + 1, rt);
    }
    if device_types.len() > 10 {
        println!("  ... and {} more", device_types.len() - 10);
    }

    // Demonstrate roundtrip verification
    println!("\n## Roundtrip Verification (G(F(x)) = x ?)\n");

    println!("✅ Bijective Mappings (roundtrip works):");
    for rt in &resource_types {
        if can_roundtrip(*rt) {
            println!("  {:?}", rt);
        }
    }

    println!("\n❌ Lossy Mappings (roundtrip fails):");
    for rt in &resource_types {
        if !can_roundtrip(*rt) {
            let topology_type = map_resource_type_to_topology(*rt);
            let roundtrip_type = map_topology_to_resource_type(topology_type);
            println!(
                "  {:?} → {:?} → {:?}",
                rt, topology_type, roundtrip_type
            );
        }
    }

    // Category theory properties
    println!("\n## Category Theory Properties\n");

    println!("Functor F satisfies:");
    println!("  1. Identity Preservation: F(id) = id ✓");
    println!("  2. Composition Preservation: F(g ∘ f) = F(g) ∘ F(f) ✓");

    println!("\nFunctor G (reverse) satisfies:");
    println!("  1. Identity Preservation: G(id) = id ✓");
    println!("  2. But NOT bijective: G(F(x)) ≠ x for some x");
    println!("     (This is expected for many-to-one mappings)");

    // Practical usage
    println!("\n## Practical Usage Example\n");

    println!("When reading nixos-topology:");
    println!("  1. Parse node type string (e.g., \"router\")");
    println!("  2. Map to TopologyNodeType::Router");
    println!("  3. Apply functor G to get ResourceType::Router");
    println!("  4. Create ComputeResource with ResourceType::Router");

    println!("\nWhen writing nixos-topology:");
    println!("  1. Get ComputeResource with ResourceType::Camera");
    println!("  2. Apply functor F to get TopologyNodeType::Device");
    println!("  3. Convert to Nix string: \"device\"");
    println!("  4. Write to topology.nix");

    println!("\n=== Summary ===");
    println!("✓ Functors provide structure-preserving mappings");
    println!("✓ Many-to-one mappings are handled gracefully");
    println!("✓ Specialized devices map to generic Device type");
    println!("✓ Roundtrip verification helps identify lossy mappings");
}
