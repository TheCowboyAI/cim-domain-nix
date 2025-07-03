//! Dead code analysis for Nix files
//!
//! This module detects unused definitions, unreachable code,
//! and redundant imports in Nix files.

use crate::parser::NixFile;
use crate::Result;
use rnix::{SyntaxKind, SyntaxNode};
use std::collections::{HashMap, HashSet};

use crate::analyzer::dependency::DependencyGraph;

/// Dead code found in a Nix file
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeadCode {
    /// Type of dead code
    pub code_type: DeadCodeType,
    /// Name of the unused item
    pub name: String,
    /// File where the dead code was found
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<usize>,
    /// Additional context
    pub context: Option<String>,
    /// Suggested action
    pub suggestion: String,
}

/// Types of dead code
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, PartialOrd, Ord)]
pub enum DeadCodeType {
    /// Unused variable binding
    UnusedVariable,
    /// Unused function parameter
    UnusedParameter,
    /// Unused attribute
    UnusedAttribute,
    /// Unused import
    UnusedImport,
    /// Unreachable code after return/throw
    UnreachableCode,
    /// Unused file (not imported anywhere)
    UnusedFile,
    /// Redundant definition
    RedundantDefinition,
    /// Unused overlay
    UnusedOverlay,
}

/// Dead code analyzer
pub struct DeadCodeAnalyzer;

impl DeadCodeAnalyzer {
    /// Analyze files for dead code
    pub fn analyze(files: &[NixFile], dep_graph: &DependencyGraph) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();

        // Analyze individual files
        for file in files {
            dead_code.extend(Self::analyze_file(file)?);
        }

        // Analyze across files using dependency graph
        dead_code.extend(Self::find_unused_files(files, dep_graph)?);

        // Sort by file and type
        dead_code.sort_by(|a, b| {
            a.file.cmp(&b.file)
                .then_with(|| a.code_type.cmp(&b.code_type))
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(dead_code)
    }

    /// Analyze a single file for dead code
    fn analyze_file(file: &NixFile) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();

        // Build symbol tables
        let mut defined_symbols = HashSet::new();
        let mut used_symbols = HashSet::new();
        
        Self::collect_symbols(&file.ast, &mut defined_symbols, &mut used_symbols)?;

        // Find unused definitions
        for symbol in &defined_symbols {
            if !used_symbols.contains(symbol) {
                dead_code.push(DeadCode {
                    code_type: Self::classify_symbol(symbol),
                    name: symbol.clone(),
                    file: file.source.as_ref().map(|p| p.display().to_string()),
                    line: None,
                    context: None,
                    suggestion: "Remove unused definition or export it if needed".to_string(),
                });
            }
        }

        // Check for other dead code patterns
        dead_code.extend(Self::find_unreachable_code(&file.ast, &file.source)?);
        dead_code.extend(Self::find_redundant_definitions(&file.ast, &file.source)?);

        Ok(dead_code)
    }

    /// Collect defined and used symbols
    fn collect_symbols(
        node: &SyntaxNode,
        defined: &mut HashSet<String>,
        used: &mut HashSet<String>,
    ) -> Result<()> {
        match node.kind() {
            // Let bindings define symbols
            SyntaxKind::NODE_LET_IN => {
                Self::collect_let_bindings(node, defined)?;
            }
            
            // Function parameters define symbols
            SyntaxKind::NODE_LAMBDA => {
                if let Some(param) = node.children().find(|n| n.kind() == SyntaxKind::NODE_IDENT) {
                    defined.insert(param.text().to_string());
                }
            }

            // Attribute sets might define symbols
            SyntaxKind::NODE_ATTR_SET => {
                Self::collect_attribute_definitions(node, defined)?;
            }

            // Identifiers are uses (unless they're definitions)
            SyntaxKind::NODE_IDENT => {
                let name = node.text().to_string();
                // Check if this is a use context
                if !Self::is_definition_context(node) {
                    used.insert(name);
                }
            }

            _ => {}
        }

        // Recurse
        for child in node.children() {
            Self::collect_symbols(&child, defined, used)?;
        }

        Ok(())
    }

    /// Collect let binding definitions
    fn collect_let_bindings(let_node: &SyntaxNode, defined: &mut HashSet<String>) -> Result<()> {
        // In let expressions, bindings are direct children of NODE_LET_IN
        for child in let_node.children() {
            if child.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                // Get the attrpath which contains the identifier
                if let Some(attrpath) = child.children().find(|n| n.kind() == SyntaxKind::NODE_ATTRPATH) {
                    // Get the identifier from the attrpath
                    if let Some(ident) = attrpath.children().find(|n| n.kind() == SyntaxKind::NODE_IDENT) {
                        let name = ident.text().to_string();
                        defined.insert(name);
                    }
                }
            }
        }
        Ok(())
    }

    /// Collect attribute definitions
    fn collect_attribute_definitions(
        attr_set: &SyntaxNode,
        defined: &mut HashSet<String>,
    ) -> Result<()> {
        for child in attr_set.children() {
            let text = child.text().to_string();
            let trimmed = text.trim();
            
            // Look for attribute assignments
            if trimmed.contains('=') && !trimmed.starts_with('#') {
                // Extract the key part
                if let Some(key_part) = trimmed.split('=').next() {
                    let name = key_part.trim();
                    // Skip if it's a complex expression
                    if !name.contains(' ') && !name.contains('.') {
                        defined.insert(name.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if a node is in a definition context
    fn is_definition_context(node: &SyntaxNode) -> bool {
        if let Some(parent) = node.parent() {
            // Check if this identifier is inside an attrpath
            if parent.kind() == SyntaxKind::NODE_ATTRPATH {
                // Check if the attrpath is part of an attrpath-value (definition)
                if let Some(grandparent) = parent.parent() {
                    if grandparent.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                        // Check if the attrpath is the first child (the key)
                        if let Some(first_child) = grandparent.children().next() {
                            return first_child == parent;
                        }
                    }
                }
            }
        }
        false
    }

    /// Classify the type of symbol
    fn classify_symbol(name: &str) -> DeadCodeType {
        if name.starts_with('_') {
            // Convention: underscore prefix often indicates unused
            DeadCodeType::UnusedParameter
        } else if name.contains("import") || name.contains("Import") {
            DeadCodeType::UnusedImport
        } else {
            DeadCodeType::UnusedVariable
        }
    }

    /// Find unreachable code
    fn find_unreachable_code(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();

        // Look for throw or abort in let bindings
        if node.kind() == SyntaxKind::NODE_LET_IN {
            let mut found_throw = false;
            
            // Check each binding (direct children of NODE_LET_IN)
            for binding in node.children() {
                if binding.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                    let binding_text = binding.text().to_string();
                    
                    // If we already found a throw, this binding is unreachable
                    if found_throw {
                        // Get the identifier from the attrpath
                        if let Some(attrpath) = binding.children().find(|n| n.kind() == SyntaxKind::NODE_ATTRPATH) {
                            if let Some(ident) = attrpath.children().find(|n| n.kind() == SyntaxKind::NODE_IDENT) {
                                let name = ident.text().to_string();
                                dead_code.push(DeadCode {
                                    code_type: DeadCodeType::UnreachableCode,
                                    name: format!("binding '{name}'"),
                                    file: file.as_ref().map(|p| p.display().to_string()),
                                    line: None,
                                    context: Some("Code after throw/abort is unreachable".to_string()),
                                    suggestion: "Remove unreachable code".to_string(),
                                });
                            }
                        }
                    }
                    
                    // Check if this binding contains throw or abort
                    if binding_text.contains("throw") || binding_text.contains("abort") {
                        found_throw = true;
                    }
                }
            }
        }

        // Recurse into children
        for child in node.children() {
            dead_code.extend(Self::find_unreachable_code(&child, file)?);
        }

        Ok(dead_code)
    }

    /// Find redundant definitions
    fn find_redundant_definitions(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
    ) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();
        let mut seen_definitions = HashMap::new();

        Self::check_redundant_definitions_recursive(node, file, &mut seen_definitions, &mut dead_code)?;

        Ok(dead_code)
    }

    /// Recursively check for redundant definitions
    fn check_redundant_definitions_recursive(
        node: &SyntaxNode,
        file: &Option<std::path::PathBuf>,
        seen: &mut HashMap<String, usize>,
        dead_code: &mut Vec<DeadCode>,
    ) -> Result<()> {
        // Look for attribute assignments
        let text = node.text().to_string();
        let trimmed = text.trim();
        
        if trimmed.contains('=') && !trimmed.starts_with('#') {
            // Extract the key part
            if let Some(key_part) = trimmed.split('=').next() {
                let name = key_part.trim();
                
                // Skip if it's a complex expression
                if !name.contains(' ') && !name.contains('.') {
                    if let Some(&count) = seen.get(name) {
                        dead_code.push(DeadCode {
                            code_type: DeadCodeType::RedundantDefinition,
                            name: name.to_string(),
                            file: file.as_ref().map(|p| p.display().to_string()),
                            line: None,
                            context: Some(format!("Defined {} times", count + 1)),
                            suggestion: "Remove redundant definitions or rename if they serve different purposes".to_string(),
                        });
                        seen.insert(name.to_string(), count + 1);
                    } else {
                        seen.insert(name.to_string(), 1);
                    }
                }
            }
        }

        // Only recurse into let bindings and attribute sets
        match node.kind() {
            SyntaxKind::NODE_LET_IN | SyntaxKind::NODE_ATTR_SET => {
                for child in node.children() {
                    Self::check_redundant_definitions_recursive(&child, file, seen, dead_code)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Find unused files using the dependency graph
    fn find_unused_files(_files: &[NixFile], graph: &DependencyGraph) -> Result<Vec<DeadCode>> {
        let mut dead_code = Vec::new();

        // Find files with no incoming edges (not imported by anything)
        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];
            let incoming_count = graph.edges_directed(node_idx, petgraph::Direction::Incoming).count();
            
            if incoming_count == 0 && !Self::is_entry_point(&node.path) {
                dead_code.push(DeadCode {
                    code_type: DeadCodeType::UnusedFile,
                    name: node.path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    file: Some(node.path.display().to_string()),
                    line: None,
                    context: Some("File is not imported by any other file".to_string()),
                    suggestion: "Remove file if unused, or add it to imports if needed".to_string(),
                });
            }
        }

        Ok(dead_code)
    }

    /// Check if a file is an entry point
    fn is_entry_point(path: &std::path::Path) -> bool {
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            // Common entry points
            matches!(
                file_name,
                "flake.nix" | "default.nix" | "shell.nix" | "configuration.nix" | "home.nix"
            )
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::DiGraph;

    #[test]
    fn test_detect_unused_variable() {
        let content = r#"
        let
          used = 42;
          unused = 99;
        in used
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let graph = DiGraph::new();
        let dead_code = DeadCodeAnalyzer::analyze(&[file], &graph).unwrap();

        // Debug: print what we found
        eprintln!("Found {} dead code items:", dead_code.len());
        for dc in &dead_code {
            eprintln!("  - Type: {:?}, Name: {}", dc.code_type, dc.name);
        }

        assert!(dead_code.iter().any(|d| {
            d.code_type == DeadCodeType::UnusedVariable && d.name == "unused"
        }));
    }

    #[test]
    fn test_detect_unreachable_code() {
        let content = r#"
        let
          result = throw "error";
          unreachable = 42;
        in result
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let graph = DiGraph::new();
        let dead_code = DeadCodeAnalyzer::analyze(&[file], &graph).unwrap();

        assert!(dead_code.iter().any(|d| {
            d.code_type == DeadCodeType::UnreachableCode
        }));
    }

    #[test]
    fn test_no_false_positives() {
        let content = r#"
        let
          helper = x: x + 1;
          result = helper 5;
        in result
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let graph = DiGraph::new();
        let dead_code = DeadCodeAnalyzer::analyze(&[file], &graph).unwrap();

        // Should not report helper as unused
        assert!(!dead_code.iter().any(|d| d.name == "helper"));
    }
} 