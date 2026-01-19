// Copyright 2025 Cowboy AI, LLC.

//! AST to Value Converter
//!
//! Converts rnix AST nodes to NixValue semantic representation.
//!
//! This bridges the gap between syntax (AST) and semantics (NixValue),
//! enabling direct file parsing in the I/O layer.

use super::ast::{AstError, NixAst, Result};
use super::value_objects::*;
use rnix::{SyntaxKind, SyntaxNode};

// ============================================================================
// Converter
// ============================================================================

/// Converts rnix AST to NixValue
pub struct AstConverter;

impl AstConverter {
    /// Create a new AST converter
    pub fn new() -> Self {
        Self
    }

    /// Convert an AST to a NixValue
    pub fn convert(&self, ast: &NixAst) -> Result<NixValue> {
        // Get the root syntax node
        let root_node = ast.root();
        let syntax_node = root_node.syntax();

        // Find the actual expression within the root
        if let Some(expr_node) = self.find_expression(syntax_node) {
            self.convert_node(&expr_node)
        } else {
            Err(AstError::InvalidSyntax("No expression found in AST".to_string()))
        }
    }

    /// Find the expression node in the tree
    fn find_expression(&self, node: &SyntaxNode) -> Option<SyntaxNode> {
        // Check if this node itself is an expression
        if self.is_expression(node) {
            return Some(node.clone());
        }

        // Otherwise search children
        for child in node.children() {
            if let Some(expr) = self.find_expression(&child) {
                return Some(expr);
            }
        }

        None
    }

    /// Check if a node is an expression
    fn is_expression(&self, node: &SyntaxNode) -> bool {
        matches!(
            node.kind(),
            SyntaxKind::NODE_ATTR_SET
                | SyntaxKind::NODE_LIST
                | SyntaxKind::NODE_STRING
                | SyntaxKind::NODE_IDENT
                | SyntaxKind::NODE_LITERAL
        )
    }

    /// Convert a syntax node to NixValue
    fn convert_node(&self, node: &SyntaxNode) -> Result<NixValue> {
        match node.kind() {
            SyntaxKind::NODE_ATTR_SET => self.convert_attrset(node),
            SyntaxKind::NODE_LIST => self.convert_list(node),
            SyntaxKind::NODE_STRING => self.convert_string(node),
            SyntaxKind::NODE_LITERAL => self.convert_literal(node),
            SyntaxKind::NODE_IDENT => self.convert_ident(node),
            _ => {
                // Try to find an expression in children
                for child in node.children() {
                    if self.is_expression(&child) {
                        return self.convert_node(&child);
                    }
                }
                Err(AstError::InvalidSyntax(format!(
                    "Unsupported node kind: {:?}",
                    node.kind()
                )))
            }
        }
    }

    /// Convert an attribute set node
    fn convert_attrset(&self, node: &SyntaxNode) -> Result<NixValue> {
        let mut attrs = NixAttrset::new();

        // Find all attribute bindings
        for child in node.children() {
            if child.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                if let Some((key, value)) = self.extract_binding(&child)? {
                    attrs.insert(key, value);
                }
            }
        }

        Ok(NixValue::Attrset(attrs))
    }

    /// Extract a key-value binding from an attribute path value node
    fn extract_binding(&self, node: &SyntaxNode) -> Result<Option<(String, NixValue)>> {
        let mut key: Option<String> = None;
        let mut value: Option<NixValue> = None;

        for child in node.children() {
            match child.kind() {
                SyntaxKind::NODE_ATTRPATH => {
                    // Extract the attribute name
                    key = self.extract_attr_name(&child);
                }
                _ => {
                    // Try to convert as expression (the value)
                    if self.is_expression(&child) {
                        value = Some(self.convert_node(&child)?);
                    }
                }
            }
        }

        if let (Some(k), Some(v)) = (key, value) {
            Ok(Some((k, v)))
        } else {
            Ok(None)
        }
    }

    /// Extract attribute name from attr path
    fn extract_attr_name(&self, node: &SyntaxNode) -> Option<String> {
        // Look for identifier or string in the path
        for child in node.descendants() {
            match child.kind() {
                SyntaxKind::NODE_IDENT => {
                    return Some(child.text().to_string());
                }
                SyntaxKind::TOKEN_IDENT => {
                    return Some(child.text().to_string());
                }
                _ => {}
            }
        }
        None
    }

    /// Convert a list node
    fn convert_list(&self, node: &SyntaxNode) -> Result<NixValue> {
        let mut elements = Vec::new();

        for child in node.children() {
            if self.is_expression(&child) {
                elements.push(self.convert_node(&child)?);
            }
        }

        Ok(NixValue::List(NixList { elements }))
    }

    /// Convert a string node
    fn convert_string(&self, node: &SyntaxNode) -> Result<NixValue> {
        // Extract string content, removing quotes
        let text = node.text().to_string();
        let content = text.trim_matches('"').to_string();
        Ok(NixValue::String(NixString::new(&content)))
    }

    /// Convert a literal node (integer, float, bool, null)
    fn convert_literal(&self, node: &SyntaxNode) -> Result<NixValue> {
        let text = node.text().to_string();

        // Try to parse as integer
        if let Ok(n) = text.parse::<i64>() {
            return Ok(NixValue::Integer(NixInteger { value: n }));
        }

        // Try to parse as float
        if let Ok(f) = text.parse::<f64>() {
            return Ok(NixValue::Float(NixFloat { value: f }));
        }

        // Check for boolean
        match text.as_str() {
            "true" => return Ok(NixValue::Bool(NixBool { value: true })),
            "false" => return Ok(NixValue::Bool(NixBool { value: false })),
            "null" => return Ok(NixValue::Null(NixNull)),
            _ => {}
        }

        Err(AstError::InvalidSyntax(format!("Unknown literal: {}", text)))
    }

    /// Convert an identifier node (might be a string in some contexts)
    fn convert_ident(&self, node: &SyntaxNode) -> Result<NixValue> {
        let text = node.text().to_string();

        // In our simplified model, identifiers become strings
        Ok(NixValue::String(NixString::new(&text)))
    }
}

impl Default for AstConverter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Convert AST to NixValue
pub fn ast_to_value(ast: &NixAst) -> Result<NixValue> {
    let converter = AstConverter::new();
    converter.convert(ast)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nix::parser::NixParser;

    #[test]
    fn test_convert_empty_attrset() {
        let parser = NixParser::new();
        let ast = parser.parse_str("{ }").unwrap();

        let converter = AstConverter::new();
        let result = converter.convert(&ast);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            NixValue::Attrset(attrs) => {
                assert_eq!(attrs.attributes.len(), 0);
            }
            _ => panic!("Expected attrset"),
        }
    }

    #[test]
    fn test_convert_simple_attrset() {
        let parser = NixParser::new();
        let ast = parser.parse_str(r#"{ name = "test"; }"#).unwrap();

        let converter = AstConverter::new();
        let result = converter.convert(&ast);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            NixValue::Attrset(attrs) => {
                assert!(attrs.attributes.contains_key("name"));
                if let Some(NixValue::String(s)) = attrs.attributes.get("name") {
                    assert_eq!(s.value, "test");
                } else {
                    panic!("Expected string value");
                }
            }
            _ => panic!("Expected attrset"),
        }
    }

    #[test]
    fn test_convert_integer() {
        let parser = NixParser::new();
        let ast = parser.parse_str("42").unwrap();

        let converter = AstConverter::new();
        let result = converter.convert(&ast);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            NixValue::Integer(i) => assert_eq!(i.value, 42),
            _ => panic!("Expected integer"),
        }
    }

    #[test]
    fn test_convert_list() {
        let parser = NixParser::new();
        let ast = parser.parse_str("[ 1 2 3 ]").unwrap();

        let converter = AstConverter::new();
        let result = converter.convert(&ast);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            NixValue::List(list) => {
                assert_eq!(list.elements.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_convert_nested_attrset() {
        let parser = NixParser::new();
        let ast = parser.parse_str(r#"{ outer = { inner = "value"; }; }"#).unwrap();

        let converter = AstConverter::new();
        let result = converter.convert(&ast);
        assert!(result.is_ok());

        let value = result.unwrap();
        match value {
            NixValue::Attrset(attrs) => {
                assert!(attrs.attributes.contains_key("outer"));
                if let Some(NixValue::Attrset(inner)) = attrs.attributes.get("outer") {
                    assert!(inner.attributes.contains_key("inner"));
                } else {
                    panic!("Expected nested attrset");
                }
            }
            _ => panic!("Expected attrset"),
        }
    }

    #[test]
    fn test_ast_to_value_convenience() {
        let parser = NixParser::new();
        let ast = parser.parse_str("{ x = 1; }").unwrap();

        let result = ast_to_value(&ast);
        assert!(result.is_ok());
    }
}
