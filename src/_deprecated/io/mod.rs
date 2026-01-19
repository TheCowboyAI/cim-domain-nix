// Copyright 2025 Cowboy AI, LLC.

//! Input/Output Adapters for Nix Files
//!
//! This module implements the I/O layer that connects the Category Theory functor
//! to actual Nix files on the filesystem.
//!
//! ## Architecture
//!
//! ```text
//! Nix Files ──read──→ Reader ──→ NixTopology ──┐
//!                                               │
//!                                         Functor (Phase 3)
//!                                               │
//! Nix Files ←─write─── Writer ←─ NixTopology ──┘
//! ```
//!
//! ## Components
//!
//! 1. **Reader**: Parse Nix files into domain objects
//!    - nix-topology format support
//!    - Flake parsing
//!    - Error recovery
//!
//! 2. **Writer**: Serialize domain objects to Nix files
//!    - nix-topology format generation
//!    - Pretty printing
//!    - Validation before write
//!
//! 3. **Validator**: Schema and consistency validation
//!    - Structure validation
//!    - Semantic validation
//!    - Version compatibility
//!
//! 4. **Error Recovery**: Graceful handling of malformed files
//!    - Partial parsing
//!    - Error reporting
//!    - Rollback support

use std::path::Path;
use std::fs;
use std::io;
use thiserror::Error;

use crate::nix::topology::*;

// Submodules
pub mod reader;
pub mod writer;
pub mod validator;

// Re-exports
pub use reader::{NixReader, TopologyReader};
pub use writer::{NixWriter, TopologyWriter};
pub use validator::{NixValidator, ValidationResult};

// ============================================================================
// Error Types
// ============================================================================

/// I/O operation errors
#[derive(Debug, Error, Clone, PartialEq)]
pub enum IoError {
    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Failed to read file
    #[error("Failed to read file: {0}")]
    ReadError(String),

    /// Failed to write file
    #[error("Failed to write file: {0}")]
    WriteError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(String),
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> Self {
        IoError::Io(err.to_string())
    }
}

/// Result type for I/O operations
pub type Result<T> = std::result::Result<T, IoError>;

// ============================================================================
// Public API
// ============================================================================

/// Read a NixTopology from a file
///
/// # Example
///
/// ```no_run
/// use cim_domain_nix::io::read_topology;
///
/// let topology = read_topology("topology.nix")?;
/// println!("Loaded {} nodes", topology.nodes.len());
/// # Ok::<(), cim_domain_nix::io::IoError>(())
/// ```
pub fn read_topology<P: AsRef<Path>>(path: P) -> Result<NixTopology> {
    let reader = TopologyReader::new();
    reader.read_file(path)
}

/// Write a NixTopology to a file
///
/// # Example
///
/// ```no_run
/// use cim_domain_nix::io::write_topology;
/// use cim_domain_nix::nix::topology::*;
///
/// let topology = NixTopology::new("my-infra".to_string());
/// write_topology(&topology, "topology.nix")?;
/// # Ok::<(), cim_domain_nix::io::IoError>(())
/// ```
pub fn write_topology<P: AsRef<Path>>(topology: &NixTopology, path: P) -> Result<()> {
    let writer = TopologyWriter::new();
    writer.write_file(topology, path)
}

/// Validate a NixTopology file
///
/// # Example
///
/// ```no_run
/// use cim_domain_nix::io::validate_topology_file;
///
/// let result = validate_topology_file("topology.nix")?;
/// if result.is_valid() {
///     println!("Topology is valid!");
/// }
/// # Ok::<(), cim_domain_nix::io::IoError>(())
/// ```
pub fn validate_topology_file<P: AsRef<Path>>(path: P) -> Result<ValidationResult> {
    let validator = NixValidator::new();
    let content = fs::read_to_string(&path)?;
    validator.validate_topology_content(&content)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = IoError::FileNotFound("test.nix".to_string());
        assert_eq!(format!("{}", err), "File not found: test.nix");
    }

    #[test]
    fn test_io_error_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err: IoError = io_err.into();
        assert!(matches!(err, IoError::Io(_)));
    }

    // ============================================================================
    // Additional Tests for 90% Coverage
    // ============================================================================

    #[test]
    fn test_io_error_read_error_display() {
        let err = IoError::ReadError("permission denied".to_string());
        assert_eq!(format!("{}", err), "Failed to read file: permission denied");
    }

    #[test]
    fn test_io_error_write_error_display() {
        let err = IoError::WriteError("disk full".to_string());
        assert_eq!(format!("{}", err), "Failed to write file: disk full");
    }

    #[test]
    fn test_io_error_parse_error_display() {
        let err = IoError::ParseError("unexpected token".to_string());
        assert_eq!(format!("{}", err), "Parse error: unexpected token");
    }

    #[test]
    fn test_io_error_validation_error_display() {
        let err = IoError::ValidationError("invalid node reference".to_string());
        assert_eq!(format!("{}", err), "Validation error: invalid node reference");
    }

    #[test]
    fn test_io_error_invalid_path_display() {
        let err = IoError::InvalidPath("/nonexistent/path".to_string());
        assert_eq!(format!("{}", err), "Invalid path: /nonexistent/path");
    }

    #[test]
    fn test_io_error_io_display() {
        let err = IoError::Io("connection refused".to_string());
        assert_eq!(format!("{}", err), "I/O error: connection refused");
    }

    #[test]
    fn test_io_error_equality() {
        let err1 = IoError::FileNotFound("test.nix".to_string());
        let err2 = IoError::FileNotFound("test.nix".to_string());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_io_error_clone() {
        let err1 = IoError::ParseError("error".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
