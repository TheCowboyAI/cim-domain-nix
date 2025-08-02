//! Example demonstrating the Nix analyzer functionality

use cim_domain_nix::analyzer::{NixAnalyzer, AnalyzerConfig};
use cim_domain_nix::formatter::NixFormatter;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🔍 Nix Analyzer Demo\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let repo_path = args.get(1)
        .map(Path::new)
        .unwrap_or(Path::new("."));
    
    let check_formatting = args.iter().any(|arg| arg == "--check-formatting");

    // Configure the analyzer
    let config = AnalyzerConfig {
        parallel_parsing: true,
        max_files: Some(100),
        follow_symlinks: false,
        ignore_patterns: vec![
            "result".to_string(),
            "result-*".to_string(),
            ".git".to_string(),
        ],
        check_formatting,
        formatter: None, // Auto-detect
    };

    // Create analyzer
    let mut analyzer = NixAnalyzer::with_config(config);

    println!("Analyzing Nix files in: {}", repo_path.display());
    if check_formatting {
        println!("Formatting check enabled");
    }
    println!("This may take a moment...\n");

    // Analyze repository
    let report = analyzer.analyze_repository(repo_path).await?;

    // Print summary
    println!("✅ Analysis Complete!\n");

    println!("📊 Summary:");
    println!("  - Files analyzed: {}", report.files_analyzed);
    println!("  - Analysis time: {:.2?}", report.duration);

    // Dependency information
    println!("\n🔗 Dependency Graph:");
    println!("  - Total files: {}", report.dependency_graph.node_count());
    println!("  - Dependencies: {}", report.dependency_graph.edge_count());
    
    // Calculate max depth
    let depths = petgraph::algo::dijkstra(
        &report.dependency_graph,
        petgraph::graph::NodeIndex::new(0),
        None,
        |_| 1,
    );
    let max_depth = depths.values().max().unwrap_or(&0);
    println!("  - Max dependency depth: {}", max_depth);

    // Security issues
    if report.security_issues.is_empty() {
        println!("\n🔒 No security issues found! ✨");
    } else {
        use cim_domain_nix::analyzer::Severity;
        
        let critical = report.security_issues.iter().filter(|i| i.severity == Severity::Critical).count();
        let high = report.security_issues.iter().filter(|i| i.severity == Severity::High).count();
        let medium = report.security_issues.iter().filter(|i| i.severity == Severity::Medium).count();
        let low = report.security_issues.iter().filter(|i| i.severity == Severity::Low).count();

        println!("\n🔒 Security Issues Found: {}", report.security_issues.len());
        if critical > 0 { println!("  - 🚨 Critical: {}", critical); }
        if high > 0 { println!("  - ❗ High: {}", high); }
        if medium > 0 { println!("  - ⚠️  Medium: {}", medium); }
        if low > 0 { println!("  - ℹ️  Low: {}", low); }

        println!("\n  Top issues:");
        for (i, issue) in report.security_issues.iter().take(5).enumerate() {
            println!("  {}. [{:?}] {}", i + 1, issue.severity, issue.description);
            if let Some(file) = &issue.file {
                println!("     File: {}", file);
            }
            if let Some(suggestion) = &issue.suggestion {
                println!("     💡 {}", suggestion);
            }
        }
        if report.security_issues.len() > 5 {
            println!("  ... and {} more", report.security_issues.len() - 5);
        }
    }

    // Performance issues
    if report.performance_issues.is_empty() {
        println!("\n⚡ No performance issues found! 🚀");
    } else {
        use cim_domain_nix::analyzer::performance::ImpactLevel;
        
        let high = report.performance_issues.iter().filter(|i| i.impact == ImpactLevel::High).count();
        let medium = report.performance_issues.iter().filter(|i| i.impact == ImpactLevel::Medium).count();
        let low = report.performance_issues.iter().filter(|i| i.impact == ImpactLevel::Low).count();

        println!("\n⚡ Performance Issues Found: {}", report.performance_issues.len());
        if high > 0 { println!("  - 🔴 High impact: {}", high); }
        if medium > 0 { println!("  - 🟡 Medium impact: {}", medium); }
        if low > 0 { println!("  - 🟢 Low impact: {}", low); }

        println!("\n  Top issues:");
        for (i, issue) in report.performance_issues.iter().take(3).enumerate() {
            println!("  {}. [{:?}] {}", i + 1, issue.impact, issue.description);
            if let Some(file) = &issue.file {
                println!("     File: {}", file);
            }
            if let Some(cost) = &issue.cost_estimate {
                println!("     Cost: {}", cost);
            }
            if let Some(suggestion) = &issue.suggestion {
                println!("     💡 {}", suggestion);
            }
        }
        if report.performance_issues.len() > 3 {
            println!("  ... and {} more", report.performance_issues.len() - 3);
        }
    }

    // Dead code
    if report.dead_code.is_empty() {
        println!("\n🧹 No dead code found! 💯");
    } else {
        use std::collections::HashMap;
        let mut by_type = HashMap::new();
        for dead in &report.dead_code {
            *by_type.entry(format!("{:?}", dead.code_type)).or_insert(0) += 1;
        }

        println!("\n🧹 Dead Code Found: {}", report.dead_code.len());
        for (code_type, count) in by_type {
            println!("  - {}: {}", code_type, count);
        }

        println!("\n  Examples:");
        for (i, dead) in report.dead_code.iter().take(3).enumerate() {
            println!("  {}. {:?}: {}", i + 1, dead.code_type, dead.name);
            if let Some(file) = &dead.file {
                println!("     File: {}", file);
            }
        }
        if report.dead_code.len() > 3 {
            println!("  ... and {} more", report.dead_code.len() - 3);
        }
    }

    // Formatting issues
    if let Some(formatting_issues) = &report.formatting_issues {
        if formatting_issues.is_empty() {
            println!("\n✨ All files are properly formatted!");
        } else {
            println!("\n📐 Formatting Issues Found: {}", formatting_issues.len());
            println!("  Files that need formatting:");
            for (i, file) in formatting_issues.iter().take(5).enumerate() {
                println!("  {}. {}", i + 1, file);
            }
            if formatting_issues.len() > 5 {
                println!("  ... and {} more", formatting_issues.len() - 5);
            }
            println!("\n  💡 Run a Nix formatter to fix these issues");
        }
    }

    // Save report if requested
    if std::env::var("SAVE_REPORT").is_ok() {
        let report_json = serde_json::to_string_pretty(&report)?;
        std::fs::write("nix-analysis-report.json", report_json)?;
        println!("\n📄 Full report saved to: nix-analysis-report.json");
    }

    Ok(())
} 