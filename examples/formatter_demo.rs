//! Example demonstrating the Nix formatter functionality

use cim_domain_nix::formatter::{NixFormatter, FormatterService};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🎨 Nix Formatter Demo\n");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1)
        .map(Path::new)
        .unwrap_or(Path::new("."));
    
    let check_only = args.iter().any(|arg| arg == "--check");
    let formatter_name = args.iter()
        .find_map(|arg| {
            if arg.starts_with("--formatter=") {
                arg.strip_prefix("--formatter=")
            } else {
                None
            }
        });

    // Detect or select formatter
    let formatter = if let Some(name) = formatter_name {
        match name {
            "nixpkgs-fmt" => Some(NixFormatter::NixpkgsFmt),
            "alejandra" => Some(NixFormatter::Alejandra),
            "nixfmt" => Some(NixFormatter::NixFmt),
            "nixfmt-rfc" => Some(NixFormatter::NixFmtRfc),
            _ => {
                eprintln!("Unknown formatter: {name}");
                eprintln!("Available formatters: nixpkgs-fmt, alejandra, nixfmt, nixfmt-rfc");
                return Ok(());
            }
        }
    } else {
        NixFormatter::detect_from_project(path).await
    };

    let Some(formatter) = formatter else {
        eprintln!("❌ No Nix formatter found!");
        eprintln!("\nPlease install one of the following:");
        eprintln!("  - nixpkgs-fmt: nix-env -iA nixpkgs.nixpkgs-fmt");
        eprintln!("  - alejandra: nix-env -iA nixpkgs.alejandra");
        eprintln!("  - nixfmt: nix-env -iA nixpkgs.nixfmt");
        return Ok(());
    };

    println!("Using formatter: {:?}", formatter.command());
    println!("Target: {}", path.display());
    if check_only {
        println!("Mode: Check only (no files will be modified)");
    } else {
        println!("Mode: Format files");
    }
    println!();

    // Create formatter service
    let service = if check_only {
        FormatterService::check_only(formatter)
    } else {
        FormatterService::new(formatter)
    };

    // Format based on path type
    let report = if path.is_file() {
        // Format single file
        let result = service.format_file(path).await?;
        let mut report = cim_domain_nix::formatter::FormattingReport::default();
        report.total_files = 1;
        
        if result.formatted {
            report.formatted_files.push(path.display().to_string());
        } else if result.needs_formatting {
            report.needs_formatting.push(path.display().to_string());
        }
        
        if let Some(error) = result.error {
            report.errors.push((path.display().to_string(), error));
        }
        
        report
    } else {
        // Format directory
        service.format_directory(path).await?
    };

    // Print report
    println!("{report.summary(}"));
    
    if !report.formatted_files.is_empty() {
        println!("\n✅ Formatted files:");
        for file in &report.formatted_files {
            println!("  - {file}");
        }
    }
    
    if !report.needs_formatting.is_empty() {
        println!("\n⚠️  Files need formatting:");
        for file in &report.needs_formatting {
            println!("  - {file}");
        }
    }
    
    if !report.errors.is_empty() {
        println!("\n❌ Errors:");
        for (file, error) in &report.errors {
            println!("  - {file}: {error}");
        }
    }

    // Exit with appropriate code
    if check_only && !report.all_formatted() {
        std::process::exit(1);
    }

    Ok(())
} 