// Example: Complete Pipeline - Nix File → Infrastructure Domain → Nix File
//
// This example demonstrates the full bidirectional data flow through all 5 phases

use cim_domain_nix::*;
use cim_domain_nix::io::{NixReader, NixWriter};

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== CIM Domain Nix: Complete Pipeline Example ===\n");

    // ========================================================================
    // Phase 1: Create Infrastructure Domain Objects
    // ========================================================================

    println!("Phase 1: Creating Infrastructure Domain...");

    let infrastructure_id = infrastructure::InfrastructureId::new();
    let mut infrastructure = infrastructure::InfrastructureAggregate::new(infrastructure_id);
    let identity = infrastructure::MessageIdentity::new_root();

    // Register a compute resource
    let spec = infrastructure::ComputeResourceSpec {
        id: infrastructure::ResourceId::new("web-server-01")?,
        resource_type: infrastructure::ComputeType::Physical,
        hostname: infrastructure::Hostname::new("web-server-01.example.com")?,
        system: infrastructure::SystemArchitecture::x86_64_linux(),
        system_description: None,
        capabilities: infrastructure::ResourceCapabilities::new(),
    };

    infrastructure.handle_register_compute_resource(spec, &identity)?;
    println!("  ✓ Registered compute resource");

    // Define a network
    let network_spec = infrastructure::NetworkSpec {
        id: infrastructure::NetworkId::new("production-lan")?,
        name: "Production LAN".to_string(),
        cidr_v4: Some(infrastructure::Ipv4Network::new(
            std::net::Ipv4Addr::new(10, 0, 1, 0),
            24,
        )?),
        cidr_v6: None,
    };

    infrastructure.handle_define_network(network_spec, &identity)?;
    println!("  ✓ Defined network");

    let events = infrastructure.take_uncommitted_events();
    println!("  ✓ Generated {} events\n", events.len());

    // ========================================================================
    // Phase 2 & 3: Project Infrastructure to Nix Topology
    // ========================================================================

    println!("Phase 2-3: Projecting to Nix Topology...");

    let functor = functor::NixInfrastructureFunctor::new();
    let topology = functor.project_topology(&infrastructure)?;

    println!("  ✓ Projected to Nix topology");
    println!("  ✓ Nodes: {}", topology.nodes.len());
    println!("  ✓ Networks: {}\n", topology.networks.len());

    // ========================================================================
    // Phase 4: Write to Nix String
    // ========================================================================

    println!("Phase 4: Writing to Nix format...");

    let writer = io::TopologyWriter::new();
    let nix_content = writer.write_string(&topology)?;

    println!("  ✓ Generated Nix file content:");
    println!("  {}", "-".repeat(60));
    for (i, line) in nix_content.lines().enumerate().take(15) {
        println!("  {}", line);
    }
    if nix_content.lines().count() > 15 {
        println!("  ... ({} more lines)", nix_content.lines().count() - 15);
    }
    println!("  {}\n", "-".repeat(60));

    // ========================================================================
    // Phase 5: Read from Nix String (Round-Trip)
    // ========================================================================

    println!("Phase 5: Reading back from Nix format...");

    let reader = io::TopologyReader::new();
    let topology_roundtrip = reader.read_string(&nix_content)?;

    println!("  ✓ Parsed Nix content");
    println!("  ✓ Nodes: {}", topology_roundtrip.nodes.len());
    println!("  ✓ Networks: {}\n", topology_roundtrip.networks.len());

    // ========================================================================
    // Verification: Compare Original and Round-Trip
    // ========================================================================

    println!("Verification: Round-Trip Integrity...");

    // Check counts match
    assert_eq!(topology.nodes.len(), topology_roundtrip.nodes.len());
    assert_eq!(topology.networks.len(), topology_roundtrip.networks.len());

    println!("  ✓ Node count preserved");
    println!("  ✓ Network count preserved");

    // Check names preserved
    for (name, _) in &topology.nodes {
        assert!(topology_roundtrip.nodes.contains_key(name));
    }
    for (name, _) in &topology.networks {
        assert!(topology_roundtrip.networks.contains_key(name));
    }

    println!("  ✓ All names preserved");
    println!("  ✓ Round-trip successful!\n");

    // ========================================================================
    // Phase 3 (Reverse): Map Nix Topology Back to Infrastructure
    // ========================================================================

    println!("Phase 3 (Reverse): Mapping back to Infrastructure...");

    let infrastructure_mapped = functor.map_topology(&topology_roundtrip)?;

    println!("  ✓ Mapped to Infrastructure");
    println!("  ✓ Resources: {}", infrastructure_mapped.resources.len());
    println!("  ✓ Networks: {}\n", infrastructure_mapped.networks.len());

    // ========================================================================
    // Validation
    // ========================================================================

    println!("Validation: Checking topology validity...");

    let validator = io::NixValidator::new();
    let validation_result = validator.validate_topology(&topology_roundtrip);

    if validation_result.is_valid() {
        println!("  ✓ Topology is valid");
    } else {
        println!("  ✗ Topology has errors:");
        for error in &validation_result.errors {
            println!("    - {}", error);
        }
    }

    if !validation_result.warnings.is_empty() {
        println!("  ⚠ Warnings:");
        for warning in &validation_result.warnings {
            println!("    - {}", warning);
        }
    }
    println!();

    // ========================================================================
    // Summary
    // ========================================================================

    println!("=== Pipeline Complete ===");
    println!();
    println!("Data Flow:");
    println!("  1. Infrastructure Domain (Phase 1)");
    println!("     └─> 2 resources, 2 events");
    println!("  2. Functor Projection (Phase 3)");
    println!("     └─> NixTopology object");
    println!("  3. Nix String Writer (Phase 4)");
    println!("     └─> {} bytes of Nix code", nix_content.len());
    println!("  4. Nix String Parser (Phase 5)");
    println!("     └─> NixTopology object (round-trip)");
    println!("  5. Functor Mapping (Phase 3)");
    println!("     └─> Infrastructure Domain (reconstructed)");
    println!();
    println!("✨ All phases working correctly!");

    Ok(())
}
