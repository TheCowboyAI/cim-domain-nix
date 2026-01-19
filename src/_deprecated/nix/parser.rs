// Copyright 2025 Cowboy AI, LLC.

//! Nix Parser Module
//!
//! This module provides high-level parsing functionality for Nix files,
//! converting Nix source code into our domain objects.
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::nix::parser::*;
//!
//! // Parse a Nix expression
//! let parser = NixParser::new();
//! let ast = parser.parse_str("{ x = 1; y = 2; }").unwrap();
//! ```

use super::ast::{AstError, NixAst};
use super::objects::*;
use super::value_objects::*;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Parser errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseError {
    /// AST error
    #[error("AST error: {0}")]
    AstError(#[from] AstError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// Invalid Nix file
    #[error("Invalid Nix file: {0}")]
    InvalidFile(String),

    /// Unsupported expression
    #[error("Unsupported expression: {0}")]
    UnsupportedExpression(String),

    /// Conversion error
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Result type for parser operations
pub type ParseResult<T> = std::result::Result<T, ParseError>;

/// Nix Parser
///
/// Parses Nix source code and converts it into our domain objects.
pub struct NixParser {
    /// Parser configuration
    config: ParserConfig,
}

/// Parser configuration
#[derive(Debug, Clone)]
pub struct ParserConfig {
    /// Whether to allow parse warnings
    pub allow_warnings: bool,
    /// Whether to parse recursively (follow imports)
    pub follow_imports: bool,
    /// Maximum recursion depth for imports
    pub max_import_depth: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            allow_warnings: true,
            follow_imports: false,
            max_import_depth: 10,
        }
    }
}

impl NixParser {
    /// Create a new parser with default configuration
    pub fn new() -> Self {
        Self {
            config: ParserConfig::default(),
        }
    }

    /// Create a parser with custom configuration
    pub fn with_config(config: ParserConfig) -> Self {
        Self { config }
    }

    /// Parse a Nix file
    pub fn parse_file(&self, path: impl AsRef<Path>) -> ParseResult<ParsedFile> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|e| ParseError::IoError(format!("Failed to read file: {}", e)))?;

        let ast = NixAst::parse(&source)?;

        Ok(ParsedFile {
            path: path.to_path_buf(),
            source,
            ast,
        })
    }

    /// Parse a Nix string
    pub fn parse_str(&self, source: impl AsRef<str>) -> ParseResult<NixAst> {
        Ok(NixAst::parse(source.as_ref())?)
    }

    /// Parse a Nix attrset from source
    pub fn parse_attrset(&self, source: impl AsRef<str>) -> ParseResult<NixAttrsetObject> {
        let _ast = self.parse_str(source)?;
        // For now, return a placeholder
        // Full implementation will convert AST to attrset
        Ok(NixAttrsetObject::new(NixAttrset::new()))
    }

    /// Parse a flake.nix file
    pub fn parse_flake(&self, path: impl AsRef<Path>) -> ParseResult<NixFlake> {
        let parsed = self.parse_file(path.as_ref())?;
        self.ast_to_flake(&parsed.ast, path.as_ref().to_path_buf())
    }

    /// Parse a NixOS module
    pub fn parse_module(&self, path: impl AsRef<Path>) -> ParseResult<NixModule> {
        let parsed = self.parse_file(path.as_ref())?;
        self.ast_to_module(&parsed.ast, path.as_ref().to_path_buf())
    }

    /// Parse a package definition
    pub fn parse_package(&self, source: impl AsRef<str>, system: String) -> ParseResult<NixPackage> {
        let ast = self.parse_str(source)?;
        self.ast_to_package(&ast, system)
    }

    /// Convert AST to Flake
    fn ast_to_flake(&self, _ast: &NixAst, flake_path: PathBuf) -> ParseResult<NixFlake> {
        // Placeholder implementation
        // Full implementation will parse flake structure
        Ok(NixFlake::new(
            "Parsed flake".to_string(),
            flake_path,
        ))
    }

    /// Convert AST to Module
    fn ast_to_module(&self, _ast: &NixAst, source_path: PathBuf) -> ParseResult<NixModule> {
        // Placeholder implementation
        // Full implementation will parse module structure
        Ok(NixModule::new("parsed-module".to_string())
            .with_source_path(source_path))
    }

    /// Convert AST to Package
    fn ast_to_package(&self, _ast: &NixAst, system: String) -> ParseResult<NixPackage> {
        // Placeholder implementation
        // Full implementation will parse package structure
        Ok(NixPackage::new("parsed-package".to_string(), system))
    }
}

impl Default for NixParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parsed Nix file
#[derive(Debug, Clone)]
pub struct ParsedFile {
    /// File path
    pub path: PathBuf,
    /// Source text
    pub source: String,
    /// Parsed AST
    pub ast: NixAst,
}

impl ParsedFile {
    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the source text
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the AST
    pub fn ast(&self) -> &NixAst {
        &self.ast
    }
}

// ============================================================================
// Specialized Parsers
// ============================================================================

/// Flake Parser - Specialized for parsing flake.nix files
pub struct FlakeParser {
    parser: NixParser,
}

impl FlakeParser {
    /// Create a new flake parser
    pub fn new() -> Self {
        Self {
            parser: NixParser::new(),
        }
    }

    /// Parse a flake.nix file
    pub fn parse(&self, flake_dir: impl AsRef<Path>) -> ParseResult<NixFlake> {
        let flake_path = flake_dir.as_ref().join("flake.nix");
        self.parser.parse_flake(flake_path)
    }

    /// Parse flake.lock file
    pub fn parse_lock(&self, flake_dir: impl AsRef<Path>) -> ParseResult<FlakeLock> {
        let lock_path = flake_dir.as_ref().join("flake.lock");
        let content = fs::read_to_string(&lock_path)
            .map_err(|e| ParseError::IoError(format!("Failed to read flake.lock: {}", e)))?;

        let lock: FlakeLock = serde_json::from_str(&content)
            .map_err(|e| ParseError::ConversionError(format!("Invalid flake.lock: {}", e)))?;

        Ok(lock)
    }
}

impl Default for FlakeParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Flake Lock File
///
/// Represents the flake.lock file that pins input versions
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FlakeLock {
    /// Lock file version
    pub version: u32,
    /// Locked nodes (inputs)
    pub nodes: std::collections::HashMap<String, LockedNode>,
}

/// A locked node in the flake lock file
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LockedNode {
    /// Locked reference
    pub locked: Option<LockedRef>,
    /// Original reference
    pub original: Option<OriginalRef>,
    /// Input nodes this depends on
    pub inputs: Option<std::collections::HashMap<String, String>>,
}

/// Locked reference
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LockedRef {
    /// Reference type (github, git, etc.)
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Owner (for github)
    pub owner: Option<String>,
    /// Repository (for github)
    pub repo: Option<String>,
    /// Revision hash
    pub rev: Option<String>,
    /// Last modified timestamp
    #[serde(rename = "lastModified")]
    pub last_modified: Option<u64>,
}

/// Original reference
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OriginalRef {
    /// Reference type
    #[serde(rename = "type")]
    pub ref_type: String,
    /// Owner (for github)
    pub owner: Option<String>,
    /// Repository (for github)
    pub repo: Option<String>,
    /// Reference (branch/tag)
    #[serde(rename = "ref")]
    pub git_ref: Option<String>,
}

/// Module Parser - Specialized for parsing NixOS modules
pub struct ModuleParser {
    parser: NixParser,
}

impl ModuleParser {
    /// Create a new module parser
    pub fn new() -> Self {
        Self {
            parser: NixParser::new(),
        }
    }

    /// Parse a NixOS module file
    pub fn parse(&self, path: impl AsRef<Path>) -> ParseResult<NixModule> {
        self.parser.parse_module(path)
    }
}

impl Default for ModuleParser {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = NixParser::new();
        assert!(parser.config.allow_warnings);
    }

    #[test]
    fn test_parser_with_config() {
        let config = ParserConfig {
            allow_warnings: false,
            follow_imports: true,
            max_import_depth: 5,
        };
        let parser = NixParser::with_config(config);
        assert!(!parser.config.allow_warnings);
        assert!(parser.config.follow_imports);
        assert_eq!(parser.config.max_import_depth, 5);
    }

    #[test]
    fn test_parse_simple_attrset() {
        let parser = NixParser::new();
        let result = parser.parse_str("{ x = 1; y = 2; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let parser = NixParser::new();
        let result = parser.parse_str("{ x = ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_list() {
        let parser = NixParser::new();
        let result = parser.parse_str("[ 1 2 3 ]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_in() {
        let parser = NixParser::new();
        let result = parser.parse_str("let x = 1; in x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda() {
        let parser = NixParser::new();
        let result = parser.parse_str("x: x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_pattern_lambda() {
        let parser = NixParser::new();
        let result = parser.parse_str("{ x, y }: x + y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_rec_attrset() {
        let parser = NixParser::new();
        let result = parser.parse_str("rec { x = 1; y = x + 1; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with() {
        let parser = NixParser::new();
        let result = parser.parse_str("with pkgs; [ hello ]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_then_else() {
        let parser = NixParser::new();
        let result = parser.parse_str("if true then 1 else 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string_interpolation() {
        let parser = NixParser::new();
        let result = parser.parse_str(r#""hello ${name}""#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_flake_parser_creation() {
        let parser = FlakeParser::new();
        assert!(parser.parser.config.allow_warnings);
    }

    #[test]
    fn test_module_parser_creation() {
        let parser = ModuleParser::new();
        assert!(parser.parser.config.allow_warnings);
    }

    #[test]
    fn test_parse_attrset() {
        let parser = NixParser::new();
        let result = parser.parse_attrset("{ x = 1; }");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_package() {
        let parser = NixParser::new();
        let result = parser.parse_package(
            "{ pkgs }: pkgs.hello",
            "x86_64-linux".to_string(),
        );
        assert!(result.is_ok());
    }
}
