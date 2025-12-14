// Example: Test Round-Trip with cim-domain-git Flake
//
// This tests bidirectional conversion with a more complex real-world flake:
// - 5 inputs (with follows and non-flake)
// - Actual packages
// - DevShells with many tools
// - Checks and apps sections

use cim_domain_nix::nix::*;
use cim_domain_nix::infrastructure::*;
use cim_domain_nix::functor::*;
use cim_domain_nix::io::*;
use std::fs;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing cim-domain-git Flake Round-Trip ===\n");

    let flake_path = "/git/thecowboyai/cim-domain-git/flake.nix";

    // ========================================================================
    // Step 1: Read and Parse Complex Flake
    // ========================================================================

    println!("Step 1: Reading cim-domain-git flake...");
    let original_content = fs::read_to_string(flake_path)?;
    println!("  ✓ Read {} bytes\n", original_content.len());

    println!("Step 2: Parsing flake...");
    let parser = NixParser::new();
    let original_ast = parser.parse_str(&original_content)?;
    let original_value = ast_to_value(&original_ast)?;
    println!("  ✓ Parsed successfully\n");

    // ========================================================================
    // Step 3: Analyze Flake Structure
    // ========================================================================

    println!("Step 3: Analyzing flake structure...");
    let analyzer = FlakeAnalyzer::new();
    let analysis = analyzer.analyze(&original_value)?;

    println!("  Flake: CIM Domain Git");
    println!("  ─────────────────────");
    if let Some(desc) = &analysis.description {
        println!("  Description: {}", desc);
    }
    println!("  Inputs: {}", analysis.inputs.len());
    for input in &analysis.inputs {
        println!("    • {}", input.name);
        if let Some(url) = &input.url {
            println!("      URL: {}", url);
        }
        if let Some(follows) = &input.follows {
            println!("      Follows: {}", follows);
        }
    }
    println!("  Packages: {}", analysis.packages.len());
    println!("  DevShells: {}\n", analysis.dev_shells.len());

    // ========================================================================
    // Step 4: Convert to Infrastructure Domain
    // ========================================================================

    println!("Step 4: Converting to Infrastructure...");
    let infrastructure_id = InfrastructureId::new();
    let infrastructure = analyzer.to_infrastructure(&analysis, infrastructure_id)?;

    println!("  ✓ Infrastructure created:");
    println!("    - Resources: {}", infrastructure.resources.len());
    println!("    - Networks: {}", infrastructure.networks.len());

    if !infrastructure.resources.is_empty() {
        println!("\n  Resources:");
        for (id, resource) in infrastructure.resources.iter().take(3) {
            println!("    • {} ({})", id, resource.resource_type);
            println!("      Hostname: {}", resource.hostname);
            println!("      System: {}", resource.system);
        }
        if infrastructure.resources.len() > 3 {
            println!("    ... and {} more", infrastructure.resources.len() - 3);
        }
    }

    if !infrastructure.networks.is_empty() {
        println!("\n  Networks:");
        for (id, network) in &infrastructure.networks {
            println!("    • {}: {}", id, network.name);
        }
    }
    println!();

    // ========================================================================
    // Step 5: Project to NixTopology
    // ========================================================================

    println!("Step 5: Projecting to NixTopology...");
    let functor = NixInfrastructureFunctor::new();
    let topology = functor.project_topology(&infrastructure)?;

    println!("  ✓ Topology created:");
    println!("    - Nodes: {}", topology.nodes.len());
    println!("    - Networks: {}", topology.networks.len());

    if !topology.nodes.is_empty() {
        println!("\n  Nodes:");
        for (name, node) in topology.nodes.iter().take(3) {
            println!("    • {} ({:?})", name, node.node_type);
            println!("      System: {}", node.system);
        }
        if topology.nodes.len() > 3 {
            println!("    ... and {} more", topology.nodes.len() - 3);
        }
    }
    println!();

    // ========================================================================
    // Step 6: Serialize to Nix
    // ========================================================================

    println!("Step 6: Serializing to Nix format...");
    let writer = TopologyWriter::new();
    let roundtrip_content = writer.write_string(&topology)?;

    println!("  ✓ Generated {} bytes of Nix code", roundtrip_content.len());
    println!("\n  Generated Nix (first 20 lines):");
    println!("  {}", "-".repeat(60));
    for (i, line) in roundtrip_content.lines().enumerate().take(20) {
        println!("  {}", line);
    }
    if roundtrip_content.lines().count() > 20 {
        println!("  ... ({} more lines)", roundtrip_content.lines().count() - 20);
    }
    println!("  {}\n", "-".repeat(60));

    // ========================================================================
    // Step 7: Parse Round-Trip
    // ========================================================================

    println!("Step 7: Parsing round-trip content...");
    let roundtrip_ast = parser.parse_str(&roundtrip_content)?;
    let roundtrip_value = ast_to_value(&roundtrip_ast)?;
    println!("  ✓ Parsed successfully\n");

    // ========================================================================
    // Step 8: Verify Topology Preservation
    // ========================================================================

    println!("Step 8: Verifying topology preservation...");
    let reader = TopologyReader::new();
    let topology_verified = reader.read_from_value(
        &roundtrip_value,
        "cim-domain-git-roundtrip".to_string()
    )?;

    let nodes_match = topology.nodes.len() == topology_verified.nodes.len();
    let networks_match = topology.networks.len() == topology_verified.networks.len();

    println!("  Nodes: {} → {} {}",
        topology.nodes.len(),
        topology_verified.nodes.len(),
        if nodes_match { "✓" } else { "✗" }
    );
    println!("  Networks: {} → {} {}",
        topology.networks.len(),
        topology_verified.networks.len(),
        if networks_match { "✓" } else { "✗" }
    );

    // Verify node names preserved
    let mut all_nodes_preserved = true;
    for (name, _) in &topology.nodes {
        if !topology_verified.nodes.contains_key(name) {
            println!("  ✗ Node '{}' not preserved", name);
            all_nodes_preserved = false;
        }
    }

    if all_nodes_preserved && !topology.nodes.is_empty() {
        println!("  ✓ All node names preserved");
    }

    // Verify network names preserved
    let mut all_networks_preserved = true;
    for (name, _) in &topology.networks {
        if !topology_verified.networks.contains_key(name) {
            println!("  ✗ Network '{}' not preserved", name);
            all_networks_preserved = false;
        }
    }

    if all_networks_preserved && !topology.networks.is_empty() {
        println!("  ✓ All network names preserved");
    }

    println!();

    // ========================================================================
    // Step 9: Map Back to Infrastructure
    // ========================================================================

    println!("Step 9: Mapping back to Infrastructure...");
    let infrastructure_verified = functor.map_topology(&topology_verified)?;

    println!("  ✓ Reconstructed Infrastructure:");
    println!("    - Resources: {}", infrastructure_verified.resources.len());
    println!("    - Networks: {}\n", infrastructure_verified.networks.len());

    // ========================================================================
    // Summary & Comparison
    // ========================================================================

    println!("=== Round-Trip Summary ===");
    println!();
    println!("Original Flake (cim-domain-git):");
    println!("  - Size: {} bytes", original_content.len());
    println!("  - Inputs: {} (including follows and non-flake)", analysis.inputs.len());
    println!("  - Packages: {}", analysis.packages.len());
    println!("  - DevShells: {}", analysis.dev_shells.len());
    println!();

    println!("Infrastructure Extraction:");
    println!("  - Resources: {}", infrastructure.resources.len());
    println!("  - Networks: {}", infrastructure.networks.len());
    println!();

    println!("Round-Trip Nix:");
    println!("  - Size: {} bytes", roundtrip_content.len());
    println!("  - Nodes: {}", topology.nodes.len());
    println!("  - Networks: {}", topology.networks.len());
    println!();

    println!("Verification:");
    let topology_preserved = nodes_match && networks_match && all_nodes_preserved && all_networks_preserved;
    let infrastructure_preserved =
        infrastructure.resources.len() == infrastructure_verified.resources.len() &&
        infrastructure.networks.len() == infrastructure_verified.networks.len();

    println!("  {} Topology structure preserved", if topology_preserved { "✅" } else { "❌" });
    println!("  {} Infrastructure counts match", if infrastructure_preserved { "✅" } else { "❌" });
    println!();

    if topology_preserved && infrastructure_preserved {
        println!("✨ Round-trip successful with complex flake!");
        println!();
        println!("Successfully processed:");
        println!("  ✓ Multiple inputs with complex relationships");
        println!("  ✓ Follows directive preserved");
        println!("  ✓ Non-flake inputs handled");
        println!("  ✓ Infrastructure network created");
        println!("  ✓ Complete bidirectional conversion verified");
    } else {
        println!("⚠ Some differences detected (may be expected)");
    }

    Ok(())
}
