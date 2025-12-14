// Copyright 2025 Cowboy AI, LLC.

//! Nix Language Representation Module
//!
//! This module implements the SOURCE category in our Category Theory functor:
//!
//! ```text
//! Category(Nix) ──Functor F──> Category(Infrastructure)
//!   (This Module)                (infrastructure module)
//! ```
//!
//! ## Purpose
//!
//! Nix is our **data storage format**. This module provides:
//! - Rust representations of Nix language constructs
//! - Parsing of Nix files using rnix-parser
//! - Serialization/deserialization of Nix AST
//! - Type-safe access to Nix data structures
//!
//! ## Nix Language Overview
//!
//! Nix has 9 types divided into primitives and compounds:
//!
//! **Primitive Types (7)**:
//! - String - Text values
//! - Integer - Whole numbers
//! - Float - Decimal numbers
//! - Bool - true/false
//! - Null - absence of value
//! - Path - Filesystem paths
//! - LookupPath - Nix search path entries (<nixpkgs>)
//!
//! **Compound Types (2)**:
//! - Attribute Set - Key-value mappings (like objects/dicts)
//! - List - Ordered collections
//!
//! ## Nix Objects
//!
//! Higher-level Nix constructs we map to Infrastructure:
//! - **Attrset** - Basic attribute set
//! - **Derivation** - Build specification (.drv)
//! - **Package** - Installable software
//! - **Module** - NixOS module with options/config
//! - **Overlay** - Package set modifications
//! - **Flake** - Top-level composition (inputs/outputs/lock)
//! - **Application** - Executable programs
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::nix::*;
//!
//! // Parse a Nix file
//! let source = r#"
//!   { pkgs ? import <nixpkgs> {} }:
//!   pkgs.hello
//! "#;
//!
//! let ast = NixParser::parse(source)?;
//!
//! // Access Nix values
//! let attrset = NixAttrset::from_ast(&ast)?;
//! ```

pub mod ast;
pub mod ast_converter;
pub mod flake_analyzer;
pub mod flake_evaluator;
pub mod objects;
pub mod parser;
pub mod topology;
pub mod value_objects;

// Re-export commonly used types
pub use ast::{NixAst, NixExpression, NixNode};
pub use ast_converter::{ast_to_value, AstConverter};
pub use flake_analyzer::{
    analyze_flake, flake_to_infrastructure, FlakeAnalysis, FlakeAnalyzer, FlakeDevShell,
    FlakeInput, FlakePackage,
};
pub use flake_evaluator::{
    evaluate_flake, nix_available, AppInfo, CheckInfo, DevShellInfo, EvaluatedFlake,
    EvaluationError, FlakeEvaluator, PackageInfo,
};
pub use objects::{
    NixApplication, NixAttrsetObject, NixDerivation, NixFlake, NixModule, NixObject, NixOverlay,
    NixPackage,
};
pub use parser::{NixParser, ParseError, ParseResult};
pub use topology::{NixTopology, TopologyNode, TopologyNetwork};
pub use value_objects::{
    NixAttrset, NixBool, NixFloat, NixInteger, NixList, NixLookupPath, NixNull, NixPath,
    NixString, NixValue,
};
