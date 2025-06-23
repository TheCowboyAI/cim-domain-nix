# Nix Parser Integration for CIM Domain Nix

## Overview

This document outlines how we can integrate Rust Nix parsing libraries to enhance the Nix domain with proper AST manipulation capabilities instead of string manipulation.

## Current Approach vs. Enhanced Approach

### Current Approach (String Manipulation)
```rust
// Current implementation in handlers/mod.rs
let new_content = if content.contains("inputs = {") {
    content.replace(
        "inputs = {",
        &format!("inputs = {{\n    {} = \"{}\";", cmd.name, cmd.url)
    )
} else {
    // Create inputs section
    content.replace(
        '{',
        &format!("{{\n  inputs = {{\n    {} = \"{}\";\n  }};\n", cmd.name, cmd.url)
    )
};
```

### Enhanced Approach (AST Manipulation)
```rust
use rnix::{SyntaxNode, SyntaxKind};
use rnix_parser::{parse, AST};

// Parse the flake.nix file
let ast = parse(&content);

// Find or create the inputs attribute set
let inputs = ast.find_attr_set("inputs")
    .unwrap_or_else(|| ast.create_attr_set("inputs"));

// Add new input as AST node
inputs.add_attribute(&cmd.name, &format!("\"{}\"", cmd.url));

// Regenerate the file with formatting preserved
let new_content = ast.to_string();
```

## Proposed Integration

### 1. Add Dependencies

```toml
# cim-domain-nix/Cargo.toml
[dependencies]
# ... existing dependencies ...

# Nix parsing and manipulation
rnix = "0.11"
rowan = "0.15"  # AST library used by rnix

# Optional: For specific use cases
nix-config-parser = "0.2"  # For nix.conf files
```

### 2. Create Parser Service

```rust
// cim-domain-nix/src/services/parser.rs
use rnix::{SyntaxNode, SyntaxKind, types::*};
use std::path::Path;
use crate::{Result, NixDomainError};

pub struct NixParser;

impl NixParser {
    /// Parse a Nix file and return its AST
    pub fn parse_file(path: &Path) -> Result<SyntaxNode> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_string(&content)
    }

    /// Parse a Nix string and return its AST
    pub fn parse_string(content: &str) -> Result<SyntaxNode> {
        let parse_result = rnix::parse(content);
        
        if !parse_result.errors().is_empty() {
            let errors: Vec<_> = parse_result.errors()
                .iter()
                .map(|e| format!("{:?}", e))
                .collect();
            return Err(NixDomainError::ParseError(errors.join(", ")));
        }
        
        Ok(parse_result.node())
    }

    /// Find an attribute in an attribute set
    pub fn find_attribute(node: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
        // Implementation using rnix traversal
        todo!()
    }

    /// Add an attribute to an attribute set
    pub fn add_attribute(
        node: &mut SyntaxNode, 
        name: &str, 
        value: &str
    ) -> Result<()> {
        // Implementation using rnix manipulation
        todo!()
    }
}
```

### 3. Create Flake Manipulator

```rust
// cim-domain-nix/src/services/flake_manipulator.rs
use rnix::SyntaxNode;
use crate::{Result, NixDomainError, value_objects::FlakeRef};

pub struct FlakeManipulator {
    ast: SyntaxNode,
}

impl FlakeManipulator {
    pub fn from_file(path: &Path) -> Result<Self> {
        let ast = NixParser::parse_file(path)?;
        Ok(Self { ast })
    }

    pub fn add_input(&mut self, name: &str, flake_ref: &FlakeRef) -> Result<()> {
        // Find or create inputs attrset
        let inputs = self.find_or_create_inputs()?;
        
        // Add the new input
        self.add_flake_input(inputs, name, flake_ref)?;
        
        Ok(())
    }

    pub fn update_description(&mut self, description: &str) -> Result<()> {
        // Find and update the description attribute
        todo!()
    }

    pub fn add_output(&mut self, name: &str, output_expr: &str) -> Result<()> {
        // Add to outputs function
        todo!()
    }

    pub fn to_string(&self) -> String {
        self.ast.to_string()
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.to_string())?;
        Ok(())
    }
}
```

### 4. Update Command Handlers

```rust
// Updated FlakeCommandHandler::add_flake_input
async fn add_flake_input(&self, cmd: AddFlakeInput) -> Result<Vec<Box<dyn NixDomainEvent>>> {
    let flake_path = cmd.path.join("flake.nix");
    
    // Use the new parser-based approach
    let mut manipulator = FlakeManipulator::from_file(&flake_path)?;
    manipulator.add_input(&cmd.name, &FlakeRef::new(&cmd.url))?;
    manipulator.write_to_file(&flake_path)?;

    let event = FlakeInputAdded {
        flake_id: Uuid::new_v4(),
        path: cmd.path.clone(),
        input_name: cmd.name,
        input_url: cmd.url,
        timestamp: Utc::now(),
    };

    Ok(vec![Box::new(event)])
}
```

### 5. Add Query Support

```rust
// cim-domain-nix/src/queries/flake_analyzer.rs
use rnix::SyntaxNode;
use crate::{Result, value_objects::*};

pub struct FlakeAnalyzer;

impl FlakeAnalyzer {
    /// Extract all inputs from a flake
    pub fn extract_inputs(ast: &SyntaxNode) -> Result<FlakeInputs> {
        // Parse inputs from AST
        todo!()
    }

    /// Extract all outputs from a flake
    pub fn extract_outputs(ast: &SyntaxNode) -> Result<FlakeOutputs> {
        // Parse outputs from AST
        todo!()
    }

    /// Validate flake structure
    pub fn validate_flake(ast: &SyntaxNode) -> Result<Vec<ValidationIssue>> {
        // Check for required attributes, proper structure, etc.
        todo!()
    }
}
```

## Benefits

1. **Correctness**: No more regex or string manipulation errors
2. **Preservation**: Comments and formatting are preserved
3. **Validation**: Can validate Nix syntax before writing
4. **Rich Queries**: Can analyze flake structure, dependencies, etc.
5. **Refactoring**: Can implement safe refactoring operations
6. **Type Safety**: AST nodes are strongly typed

## Implementation Plan

1. **Phase 1**: Add `rnix` dependency and create basic parser service
2. **Phase 2**: Implement flake manipulation for `add_flake_input` command
3. **Phase 3**: Extend to other modification commands
4. **Phase 4**: Add analysis and query capabilities
5. **Phase 5**: Add validation and linting features

## Example Use Cases

### 1. Safe Flake Input Addition
```rust
// Instead of string manipulation, use AST
let mut flake = FlakeManipulator::from_file("flake.nix")?;
flake.add_input("nixpkgs", &FlakeRef::new("github:NixOS/nixpkgs/nixos-23.11"))?;
flake.write_to_file("flake.nix")?;
```

### 2. Flake Analysis
```rust
let ast = NixParser::parse_file("flake.nix")?;
let inputs = FlakeAnalyzer::extract_inputs(&ast)?;
println!("Flake has {} inputs", inputs.inputs.len());
```

### 3. Flake Validation
```rust
let ast = NixParser::parse_file("flake.nix")?;
let issues = FlakeAnalyzer::validate_flake(&ast)?;
if !issues.is_empty() {
    println!("Flake validation issues found:");
    for issue in issues {
        println!("  - {}", issue);
    }
}
```

## Conclusion

Integrating proper Nix parsing libraries will significantly improve the robustness and capabilities of the Nix domain. It moves us from fragile string manipulation to proper AST-based operations, enabling more sophisticated features while maintaining correctness. 