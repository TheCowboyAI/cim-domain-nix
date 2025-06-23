//! Domain model for Nix operations
//!
//! This crate provides a domain-driven design (DDD) model for interacting
//! with Nix, including flakes, packages, modules, overlays, and NixOS configurations.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

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
use thiserror::Error;

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