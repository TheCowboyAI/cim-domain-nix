//! AST helper functions for working with Nix syntax trees

use rnix::{SyntaxNode, SyntaxKind};
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

/// Represents a parsed Nix expression with full AST information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NixAst {
    /// Integer literal
    Integer(i64),
    
    /// Float literal
    Float(f64),
    
    /// String literal
    String(String),
    
    /// Path literal
    Path(PathBuf),
    
    /// Boolean literal
    Bool(bool),
    
    /// Null literal
    Null,
    
    /// Identifier
    Identifier(String),
    
    /// Attribute set
    AttrSet {
        recursive: bool,
        bindings: Vec<Binding>,
    },
    
    /// List
    List(Vec<NixAst>),
    
    /// Function (lambda)
    Function {
        param: FunctionParam,
        body: Box<NixAst>,
    },
    
    /// Function application
    Apply {
        function: Box<NixAst>,
        argument: Box<NixAst>,
    },
    
    /// Let expression
    Let {
        bindings: Vec<Binding>,
        body: Box<NixAst>,
    },
    
    /// If expression
    If {
        condition: Box<NixAst>,
        then_branch: Box<NixAst>,
        else_branch: Box<NixAst>,
    },
    
    /// With expression
    With {
        namespace: Box<NixAst>,
        body: Box<NixAst>,
    },
    
    /// Assert expression
    Assert {
        condition: Box<NixAst>,
        body: Box<NixAst>,
    },
    
    /// Binary operation
    BinaryOp {
        op: BinaryOperator,
        left: Box<NixAst>,
        right: Box<NixAst>,
    },
    
    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        operand: Box<NixAst>,
    },
    
    /// Attribute selection (a.b)
    Select {
        expr: Box<NixAst>,
        attr_path: AttrPath,
        default: Option<Box<NixAst>>,
    },
    
    /// Has attribute (a ? b)
    HasAttr {
        expr: Box<NixAst>,
        attr_path: AttrPath,
    },
    
    /// Import expression
    Import(Box<NixAst>),
    
    /// Inherit expression (for use in bindings)
    Inherit {
        from: Option<Box<NixAst>>,
        attrs: Vec<String>,
    },
}

/// Function parameter pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionParam {
    /// Simple identifier parameter
    Identifier(String),
    
    /// Pattern parameter with optional fields
    Pattern {
        fields: Vec<PatternField>,
        bind: Option<String>,
        ellipsis: bool,
    },
}

/// Field in a function pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternField {
    pub name: String,
    pub default: Option<NixAst>,
}

/// Attribute path (a.b.c)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrPath {
    pub segments: Vec<AttrPathSegment>,
}

/// Segment of an attribute path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttrPathSegment {
    /// Simple identifier
    Identifier(String),
    
    /// Dynamic segment (${expr})
    Dynamic(NixAst),
}

/// Binding in an attribute set or let expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub attr_path: AttrPath,
    pub value: BindingValue,
}

/// Value of a binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingValue {
    /// Regular value binding
    Value(NixAst),
    
    /// Inherit binding
    Inherit {
        from: Option<NixAst>,
        attrs: Vec<String>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    Implies,
    
    // List
    Concat,
    
    // Attribute set
    Update,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
}

/// Location information for AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: Option<PathBuf>,
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    pub length: usize,
}

/// AST node with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatedAst {
    pub ast: NixAst,
    pub location: Location,
}

/// Find an attribute by name in an attribute set
pub fn find_attribute(attrset: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
    // First, if this is the root node, find the actual attribute set
    let attr_set_node = if attrset.kind() == SyntaxKind::NODE_ATTR_SET {
        attrset.clone()
    } else {
        // Look for an attribute set child
        attrset.children().find(|child| child.kind() == SyntaxKind::NODE_ATTR_SET)?
    };
    
    // Now look for the attribute within the attribute set
    // In Nix, attributes are direct children of the attribute set
    attr_set_node.children()
        .find(|child| {
            // Check if this child's text starts with the attribute name
            let text = child.text().to_string();
            let trimmed = text.trim();
            trimmed.starts_with(name) && trimmed.contains('=')
        })
}

/// Get the value of an attribute
pub fn get_attribute_value(attrset: &SyntaxNode, name: &str) -> Option<SyntaxNode> {
    // First, if this is the root node, find the actual attribute set
    let attr_set_node = if attrset.kind() == SyntaxKind::NODE_ATTR_SET {
        attrset.clone()
    } else {
        attrset.children().find(|child| child.kind() == SyntaxKind::NODE_ATTR_SET)?
    };
    
    // Look through all children to find the attribute
    for child in attr_set_node.children() {
        let text = child.text().to_string();
        let trimmed = text.trim();
        
        // Check if this is our attribute
        if trimmed.starts_with(name) && trimmed.contains('=') {
            // Find the value part (after the equals sign)
            // This is a bit hacky but works for simple cases
            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                // The value is everything after the equals sign
                // We need to find the actual value node
                for value_child in child.children() {
                    // Skip identifier and equals
                    if value_child.kind() == SyntaxKind::NODE_STRING 
                        || value_child.kind() == SyntaxKind::NODE_ATTR_SET
                        || value_child.kind() == SyntaxKind::NODE_LIST {
                        return Some(value_child);
                    }
                }
            }
        }
    }
    
    None
}

/// Extract a string value from a node
pub fn extract_string_value(node: &SyntaxNode) -> Option<String> {
    // Handle different node types that might contain strings
    match node.kind() {
        SyntaxKind::NODE_STRING => {
            // Remove quotes and handle escape sequences
            let text = node.text().to_string();
            let trimmed = text.trim_start_matches('"').trim_end_matches('"');
            Some(trimmed.to_string())
        }
        _ => {
            // Sometimes the string is a child of the current node
            node.children()
                .find(|child| child.kind() == SyntaxKind::NODE_STRING)
                .and_then(|string_node| {
                    let text = string_node.text().to_string();
                    let trimmed = text.trim_start_matches('"').trim_end_matches('"');
                    Some(trimmed.to_string())
                })
        }
    }
}

/// Check if a node is an attribute set
pub fn is_attrset(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::NODE_ATTR_SET
}

/// Check if a node is a lambda
pub fn is_lambda(node: &SyntaxNode) -> bool {
    node.kind() == SyntaxKind::NODE_LAMBDA
}

/// Get the parameter of a lambda
pub fn get_lambda_param(lambda: &SyntaxNode) -> Option<String> {
    lambda.children()
        .next()
        .filter(|child| child.kind() == SyntaxKind::NODE_IDENT)
        .map(|ident| ident.text().to_string())
}

/// Get the body of a lambda
pub fn get_lambda_body(lambda: &SyntaxNode) -> Option<SyntaxNode> {
    lambda.children().nth(1)
}

/// Walk the AST and visit each node
pub fn walk_ast<F>(node: &SyntaxNode, visitor: &mut F)
where
    F: FnMut(&SyntaxNode),
{
    visitor(node);
    for child in node.children() {
        walk_ast(&child, visitor);
    }
}

/// Find all nodes of a specific kind
pub fn find_all_by_kind(root: &SyntaxNode, kind: SyntaxKind) -> Vec<SyntaxNode> {
    let mut results = Vec::new();
    walk_ast(root, &mut |node| {
        if node.kind() == kind {
            results.push(node.clone());
        }
    });
    results
}

/// Extract all imports from a module
pub fn extract_imports(module: &SyntaxNode) -> Vec<String> {
    let mut imports = Vec::new();
    
    if let Some(imports_attr) = get_attribute_value(module, "imports") {
        if imports_attr.kind() == SyntaxKind::NODE_LIST {
            for child in imports_attr.children() {
                if let Some(import_path) = extract_string_value(&child) {
                    imports.push(import_path);
                }
            }
        }
    }
    
    imports
}

/// Check if a node contains a specific function call
pub fn contains_function_call(node: &SyntaxNode, function_name: &str) -> bool {
    let mut found = false;
    walk_ast(node, &mut |n| {
        if n.kind() == SyntaxKind::NODE_APPLY {
            let text = n.text().to_string();
            if text.contains(function_name) {
                found = true;
            }
        }
    });
    found
}

/// Extract attribute paths (e.g., `foo.bar.baz`)
pub fn extract_attr_path(node: &SyntaxNode) -> Option<Vec<String>> {
    if node.kind() == SyntaxKind::NODE_ATTRPATH {
        let parts: Vec<String> = node.children()
            .filter(|child| child.kind() == SyntaxKind::NODE_IDENT)
            .map(|ident| ident.text().to_string())
            .collect();
        
        if parts.is_empty() {
            None
        } else {
            Some(parts)
        }
    } else {
        None
    }
}

/// Create a new attribute set node
pub fn create_attrset(attrs: Vec<(&str, &str)>) -> String {
    let mut result = String::from("{\n");
    
    for (key, value) in attrs {
        result.push_str(&format!("  {} = {};\n", key, value));
    }
    
    result.push('}');
    result
}

/// Create a new lambda expression
pub fn create_lambda(params: &[&str], body: &str) -> String {
    let param_str = params.join(": ");
    format!("{}: {}", param_str, body)
}

/// Convert from rnix SyntaxNode to our AST representation
pub fn from_syntax_node(node: &SyntaxNode) -> Result<NixAst, AstError> {
    match node.kind() {
        SyntaxKind::NODE_LITERAL => parse_literal(node),
        SyntaxKind::NODE_IDENT => parse_identifier(node),
        SyntaxKind::NODE_ATTR_SET => parse_attr_set(node),
        SyntaxKind::NODE_LIST => parse_list(node),
        SyntaxKind::NODE_LAMBDA => parse_function(node),
        SyntaxKind::NODE_APPLY => parse_apply(node),
        SyntaxKind::NODE_LET_IN => parse_let(node),
        SyntaxKind::NODE_IF_ELSE => parse_if(node),
        SyntaxKind::NODE_WITH => parse_with(node),
        SyntaxKind::NODE_ASSERT => parse_assert(node),
        SyntaxKind::NODE_BIN_OP => parse_binary_op(node),
        SyntaxKind::NODE_UNARY_OP => parse_unary_op(node),
        SyntaxKind::NODE_SELECT => parse_select(node),
        SyntaxKind::NODE_HAS_ATTR => parse_has_attr(node),
        _ => Err(AstError::UnsupportedNode(format!("{:?}", node.kind()))),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("Unsupported node type: {0}")]
    UnsupportedNode(String),
    
    #[error("Invalid node structure: {0}")]
    InvalidStructure(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Placeholder implementations for parsing functions
fn parse_literal(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement literal parsing
    Err(AstError::UnsupportedNode("literal parsing not implemented".to_string()))
}

fn parse_identifier(node: &SyntaxNode) -> Result<NixAst, AstError> {
    Ok(NixAst::Identifier(node.text().to_string()))
}

fn parse_attr_set(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement attribute set parsing
    Err(AstError::UnsupportedNode("attr set parsing not implemented".to_string()))
}

fn parse_list(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement list parsing
    Err(AstError::UnsupportedNode("list parsing not implemented".to_string()))
}

fn parse_function(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement function parsing
    Err(AstError::UnsupportedNode("function parsing not implemented".to_string()))
}

fn parse_apply(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement application parsing
    Err(AstError::UnsupportedNode("apply parsing not implemented".to_string()))
}

fn parse_let(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement let parsing
    Err(AstError::UnsupportedNode("let parsing not implemented".to_string()))
}

fn parse_if(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement if parsing
    Err(AstError::UnsupportedNode("if parsing not implemented".to_string()))
}

fn parse_with(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement with parsing
    Err(AstError::UnsupportedNode("with parsing not implemented".to_string()))
}

fn parse_assert(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement assert parsing
    Err(AstError::UnsupportedNode("assert parsing not implemented".to_string()))
}

fn parse_binary_op(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement binary op parsing
    Err(AstError::UnsupportedNode("binary op parsing not implemented".to_string()))
}

fn parse_unary_op(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement unary op parsing
    Err(AstError::UnsupportedNode("unary op parsing not implemented".to_string()))
}

fn parse_select(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement select parsing
    Err(AstError::UnsupportedNode("select parsing not implemented".to_string()))
}

fn parse_has_attr(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // TODO: Implement has_attr parsing
    Err(AstError::UnsupportedNode("has_attr parsing not implemented".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::NixFile;

    #[test]
    fn test_find_attribute() {
        let content = r#"{ foo = "bar"; baz = 42; }"#;
        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        
        let attrset = &file.ast;
        assert!(find_attribute(attrset, "foo").is_some());
        assert!(find_attribute(attrset, "missing").is_none());
    }

    #[test]
    fn test_extract_string_value() {
        let content = r#"{ message = "hello world"; }"#;
        let file = NixFile::parse_string(content.to_string(), None).unwrap();
        
        if let Some(value_node) = get_attribute_value(&file.ast, "message") {
            let value = extract_string_value(&value_node);
            assert_eq!(value, Some("hello world".to_string()));
        } else {
            panic!("Failed to find message attribute");
        }
    }

    #[test]
    fn test_create_attrset() {
        let attrs = vec![
            ("name", "\"my-package\""),
            ("version", "\"1.0.0\""),
        ];
        
        let result = create_attrset(attrs);
        assert!(result.contains("name = \"my-package\""));
        assert!(result.contains("version = \"1.0.0\""));
    }

    #[test]
    fn test_ast_serialization() {
        let ast = NixAst::AttrSet {
            recursive: false,
            bindings: vec![
                Binding {
                    attr_path: AttrPath {
                        segments: vec![AttrPathSegment::Identifier("foo".to_string())],
                    },
                    value: BindingValue::Value(NixAst::Integer(42)),
                },
            ],
        };

        let json = serde_json::to_string(&ast).unwrap();
        let deserialized: NixAst = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            NixAst::AttrSet { bindings, .. } => {
                assert_eq!(bindings.len(), 1);
            }
            _ => panic!("Expected AttrSet"),
        }
    }
} 