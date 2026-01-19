// Copyright 2025 Cowboy AI, LLC.

//! Nix Abstract Syntax Tree (AST) Representation
//!
//! This module provides a Rust-friendly wrapper around the rnix parser's
//! syntax tree. It converts rnix's raw AST into our domain types.
//!
//! ## Architecture
//!
//! ```text
//! Nix Source → rnix Parser → Rowan AST → Our AST → Domain Objects
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::nix::ast::*;
//!
//! let source = "{ x = 1; y = 2; }";
//! let ast = NixAst::parse(source).unwrap();
//! let root = ast.root();
//! ```

use rnix::{SyntaxKind, SyntaxNode};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Errors that can occur during AST operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum AstError {
    /// Parse error from rnix
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Unexpected node type
    #[error("Unexpected node type: expected {expected}, got {got}")]
    UnexpectedNodeType { expected: String, got: String },

    /// Missing node
    #[error("Missing expected node: {0}")]
    MissingNode(String),

    /// Invalid syntax
    #[error("Invalid syntax: {0}")]
    InvalidSyntax(String),
}

/// Result type for AST operations
pub type Result<T> = std::result::Result<T, AstError>;

// ============================================================================
// NixAst - Top-level AST wrapper
// ============================================================================

/// Nix Abstract Syntax Tree
///
/// This wraps rnix's Parse result and provides a convenient interface
/// for working with the parsed Nix code.
#[derive(Clone)]
pub struct NixAst {
    /// The rnix parse result
    parse: rnix::Parse<rnix::Root>,
    /// Original source text
    source: String,
}

impl std::fmt::Debug for NixAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NixAst")
            .field("source", &self.source)
            .finish()
    }
}

impl NixAst {
    /// Parse Nix source code into an AST
    pub fn parse(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        let parse = rnix::Root::parse(&source);

        // Check for parse errors
        let errors: Vec<_> = parse.errors().into_iter().collect();
        if !errors.is_empty() {
            let error_msg = errors
                .into_iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            return Err(AstError::ParseError(error_msg));
        }

        Ok(Self { parse, source })
    }

    /// Get the root node
    pub fn root(&self) -> NixNode {
        NixNode::new(self.parse.syntax().clone())
    }

    /// Get the root expression
    pub fn root_expr(&self) -> Option<NixExpression> {
        self.parse.tree().expr().map(|expr| NixExpression::from_rnix(expr))
    }

    /// Get the original source text
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Check if the AST has any errors
    pub fn has_errors(&self) -> bool {
        !self.parse.errors().is_empty()
    }
}

// ============================================================================
// NixNode - Generic syntax node wrapper
// ============================================================================

/// Generic Nix syntax node
///
/// This wraps a rowan SyntaxNode and provides convenient methods
/// for traversing the tree.
#[derive(Debug, Clone)]
pub struct NixNode {
    /// The underlying syntax node
    node: SyntaxNode,
}

impl NixNode {
    /// Create a new node wrapper
    pub fn new(node: SyntaxNode) -> Self {
        Self { node }
    }

    /// Get the node kind
    pub fn kind(&self) -> SyntaxKind {
        self.node.kind()
    }

    /// Get the node text
    pub fn text(&self) -> String {
        self.node.text().to_string()
    }

    /// Get child nodes
    pub fn children(&self) -> Vec<NixNode> {
        self.node
            .children()
            .map(|n| NixNode::new(n))
            .collect()
    }

    /// Get the first child
    pub fn first_child(&self) -> Option<NixNode> {
        self.node.first_child().map(NixNode::new)
    }

    /// Get the parent node
    pub fn parent(&self) -> Option<NixNode> {
        self.node.parent().map(NixNode::new)
    }

    /// Check if this is a specific kind
    pub fn is_kind(&self, kind: SyntaxKind) -> bool {
        self.kind() == kind
    }

    /// Get the underlying syntax node (for advanced use)
    pub fn syntax(&self) -> &SyntaxNode {
        &self.node
    }
}

impl fmt::Display for NixNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind(), self.text())
    }
}

// ============================================================================
// NixExpression - Typed expression wrapper
// ============================================================================

/// Nix Expression
///
/// This represents a typed Nix expression (the result of evaluating
/// Nix syntax).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NixExpression {
    /// Literal value (string, int, float, bool, null, path)
    Literal(LiteralExpr),
    /// Identifier/variable reference
    Ident(String),
    /// Attribute set { ... }
    AttrSet(AttrSetExpr),
    /// List [ ... ]
    List(ListExpr),
    /// Function application (f x)
    Apply(Box<ApplyExpr>),
    /// Lambda (x: body) or ({ x, y }: body)
    Lambda(Box<LambdaExpr>),
    /// Let expression (let x = 1; in x)
    LetIn(Box<LetInExpr>),
    /// With expression (with pkgs; ...)
    With(Box<WithExpr>),
    /// If-then-else
    IfThenElse(Box<IfThenElseExpr>),
    /// Binary operation (a + b, a // b, etc.)
    BinOp(Box<BinOpExpr>),
    /// Unary operation (!a, -a)
    UnaryOp(Box<UnaryOpExpr>),
    /// Attribute access (a.b.c)
    Select(Box<SelectExpr>),
    /// String interpolation
    StringInterpolation(StringInterpolationExpr),
    /// Path interpolation
    PathInterpolation(PathInterpolationExpr),
}

impl NixExpression {
    /// Convert from rnix expression
    pub fn from_rnix(_expr: rnix::ast::Expr) -> Self {
        // For now, return a placeholder
        // Full implementation will parse each expression type
        NixExpression::Ident("placeholder".to_string())
    }

    /// Get a human-readable type name
    pub fn type_name(&self) -> &'static str {
        match self {
            NixExpression::Literal(_) => "literal",
            NixExpression::Ident(_) => "identifier",
            NixExpression::AttrSet(_) => "attrset",
            NixExpression::List(_) => "list",
            NixExpression::Apply(_) => "apply",
            NixExpression::Lambda(_) => "lambda",
            NixExpression::LetIn(_) => "let-in",
            NixExpression::With(_) => "with",
            NixExpression::IfThenElse(_) => "if-then-else",
            NixExpression::BinOp(_) => "binary-op",
            NixExpression::UnaryOp(_) => "unary-op",
            NixExpression::Select(_) => "select",
            NixExpression::StringInterpolation(_) => "string-interpolation",
            NixExpression::PathInterpolation(_) => "path-interpolation",
        }
    }
}

// ============================================================================
// Expression Types
// ============================================================================

/// Literal expression (string, int, float, bool, null, path)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralExpr {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    Null,
    Path(String),
}

/// Attribute set expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttrSetExpr {
    pub recursive: bool,
    pub bindings: Vec<Binding>,
}

/// Attribute binding (key = value)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub key: String,
    pub value: NixExpression,
}

/// List expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExpr {
    pub elements: Vec<NixExpression>,
}

/// Function application (f x)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplyExpr {
    pub function: NixExpression,
    pub argument: NixExpression,
}

/// Lambda expression (x: body)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LambdaExpr {
    pub param: LambdaParam,
    pub body: NixExpression,
}

/// Lambda parameter
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LambdaParam {
    /// Simple parameter (x: ...)
    Ident(String),
    /// Pattern parameter ({ x, y }: ...)
    Pattern {
        bindings: Vec<String>,
        at_param: Option<String>,
    },
}

/// Let-in expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LetInExpr {
    pub bindings: Vec<Binding>,
    pub body: NixExpression,
}

/// With expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WithExpr {
    pub namespace: NixExpression,
    pub body: NixExpression,
}

/// If-then-else expression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IfThenElseExpr {
    pub condition: NixExpression,
    pub then_expr: NixExpression,
    pub else_expr: NixExpression,
}

/// Binary operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BinOpExpr {
    pub op: BinOp,
    pub left: NixExpression,
    pub right: NixExpression,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    Add,        // +
    Sub,        // -
    Mul,        // *
    Div,        // /
    Update,     // //
    Concat,     // ++
    Equal,      // ==
    NotEqual,   // !=
    Less,       // <
    LessEq,     // <=
    Greater,    // >
    GreaterEq,  // >=
    And,        // &&
    Or,         // ||
    Implication, // ->
}

/// Unary operation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnaryOpExpr {
    pub op: UnaryOp,
    pub expr: NixExpression,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Not,    // !
    Negate, // -
}

/// Select expression (attribute access)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectExpr {
    pub expr: NixExpression,
    pub path: Vec<String>,
    pub default: Option<NixExpression>,
}

/// String interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringInterpolationExpr {
    pub parts: Vec<StringPart>,
}

/// Part of an interpolated string
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    Interpolation(NixExpression),
}

/// Path interpolation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PathInterpolationExpr {
    pub parts: Vec<PathPart>,
}

/// Part of an interpolated path
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PathPart {
    Literal(String),
    Interpolation(NixExpression),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_attrset() {
        let source = "{ x = 1; y = 2; }";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_simple_list() {
        let source = "[ 1 2 3 ]";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_simple_let() {
        let source = "let x = 1; in x";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_simple_lambda() {
        let source = "x: x + 1";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_pattern_lambda() {
        let source = "{ x, y }: x + y";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_if_then_else() {
        let source = "if true then 1 else 2";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_with() {
        let source = "with pkgs; [ hello ]";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_string_interpolation() {
        let source = r#""hello ${name}""#;
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_rec_attrset() {
        let source = "rec { x = 1; y = x + 1; }";
        let ast = NixAst::parse(source);
        assert!(ast.is_ok());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "{ x = ";
        let ast = NixAst::parse(source);
        assert!(ast.is_err());
    }

    #[test]
    fn test_ast_root() {
        let source = "{ x = 1; }";
        let ast = NixAst::parse(source).unwrap();
        let root = ast.root();
        assert!(!root.text().is_empty());
    }

    #[test]
    fn test_ast_source() {
        let source = "{ x = 1; }";
        let ast = NixAst::parse(source).unwrap();
        assert_eq!(ast.source(), source);
    }

    #[test]
    fn test_expression_type_name() {
        let expr = NixExpression::Literal(LiteralExpr::Integer(42));
        assert_eq!(expr.type_name(), "literal");

        let expr = NixExpression::Ident("x".to_string());
        assert_eq!(expr.type_name(), "identifier");
    }

    #[test]
    fn test_literal_expressions() {
        let int_expr = LiteralExpr::Integer(42);
        let string_expr = LiteralExpr::String("hello".to_string());
        let bool_expr = LiteralExpr::Bool(true);
        let null_expr = LiteralExpr::Null;

        assert!(matches!(int_expr, LiteralExpr::Integer(42)));
        assert!(matches!(string_expr, LiteralExpr::String(_)));
        assert!(matches!(bool_expr, LiteralExpr::Bool(true)));
        assert!(matches!(null_expr, LiteralExpr::Null));
    }

    #[test]
    fn test_binary_operators() {
        let ops = vec![
            BinOp::Add,
            BinOp::Sub,
            BinOp::Mul,
            BinOp::Div,
            BinOp::Update,
            BinOp::Concat,
            BinOp::Equal,
            BinOp::And,
            BinOp::Or,
        ];
        assert_eq!(ops.len(), 9);
    }

    #[test]
    fn test_unary_operators() {
        let not_op = UnaryOp::Not;
        let neg_op = UnaryOp::Negate;
        assert!(matches!(not_op, UnaryOp::Not));
        assert!(matches!(neg_op, UnaryOp::Negate));
    }
}
