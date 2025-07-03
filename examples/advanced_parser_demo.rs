//! Demonstrates the advanced Nix parser and AST manipulation capabilities

use cim_domain_nix::parser::{AdvancedParser, AstManipulator, AstBuilder, NixAst};
use cim_domain_nix::parser::ast::{BindingValue, AttrPathSegment, Binding, AttrPath};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Advanced Nix Parser Demo ===\n");

    // Create parser
    let mut parser = AdvancedParser::new();

    // Example 1: Parse and analyze a simple flake
    println!("1. Parsing a simple flake structure:");
    let flake_content = r#"{
  description = "Example flake";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        packages.hello = pkgs.hello;
        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.rustc pkgs.cargo ];
        };
      }
    );
}"#;

    let parsed = parser.parse_string(flake_content)?;
    println!("Successfully parsed flake!");
    println!("AST root: {:?}\n", parsed.ast);

    // Example 2: Build a new attribute set from scratch
    println!("2. Building a new attribute set programmatically:");
    
    let new_ast = AstBuilder::new()
        .attr_set()
        .add_attr("name", NixAst::String("my-package".to_string()))
        .add_attr("version", NixAst::String("1.0.0".to_string()))
        .add_attr("src", NixAst::Path(PathBuf::from("./src")))
        .add_attr("buildInputs", NixAst::List(vec![
            NixAst::Identifier("pkgs.rustc".to_string()),
            NixAst::Identifier("pkgs.cargo".to_string()),
        ]))
        .build();
    
    println!("Built AST: {:#?}\n", new_ast);

    // Example 3: Manipulate existing AST
    println!("3. Manipulating the parsed AST:");
    
    // Add a new output to the flake
    if let NixAst::AttrSet { bindings, .. } = &parsed.ast {
        // Find the outputs binding
        for binding in bindings {
            if let Some(first_segment) = binding.attr_path.segments.first() {
                if let AttrPathSegment::Identifier(name) = first_segment {
                    if name == "outputs" {
                        println!("Found outputs attribute!");
                        // In a real implementation, we would modify the function body
                    }
                }
            }
        }
    }

    // Example 4: Query the AST
    println!("4. Querying the AST:");
    
    let strings = AstManipulator::find_nodes(&parsed.ast, &|node| {
        matches!(node, NixAst::String(_))
    });
    
    println!("Found {strings.len(} string literals in the flake"));
    for (i, s) in strings.iter().enumerate() {
        if let NixAst::String(content) = s {
            println!("  {i + 1}: {content}");
        }
    }

    // Example 5: Transform the AST
    println!("\n5. Transforming the AST:");
    
    let mut transformed = parsed.ast.clone();
    AstManipulator::transform_nodes(&mut transformed, &|node| {
        match node {
            NixAst::String(s) if s == "Example flake" => {
                Some(NixAst::String("Enhanced flake with AST manipulation".to_string()))
            }
            _ => None,
        }
    });
    
    println!("Transformed description successfully!");

    // Example 6: Add a new input to the flake
    println!("\n6. Adding a new input to the flake:");
    
    if let NixAst::AttrSet { bindings, .. } = &mut transformed {
        // Find the inputs attribute
        for binding in bindings.iter_mut() {
            if let Some(first_segment) = binding.attr_path.segments.first() {
                if let AttrPathSegment::Identifier(name) = first_segment {
                    if name == "inputs" {
                        // Get the inputs attrset
                        if let BindingValue::Value(NixAst::AttrSet { bindings: input_bindings, .. }) = &mut binding.value {
                            // Check if we're looking at nixpkgs or flake-utils
                            if let Some(first_input) = input_bindings.first() {
                                if let Some(first_segment) = first_input.attr_path.segments.first() {
                                    if let AttrPathSegment::Identifier(name) = first_segment {
                                        if name == "nixpkgs" || name == "flake-utils" {
                                            // This is the inputs set, add a new input
                                            let rust_overlay_ast = AstBuilder::new()
                                                .attr_set()
                                                .add_attr("url", NixAst::String("github:oxalica/rust-overlay".to_string()))
                                                .build();
                                            
                                            input_bindings.push(Binding {
                                                attr_path: AttrPath {
                                                    segments: vec![AttrPathSegment::Identifier("rust-overlay".to_string())],
                                                },
                                                value: BindingValue::Value(rust_overlay_ast),
                                            });
                                            
                                            println!("Added rust-overlay input!");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Example 7: Pretty print the AST
    println!("\n7. AST Statistics:");
    let stats = analyze_ast(&parsed.ast);
    println!("  Total nodes: {stats.total_nodes}");
    println!("  Node types:");
    for (node_type, count) in stats.node_counts {
        println!("    {node_type}: {count}");
    }

    Ok(())
}

#[derive(Default)]
struct AstStats {
    total_nodes: usize,
    node_counts: std::collections::HashMap<String, usize>,
}

fn analyze_ast(ast: &NixAst) -> AstStats {
    let mut stats = AstStats::default();
    count_nodes(ast, &mut stats.node_counts);
    stats.total_nodes = stats.node_counts.values().sum();
    stats
}

fn count_nodes(ast: &NixAst, counts: &mut std::collections::HashMap<String, usize>) {
    let node_type = match ast {
        NixAst::Integer(_) => "Integer",
        NixAst::Float(_) => "Float",
        NixAst::String(_) => "String",
        NixAst::Path(_) => "Path",
        NixAst::Bool(_) => "Bool",
        NixAst::Null => "Null",
        NixAst::Identifier(_) => "Identifier",
        NixAst::AttrSet { .. } => "AttrSet",
        NixAst::List(_) => "List",
        NixAst::Function { .. } => "Function",
        NixAst::Apply { .. } => "Apply",
        NixAst::Let { .. } => "Let",
        NixAst::If { .. } => "If",
        NixAst::With { .. } => "With",
        NixAst::Assert { .. } => "Assert",
        NixAst::BinaryOp { .. } => "BinaryOp",
        NixAst::UnaryOp { .. } => "UnaryOp",
        NixAst::Select { .. } => "Select",
        NixAst::HasAttr { .. } => "HasAttr",
        NixAst::Import(_) => "Import",
        NixAst::Inherit { .. } => "Inherit",
    };
    
    *counts.entry(node_type.to_string()).or_insert(0) += 1;

    // Recurse into child nodes
    match ast {
        NixAst::AttrSet { bindings, .. } => {
            for binding in bindings {
                match &binding.value {
                    BindingValue::Value(v) => count_nodes(v, counts),
                    BindingValue::Inherit { from: Some(f), .. } => {
                        count_nodes(f, counts);
                    }
                    _ => {}
                }
            }
        }
        NixAst::List(elements) => {
            for elem in elements {
                count_nodes(elem, counts);
            }
        }
        NixAst::Function { body, .. } => count_nodes(body, counts),
        NixAst::Apply { function, argument } => {
            count_nodes(function, counts);
            count_nodes(argument, counts);
        }
        NixAst::Let { bindings, body } => {
            for binding in bindings {
                match &binding.value {
                    BindingValue::Value(v) => count_nodes(v, counts),
                    _ => {}
                }
            }
            count_nodes(body, counts);
        }
        NixAst::If { condition, then_branch, else_branch } => {
            count_nodes(condition, counts);
            count_nodes(then_branch, counts);
            count_nodes(else_branch, counts);
        }
        NixAst::With { namespace, body } => {
            count_nodes(namespace, counts);
            count_nodes(body, counts);
        }
        NixAst::Assert { condition, body } => {
            count_nodes(condition, counts);
            count_nodes(body, counts);
        }
        NixAst::BinaryOp { left, right, .. } => {
            count_nodes(left, counts);
            count_nodes(right, counts);
        }
        NixAst::UnaryOp { operand, .. } => count_nodes(operand, counts),
        NixAst::Select { expr, default, .. } => {
            count_nodes(expr, counts);
            if let Some(d) = default {
                count_nodes(d, counts);
            }
        }
        NixAst::HasAttr { expr, .. } => count_nodes(expr, counts),
        NixAst::Import(path) => count_nodes(path, counts),
        NixAst::Inherit { from, .. } => {
            if let Some(f) = from {
                count_nodes(f, counts);
            }
        }
        _ => {}
    }
} 