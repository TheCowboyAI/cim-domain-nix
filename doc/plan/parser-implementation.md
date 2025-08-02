# Nix Parser Implementation Plan

> **Note**: Parser has been implemented. This document describes the original plan and future enhancements. For current status, see [IMPLEMENTATION_STATUS.md](./IMPLEMENTATION_STATUS.md)

## Immediate Actions

### Step 1: Add Parser Dependencies

Update `Cargo.toml`:
```toml
[dependencies]
# Existing dependencies...

# Nix parsing
rnix = "0.11"              # Core Nix parser
rowan = "0.15"             # Syntax tree library
nixel = { version = "5.1", optional = true }  # Alternative parser
nix-editor = "0.3"         # For AST manipulation

# Analysis
petgraph = "0.6"           # For dependency graphs
rayon = "1.7"              # For parallel parsing
```

### Step 2: Create Parser Module Structure

```
cim-domain-nix/src/
├── parser/
│   ├── mod.rs            # Core parser types
│   ├── ast.rs            # AST helpers
│   ├── flake.rs          # Flake-specific parsing
│   ├── module.rs         # NixOS module parsing
│   ├── derivation.rs     # Package/derivation parsing
│   └── error.rs          # Parse error types
├── analyzer/
│   ├── mod.rs            # Analysis orchestration
│   ├── dependency.rs     # Dependency analysis
│   ├── security.rs       # Security scanning
│   ├── performance.rs    # Performance analysis
│   └── dead_code.rs      # Dead code detection
├── manipulator/
│   ├── mod.rs            # AST manipulation
│   ├── flake_editor.rs   # Flake editing
│   ├── module_editor.rs  # Module editing
│   └── formatter.rs      # Code formatting
└── workflow/
    ├── mod.rs            # Workflow generation
    ├── from_flake.rs     # Flake → Workflow
    ├── from_module.rs    # Module → Workflow
    └── templates.rs      # Workflow templates
```

### Step 3: Implement Core Parser

```rust
// cim-domain-nix/src/parser/mod.rs
use rnix::{SyntaxNode, SyntaxKind, SyntaxElement};
use rowan::{GreenNode, TextRange};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct NixFile {
    /// The parsed syntax tree
    pub ast: SyntaxNode,
    /// The green tree for incremental updates
    pub green: GreenNode,
    /// Source file path if available
    pub source: Option<PathBuf>,
    /// Original content
    pub content: String,
    /// Parse errors
    pub errors: Vec<ParseError>,
}

impl NixFile {
    /// Parse a Nix file from disk
    pub fn parse_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_string(content, Some(path.to_path_buf()))
    }

    /// Parse Nix content from a string
    pub fn parse_string(content: String, source: Option<PathBuf>) -> Result<Self> {
        let parsed = rnix::parse(&content);
        
        let errors = parsed.errors()
            .into_iter()
            .map(|e| ParseError::from_rnix(e))
            .collect();

        Ok(Self {
            ast: parsed.node(),
            green: parsed.green().into_owned(),
            source,
            content,
            errors,
        })
    }

    /// Check if the file has parse errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the file type based on content
    pub fn file_type(&self) -> NixFileType {
        NixFileType::detect(&self.ast)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NixFileType {
    Flake,
    Module,
    Overlay,
    Derivation,
    Configuration,
    Unknown,
}

impl NixFileType {
    pub fn detect(ast: &SyntaxNode) -> Self {
        // Analyze AST to determine file type
        if Self::is_flake(ast) {
            NixFileType::Flake
        } else if Self::is_module(ast) {
            NixFileType::Module
        } else if Self::is_overlay(ast) {
            NixFileType::Overlay
        } else if Self::is_derivation(ast) {
            NixFileType::Derivation
        } else {
            NixFileType::Unknown
        }
    }

    fn is_flake(ast: &SyntaxNode) -> bool {
        // Check for flake structure: { description, inputs, outputs }
        ast.children()
            .any(|child| {
                // Look for attribute set with flake keys
                false // TODO: Implement
            })
    }

    fn is_module(ast: &SyntaxNode) -> bool {
        // Check for module structure: { options, config, imports }
        false // TODO: Implement
    }

    fn is_overlay(ast: &SyntaxNode) -> bool {
        // Check for overlay pattern: self: super: { ... }
        false // TODO: Implement
    }

    fn is_derivation(ast: &SyntaxNode) -> bool {
        // Check for derivation calls
        false // TODO: Implement
    }
}
```

### Step 4: Implement Flake Parser

```rust
// cim-domain-nix/src/parser/flake.rs
use super::*;
use crate::value_objects::{FlakeRef, FlakeInputs, FlakeOutputs};
use rnix::ast::{AttrSet, Ident};

pub struct FlakeParser;

impl FlakeParser {
    pub fn parse(file: &NixFile) -> Result<ParsedFlake> {
        let flake_node = Self::find_flake_attrset(&file.ast)?;
        
        let description = Self::extract_description(&flake_node)?;
        let inputs = Self::extract_inputs(&flake_node)?;
        let outputs = Self::extract_outputs(&flake_node)?;
        let nix_config = Self::extract_nix_config(&flake_node)?;

        Ok(ParsedFlake {
            file: file.clone(),
            description,
            inputs,
            outputs,
            nix_config,
        })
    }

    fn find_flake_attrset(ast: &SyntaxNode) -> Result<AttrSet> {
        // Find the top-level attribute set
        ast.children()
            .find_map(AttrSet::cast)
            .ok_or_else(|| NixDomainError::ParseError("Not a valid flake".into()))
    }

    fn extract_description(attrset: &AttrSet) -> Result<Option<String>> {
        // Extract description attribute
        Ok(None) // TODO: Implement
    }

    fn extract_inputs(attrset: &AttrSet) -> Result<FlakeInputs> {
        // Extract and parse inputs
        Ok(FlakeInputs::default()) // TODO: Implement
    }

    fn extract_outputs(attrset: &AttrSet) -> Result<FlakeOutputs> {
        // Extract and parse outputs
        Ok(FlakeOutputs::default()) // TODO: Implement
    }

    fn extract_nix_config(attrset: &AttrSet) -> Result<Option<HashMap<String, String>>> {
        // Extract nixConfig if present
        Ok(None) // TODO: Implement
    }
}

pub struct ParsedFlake {
    pub file: NixFile,
    pub description: Option<String>,
    pub inputs: FlakeInputs,
    pub outputs: FlakeOutputs,
    pub nix_config: Option<HashMap<String, String>>,
}

impl ParsedFlake {
    /// Add a new input to the flake
    pub fn add_input(&mut self, name: &str, url: &str) -> Result<()> {
        // Use AST manipulation to add input
        let mut manipulator = FlakeManipulator::new(&self.file);
        manipulator.add_input(name, url)?;
        
        // Update our parsed representation
        self.file = manipulator.generate()?;
        self.inputs = FlakeParser::extract_inputs(&self.file.ast)?;
        
        Ok(())
    }

    /// Update an existing input
    pub fn update_input(&mut self, name: &str, new_url: &str) -> Result<()> {
        // Use AST manipulation to update input
        let mut manipulator = FlakeManipulator::new(&self.file);
        manipulator.update_input(name, new_url)?;
        
        // Update our parsed representation
        self.file = manipulator.generate()?;
        self.inputs = FlakeParser::extract_inputs(&self.file.ast)?;
        
        Ok(())
    }

    /// Generate a workflow from this flake
    pub fn to_workflow(&self) -> Result<WorkflowGraph> {
        WorkflowGenerator::from_flake(self)
    }
}
```

### Step 5: Implement AST Manipulation

```rust
// cim-domain-nix/src/manipulator/flake_editor.rs
use rnix::{SyntaxNode, SyntaxElement, SyntaxKind};
use rowan::{GreenNodeBuilder, NodeOrToken};

pub struct FlakeManipulator {
    original: NixFile,
    builder: GreenNodeBuilder,
}

impl FlakeManipulator {
    pub fn new(file: &NixFile) -> Self {
        Self {
            original: file.clone(),
            builder: GreenNodeBuilder::new(),
        }
    }

    pub fn add_input(&mut self, name: &str, url: &str) -> Result<()> {
        // Walk the AST and modify the inputs section
        self.visit_and_modify(|element| {
            match element {
                NodeOrToken::Node(node) => {
                    if self.is_inputs_attrset(node) {
                        self.insert_input_attribute(node, name, url)
                    } else {
                        self.copy_node(node)
                    }
                }
                NodeOrToken::Token(token) => {
                    self.copy_token(token)
                }
            }
        })
    }

    pub fn update_input(&mut self, name: &str, new_url: &str) -> Result<()> {
        // Similar to add_input but replaces existing
        todo!()
    }

    pub fn generate(self) -> Result<NixFile> {
        let green = self.builder.finish();
        let ast = SyntaxNode::new_root(green.clone());
        
        // Format the output
        let formatted = NixFormatter::format(&ast)?;
        
        Ok(NixFile {
            ast,
            green,
            source: self.original.source.clone(),
            content: formatted,
            errors: vec![],
        })
    }

    fn is_inputs_attrset(&self, node: &SyntaxNode) -> bool {
        // Check if this is the inputs attribute set
        false // TODO: Implement
    }

    fn insert_input_attribute(&mut self, inputs_node: &SyntaxNode, name: &str, url: &str) -> Result<()> {
        // Insert a new input into the attribute set
        Ok(()) // TODO: Implement
    }

    fn copy_node(&mut self, node: &SyntaxNode) -> Result<()> {
        // Copy a node to the new tree
        Ok(()) // TODO: Implement
    }

    fn copy_token(&mut self, token: &SyntaxToken) -> Result<()> {
        // Copy a token to the new tree
        Ok(()) // TODO: Implement
    }
}
```

### Step 6: Update Command Handlers

```rust
// cim-domain-nix/src/handlers/mod.rs
use crate::parser::{NixFile, FlakeParser, ParsedFlake};
use crate::manipulator::FlakeManipulator;

impl NixCommandHandler {
    pub async fn handle_add_flake_input(&self, cmd: AddFlakeInput) -> Result<Vec<DomainEvent>> {
        // OLD: String manipulation
        // let content = std::fs::read_to_string(&cmd.flake_path)?;
        // let new_content = /* string manipulation */;

        // NEW: AST manipulation
        let file = NixFile::parse_file(&cmd.flake_path)?;
        let mut flake = FlakeParser::parse(&file)?;
        
        // Add the input using AST manipulation
        flake.add_input(&cmd.name, &cmd.url)?;
        
        // Write back the formatted result
        std::fs::write(&cmd.flake_path, &flake.file.content)?;
        
        // Run nix flake update
        self.run_nix_command(&["flake", "update", &cmd.name], Some(&cmd.flake_path)).await?;
        
        Ok(vec![DomainEvent::FlakeInputAdded {
            flake_path: cmd.flake_path,
            input_name: cmd.name,
            input_url: cmd.url,
        }])
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flake() {
        let content = r#"{
            description = "A test flake";
            inputs.nixpkgs.url = "github:NixOS/nixpkgs";
            outputs = { self, nixpkgs }: {
                packages.x86_64-linux.hello = nixpkgs.hello;
            };
        }"#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        assert!(!file.has_errors());
        assert_eq!(file.file_type(), NixFileType::Flake);

        let flake = FlakeParser::parse(&file).unwrap();
        assert_eq!(flake.description, Some("A test flake".to_string()));
        assert_eq!(flake.inputs.inputs.len(), 1);
    }

    #[test]
    fn test_add_flake_input() {
        let content = r#"{
            description = "Test";
            inputs = {
                nixpkgs.url = "github:NixOS/nixpkgs";
            };
            outputs = { self, nixpkgs }: { };
        }"#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let mut flake = FlakeParser::parse(&file).unwrap();
        
        flake.add_input("flake-utils", "github:numtide/flake-utils").unwrap();
        
        assert!(flake.file.content.contains("flake-utils"));
        assert_eq!(flake.inputs.inputs.len(), 2);
    }

    #[test]
    fn test_detect_file_types() {
        let flake = r#"{ description = "test"; inputs = {}; outputs = {}; }"#;
        let module = r#"{ options = {}; config = {}; }"#;
        let overlay = r#"self: super: { }"#;

        assert_eq!(detect_type(flake), NixFileType::Flake);
        assert_eq!(detect_type(module), NixFileType::Module);
        assert_eq!(detect_type(overlay), NixFileType::Overlay);
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_flake_workflow_generation() {
    let test_flake = "tests/fixtures/complex-flake.nix";
    let file = NixFile::parse_file(Path::new(test_flake)).unwrap();
    let flake = FlakeParser::parse(&file).unwrap();
    
    let workflow = flake.to_workflow().unwrap();
    
    assert!(workflow.nodes().len() > 0);
    assert!(workflow.edges().len() > 0);
    assert_eq!(workflow.metadata().source, WorkflowSource::NixFlake);
}
```

## Migration Path

1. **Phase 1**: Add parser dependencies and basic types (1 week)
   - Add crates to Cargo.toml
   - Create parser module structure
   - Implement basic NixFile parsing

2. **Phase 2**: Implement flake parsing (1 week)
   - Complete FlakeParser implementation
   - Add AST analysis for flake structure
   - Create tests with real flake examples

3. **Phase 3**: AST manipulation (2 weeks)
   - Implement FlakeManipulator
   - Replace string manipulation in handlers
   - Ensure formatting preservation

4. **Phase 4**: Advanced analysis (2 weeks)
   - Dependency graph generation
   - Security scanning
   - Performance analysis

5. **Phase 5**: Workflow generation (1 week)
   - Flake to workflow conversion
   - Module to workflow conversion
   - Integration with Graph domain

## Success Criteria

- [ ] All existing tests pass with new parser
- [ ] Can parse and manipulate nixpkgs flakes
- [ ] AST manipulation preserves formatting
- [ ] Performance: Parse 1000 files in < 5 seconds
- [ ] Generate accurate dependency graphs
- [ ] Convert flakes to CIM workflows 