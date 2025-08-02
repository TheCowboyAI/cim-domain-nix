//! Query handlers for Nix domain

use crate::{
    projections::NixProjection,
    value_objects::{FlakeOutputs, FlakeRef},
    Result,
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Query to find a flake by path
#[derive(Debug, Clone)]
pub struct FindFlakeQuery {
    /// Path to the flake directory
    pub path: PathBuf,
}

/// Query to find a package by name
#[derive(Debug, Clone)]
pub struct FindPackageQuery {
    /// Package name to search for
    pub name: String,
    /// Optional system architecture (defaults to x86_64-linux)
    pub system: Option<String>,
}

/// Query to find a configuration by name
#[derive(Debug, Clone)]
pub struct FindConfigurationQuery {
    /// Configuration name (e.g., hostname)
    pub name: String,
}

/// Query to search nixpkgs
#[derive(Debug, Clone)]
pub struct SearchNixPackagesQuery {
    /// Search query string
    pub query: String,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

/// Basic query handler for Nix domain
pub struct NixQueryHandler {
    projection: NixProjection,
}

impl NixQueryHandler {
    /// Create a new query handler with the given projection
    #[must_use]
    pub fn new(projection: NixProjection) -> Self {
        Self { projection }
    }

    /// Find a flake by its file path
    pub fn find_flake(&self, query: FindFlakeQuery) -> Result<Option<FlakeView>> {
        if let Some(flake_id) = self
            .projection
            .flake_projection
            .flakes_by_path
            .get(&query.path)
        {
            Ok(self
                .projection
                .flake_projection
                .flakes
                .get(flake_id)
                .map(|info| FlakeView {
                    id: info.id,
                    path: info.path.clone(),
                    description: info.description.clone(),
                    inputs: info.inputs.clone(),
                    outputs: info.outputs.clone(),
                }))
        } else {
            Ok(None)
        }
    }

    /// Find a package by name and optional system architecture
    pub fn find_package(&self, query: FindPackageQuery) -> Result<Option<PackageView>> {
        let key = format!(
            "{}#{}",
            query.system.as_deref().unwrap_or("x86_64-linux"),
            query.name
        );

        Ok(self
            .projection
            .package_projection
            .packages
            .get(&key)
            .map(|info| PackageView {
                name: info.name.clone(),
                system: info.system.clone(),
                version: info.version.clone(),
                description: info.description.clone(),
            }))
    }

    /// Find a `NixOS` configuration by name
    pub fn find_configuration(
        &self,
        query: FindConfigurationQuery,
    ) -> Result<Option<ConfigurationView>> {
        Ok(self
            .projection
            .configuration_projection
            .configurations
            .get(&query.name)
            .map(|info| ConfigurationView {
                id: info.id,
                name: info.name.clone(),
                system: info.system.clone(),
                current_generation: info.current_generation,
                last_activated: info.last_activated,
            }))
    }
}

/// Advanced query handler with external data sources
pub struct AdvancedNixQueryHandler {
    projection: NixProjection,
}

impl AdvancedNixQueryHandler {
    /// Create a new advanced query handler with the given projection
    #[must_use]
    pub fn new(projection: NixProjection) -> Self {
        Self { projection }
    }

    /// Search nixpkgs for packages
    pub async fn search_nixpkgs(
        &self,
        query: SearchNixPackagesQuery,
    ) -> Result<Vec<PackageSearchResult>> {
        // Search in our projection first
        let mut results = vec![];

        for (key, package) in &self.projection.package_projection.packages {
            if package.name.contains(&query.query)
                || package
                    .description
                    .as_ref()
                    .is_some_and(|d| d.contains(&query.query))
            {
                results.push(PackageSearchResult {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    description: package.description.clone(),
                    attribute_path: key.clone(),
                });

                if let Some(limit) = query.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}

/// View model for flake information
#[derive(Debug, Clone)]
pub struct FlakeView {
    /// Unique identifier for the flake
    pub id: Uuid,
    /// Path to the flake directory
    pub path: PathBuf,
    /// Human-readable description of the flake
    pub description: String,
    /// Input dependencies of the flake
    pub inputs: HashMap<String, FlakeRef>,
    /// Outputs provided by the flake
    pub outputs: FlakeOutputs,
}

/// View model for package information
#[derive(Debug, Clone)]
pub struct PackageView {
    /// Package name
    pub name: String,
    /// Target system architecture
    pub system: String,
    /// Package version if available
    pub version: Option<String>,
    /// Package description if available
    pub description: Option<String>,
}

/// View model for configuration information
#[derive(Debug, Clone)]
pub struct ConfigurationView {
    /// Unique identifier for the configuration
    pub id: Uuid,
    /// Configuration name (usually hostname)
    pub name: String,
    /// Target system architecture
    pub system: String,
    /// Current generation number
    pub current_generation: u32,
    /// Last activation timestamp
    pub last_activated: Option<DateTime<Utc>>,
}

/// Search result for packages
#[derive(Debug, Clone)]
pub struct PackageSearchResult {
    /// Package name
    pub name: String,
    /// Package version if available
    pub version: Option<String>,
    /// Package description if available
    pub description: Option<String>,
    /// Nix attribute path to the package
    pub attribute_path: String,
}
