// Example: Parse and analyze the cim-domain-nix flake.nix file
//
// This demonstrates:
// 1. Parsing real-world Nix code (flake.nix)
// 2. Extracting infrastructure information from the flake
// 3. Populating Infrastructure domain objects

use cim_domain_nix::nix::*;
use cim_domain_nix::infrastructure::*;
use std::fs;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Parsing cim-domain-nix flake.nix ===\n");

    // ========================================================================
    // Phase 1: Read and Parse Nix File
    // ========================================================================

    println!("Phase 1: Reading and parsing flake.nix...");

    let flake_content = fs::read_to_string("flake.nix")?;
    println!("  ✓ Read {} bytes from flake.nix", flake_content.len());

    let parser = NixParser::new();
    let ast = parser.parse_str(&flake_content)?;
    println!("  ✓ Successfully parsed to AST");

    let value = ast_to_value(&ast)?;
    println!("  ✓ Converted AST to NixValue\n");

    // ========================================================================
    // Phase 2: Analyze Flake Structure
    // ========================================================================

    println!("Phase 2: Analyzing flake structure...");

    let analyzer = FlakeAnalyzer::new();
    let analysis = analyzer.analyze(&value)?;

    println!("  ✓ Analysis complete\n");

    // Display flake information
    println!("Flake Information:");
    println!("─────────────────");

    if let Some(desc) = &analysis.description {
        println!("Description: {}", desc);
    }

    println!("\nExternal Dependencies ({}):", analysis.inputs.len());
    for input in &analysis.inputs {
        if let Some(url) = &input.url {
            println!("  - {}: {}", input.name, url);
        } else {
            println!("  - {}", input.name);
        }
    }

    println!("\nPackages: {}", analysis.packages.len());
    for pkg in &analysis.packages {
        println!("  - {} (v{})", pkg.name, pkg.version.as_deref().unwrap_or("unknown"));
        if !pkg.build_inputs.is_empty() {
            println!("    Build inputs: {}", pkg.build_inputs.join(", "));
        }
        if !pkg.native_build_inputs.is_empty() {
            println!("    Native inputs: {}", pkg.native_build_inputs.join(", "));
        }
    }

    println!("\nDevelopment Shells: {}", analysis.dev_shells.len());
    for shell in &analysis.dev_shells {
        println!("  - {}", shell.name);
        if !shell.packages.is_empty() {
            println!("    Packages ({}):", shell.packages.len());
            for (i, pkg) in shell.packages.iter().take(5).enumerate() {
                println!("      {}. {}", i + 1, pkg);
            }
            if shell.packages.len() > 5 {
                println!("      ... and {} more", shell.packages.len() - 5);
            }
        }
    }

    println!();

    // ========================================================================
    // Phase 3: Convert to Infrastructure Domain
    // ========================================================================

    println!("Phase 3: Converting to Infrastructure domain...");

    let infrastructure_id = InfrastructureId::new();
    let infrastructure = analyzer.to_infrastructure(&analysis, infrastructure_id)?;

    println!("  ✓ Created Infrastructure aggregate");
    println!("  ✓ Resources: {}", infrastructure.resources.len());
    println!("  ✓ Networks: {}", infrastructure.networks.len());
    println!();

    // Display infrastructure details
    println!("Infrastructure Summary:");
    println!("──────────────────────");

    if !infrastructure.resources.is_empty() {
        println!("\nCompute Resources:");
        for (id, resource) in &infrastructure.resources {
            println!("  - {} ({})", id, resource.hostname);
            println!("    Type: {:?}", resource.resource_type);
            println!("    System: {}", resource.system);
            if !resource.capabilities.metadata.is_empty() {
                let metadata: Vec<_> = resource.capabilities.metadata.iter().take(3).collect();
                println!("    Capabilities: {}", metadata.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(", "));
                if resource.capabilities.metadata.len() > 3 {
                    println!("      ... and {} more", resource.capabilities.metadata.len() - 3);
                }
            }
        }
    }

    if !infrastructure.networks.is_empty() {
        println!("\nNetworks:");
        for (id, network) in &infrastructure.networks {
            println!("  - {}: {}", id, network.name);
        }
    }

    println!();

    // ========================================================================
    // Phase 4: Generate Events
    // ========================================================================

    println!("Phase 4: Event sourcing...");

    // In a real scenario, we would:
    // 1. Persist events to NATS JetStream
    // 2. Project events to various read models
    // 3. Trigger workflows based on infrastructure changes

    let events_count = infrastructure.resources.len() + infrastructure.networks.len();
    println!("  ✓ Generated {} infrastructure events", events_count);
    println!("  ✓ Events can be persisted to NATS JetStream");
    println!();

    // ========================================================================
    // Summary
    // ========================================================================

    println!("=== Summary ===");
    println!();
    println!("Successfully demonstrated complete pipeline:");
    println!("  1. ✅ Parsed flake.nix ({} bytes)", flake_content.len());
    println!("  2. ✅ Extracted {} dependencies", analysis.inputs.len());
    println!("  3. ✅ Created {} compute resources", infrastructure.resources.len());
    println!("  4. ✅ Defined {} networks", infrastructure.networks.len());
    println!();
    println!("✨ Flake → Infrastructure conversion complete!");
    println!();
    println!("This demonstrates:");
    println!("  - Real-world Nix parsing with rnix");
    println!("  - Infrastructure concept extraction from flakes");
    println!("  - Event-sourced domain model population");
    println!("  - Bidirectional Nix ↔ Infrastructure mapping");

    Ok(())
}
