//! Dependency analysis for Nix files
//!
//! This module analyzes dependencies between Nix files, including imports,
//! flake inputs, and package references.

use crate::parser::{NixFile, NixFileType};
use crate::Result;
use petgraph::graph::DiGraph;
use petgraph::visit::EdgeRef;
use rnix::{SyntaxKind, SyntaxNode};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// A node in the dependency graph representing a file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileNode {
    /// Path to the file
    pub path: PathBuf,
    /// Type of Nix file
    pub file_type: NixFileType,
    /// Whether the file has parse errors
    pub has_errors: bool,
}

/// An edge in the dependency graph representing a dependency
#[derive(Debug, Clone)]
pub struct DependencyEdge {
    /// Type of dependency
    pub dep_type: DependencyType,
    /// Path to the dependency
    pub path: PathBuf,
    /// Whether the dependency is optional
    pub optional: bool,
    /// Line number where the dependency is declared
    pub line: Option<usize>,
}

/// Types of dependencies between Nix files
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    /// Import statement
    Import,
    /// Flake input
    FlakeInput,
    /// Relative path reference
    PathReference,
    /// Fetchurl or similar
    FetchUrl,
    /// Package reference
    PackageRef,
}

/// Dependency graph type alias
pub type DependencyGraph = DiGraph<FileNode, DependencyEdge>;

/// Analyzer for finding dependencies in Nix files
pub struct DependencyAnalyzer;

impl DependencyAnalyzer {
    /// Find all dependencies in a Nix file
    ///
    /// # Errors
    ///
    /// Returns an error if parsing fails or dependency analysis encounters issues
    pub fn find_dependencies(file: &NixFile) -> Result<Vec<DependencyEdge>> {
        let mut dependencies = Vec::new();

        // Different analysis based on file type
        match file.file_type() {
            NixFileType::Flake => {
                dependencies.extend(Self::find_flake_dependencies(&file.ast)?);
            }
            NixFileType::Module => {
                dependencies.extend(Self::find_module_dependencies(&file.ast)?);
            }
            _ => {
                // General dependency finding
                dependencies.extend(Self::find_import_dependencies(&file.ast)?);
                dependencies.extend(Self::find_path_dependencies(&file.ast)?);
            }
        }

        Ok(dependencies)
    }

    /// Find dependencies in a flake file
    ///
    /// # Errors
    ///
    /// Returns an error if flake dependency extraction fails
    fn find_flake_dependencies(ast: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        // Find the inputs attribute
        if let Some(inputs_node) = Self::find_attribute(ast, "inputs") {
            deps.extend(Self::extract_flake_inputs(&inputs_node)?);
        }

        // Also look for imports in outputs
        if let Some(outputs_node) = Self::find_attribute(ast, "outputs") {
            deps.extend(Self::find_import_dependencies(&outputs_node)?);
        }

        Ok(deps)
    }

    /// Extract flake inputs as dependencies
    ///
    /// # Errors
    ///
    /// Returns an error if input extraction fails
    fn extract_flake_inputs(inputs_node: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        // In rnix 0.11, key-value pairs are not explicitly marked
        // We need to look for patterns like "name = value"
        for child in inputs_node.children() {
            let text = child.text().to_string();
            let trimmed = text.trim();
            
            // Check if this looks like an attribute assignment
            if trimmed.contains('=') && !trimmed.starts_with('#') {
                // Extract the key part
                if let Some(key_part) = trimmed.split('=').next() {
                    let _input_name = key_part.trim().to_string();
                    
                    // Try to extract URL from the value
                    // Look for string nodes in children
                    for value_child in child.children() {
                        if let Some(url) = Self::extract_url_from_node(&value_child) {
                            deps.push(DependencyEdge {
                                dep_type: DependencyType::FlakeInput,
                                path: PathBuf::from(url),
                                optional: false,
                                line: None, // TODO: Extract line number
                            });
                            break;
                        }
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Find dependencies in a `NixOS` module
    ///
    /// # Errors
    ///
    /// Returns an error if module dependency analysis fails
    fn find_module_dependencies(ast: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        // Find imports
        if let Some(imports_node) = Self::find_attribute(ast, "imports") {
            deps.extend(Self::extract_imports(&imports_node)?);
        }

        // Find other dependencies
        deps.extend(Self::find_import_dependencies(ast)?);
        deps.extend(Self::find_path_dependencies(ast)?);

        Ok(deps)
    }

    /// Find import statements
    ///
    /// # Errors
    ///
    /// Returns an error if import statement parsing fails
    fn find_import_dependencies(node: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        // Recursively search for import expressions
        if node.kind() == SyntaxKind::NODE_APPLY {
            let text = node.text().to_string();
            if text.starts_with("import ") {
                // Extract the path being imported
                if let Some(path_node) = node.children().nth(1) {
                    if let Some(path) = Self::extract_path_from_node(&path_node) {
                        deps.push(DependencyEdge {
                            dep_type: DependencyType::Import,
                            path: PathBuf::from(path),
                            optional: false,
                            line: None,
                        });
                    }
                }
            }
        }

        // Recurse into children
        for child in node.children() {
            deps.extend(Self::find_import_dependencies(&child)?);
        }

        Ok(deps)
    }

    /// Find path references (./foo.nix, ../bar/baz.nix, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if path reference extraction fails
    fn find_path_dependencies(node: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        if node.kind() == SyntaxKind::NODE_PATH {
            let path_text = node.text().to_string();
            if path_text.ends_with(".nix") {
                deps.push(DependencyEdge {
                    dep_type: DependencyType::PathReference,
                    path: PathBuf::from(path_text),
                    optional: false,
                    line: None,
                });
            }
        }

        // Recurse into children
        for child in node.children() {
            deps.extend(Self::find_path_dependencies(&child)?);
        }

        Ok(deps)
    }

    /// Extract imports from a list node
    ///
    /// # Errors
    ///
    /// Returns an error if import extraction fails
    fn extract_imports(list_node: &SyntaxNode) -> Result<Vec<DependencyEdge>> {
        let mut deps = Vec::new();

        for child in list_node.children() {
            if let Some(path) = Self::extract_path_from_node(&child) {
                deps.push(DependencyEdge {
                    dep_type: DependencyType::Import,
                    path: PathBuf::from(path),
                    optional: false,
                    line: None,
                });
            }
        }

        Ok(deps)
    }

    /// Find an attribute by name in an attribute set
    fn find_attribute(node: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
        for child in node.children() {
            let text = child.text().to_string();
            let trimmed = text.trim();
            
            // Check if this child's text starts with the attribute name
            if trimmed.starts_with(name) && trimmed.contains('=') {
                // Find the value part after the equals sign
                for value_child in child.children() {
                    // Skip identifier and look for actual value nodes
                    match value_child.kind() {
                        SyntaxKind::NODE_STRING 
                        | SyntaxKind::NODE_ATTR_SET
                        | SyntaxKind::NODE_LIST
                        | SyntaxKind::NODE_PATH => {
                            return Some(value_child);
                        }
                        _ => {} // Do nothing for other node types
                    }
                }
            }
            
            // Recurse into attribute sets
            if child.kind() == SyntaxKind::NODE_ATTR_SET {
                if let Some(found) = Self::find_attribute(&child, name) {
                    return Some(found);
                }
            }
        }
        None
    }

    /// Extract a path string from a node
    fn extract_path_from_node(node: &SyntaxNode) -> Option<String> {
        match node.kind() {
            SyntaxKind::NODE_PATH => Some(node.text().to_string()),
            SyntaxKind::NODE_STRING => {
                // Remove quotes
                let text = node.text().to_string();
                Some(text.trim_matches('"').to_string())
            }
            _ => None,
        }
    }

    /// Extract URL from a flake input value
    fn extract_url_from_node(node: &SyntaxNode) -> Option<String> {
        // Handle different input formats
        match node.kind() {
            SyntaxKind::NODE_STRING => {
                let text = node.text().to_string();
                Some(text.trim_matches('"').to_string())
            }
            SyntaxKind::NODE_ATTR_SET => {
                // Look for url attribute
                if let Some(url_node) = Self::find_attribute(node, "url") {
                    Self::extract_url_from_node(&url_node)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Analyze a dependency graph for issues
    pub fn analyze_graph(graph: &DependencyGraph) -> DependencyAnalysis {
        let mut analysis = DependencyAnalysis::default();

        // Find cycles
        analysis.cycles = Self::find_cycles(graph);

        // Find missing dependencies
        analysis.missing_deps = Self::find_missing_dependencies(graph);

        // Calculate depth
        analysis.max_depth = Self::calculate_max_depth(graph);

        // Find unused files
        analysis.unused_files = Self::find_unused_files(graph);

        analysis
    }

    /// Find dependency cycles
    fn find_cycles(graph: &DependencyGraph) -> Vec<Vec<PathBuf>> {
        use petgraph::algo::kosaraju_scc;

        let sccs = kosaraju_scc(graph);
        let mut cycles = Vec::new();

        for scc in sccs {
            if scc.len() > 1 {
                let cycle: Vec<PathBuf> = scc.iter()
                    .map(|&idx| graph[idx].path.clone())
                    .collect();
                cycles.push(cycle);
            }
        }

        cycles
    }

    /// Find missing dependencies (referenced but not found)
    fn find_missing_dependencies(graph: &DependencyGraph) -> Vec<PathBuf> {
        let mut missing = Vec::new();
        let existing_paths: HashSet<_> = graph.node_indices()
            .map(|idx| &graph[idx].path)
            .collect();

        for edge in graph.edge_references() {
            let dep = edge.weight();
            if !existing_paths.contains(&dep.path) && !dep.optional {
                missing.push(dep.path.clone());
            }
        }

        missing.sort();
        missing.dedup();
        missing
    }

    /// Calculate maximum dependency depth
    fn calculate_max_depth(graph: &DependencyGraph) -> usize {
        use petgraph::algo::toposort;

        if let Ok(sorted) = toposort(graph, None) {
            // Calculate depth for each node
            let mut depths = HashMap::new();
            let mut max_depth = 0;

            for node in sorted {
                let depth = graph.edges_directed(node, petgraph::Direction::Incoming)
                    .map(|edge| depths.get(&edge.source()).copied().unwrap_or(0) + 1)
                    .max()
                    .unwrap_or(0);
                
                depths.insert(node, depth);
                max_depth = max_depth.max(depth);
            }

            max_depth
        } else {
            // Graph has cycles
            0
        }
    }

    /// Find files that are not dependencies of anything
    fn find_unused_files(graph: &DependencyGraph) -> Vec<PathBuf> {
        graph.node_indices()
            .filter(|&idx| {
                graph.edges_directed(idx, petgraph::Direction::Incoming).count() == 0
            })
            .map(|idx| graph[idx].path.clone())
            .collect()
    }
}

/// Results of dependency analysis
#[derive(Debug, Default)]
pub struct DependencyAnalysis {
    /// Dependency cycles found
    pub cycles: Vec<Vec<PathBuf>>,
    /// Missing dependencies
    pub missing_deps: Vec<PathBuf>,
    /// Maximum dependency depth
    pub max_depth: usize,
    /// Files not used by anything
    pub unused_files: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_import_dependencies() {
        let content = r#"
        {
            imports = [ ./hardware.nix ./network.nix ];
            config = import ./config.nix;
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let deps = DependencyAnalyzer::find_dependencies(&file).unwrap();

        assert!(deps.len() >= 3);
        assert!(deps.iter().any(|d| d.path == PathBuf::from("./hardware.nix")));
        assert!(deps.iter().any(|d| d.path == PathBuf::from("./network.nix")));
        assert!(deps.iter().any(|d| d.path == PathBuf::from("./config.nix")));
    }

    #[test]
    fn test_find_flake_inputs() {
        let content = r#"
        {
            description = "Test flake";
            inputs = {
                nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
                flake-utils.url = "github:numtide/flake-utils";
            };
            outputs = { self, nixpkgs, flake-utils }: { };
        }
        "#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let deps = DependencyAnalyzer::find_dependencies(&file).unwrap();

        let flake_inputs: Vec<_> = deps.iter()
            .filter(|d| d.dep_type == DependencyType::FlakeInput)
            .collect();

        assert_eq!(flake_inputs.len(), 2);
    }
} 