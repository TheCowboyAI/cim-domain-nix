// Example: Evaluate cim-domain-git Flake with Full Outputs
//
// This demonstrates using Nix evaluation to extract packages, devShells,
// checks, and apps from the outputs function.

use cim_domain_nix::nix::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Evaluating cim-domain-git Flake ===\n");

    let flake_path = "/git/thecowboyai/cim-domain-git";

    // ========================================================================
    // Step 1: Check if Nix is Available
    // ========================================================================

    println!("Step 1: Checking Nix availability...");
    if !nix_available() {
        println!("  ✗ Nix command not available");
        println!("\n  To use flake evaluation, install Nix:");
        println!("    https://nixos.org/download.html");
        println!("\n  Continuing with static analysis only...\n");
        return static_analysis_only(flake_path);
    }
    println!("  ✓ Nix is available\n");

    // ========================================================================
    // Step 2: Static Analysis
    // ========================================================================

    println!("Step 2: Performing static analysis...");
    let content = std::fs::read_to_string(format!("{}/flake.nix", flake_path))?;
    let parser = NixParser::new();
    let ast = parser.parse_str(&content)?;
    let value = ast_to_value(&ast)?;

    let analyzer = FlakeAnalyzer::new();
    let static_analysis = analyzer.analyze(&value)?;

    println!("  Static Analysis Results:");
    println!("    - Description: {}", static_analysis.description.as_deref().unwrap_or("None"));
    println!("    - Inputs: {}", static_analysis.inputs.len());
    println!("    - Packages: {} (from static analysis)", static_analysis.packages.len());
    println!("    - DevShells: {} (from static analysis)\n", static_analysis.dev_shells.len());

    // ========================================================================
    // Step 3: Evaluate with Nix
    // ========================================================================

    println!("Step 3: Evaluating flake outputs with Nix...");
    println!("  (This may take a moment...)");

    let evaluator = FlakeEvaluator::new();
    let evaluated = match evaluator.evaluate(flake_path) {
        Ok(e) => e,
        Err(e) => {
            println!("  ✗ Evaluation failed: {}", e);
            println!("\n  Falling back to static analysis...\n");
            return static_analysis_only(flake_path);
        }
    };

    println!("  ✓ Evaluation complete!\n");

    // ========================================================================
    // Step 4: Display Evaluated Results
    // ========================================================================

    println!("Step 4: Analyzing evaluated outputs...");
    println!();

    // Count total items across all systems
    let total_packages: usize = evaluated.packages.values()
        .map(|m| m.len())
        .sum();
    let total_devshells: usize = evaluated.dev_shells.values()
        .map(|m| m.len())
        .sum();
    let total_checks: usize = evaluated.checks.values()
        .map(|m| m.len())
        .sum();
    let total_apps: usize = evaluated.apps.values()
        .map(|m| m.len())
        .sum();

    println!("  Evaluated Flake Summary:");
    println!("  ─────────────────────────");
    if let Some(desc) = &evaluated.description {
        println!("  Description: {}", desc);
    }
    println!("  Systems: {}", evaluated.packages.keys().len());
    println!("  Total Packages: {}", total_packages);
    println!("  Total DevShells: {}", total_devshells);
    println!("  Total Checks: {}", total_checks);
    println!("  Total Apps: {}", total_apps);
    println!();

    // ========================================================================
    // Step 5: Display Per-System Details
    // ========================================================================

    for system in evaluated.packages.keys() {
        println!("  System: {}", system);
        println!("  {}", "-".repeat(60));

        // Packages
        if let Some(packages) = evaluated.packages.get(system) {
            if !packages.is_empty() {
                println!("    Packages ({}):", packages.len());
                for (name, pkg) in packages {
                    println!("      • {} ({})", name, pkg.pkg_type);
                    if let Some(desc) = &pkg.description {
                        println!("        {}", desc);
                    }
                }
                println!();
            }
        }

        // DevShells
        if let Some(shells) = evaluated.dev_shells.get(system) {
            if !shells.is_empty() {
                println!("    DevShells ({}):", shells.len());
                for (name, shell) in shells {
                    println!("      • {} ({})", name, shell.shell_type);
                    if let Some(desc) = &shell.description {
                        println!("        {}", desc);
                    }
                }
                println!();
            }
        }

        // Checks
        if let Some(checks) = evaluated.checks.get(system) {
            if !checks.is_empty() {
                println!("    Checks ({}):", checks.len());
                for (name, check) in checks.iter().take(5) {
                    println!("      • {} ({})", name, check.check_type);
                }
                if checks.len() > 5 {
                    println!("      ... and {} more", checks.len() - 5);
                }
                println!();
            }
        }

        // Apps
        if let Some(apps) = evaluated.apps.get(system) {
            if !apps.is_empty() {
                println!("    Apps ({}):", apps.len());
                for (name, app) in apps {
                    println!("      • {} ({})", name, app.app_type);
                }
                println!();
            }
        }
    }

    // ========================================================================
    // Step 6: Compare Static vs Evaluated
    // ========================================================================

    println!("=== Comparison: Static vs Evaluated ===");
    println!();
    println!("  Static Analysis (AST parsing only):");
    println!("    - Inputs: {} ✓ (can extract)", static_analysis.inputs.len());
    println!("    - Packages: {} (inside function)", static_analysis.packages.len());
    println!("    - DevShells: {} (inside function)", static_analysis.dev_shells.len());
    println!();
    println!("  Evaluated Analysis (Nix evaluation):");
    println!("    - Packages: {} ✓ (extracted!)", total_packages);
    println!("    - DevShells: {} ✓ (extracted!)", total_devshells);
    println!("    - Checks: {} ✓ (extracted!)", total_checks);
    println!("    - Apps: {} ✓ (extracted!)", total_apps);
    println!();

    println!("✨ With evaluation, we get the COMPLETE flake structure!");
    println!();
    println!("This includes everything inside the outputs function:");
    println!("  ✓ Package definitions");
    println!("  ✓ Development shell configurations");
    println!("  ✓ CI/CD checks");
    println!("  ✓ Executable applications");

    Ok(())
}

fn static_analysis_only(flake_path: &str) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(format!("{}/flake.nix", flake_path))?;
    let parser = NixParser::new();
    let ast = parser.parse_str(&content)?;
    let value = ast_to_value(&ast)?;

    let analyzer = FlakeAnalyzer::new();
    let analysis = analyzer.analyze(&value)?;

    println!("Static Analysis (without Nix evaluation):");
    println!("  - Description: {}", analysis.description.as_deref().unwrap_or("None"));
    println!("  - Inputs: {}", analysis.inputs.len());
    for input in &analysis.inputs {
        println!("    • {}", input.name);
        if let Some(url) = &input.url {
            println!("      URL: {}", url);
        }
    }
    println!();
    println!("Note: Install Nix to get packages, devShells, checks, and apps");

    Ok(())
}
