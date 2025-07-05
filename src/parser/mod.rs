//! Nix file parsing and AST manipulation
//!
//! This module provides comprehensive parsing capabilities for all Nix file types,
//! including flakes, modules, overlays, and derivations.

pub mod ast;
pub mod error;
pub mod flake;
pub mod module;
pub mod derivation;
pub mod advanced;
pub mod manipulator;

use rnix::{SyntaxNode, SyntaxKind};
use rnix::tokenizer::tokenize;
use rnix::parser;
use rowan::GreenNode;
use std::path::{Path, PathBuf};
use crate::{Result, NixDomainError};

pub use error::ParseError;
pub use flake::{FlakeParser, ParsedFlake};
pub use module::{ModuleParser, ParsedModule};
pub use ast::{NixAst, FunctionParam, AttrPath, Binding, BinaryOperator, UnaryOperator, AttrPathSegment, BindingValue};
pub use advanced::AdvancedParser;
pub use manipulator::{AstManipulator, AstBuilder};

/// A parsed Nix file with its AST and metadata
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
        let content = std::fs::read_to_string(path)
            .map_err(NixDomainError::IoError)?;
        Self::parse_string(content, Some(path.to_path_buf()))
    }

    /// Parse Nix content from a string
    pub fn parse_string(content: String, source: Option<PathBuf>) -> Result<Self> {
        // Tokenize first
        let tokens = tokenize(&content);
        
        // Then parse - tokenize returns a Vec, but parse expects an iterator
        let (green, parse_errors) = parser::parse(tokens.into_iter());
        
        let errors = parse_errors
            .into_iter()
            .map(ParseError::from_rnix)
            .collect();

        // Create syntax node from green node
        let ast = SyntaxNode::new_root(green.clone());

        Ok(Self {
            ast,
            green,
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

    /// Get a formatted version of the content
    pub fn format(&self) -> Result<String> {
        // TODO: Implement proper formatting
        Ok(self.content.clone())
    }
    
    /// Parse to our `NixAst` representation
    pub fn to_ast(&self) -> Result<NixAst> {
        ast::from_syntax_node(&self.ast)
            .map_err(|e| NixDomainError::ParseError(e.to_string()))
    }
}

/// Types of Nix files we can parse and analyze
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NixFileType {
    /// A Nix flake (flake.nix)
    Flake,
    /// A `NixOS` module
    Module,
    /// An overlay
    Overlay,
    /// A derivation/package definition
    Derivation,
    /// A `NixOS` configuration
    Configuration,
    /// Unknown file type
    Unknown,
}

impl NixFileType {
    /// Detect the file type by analyzing the AST
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
        // A flake has the structure: { description = ...; inputs = ...; outputs = ...; }
        ast.children()
            .any(|child| {
                if child.kind() == SyntaxKind::NODE_ATTR_SET {
                    let has_description = Self::has_attribute(&child, "description");
                    let has_outputs = Self::has_attribute(&child, "outputs");
                    has_description && has_outputs
                } else {
                    false
                }
            })
    }

    fn is_module(ast: &SyntaxNode) -> bool {
        // A module typically has: { options = ...; config = ...; } or { imports = ...; }
        ast.children()
            .any(|child| {
                if child.kind() == SyntaxKind::NODE_ATTR_SET {
                    let has_options = Self::has_attribute(&child, "options");
                    let has_config = Self::has_attribute(&child, "config");
                    let has_imports = Self::has_attribute(&child, "imports");
                    (has_options && has_config) || has_imports
                } else {
                    false
                }
            })
    }

    fn is_overlay(ast: &SyntaxNode) -> bool {
        // An overlay has the pattern: self: super: { ... }
        // Look for a lambda with two parameters
        ast.children()
            .any(|child| {
                if child.kind() == SyntaxKind::NODE_LAMBDA {
                    // Check if it has the self: super: pattern
                    Self::count_lambda_params(&child) >= 2
                } else {
                    false
                }
            })
    }

    fn is_derivation(ast: &SyntaxNode) -> bool {
        // A derivation typically calls mkDerivation or stdenv.mkDerivation
        Self::contains_derivation_call(ast)
    }

    fn has_attribute(node: &SyntaxNode, attr_name: &str) -> bool {
        node.children()
            .any(|child| {
                // In rnix 0.11, key-value pairs might have a different syntax kind
                // For now, just check if the text contains the attribute
                child.text().to_string().contains(attr_name)
            })
    }

    fn count_lambda_params(lambda_node: &SyntaxNode) -> usize {
        let mut count = 0;
        let mut current = lambda_node.clone();
        
        while current.kind() == SyntaxKind::NODE_LAMBDA {
            count += 1;
            // Move to the body of the lambda
            if let Some(body) = current.children().nth(1) {
                current = body;
            } else {
                break;
            }
        }
        
        count
    }

    fn contains_derivation_call(node: &SyntaxNode) -> bool {
        // Recursively search for mkDerivation calls
        if node.kind() == SyntaxKind::NODE_APPLY {
            let text = node.text().to_string();
            if text.contains("mkDerivation") {
                return true;
            }
        }
        
        node.children().any(|child| Self::contains_derivation_call(&child))
    }
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Preserve comments in the AST
    pub preserve_comments: bool,
    /// Validate while parsing
    pub validate: bool,
    /// Parse included files recursively
    pub follow_imports: bool,
    /// Maximum recursion depth for imports
    pub max_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            preserve_comments: true,
            validate: true,
            follow_imports: false,
            max_depth: 10,
        }
    }
}

/// Simple parser for basic Nix expressions
#[derive(Debug, Clone)]
pub struct NixParser {
    /// Parser configuration
    config: ParserConfig,
}

impl Default for NixParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NixParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }
    
    /// Parse a Nix file from disk
    pub fn parse_file(&self, path: &Path) -> Result<ParsedFile> {
        let nix_file = NixFile::parse_file(path)?;
        Ok(ParsedFile {
            path: path.to_path_buf(),
            expr: NixExpr::from_ast(&nix_file.ast),
            errors: nix_file.errors,
        })
    }
    
    /// Parse Nix content from a string
    pub fn parse_string(&self, content: &str) -> Result<ParsedFile> {
        let nix_file = NixFile::parse_string(content.to_string(), None)?;
        Ok(ParsedFile {
            path: PathBuf::new(),
            expr: NixExpr::from_ast(&nix_file.ast),
            errors: nix_file.errors,
        })
    }
}

/// A parsed file with its expression tree
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// Path to the file (empty for strings)
    pub path: PathBuf,
    /// The parsed expression
    pub expr: NixExpr,
    /// Any parse errors encountered
    pub errors: Vec<ParseError>,
}

/// Simple Nix expression representation
#[derive(Debug, Clone, PartialEq)]
pub enum NixExpr {
    /// String literal
    String(String),
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// Boolean literal
    Bool(bool),
    /// Path literal
    Path(PathBuf),
    /// Identifier
    Identifier(String),
    /// List
    List(Vec<NixExpr>),
    /// Attribute set
    AttrSet(std::collections::HashMap<String, NixExpr>),
    /// Function application
    Apply(Box<NixExpr>, Box<NixExpr>),
    /// Lambda
    Lambda(String, Box<NixExpr>),
    /// Let binding
    Let(Vec<(String, NixExpr)>, Box<NixExpr>),
    /// If expression
    If(Box<NixExpr>, Box<NixExpr>, Box<NixExpr>),
    /// With expression
    With(Box<NixExpr>, Box<NixExpr>),
    /// Other/Unknown
    Other(String),
}

impl NixExpr {
    /// Convert from AST node to expression
    pub fn from_ast(node: &SyntaxNode) -> Self {
        // Try to use our new parser first
        match ast::from_syntax_node(node) {
            Ok(ast) => Self::from_nix_ast(ast),
            Err(_) => {
                // Fallback to simple parsing
                match node.kind() {
                    SyntaxKind::NODE_STRING => {
                        let text = node.text().to_string();
                        // Remove quotes
                        let cleaned = text.trim_matches('"').to_string();
                        NixExpr::String(cleaned)
                    }
                    SyntaxKind::TOKEN_INTEGER => {
                        let text = node.text().to_string();
                        NixExpr::Int(text.parse().unwrap_or(0))
                    }
                    SyntaxKind::TOKEN_FLOAT => {
                        let text = node.text().to_string();
                        NixExpr::Float(text.parse().unwrap_or(0.0))
                    }
                    SyntaxKind::NODE_IDENT => {
                        NixExpr::Identifier(node.text().to_string())
                    }
                    SyntaxKind::NODE_LIST => {
                        let items = node.children()
                            .map(|child| NixExpr::from_ast(&child))
                            .collect();
                        NixExpr::List(items)
                    }
                    SyntaxKind::NODE_ATTR_SET => {
                        let mut attrs = std::collections::HashMap::new();
                        
                        // Simple attribute extraction - this is a simplified version
                        for child in node.children() {
                            if let Some(key_node) = child.children().next() {
                                if let Some(value_node) = child.children().nth(1) {
                                    let key = key_node.text().to_string();
                                    let value = NixExpr::from_ast(&value_node);
                                    attrs.insert(key, value);
                                }
                            }
                        }
                        
                        NixExpr::AttrSet(attrs)
                    }
                    _ => NixExpr::Other(node.text().to_string()),
                }
            }
        }
    }
    
    /// Convert from our `NixAst` to `NixExpr`
    fn from_nix_ast(ast: NixAst) -> Self {
        match ast {
            NixAst::Integer(i) => NixExpr::Int(i),
            NixAst::Float(f) => NixExpr::Float(f),
            NixAst::String(s) => NixExpr::String(s),
            NixAst::Path(p) => NixExpr::Path(p),
            NixAst::Bool(b) => NixExpr::Bool(b),
            NixAst::Null => NixExpr::Other("null".to_string()),
            NixAst::Identifier(id) => NixExpr::Identifier(id),
            NixAst::List(items) => {
                let exprs = items.into_iter()
                    .map(Self::from_nix_ast)
                    .collect();
                NixExpr::List(exprs)
            }
            NixAst::AttrSet { bindings, .. } => {
                let mut attrs = std::collections::HashMap::new();
                for binding in bindings {
                    // Simple conversion - just use the first segment of the path
                    if let Some(first_segment) = binding.attr_path.segments.first() {
                        if let AttrPathSegment::Identifier(name) = first_segment {
                            if let BindingValue::Value(value) = binding.value {
                                attrs.insert(name.clone(), Self::from_nix_ast(value));
                            }
                        }
                    }
                }
                NixExpr::AttrSet(attrs)
            }
            NixAst::Function { param, body } => {
                let param_name = match param {
                    FunctionParam::Identifier(id) => id,
                    FunctionParam::Pattern { .. } => "args".to_string(),
                };
                NixExpr::Lambda(param_name, Box::new(Self::from_nix_ast(*body)))
            }
            NixAst::Apply { function, argument } => {
                NixExpr::Apply(
                    Box::new(Self::from_nix_ast(*function)),
                    Box::new(Self::from_nix_ast(*argument))
                )
            }
            NixAst::Let { bindings, body } => {
                let mut let_bindings = Vec::new();
                for binding in bindings {
                    if let Some(first_segment) = binding.attr_path.segments.first() {
                        if let AttrPathSegment::Identifier(name) = first_segment {
                            if let BindingValue::Value(value) = binding.value {
                                let_bindings.push((name.clone(), Self::from_nix_ast(value)));
                            }
                        }
                    }
                }
                NixExpr::Let(let_bindings, Box::new(Self::from_nix_ast(*body)))
            }
            NixAst::If { condition, then_branch, else_branch } => {
                NixExpr::If(
                    Box::new(Self::from_nix_ast(*condition)),
                    Box::new(Self::from_nix_ast(*then_branch)),
                    Box::new(Self::from_nix_ast(*else_branch))
                )
            }
            NixAst::With { namespace, body } => {
                NixExpr::With(
                    Box::new(Self::from_nix_ast(*namespace)),
                    Box::new(Self::from_nix_ast(*body))
                )
            }
            _ => NixExpr::Other(format!("{ast:?}")),
        }
    }
}

#[cfg(test)]
mod tests; 