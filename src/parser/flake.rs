//! Flake-specific parsing and manipulation

use super::NixFile;
use crate::parser::ast::{extract_string_value, get_attribute_value};
use crate::value_objects::{FlakeInputs, FlakeOutputs, FlakeRef};
use crate::{NixDomainError, Result};
use rnix::{SyntaxKind, SyntaxNode};
use std::collections::HashMap;

/// A parser for Nix flakes
pub struct FlakeParser;

impl FlakeParser {
    /// Parse a `NixFile` as a flake
    pub fn parse(file: &NixFile) -> Result<ParsedFlake> {
        if !file.errors.is_empty() {
            return Err(NixDomainError::ParseError(
                "Cannot parse flake with syntax errors".to_string(),
            ));
        }

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

    fn find_flake_attrset(ast: &SyntaxNode) -> Result<SyntaxNode> {
        // Find the top-level attribute set
        ast.children()
            .find(|child| child.kind() == SyntaxKind::NODE_ATTR_SET)
            .ok_or_else(|| {
                NixDomainError::ParseError("Not a valid flake: no attribute set found".into())
            })
    }

    fn extract_description(attrset: &SyntaxNode) -> Result<Option<String>> {
        Ok(
            get_attribute_value(attrset, "description")
                .and_then(|node| extract_string_value(&node)),
        )
    }

    fn extract_inputs(attrset: &SyntaxNode) -> Result<FlakeInputs> {
        let mut inputs = HashMap::new();

        if let Some(inputs_node) = get_attribute_value(attrset, "inputs") {
            if inputs_node.kind() == SyntaxKind::NODE_ATTR_SET {
                // Extract each input using proper AST traversal
                for child in inputs_node.children() {
                    // In rnix 0.11, key-value pairs might be ENTRY nodes
                    if child.text().to_string().contains('=') {
                        // Find the key (input name)
                        let key_node = child
                            .children()
                            .find(|n| n.kind() == SyntaxKind::NODE_IDENT);

                        // Find the value node
                        let value_node = child.children().find(|n| {
                            n.kind() == SyntaxKind::NODE_STRING
                                || n.kind() == SyntaxKind::NODE_ATTR_SET
                        });

                        if let (Some(key), Some(value)) = (key_node, value_node) {
                            let input_name = key.text().to_string();

                            // Use extract_input_url to properly handle both string and attrset formats
                            if let Some(url) = Self::extract_input_url(&value) {
                                inputs.insert(input_name, FlakeRef::new(&url));
                            }
                        }
                    }
                }
            }
        }

        Ok(FlakeInputs { inputs })
    }

    fn extract_input_url(node: &SyntaxNode) -> Option<String> {
        // Input can be a string directly or an attrset with url field
        if node.kind() == SyntaxKind::NODE_STRING {
            extract_string_value(node)
        } else if node.kind() == SyntaxKind::NODE_ATTR_SET {
            get_attribute_value(node, "url").and_then(|url_node| extract_string_value(&url_node))
        } else {
            None
        }
    }

    fn extract_outputs(_attrset: &SyntaxNode) -> Result<FlakeOutputs> {
        let outputs = FlakeOutputs {
            packages: HashMap::new(),
            dev_shells: HashMap::new(),
            apps: HashMap::new(),
            nixos_modules: HashMap::new(),
            overlays: HashMap::new(),
        };

        // TODO: Parse outputs function and extract actual outputs
        Ok(outputs)
    }

    fn extract_nix_config(attrset: &SyntaxNode) -> Result<Option<HashMap<String, String>>> {
        if let Some(config_node) = get_attribute_value(attrset, "nixConfig") {
            if config_node.kind() == SyntaxKind::NODE_ATTR_SET {
                let mut config = HashMap::new();

                // Simple text-based extraction for now
                let text = config_node.text().to_string();
                for line in text.lines() {
                    if line.contains('=') {
                        let parts: Vec<&str> = line.split('=').collect();
                        if parts.len() == 2 {
                            let key = parts[0].trim().to_string();
                            let value = parts[1].trim().trim_end_matches(';').to_string();
                            config.insert(key, value);
                        }
                    }
                }

                return Ok(Some(config));
            }
        }

        Ok(None)
    }
}

/// A parsed flake with extracted metadata
#[derive(Debug, Clone)]
pub struct ParsedFlake {
    /// The underlying parsed file
    pub file: NixFile,
    /// Flake description
    pub description: Option<String>,
    /// Flake inputs
    pub inputs: FlakeInputs,
    /// Flake outputs
    pub outputs: FlakeOutputs,
    /// Nix configuration
    pub nix_config: Option<HashMap<String, String>>,
}

impl ParsedFlake {
    /// Add a new input to the flake
    pub fn add_input(&mut self, name: &str, url: &str) -> Result<()> {
        // Parse the URL into a FlakeRef
        let flake_ref = FlakeRef::new(url);

        // Add to our inputs
        self.inputs.inputs.insert(name.to_string(), flake_ref);

        // TODO: Update the AST
        // For now, we'll regenerate the entire flake
        self.regenerate_content()?;

        Ok(())
    }

    /// Update an existing input
    pub fn update_input(&mut self, name: &str, new_url: &str) -> Result<()> {
        if !self.inputs.inputs.contains_key(name) {
            return Err(NixDomainError::ValidationError(format!(
                "Input '{name}' not found"
            )));
        }

        let flake_ref = FlakeRef::new(new_url);
        self.inputs.inputs.insert(name.to_string(), flake_ref);

        self.regenerate_content()?;

        Ok(())
    }

    /// Remove an input
    pub fn remove_input(&mut self, name: &str) -> Result<()> {
        if self.inputs.inputs.remove(name).is_none() {
            return Err(NixDomainError::ValidationError(format!(
                "Input '{name}' not found"
            )));
        }

        self.regenerate_content()?;

        Ok(())
    }

    /// Regenerate the flake content from the parsed data
    fn regenerate_content(&mut self) -> Result<()> {
        let mut content = String::from("{\n");

        // Add description
        if let Some(desc) = &self.description {
            content.push_str(&format!("  description = \"{desc}\";\n\n"));
        }

        // Add inputs
        if !self.inputs.inputs.is_empty() {
            content.push_str("  inputs = {\n");
            for (name, flake_ref) in &self.inputs.inputs {
                content.push_str(&format!(
                    "    {}.url = \"{}\";\n",
                    name,
                    flake_ref.to_nix_string()
                ));
            }
            content.push_str("  };\n\n");
        }

        // Add outputs (simplified for now)
        content.push_str("  outputs = { self, ... }: {\n");
        content.push_str("    # TODO: Preserve original outputs\n");
        content.push_str("  };\n");

        // Add nixConfig if present
        if let Some(config) = &self.nix_config {
            content.push_str("\n  nixConfig = {\n");
            for (key, value) in config {
                content.push_str(&format!("    {key} = {value};\n"));
            }
            content.push_str("  };\n");
        }

        content.push_str("}\n");

        // Update the file content
        self.file.content = content;

        // Re-parse to update AST
        let new_file = NixFile::parse_string(self.file.content.clone(), self.file.source.clone())?;
        self.file = new_file;

        Ok(())
    }

    /// Get the flake as formatted Nix code
    pub fn to_string(&self) -> String {
        self.file.content.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flake() {
        let content = r#"{
            description = "A test flake";
            inputs = {
                nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
            };
            outputs = { self, nixpkgs }: {
                packages.x86_64-linux.hello = nixpkgs.legacyPackages.x86_64-linux.hello;
            };
        }"#;

        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        let flake = FlakeParser::parse(&file).unwrap();

        assert_eq!(flake.description, Some("A test flake".to_string()));

        // Debug output
        println!("Parsed inputs: {:?}", flake.inputs.inputs);
        println!("Number of inputs: {}", flake.inputs.inputs.len());
        for (name, ref_) in &flake.inputs.inputs {
            println!("Input: {} -> {}", name, ref_.to_nix_string());
        }

        assert_eq!(flake.inputs.inputs.len(), 1);
        assert!(flake.inputs.inputs.contains_key("nixpkgs"));
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

        flake
            .add_input("flake-utils", "github:numtide/flake-utils")
            .unwrap();

        assert_eq!(flake.inputs.inputs.len(), 2);
        assert!(flake.inputs.inputs.contains_key("flake-utils"));
        assert!(flake.to_string().contains("flake-utils"));
    }
}
