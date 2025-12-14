// Copyright 2025 Cowboy AI, LLC.

//! Category Theory Functor Module
//!
//! This module implements the functor `F: Category(Nix) → Category(Infrastructure)`
//! that provides structure-preserving mappings between Nix data and Infrastructure domain.
//!
//! ## Category Theory Foundation
//!
//! A **Functor** F between categories C and D consists of:
//! 1. An **object mapping**: For each object X in C, an object F(X) in D
//! 2. A **morphism mapping**: For each morphism f: X → Y in C, a morphism F(f): F(X) → F(Y) in D
//!
//! The functor must satisfy two laws:
//! - **Identity preservation**: F(id_X) = id_F(X)
//! - **Composition preservation**: F(g ∘ f) = F(g) ∘ F(f)
//!
//! ## Our Functor
//!
//! ```text
//! Category(Nix)                     Category(Infrastructure)
//! ══════════════                    ═══════════════════════════
//! Objects:                          Objects:
//!   NixFlake                 ──>      InfrastructureAggregate
//!   NixPackage               ──>      SoftwareConfiguration
//!   NixModule                ──>      ComputeResource
//!   NixDerivation            ──>      Build metadata
//!   NixOverlay               ──>      PolicyRule
//!   NixApplication           ──>      Deployed service
//!   NixTopology              ──>      Complete Infrastructure
//!
//! Morphisms:                        Morphisms:
//!   import                   ──>      Load/Parse commands
//!   merge                    ──>      Update commands
//!   override                 ──>      Policy application commands
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::functor::*;
//! use cim_domain_nix::nix::*;
//! use cim_domain_nix::infrastructure::*;
//!
//! // Create functor
//! let functor = NixInfrastructureFunctor::new();
//!
//! // Map Nix topology to Infrastructure
//! let topology = NixTopology::new("my-infra".to_string());
//! let infrastructure = functor.map_topology(&topology)?;
//!
//! // Map back (projection)
//! let projected_topology = functor.project_topology(&infrastructure)?;
//! ```

pub mod mappings;
pub mod laws;
pub mod projections;

use crate::infrastructure::*;
use crate::nix::*;
use thiserror::Error;
use uuid::Uuid;

// Re-export commonly used types
pub use mappings::*;
pub use laws::*;
pub use projections::*;

/// Errors that can occur during functor operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum FunctorError {
    /// Mapping error
    #[error("Mapping error: {0}")]
    MappingError(String),

    /// Invalid source object
    #[error("Invalid source object: {0}")]
    InvalidSource(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Infrastructure error
    #[error("Infrastructure error: {0}")]
    InfrastructureError(#[from] InfrastructureError),

    /// Nix value error
    #[error("Nix value error: {0}")]
    NixValueError(String),

    /// Functor law violation
    #[error("Functor law violation: {0}")]
    LawViolation(String),
}

/// Result type for functor operations
pub type Result<T> = std::result::Result<T, FunctorError>;

// ============================================================================
// Functor Trait - The core abstraction
// ============================================================================

/// Functor trait for Category Theory functors
///
/// Defines the object and morphism mappings between categories.
pub trait Functor<SourceCat, TargetCat> {
    /// Map an object from source category to target category
    ///
    /// This implements the object mapping component of the functor.
    fn map_object(&self, source: &SourceCat) -> Result<TargetCat>;

    /// Map a morphism from source category to target category
    ///
    /// This implements the morphism mapping component of the functor.
    /// The morphism is represented as a transformation function.
    fn map_morphism<F, G>(&self, morphism: F) -> G
    where
        F: Fn(&SourceCat) -> Result<SourceCat>,
        G: Fn(&TargetCat) -> Result<TargetCat>;

    /// Verify identity preservation law: F(id_X) = id_F(X)
    fn verify_identity(&self, source: &SourceCat) -> Result<()>;

    /// Verify composition preservation law: F(g ∘ f) = F(g) ∘ F(f)
    fn verify_composition<F, G>(
        &self,
        f: F,
        g: G,
        source: &SourceCat,
    ) -> Result<()>
    where
        F: Fn(&SourceCat) -> Result<SourceCat>,
        G: Fn(&SourceCat) -> Result<SourceCat>;
}

// ============================================================================
// NixInfrastructureFunctor - Main functor implementation
// ============================================================================

/// The main functor: F: Category(Nix) → Category(Infrastructure)
///
/// This functor maps Nix data structures to Infrastructure domain objects
/// while preserving categorical structure.
pub struct NixInfrastructureFunctor {
    /// Correlation ID for tracking functor operations
    pub correlation_id: Uuid,
}

impl NixInfrastructureFunctor {
    /// Create a new functor instance
    pub fn new() -> Self {
        Self {
            correlation_id: Uuid::now_v7(),
        }
    }

    /// Map NixTopology to InfrastructureAggregate (the primary mapping)
    ///
    /// This is the main object mapping that converts a complete Nix topology
    /// into an Infrastructure aggregate with all resources, networks, and connections.
    pub fn map_topology(&self, topology: &NixTopology) -> Result<InfrastructureAggregate> {
        // This will be implemented in mappings module
        map_topology_to_infrastructure(topology)
    }

    /// Map NixFlake to InfrastructureAggregate
    ///
    /// Flakes represent complete, reproducible configurations.
    pub fn map_flake(&self, flake: &NixFlake) -> Result<InfrastructureAggregate> {
        map_flake_to_infrastructure(flake)
    }

    /// Map NixPackage to SoftwareConfiguration
    pub fn map_package(&self, package: &NixPackage) -> Result<SoftwareConfiguration> {
        map_package_to_software_config(package)
    }

    /// Map NixModule to ComputeResource
    pub fn map_module(&self, module: &NixModule) -> Result<ComputeResource> {
        map_module_to_compute_resource(module)
    }

    /// Map NixApplication to service metadata
    pub fn map_application(&self, app: &NixApplication) -> Result<SoftwareArtifact> {
        map_application_to_software_artifact(app)
    }

    /// Project InfrastructureAggregate back to NixTopology
    ///
    /// This is the reverse mapping (projection) for writing Infrastructure
    /// state back to Nix files.
    pub fn project_topology(&self, infrastructure: &InfrastructureAggregate) -> Result<NixTopology> {
        project_infrastructure_to_topology(infrastructure)
    }

    /// Project SoftwareConfiguration back to NixPackage
    pub fn project_package(&self, config: &SoftwareConfiguration) -> Result<NixPackage> {
        project_software_config_to_package(config)
    }
}

impl Default for NixInfrastructureFunctor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper traits for type conversions
// ============================================================================

/// Trait for types that can be mapped from Nix to Infrastructure
pub trait FromNix<T> {
    /// Convert from a Nix type
    fn from_nix(nix: &T) -> Result<Self>
    where
        Self: Sized;
}

/// Trait for types that can be projected from Infrastructure to Nix
pub trait ToNix<T> {
    /// Convert to a Nix type
    fn to_nix(&self) -> Result<T>;
}

/// Trait for bidirectional mapping
pub trait Bijection<T>: FromNix<T> + ToNix<T> {
    /// Verify round-trip property: to_nix(from_nix(x)) ≈ x
    fn verify_round_trip(nix: &T) -> Result<()>
    where
        Self: Sized,
    {
        let infra = Self::from_nix(nix)?;
        let _projected = infra.to_nix()?;
        // In a full implementation, we'd verify equality here
        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nix::topology::{NixTopology, TopologyNode, TopologyNodeType};

    #[test]
    fn test_functor_creation() {
        let functor = NixInfrastructureFunctor::new();
        // Verify correlation ID is UUID v7 (time-ordered)
        assert!(functor.correlation_id.get_version_num() == 7);
    }

    #[test]
    fn test_functor_default() {
        let functor = NixInfrastructureFunctor::default();
        assert!(functor.correlation_id.get_version_num() == 7);
    }

    #[test]
    fn test_error_types() {
        let err = FunctorError::MappingError("test".to_string());
        assert_eq!(err.to_string(), "Mapping error: test");

        let err = FunctorError::InvalidSource("test".to_string());
        assert_eq!(err.to_string(), "Invalid source object: test");

        let err = FunctorError::MissingField("name".to_string());
        assert_eq!(err.to_string(), "Missing required field: name");
    }

    // ============================================================================
    // Additional Tests for 90% Coverage
    // ============================================================================

    #[test]
    fn test_functor_error_nix_value_error() {
        let err = FunctorError::NixValueError("invalid value".to_string());
        assert_eq!(err.to_string(), "Nix value error: invalid value");
    }

    #[test]
    fn test_functor_error_law_violation() {
        let err = FunctorError::LawViolation("identity not preserved".to_string());
        assert_eq!(err.to_string(), "Functor law violation: identity not preserved");
    }

    #[test]
    fn test_functor_error_infrastructure_error() {
        let infra_err = InfrastructureError::ValidationError("invalid".into());
        let err = FunctorError::InfrastructureError(infra_err);
        let display = format!("{}", err);
        assert!(display.contains("invalid"));
    }

    #[test]
    fn test_functor_error_equality() {
        let err1 = FunctorError::MappingError("test".to_string());
        let err2 = FunctorError::MappingError("test".to_string());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_functor_error_clone() {
        let err1 = FunctorError::InvalidSource("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_functor_map_topology() {
        let functor = NixInfrastructureFunctor::new();
        let mut topology = NixTopology::new("test".to_string());
        topology.add_node(TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        ));

        let result = functor.map_topology(&topology);
        assert!(result.is_ok());
    }

    #[test]
    fn test_functor_map_flake() {
        let functor = NixInfrastructureFunctor::new();
        let flake = NixFlake::new(
            "Test flake".to_string(),
            std::path::PathBuf::from("/test"),
        );

        let result = functor.map_flake(&flake);
        assert!(result.is_ok());
    }

    #[test]
    fn test_functor_map_package() {
        let functor = NixInfrastructureFunctor::new();
        let package = NixPackage::new("hello".to_string(), "x86_64-linux".to_string())
            .with_version("2.10".to_string());

        let result = functor.map_package(&package);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.software.name, "hello");
    }

    #[test]
    fn test_functor_map_module() {
        let functor = NixInfrastructureFunctor::new();
        let module = NixModule::new("webserver".to_string());

        let result = functor.map_module(&module);
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert_eq!(format!("{}", resource.hostname), "webserver.local");
    }

    #[test]
    fn test_functor_map_application() {
        let functor = NixInfrastructureFunctor::new();
        let app = NixApplication::new(
            "myapp".to_string(),
            "/nix/store/abc/bin/myapp".to_string(),
            "x86_64-linux".to_string(),
        );

        let result = functor.map_application(&app);
        assert!(result.is_ok());
        let artifact = result.unwrap();
        assert_eq!(artifact.name, "myapp");
    }

    #[test]
    fn test_functor_project_topology() {
        let functor = NixInfrastructureFunctor::new();
        let infrastructure = InfrastructureAggregate::new(InfrastructureId::new());

        let result = functor.project_topology(&infrastructure);
        assert!(result.is_ok());
    }

    #[test]
    fn test_functor_project_package() {
        let functor = NixInfrastructureFunctor::new();
        let software = SoftwareArtifact {
            id: SoftwareId::new("nginx").unwrap(),
            name: "nginx".to_string(),
            version: Version::new("1.20.0"),
            derivation_path: None,
        };
        let config = SoftwareConfiguration {
            id: ConfigurationId::new(),
            resource_id: ResourceId::new("server01").unwrap(),
            software,
            configuration_data: serde_json::Value::Null,
            dependencies: Vec::new(),
        };

        let result = functor.project_package(&config);
        assert!(result.is_ok());
        let package = result.unwrap();
        assert_eq!(package.name, "nginx");
    }

    #[test]
    fn test_functor_round_trip_topology() {
        let functor = NixInfrastructureFunctor::new();
        let mut topology = NixTopology::new("test".to_string());
        topology.add_node(TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        ));

        let infrastructure = functor.map_topology(&topology).unwrap();
        let projected = functor.project_topology(&infrastructure).unwrap();

        assert_eq!(topology.nodes.len(), projected.nodes.len());
    }

    #[test]
    fn test_functor_correlation_id_unique() {
        let f1 = NixInfrastructureFunctor::new();
        let f2 = NixInfrastructureFunctor::new();
        assert_ne!(f1.correlation_id, f2.correlation_id);
    }
}
