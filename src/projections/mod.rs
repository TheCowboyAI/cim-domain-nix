//! Read models and projections for the Nix domain

use crate::{
    events::{
        ActivationType, ConfigurationActivated, ConfigurationCreated, FlakeCreated,
        FlakeInputAdded, NixDomainEvent, PackageBuilt,
    },
    value_objects::{AttributePath, FlakeOutputs, FlakeRef},
};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Information about a flake
#[derive(Debug, Clone)]
pub struct FlakeInfo {
    /// Unique identifier for the flake
    pub id: Uuid,
    /// Path to the flake directory
    pub path: PathBuf,
    /// Human-readable description
    pub description: String,
    /// Flake inputs (dependencies)
    pub inputs: HashMap<String, FlakeRef>,
    /// Flake outputs (packages, shells, etc.)
    pub outputs: FlakeOutputs,
    /// Last time the flake was updated
    pub last_updated: DateTime<Utc>,
    /// Last time the flake was checked for validity
    pub last_checked: Option<DateTime<Utc>>,
}

/// Projection for flake information
#[derive(Debug, Clone, Default)]
pub struct FlakeProjection {
    /// All flakes by ID
    pub flakes: HashMap<Uuid, FlakeInfo>,
    /// Flakes by path for quick lookup
    pub flakes_by_path: HashMap<PathBuf, Uuid>,
    /// Dependency graph
    pub dependencies: HashMap<Uuid, Vec<Uuid>>,
}

impl FlakeProjection {
    /// Handle a flake created event
    pub fn handle_flake_created(&mut self, event: &FlakeCreated) {
        let info = FlakeInfo {
            id: event.flake_id,
            path: event.path.clone(),
            description: event.description.clone(),
            inputs: HashMap::new(),
            outputs: FlakeOutputs {
                packages: HashMap::new(),
                dev_shells: HashMap::new(),
                nixos_modules: HashMap::new(),
                overlays: HashMap::new(),
                apps: HashMap::new(),
            },
            last_updated: event.timestamp,
            last_checked: None,
        };

        self.flakes.insert(event.flake_id, info);
        self.flakes_by_path
            .insert(event.path.clone(), event.flake_id);
    }

    /// Handle a flake input added event
    pub fn handle_flake_input_added(&mut self, event: &FlakeInputAdded) {
        if let Some(flake) = self.flakes.get_mut(&event.flake_id) {
            flake.inputs.insert(
                event.input_name.clone(),
                FlakeRef::new(event.input_url.clone()),
            );
            flake.last_updated = event.timestamp;
        }
    }
}

/// Information about a package
#[derive(Debug, Clone)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Target system architecture
    pub system: String,
    /// Package version if available
    pub version: Option<String>,
    /// Package description if available
    pub description: Option<String>,
    /// Flake reference containing the package
    pub flake_ref: String,
    /// Attribute path to the package
    pub attribute_path: AttributePath,
}

/// Build information
#[derive(Debug, Clone)]
pub struct BuildInfo {
    /// Unique identifier for the build
    pub package_id: Uuid,
    /// Output path in the Nix store
    pub output_path: PathBuf,
    /// Time taken to build
    pub build_time: std::time::Duration,
    /// When the build occurred
    pub timestamp: DateTime<Utc>,
    /// Whether the build succeeded
    pub success: bool,
}

/// Projection for package builds
#[derive(Debug, Clone, Default)]
pub struct PackageBuildProjection {
    /// Package information by system#name
    pub packages: HashMap<String, PackageInfo>,
    /// Build history
    pub builds: Vec<BuildInfo>,
    /// Build statistics
    pub total_builds: u64,
    /// Number of successful builds
    pub successful_builds: u64,
    /// Number of failed builds
    pub failed_builds: u64,
    /// Average build time across all builds
    pub average_build_time: std::time::Duration,
}

impl PackageBuildProjection {
    /// Handle a package built event
    pub fn handle_package_built(&mut self, event: &PackageBuilt) {
        let key = format!("x86_64-linux#{}", event.attribute);

        let package_info = PackageInfo {
            name: event.attribute.to_string(),
            system: "x86_64-linux".to_string(),
            version: None,
            description: None,
            flake_ref: event.flake_ref.clone(),
            attribute_path: event.attribute.clone(),
        };

        self.packages.insert(key, package_info);

        let build_info = BuildInfo {
            package_id: event.package_id,
            output_path: event.output_path.clone(),
            build_time: event.build_time,
            timestamp: event.timestamp,
            success: true,
        };

        self.builds.push(build_info);
        self.total_builds += 1;
        self.successful_builds += 1;

        // Update average build time
        let total_time: std::time::Duration = self.builds.iter().map(|b| b.build_time).sum();
        self.average_build_time = total_time / self.total_builds as u32;
    }
}

/// Configuration information
#[derive(Debug, Clone)]
pub struct ConfigurationInfo {
    /// Unique identifier for the configuration
    pub id: Uuid,
    /// Configuration name
    pub name: String,
    /// Target system architecture
    pub system: String,
    /// Hostname for the system
    pub hostname: String,
    /// Current generation number
    pub current_generation: u32,
    /// Last activation timestamp
    pub last_activated: Option<DateTime<Utc>>,
    /// History of activations (timestamp, type, generation)
    pub activation_history: Vec<(DateTime<Utc>, ActivationType, u32)>,
}

/// Projection for `NixOS` configurations
#[derive(Debug, Clone, Default)]
pub struct ConfigurationProjection {
    /// Configurations by name
    pub configurations: HashMap<String, ConfigurationInfo>,
    /// Configurations by ID
    pub configurations_by_id: HashMap<Uuid, String>,
    /// Active configuration
    pub active_configuration: Option<String>,
}

impl ConfigurationProjection {
    /// Handle a configuration created event
    pub fn handle_configuration_created(&mut self, event: &ConfigurationCreated) {
        let info = ConfigurationInfo {
            id: event.configuration.id,
            name: event.configuration.name.clone(),
            system: event.configuration.system.clone(),
            hostname: event.configuration.hostname.clone(),
            current_generation: 0,
            last_activated: None,
            activation_history: vec![],
        };

        self.configurations
            .insert(event.configuration.name.clone(), info);
        self.configurations_by_id
            .insert(event.configuration.id, event.configuration.name.clone());
    }

    /// Handle a configuration activated event
    pub fn handle_configuration_activated(&mut self, event: &ConfigurationActivated) {
        // Find configuration by ID
        if let Some(name) = self.configurations_by_id.get(&event.configuration_id) {
            if let Some(config) = self.configurations.get_mut(name) {
                config.current_generation = event.generation;
                config.last_activated = Some(event.occurred_at());
                config.activation_history.push((
                    event.occurred_at(),
                    event.activation_type.clone(),
                    event.generation,
                ));

                if matches!(event.activation_type, ActivationType::Switch) {
                    self.active_configuration = Some(name.clone());
                }
            }
        }
    }
}

/// Combined projection for all Nix domain data
#[derive(Debug, Clone, Default)]
pub struct NixProjection {
    /// Flake-related projections
    pub flake_projection: FlakeProjection,
    /// Package build projections
    pub package_projection: PackageBuildProjection,
    /// Configuration projections
    pub configuration_projection: ConfigurationProjection,
}

impl NixProjection {
    /// Handle any domain event
    pub fn handle_event(&mut self, _event: &dyn NixDomainEvent) {
        // In a real implementation, we would downcast and dispatch
        // For now, this is a placeholder
    }
}

/// View model for flake information
pub struct FlakeView {
    /// Unique identifier for the flake
    pub id: Uuid,
    /// Path to the flake directory
    pub path: PathBuf,
    /// Human-readable description
    pub description: String,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

/// View model for package information
pub struct PackageView {
    /// Package name
    pub name: String,
    /// Attribute path to the package
    pub attribute_path: String,
    /// Store path if built
    pub store_path: Option<PathBuf>,
    /// Last build timestamp
    pub last_built: Option<DateTime<Utc>>,
}

/// View model for configuration information
pub struct ConfigurationView {
    /// Unique identifier for the configuration
    pub id: Uuid,
    /// Configuration name
    pub name: String,
    /// Target system architecture
    pub system: String,
    /// Current generation number
    pub current_generation: Option<u64>,
}
