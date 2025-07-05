//! Domain-Driven Design module for Nix ecosystem operations
//! 
//! This module provides comprehensive support for working with the Nix package manager
//! and `NixOS` configuration management within the CIM architecture.
//!
//! # Examples
//!
//! ## Creating a Nix Flake
//!
//! ```no_run
//! # use cim_domain_nix::{commands::CreateFlake, handlers::NixCommandHandler};
//! # use std::path::PathBuf;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let handler = NixCommandHandler::new();
//! 
//! let cmd = CreateFlake {
//!     path: PathBuf::from("/tmp/my-project"),
//!     description: "My Rust project".to_string(),
//!     template: Some("rust".to_string()),
//! };
//! 
//! let events = handler.handle_command(Box::new(cmd)).await?;
//! println!("Created flake with {events.len(} events"));
//! # Ok(())
//! # }
//! ```
//!
//! ## Building a Package
//!
//! ```no_run
//! # use cim_domain_nix::{commands::BuildPackage, handlers::NixCommandHandler, value_objects::AttributePath};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let handler = NixCommandHandler::new();
//! 
//! let cmd = BuildPackage {
//!     flake_ref: "nixpkgs".to_string(),
//!     attribute: AttributePath::from_str("hello"),
//!     output_path: None,
//! };
//! 
//! let events = handler.handle_command(Box::new(cmd)).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Working with Attribute Paths
//!
//! ```
//! use cim_domain_nix::value_objects::AttributePath;
//! 
//! let path = AttributePath::from_str("packages.x86_64-linux.hello");
//! assert_eq!(path.segments.len(), 3);
//! assert_eq!(path.to_string(), "packages.x86_64-linux.hello");
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

use thiserror::Error;

pub mod aggregate;
pub mod analyzer;
pub mod commands;
pub mod events;
pub mod formatter;
pub mod git_integration;
pub mod handlers;
pub mod home_manager;
pub mod parser;
pub mod projections;
pub mod queries;
pub mod services;
pub mod value_objects;
pub mod templates;
// pub mod parser; // TODO: Complete parser implementation

// Re-export commonly used types
pub use aggregate::{FlakeAggregate, ModuleAggregate, OverlayAggregate, ConfigurationAggregate};
pub use analyzer::{NixAnalyzer, AnalysisReport, SecurityIssue, PerformanceIssue, DeadCode};
pub use commands::*;
pub use events::*;
pub use formatter::{NixFormatter, FormatterService, FormattingReport};
pub use handlers::*;
pub use parser::{NixFile, NixFileType, FlakeParser, ModuleParser};
pub use projections::NixProjection;
pub use queries::*;
pub use services::*;
// pub use git_integration::*;
pub use value_objects::*;
pub use templates::*;

use std::io;

/// Result type for Nix domain operations
pub type Result<T> = std::result::Result<T, NixDomainError>;

/// Errors that can occur in Nix domain operations
#[derive(Error, Debug)]
pub enum NixDomainError {
    /// Command execution error
    #[error("Command error: {0}")]
    CommandError(String),
    
    /// Build error
    #[error("Build error: {0}")]
    BuildError(String),
    
    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Formatter error
    #[error("Formatter error: {0}")]
    FormatterError(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<parser::ParseError> for NixDomainError {
    fn from(err: parser::ParseError) -> Self {
        NixDomainError::ParseError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let nix_error: NixDomainError = io_error.into();
        assert!(matches!(nix_error, NixDomainError::IoError(_)));
    }
} 