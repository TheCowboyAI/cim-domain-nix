//! AST helper functions for working with Nix syntax trees

use rnix::{SyntaxNode, SyntaxKind};
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

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
        /// Whether this is a recursive attribute set (rec { ... })
        recursive: bool,
        /// The bindings within the attribute set
        bindings: Vec<Binding>,
    },
    
    /// List
    List(Vec<NixAst>),
    
    /// Function (lambda)
    Function {
        /// The function parameter pattern
        param: FunctionParam,
        /// The function body expression
        body: Box<NixAst>,
    },
    
    /// Function application
    Apply {
        /// The function being applied
        function: Box<NixAst>,
        /// The argument being passed to the function
        argument: Box<NixAst>,
    },
    
    /// Let expression
    Let {
        /// The bindings introduced by the let
        bindings: Vec<Binding>,
        /// The body expression where bindings are in scope
        body: Box<NixAst>,
    },
    
    /// If expression
    If {
        /// The condition to evaluate
        condition: Box<NixAst>,
        /// Expression to evaluate if condition is true
        then_branch: Box<NixAst>,
        /// Expression to evaluate if condition is false
        else_branch: Box<NixAst>,
    },
    
    /// With expression
    With {
        /// The namespace to bring into scope
        namespace: Box<NixAst>,
        /// The body where the namespace is available
        body: Box<NixAst>,
    },
    
    /// Assert expression
    Assert {
        /// The condition that must be true
        condition: Box<NixAst>,
        /// The body to evaluate if assertion passes
        body: Box<NixAst>,
    },
    
    /// Binary operation
    BinaryOp {
        /// The binary operator
        op: BinaryOperator,
        /// The left operand
        left: Box<NixAst>,
        /// The right operand
        right: Box<NixAst>,
    },
    
    /// Unary operation
    UnaryOp {
        /// The unary operator
        op: UnaryOperator,
        /// The operand
        operand: Box<NixAst>,
    },
    
    /// Attribute selection (a.b)
    Select {
        /// The expression to select from
        expr: Box<NixAst>,
        /// The attribute path to select
        attr_path: AttrPath,
        /// Optional default value if attribute doesn't exist
        default: Option<Box<NixAst>>,
    },
    
    /// Has attribute (a ? b)
    HasAttr {
        /// The expression to check
        expr: Box<NixAst>,
        /// The attribute path to check for
        attr_path: AttrPath,
    },
    
    /// Import expression
    Import(Box<NixAst>),
    
    /// Inherit expression (for use in bindings)
    Inherit {
        /// Optional expression to inherit from
        from: Option<Box<NixAst>>,
        /// Attributes to inherit
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
        /// The fields in the pattern
        fields: Vec<PatternField>,
        /// Optional identifier to bind the whole argument to
        bind: Option<String>,
        /// Whether the pattern has an ellipsis (...)
        ellipsis: bool,
    },
}

/// Field in a function pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternField {
    /// The field name
    pub name: String,
    /// Optional default value for the field
    pub default: Option<NixAst>,
}

/// Attribute path (a.b.c)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttrPath {
    /// The segments of the path
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
    /// The attribute path being bound
    pub attr_path: AttrPath,
    /// The value being bound
    pub value: BindingValue,
}

/// Value of a binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BindingValue {
    /// Regular value binding
    Value(NixAst),
    
    /// Inherit binding
    Inherit {
        /// Optional expression to inherit from
        from: Option<NixAst>,
        /// Attributes to inherit
        attrs: Vec<String>,
    },
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    /// Addition operator (+)
    Add,
    /// Subtraction operator (-)
    Subtract,
    /// Multiplication operator (*)
    Multiply,
    /// Division operator (/)
    Divide,
    
    // Comparison
    /// Equality operator (==)
    Equal,
    /// Inequality operator (!=)
    NotEqual,
    /// Less than operator (<)
    Less,
    /// Less than or equal operator (<=)
    LessEqual,
    /// Greater than operator (>)
    Greater,
    /// Greater than or equal operator (>=)
    GreaterEqual,
    
    // Logical
    /// Logical AND operator (&&)
    And,
    /// Logical OR operator (||)
    Or,
    /// Logical implication operator (->)
    Implies,
    
    // List
    /// List concatenation operator (++)
    Concat,
    
    // Attribute set
    /// Attribute set update operator (//)
    Update,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Logical NOT operator (!)
    Not,
    /// Arithmetic negation operator (-)
    Negate,
}

/// Location information for AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// The file this node comes from
    pub file: Option<PathBuf>,
    /// Line number (1-based)
    pub line: usize,
    /// Column number (1-based)
    pub column: usize,
    /// Byte offset in the file
    pub offset: usize,
    /// Length in bytes
    pub length: usize,
}

/// AST node with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocatedAst {
    /// The AST node
    pub ast: NixAst,
    /// Location information
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
        result.push_str(&format!("  {key} = {value};\n"));
    }
    
    result.push('}');
    result
}

/// Create a new lambda expression
pub fn create_lambda(params: &[&str], body: &str) -> String {
    let param_str = params.join(": ");
    format!("{param_str}: {body}")
}

/// Convert from rnix SyntaxNode to our AST representation
pub fn from_syntax_node(node: &SyntaxNode) -> Result<NixAst, AstError> {
    match node.kind() {
        SyntaxKind::NODE_LITERAL => parse_literal(node),
        SyntaxKind::NODE_IDENT => parse_identifier(node),
        SyntaxKind::NODE_ATTR_SET => parse_attr_set(node),
        SyntaxKind::NODE_LIST => parse_list(node),
        SyntaxKind::NODE_LAMBDA => parse_function(node),
        SyntaxKind::NODE_APPLY => {
            // Check if this is an import expression
            let children: Vec<_> = node.children().collect();
            if !children.is_empty() {
                if let Ok(NixAst::Identifier(name)) = from_syntax_node(&children[0]) {
                    if name == "import" && children.len() > 1 {
                        return Ok(NixAst::Import(Box::new(from_syntax_node(&children[1])?)));
                    }
                }
            }
            parse_apply(node)
        }
        SyntaxKind::NODE_LET_IN => parse_let(node),
        SyntaxKind::NODE_IF_ELSE => parse_if(node),
        SyntaxKind::NODE_WITH => parse_with(node),
        SyntaxKind::NODE_ASSERT => parse_assert(node),
        SyntaxKind::NODE_BIN_OP => parse_binary_op(node),
        SyntaxKind::NODE_UNARY_OP => parse_unary_op(node),
        SyntaxKind::NODE_SELECT => parse_select(node),
        SyntaxKind::NODE_HAS_ATTR => parse_has_attr(node),
        SyntaxKind::NODE_STRING => parse_string(node),
        SyntaxKind::NODE_PATH => parse_path(node),
        SyntaxKind::NODE_PAREN => parse_paren(node),
        SyntaxKind::NODE_ROOT => parse_root(node),
        SyntaxKind::NODE_INHERIT => {
            // Inherit nodes are handled within attr sets, but if we encounter one directly,
            // we can return a placeholder
            Err(AstError::InvalidStructure("Inherit must be inside an attribute set".to_string()))
        }
        // Handle tokens that might appear as direct children
        SyntaxKind::TOKEN_INTEGER => {
            let text = node.text().to_string();
            let value = text.parse::<i64>()
                .map_err(|e| AstError::ParseError(format!("Invalid integer: {e}")))?;
            Ok(NixAst::Integer(value))
        }
        SyntaxKind::TOKEN_FLOAT => {
            let text = node.text().to_string();
            let value = text.parse::<f64>()
                .map_err(|e| AstError::ParseError(format!("Invalid float: {e}")))?;
            Ok(NixAst::Float(value))
        }
        _ => {
            // Check if it's a boolean or null identifier
            let text = node.text().to_string();
            match text.as_str() {
                "true" => Ok(NixAst::Bool(true)),
                "false" => Ok(NixAst::Bool(false)),
                "null" => Ok(NixAst::Null),
                _ => {
                    // Try to find a child that we can parse
                    for child in node.children() {
                        if let Ok(ast) = from_syntax_node(&child) {
                            return Ok(ast);
                        }
                    }
                    
                    Err(AstError::UnsupportedNode(format!("Unsupported node kind: {:?}", node.kind())))
                }
            }
        }
    }
}

fn parse_paren(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // Parentheses just wrap an expression, so we parse the inner expression
    node.children()
        .next()
        .ok_or_else(|| AstError::InvalidStructure("Empty parentheses".to_string()))
        .and_then(|child| from_syntax_node(&child))
}

fn parse_root(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // The root node contains the top-level expression
    node.children()
        .next()
        .ok_or_else(|| AstError::InvalidStructure("Empty root node".to_string()))
        .and_then(|child| from_syntax_node(&child))
}

fn parse_import(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // Import expressions have the form: import <expr>
    // Find the expression after the import keyword
    let mut found_import = false;
    for child in node.children() {
        if found_import {
            return Ok(NixAst::Import(Box::new(from_syntax_node(&child)?)));
        }
        if child.text().to_string().trim() == "import" {
            found_import = true;
        }
    }
    
    // If we have children, try to parse the first one as the import expression
    if let Some(child) = node.children().next() {
        return Ok(NixAst::Import(Box::new(from_syntax_node(&child)?)));
    }
    
    Err(AstError::InvalidStructure("Import missing expression".to_string()))
}

/// Errors that can occur during AST parsing
#[derive(Debug, Clone, thiserror::Error)]
pub enum AstError {
    /// An unsupported node type was encountered
    #[error("Unsupported node type: {0}")]
    UnsupportedNode(String),
    
    /// The node structure was invalid or unexpected
    #[error("Invalid node structure: {0}")]
    InvalidStructure(String),
    
    /// An error occurred while parsing a value
    #[error("Parse error: {0}")]
    ParseError(String),
}

fn parse_literal(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // Handle different types of literals based on the first token
    if let Some(token) = node.first_token() {
        match token.kind() {
            SyntaxKind::TOKEN_INTEGER => {
                let text = token.text().to_string();
                let value = text.parse::<i64>()
                    .map_err(|e| AstError::ParseError(format!("Invalid integer: {e}")))?;
                Ok(NixAst::Integer(value))
            }
            SyntaxKind::TOKEN_FLOAT => {
                let text = token.text().to_string();
                let value = text.parse::<f64>()
                    .map_err(|e| AstError::ParseError(format!("Invalid float: {e}")))?;
                Ok(NixAst::Float(value))
            }
            _ => {
                // Check children for other literal types
                for child in node.children() {
                    match child.kind() {
                        SyntaxKind::NODE_STRING => return parse_string(&child),
                        SyntaxKind::NODE_PATH => return parse_path(&child),
                        _ => {}
                    }
                }
                
                // Check if it's a boolean or null by text
                let text = node.text().to_string();
                match text.as_str() {
                    "true" => Ok(NixAst::Bool(true)),
                    "false" => Ok(NixAst::Bool(false)),
                    "null" => Ok(NixAst::Null),
                    _ => Err(AstError::UnsupportedNode(format!("Unknown literal: {:?}", node.kind())))
                }
            }
        }
    } else {
        Err(AstError::InvalidStructure("Literal node has no tokens".to_string()))
    }
}

fn parse_string(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let mut content = String::new();
    
    // In rnix, string content is stored in tokens
    for token in node.children_with_tokens() {
        if let Some(token) = token.as_token() {
            match token.kind() {
                SyntaxKind::TOKEN_STRING_CONTENT => {
                    content.push_str(&token.text());
                }
                _ => {} // Ignore string delimiters and other tokens
            }
        }
    }
    
    // If no content tokens found, try to extract from the text directly
    if content.is_empty() {
        let text = node.text().to_string();
        // Remove quotes if present
        if text.starts_with('"') && text.ends_with('"') && text.len() > 1 {
            content = text[1..text.len()-1].to_string();
        } else if text.starts_with("''") && text.ends_with("''") && text.len() > 3 {
            // Handle multiline strings
            content = text[2..text.len()-2].to_string();
        } else {
            content = text;
        }
    }
    
    Ok(NixAst::String(content))
}

fn parse_path(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let path_text = node.text().to_string();
    Ok(NixAst::Path(PathBuf::from(path_text)))
}

fn parse_identifier(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let text = node.text().to_string();
    // Check for keywords that should be parsed as literals
    match text.as_str() {
        "true" => Ok(NixAst::Bool(true)),
        "false" => Ok(NixAst::Bool(false)),
        "null" => Ok(NixAst::Null),
        _ => Ok(NixAst::Identifier(text)),
    }
}

fn parse_attr_set(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let mut bindings = Vec::new();
    let mut recursive = false;
    
    // Check if it's a recursive attribute set by looking for 'rec' keyword
    for token in node.children_with_tokens() {
        if let Some(token) = token.as_token() {
            if token.kind() == SyntaxKind::TOKEN_REC {
                recursive = true;
                break;
            }
        }
    }
    
    // Parse all bindings
    for child in node.children() {
        match child.kind() {
            SyntaxKind::NODE_ATTRPATH_VALUE => {
                if let Ok(binding) = parse_binding(&child) {
                    bindings.push(binding);
                }
            }
            SyntaxKind::NODE_INHERIT => {
                if let Ok(inherit_bindings) = parse_inherit(&child) {
                    bindings.extend(inherit_bindings);
                }
            }
            _ => {}
        }
    }
    
    Ok(NixAst::AttrSet { recursive, bindings })
}

fn parse_binding(node: &SyntaxNode) -> Result<Binding, AstError> {
    let mut attr_path = None;
    let mut value_node = None;
    
    for child in node.children() {
        match child.kind() {
            SyntaxKind::NODE_ATTRPATH => {
                attr_path = Some(parse_attr_path(&child)?);
            }
            _ => {
                // The value is any other node after the attr path
                if attr_path.is_some() && value_node.is_none() {
                    value_node = Some(child);
                }
            }
        }
    }
    
    let attr_path = attr_path
        .ok_or_else(|| AstError::InvalidStructure("Binding missing attribute path".to_string()))?;
    let value_node = value_node
        .ok_or_else(|| AstError::InvalidStructure("Binding missing value".to_string()))?;
    
    let value = from_syntax_node(&value_node)?;
    
    Ok(Binding {
        attr_path,
        value: BindingValue::Value(value),
    })
}

fn parse_attr_path(node: &SyntaxNode) -> Result<AttrPath, AstError> {
    let mut segments = Vec::new();
    
    for child in node.children() {
        match child.kind() {
            SyntaxKind::NODE_IDENT => {
                segments.push(AttrPathSegment::Identifier(child.text().to_string()));
            }
            SyntaxKind::NODE_STRING => {
                // String literals in attribute paths
                if let Ok(NixAst::String(s)) = parse_string(&child) {
                    segments.push(AttrPathSegment::Identifier(s));
                }
            }
            SyntaxKind::NODE_DYNAMIC => {
                let expr = parse_dynamic(&child)?;
                segments.push(AttrPathSegment::Dynamic(expr));
            }
            _ => {}
        }
    }
    
    if segments.is_empty() {
        Err(AstError::InvalidStructure("Empty attribute path".to_string()))
    } else {
        Ok(AttrPath { segments })
    }
}

fn parse_dynamic(node: &SyntaxNode) -> Result<NixAst, AstError> {
    // Dynamic attributes are wrapped in ${}, parse the expression inside
    node.children()
        .next()
        .ok_or_else(|| AstError::InvalidStructure("Empty dynamic attribute".to_string()))
        .and_then(|child| from_syntax_node(&child))
}

fn parse_inherit(node: &SyntaxNode) -> Result<Vec<Binding>, AstError> {
    let mut bindings = Vec::new();
    let mut from_expr = None;
    let mut attrs = Vec::new();
    
    // First pass: find the from expression if any
    for child in node.children() {
        if child.kind() == SyntaxKind::NODE_INHERIT_FROM {
            // The from expression is inside this node
            if let Some(expr_node) = child.children().next() {
                from_expr = Some(Box::new(from_syntax_node(&expr_node)?));
            }
            break; // Only one from expression allowed
        }
    }
    
    // Second pass: collect all identifiers that are not inside NODE_INHERIT_FROM
    for child in node.children() {
        if child.kind() == SyntaxKind::NODE_IDENT {
            attrs.push(child.text().to_string());
        }
    }
    
    // Create individual bindings for each inherited attribute
    for attr in &attrs {
        let binding = Binding {
            attr_path: AttrPath {
                segments: vec![AttrPathSegment::Identifier(attr.clone())],
            },
            value: BindingValue::Inherit {
                from: from_expr.as_ref().map(|e| (**e).clone()),
                attrs: vec![attr.clone()],
            },
        };
        bindings.push(binding);
    }
    
    Ok(bindings)
}

fn parse_list(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let mut elements = Vec::new();
    
    for child in node.children() {
        // Skip whitespace and other trivia
        match from_syntax_node(&child) {
            Ok(elem) => elements.push(elem),
            Err(_) => {} // Skip unparseable elements
        }
    }
    
    Ok(NixAst::List(elements))
}

fn parse_function(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.is_empty() {
        return Err(AstError::InvalidStructure("Function has no children".to_string()));
    }
    
    let param = match children[0].kind() {
        SyntaxKind::NODE_IDENT | SyntaxKind::NODE_IDENT_PARAM => {
            FunctionParam::Identifier(children[0].text().to_string())
        }
        SyntaxKind::NODE_PATTERN => {
            parse_pattern(&children[0])?
        }
        _ => {
            return Err(AstError::InvalidStructure("Invalid function parameter".to_string()));
        }
    };
    
    // The body is the last child (after the parameter)
    let body = children.get(1)
        .ok_or_else(|| AstError::InvalidStructure("Function missing body".to_string()))?;
    
    Ok(NixAst::Function {
        param,
        body: Box::new(from_syntax_node(body)?),
    })
}

fn parse_pattern(node: &SyntaxNode) -> Result<FunctionParam, AstError> {
    let mut fields = Vec::new();
    let mut bind = None;
    let mut ellipsis = false;
    
    for child in node.children() {
        match child.kind() {
            SyntaxKind::NODE_PAT_ENTRY => {
                if let Ok(field) = parse_pattern_field(&child) {
                    fields.push(field);
                }
            }
            SyntaxKind::NODE_PAT_BIND => {
                // The bind is usually after an @ symbol
                if let Some(ident) = child.children().find(|c| c.kind() == SyntaxKind::NODE_IDENT) {
                    bind = Some(ident.text().to_string());
                }
            }
            _ => {}
        }
    }
    
    // Check for ellipsis token
    for token in node.children_with_tokens() {
        if let Some(token) = token.as_token() {
            if token.kind() == SyntaxKind::TOKEN_ELLIPSIS {
                ellipsis = true;
            }
        }
    }
    
    Ok(FunctionParam::Pattern { fields, bind, ellipsis })
}

fn parse_pattern_field(node: &SyntaxNode) -> Result<PatternField, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.is_empty() {
        return Err(AstError::InvalidStructure("Pattern field has no children".to_string()));
    }
    
    let name = match children[0].kind() {
        SyntaxKind::NODE_IDENT => children[0].text().to_string(),
        _ => return Err(AstError::InvalidStructure("Pattern field missing name".to_string())),
    };
    
    // Check if there's a default value (after a ? token)
    let default = if children.len() > 1 {
        Some(from_syntax_node(&children[1])?)
    } else {
        None
    };
    
    Ok(PatternField { name, default })
}

fn parse_apply(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("Apply needs at least 2 children".to_string()));
    }
    
    let function = from_syntax_node(&children[0])?;
    let argument = from_syntax_node(&children[1])?;
    
    Ok(NixAst::Apply {
        function: Box::new(function),
        argument: Box::new(argument),
    })
}

fn parse_let(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let mut bindings = Vec::new();
    let mut body = None;
    let mut found_in = false;
    
    // Parse children to find bindings and body
    for child in node.children() {
        // Check if we've hit the 'in' keyword area
        if !found_in {
            match child.kind() {
                SyntaxKind::NODE_ATTRPATH_VALUE => {
                    if let Ok(binding) = parse_binding(&child) {
                        bindings.push(binding);
                    }
                }
                SyntaxKind::NODE_INHERIT => {
                    if let Ok(inherit_bindings) = parse_inherit(&child) {
                        bindings.extend(inherit_bindings);
                    }
                }
                _ => {
                    // This might be the body after 'in'
                    // Check tokens to see if we've passed 'in'
                    for token in node.children_with_tokens() {
                        if let Some(token) = token.as_token() {
                            if token.text() == "in" {
                                found_in = true;
                                break;
                            }
                        }
                    }
                    
                    // If we found 'in', this child might be the body
                    if found_in {
                        body = Some(child);
                        break;
                    }
                }
            }
        } else {
            // Everything after 'in' is the body
            body = Some(child);
            break;
        }
    }
    
    // If we didn't find a body explicitly, the last child might be it
    if body.is_none() {
        let children: Vec<_> = node.children().collect();
        if let Some(last) = children.last() {
            // Check if the last child is not a binding
            if !matches!(last.kind(), SyntaxKind::NODE_ATTRPATH_VALUE | SyntaxKind::NODE_INHERIT) {
                body = Some(last.clone());
            }
        }
    }
    
    let body = body
        .ok_or_else(|| AstError::InvalidStructure("Let expression missing body".to_string()))?;
    
    Ok(NixAst::Let {
        bindings,
        body: Box::new(from_syntax_node(&body)?),
    })
}

fn parse_if(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 3 {
        return Err(AstError::InvalidStructure("If expression needs condition, then, and else parts".to_string()));
    }
    
    // The structure is: condition, then expression, else expression
    let condition = from_syntax_node(&children[0])?;
    let then_branch = from_syntax_node(&children[1])?;
    let else_branch = from_syntax_node(&children[2])?;
    
    Ok(NixAst::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    })
}

fn parse_with(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("With expression needs namespace and body".to_string()));
    }
    
    let namespace = from_syntax_node(&children[0])?;
    let body = from_syntax_node(&children[1])?;
    
    Ok(NixAst::With {
        namespace: Box::new(namespace),
        body: Box::new(body),
    })
}

fn parse_assert(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("Assert expression needs condition and body".to_string()));
    }
    
    let condition = from_syntax_node(&children[0])?;
    let body = from_syntax_node(&children[1])?;
    
    Ok(NixAst::Assert {
        condition: Box::new(condition),
        body: Box::new(body),
    })
}

fn parse_binary_op(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("Binary operation needs at least 2 operands".to_string()));
    }
    
    // Find the operator token
    let mut op = None;
    for token in node.children_with_tokens() {
        if let Some(token) = token.as_token() {
            op = match token.kind() {
                SyntaxKind::TOKEN_ADD => Some(BinaryOperator::Add),
                SyntaxKind::TOKEN_SUB => Some(BinaryOperator::Subtract),
                SyntaxKind::TOKEN_MUL => Some(BinaryOperator::Multiply),
                SyntaxKind::TOKEN_DIV => Some(BinaryOperator::Divide),
                SyntaxKind::TOKEN_EQUAL => Some(BinaryOperator::Equal),
                SyntaxKind::TOKEN_NOT_EQUAL => Some(BinaryOperator::NotEqual),
                SyntaxKind::TOKEN_LESS => Some(BinaryOperator::Less),
                SyntaxKind::TOKEN_LESS_OR_EQ => Some(BinaryOperator::LessEqual),
                SyntaxKind::TOKEN_MORE => Some(BinaryOperator::Greater),
                SyntaxKind::TOKEN_MORE_OR_EQ => Some(BinaryOperator::GreaterEqual),
                SyntaxKind::TOKEN_AND_AND => Some(BinaryOperator::And),
                SyntaxKind::TOKEN_OR_OR => Some(BinaryOperator::Or),
                SyntaxKind::TOKEN_IMPLICATION => Some(BinaryOperator::Implies),
                SyntaxKind::TOKEN_CONCAT => Some(BinaryOperator::Concat),
                SyntaxKind::TOKEN_UPDATE => Some(BinaryOperator::Update),
                _ => None,
            };
            if op.is_some() {
                break;
            }
        }
    }
    
    let op = op
        .ok_or_else(|| AstError::InvalidStructure("Binary operation missing operator".to_string()))?;
    
    let left = from_syntax_node(&children[0])?;
    let right = from_syntax_node(&children[1])?;
    
    Ok(NixAst::BinaryOp {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_unary_op(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.is_empty() {
        return Err(AstError::InvalidStructure("Unary operation needs an operand".to_string()));
    }
    
    // Find the operator token
    let mut op = None;
    for token in node.children_with_tokens() {
        if let Some(token) = token.as_token() {
            op = match token.kind() {
                SyntaxKind::TOKEN_INVERT => Some(UnaryOperator::Not),
                SyntaxKind::TOKEN_SUB => Some(UnaryOperator::Negate),
                _ => None,
            };
            if op.is_some() {
                break;
            }
        }
    }
    
    let op = op
        .ok_or_else(|| AstError::InvalidStructure("Unary operation missing operator".to_string()))?;
    
    let operand = from_syntax_node(&children[0])?;
    
    Ok(NixAst::UnaryOp {
        op,
        operand: Box::new(operand),
    })
}

fn parse_select(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("Select needs expression and attribute path".to_string()));
    }
    
    let expr = from_syntax_node(&children[0])?;
    let attr_path = if children[1].kind() == SyntaxKind::NODE_ATTRPATH {
        parse_attr_path(&children[1])?
    } else {
        // Try to create a simple attr path from an identifier
        AttrPath {
            segments: vec![AttrPathSegment::Identifier(children[1].text().to_string())],
        }
    };
    
    // Check for default value (after 'or' keyword)
    let default = if children.len() > 2 {
        // Look for the 'or' keyword in tokens
        let mut found_or = false;
        for token in node.children_with_tokens() {
            if let Some(token) = token.as_token() {
                if token.text() == "or" {
                    found_or = true;
                    break;
                }
            }
        }
        
        if found_or {
            Some(Box::new(from_syntax_node(&children[2])?))
        } else {
            None
        }
    } else {
        None
    };
    
    Ok(NixAst::Select {
        expr: Box::new(expr),
        attr_path,
        default,
    })
}

fn parse_has_attr(node: &SyntaxNode) -> Result<NixAst, AstError> {
    let children: Vec<_> = node.children().collect();
    
    if children.len() < 2 {
        return Err(AstError::InvalidStructure("Has attr needs expression and attribute path".to_string()));
    }
    
    let expr = from_syntax_node(&children[0])?;
    let attr_path = if children[1].kind() == SyntaxKind::NODE_ATTRPATH {
        parse_attr_path(&children[1])?
    } else {
        // Try to create a simple attr path from an identifier
        AttrPath {
            segments: vec![AttrPathSegment::Identifier(children[1].text().to_string())],
        }
    };
    
    Ok(NixAst::HasAttr {
        expr: Box::new(expr),
        attr_path,
    })
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