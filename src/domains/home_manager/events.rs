// Copyright 2025 Cowboy AI, LLC.

//! Events for Home Manager domain

use super::value_objects::*;
use crate::value_objects::{CausationId, CorrelationId};
use cim_domain::DomainEvent;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// Wrapper enum for all Home Manager events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HomeManagerDomainEvent {
    /// Home configuration created event
    ConfigCreated(HomeConfigCreated),
    /// Dotfiles imported event
    DotfilesImported(DotfilesImported),
    /// Program added event
    ProgramAdded(ProgramAdded),
    /// Program updated event
    ProgramUpdated(ProgramUpdated),
    /// Program removed event
    ProgramRemoved(ProgramRemoved),
    /// Service added event
    ServiceAdded(ServiceAdded),
    /// Service updated event
    ServiceUpdated(ServiceUpdated),
    /// Service removed event
    ServiceRemoved(ServiceRemoved),
    /// Shell configuration updated event
    ShellConfigUpdated(ShellConfigUpdated),
    /// Desktop configuration updated event
    DesktopConfigUpdated(DesktopConfigUpdated),
    /// Packages added event
    PackagesAdded(PackagesAdded),
    /// Packages removed event
    PackagesRemoved(PackagesRemoved),
    /// Configuration generated event
    ConfigGenerated(ConfigGenerated),
    /// Configuration validated event
    ConfigValidated(ConfigValidated),
    /// Migration started event
    MigrationStarted(MigrationStarted),
    /// Migration completed event
    MigrationCompleted(MigrationCompleted),
    /// Migration failed event
    MigrationFailed(MigrationFailed),
}

impl HomeManagerDomainEvent {
    /// Convert to trait object
    pub fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl DomainEvent for HomeManagerDomainEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::ConfigCreated(_) => "HomeConfigCreated",
            Self::DotfilesImported(_) => "DotfilesImported",
            Self::ProgramAdded(_) => "ProgramAdded",
            Self::ProgramUpdated(_) => "ProgramUpdated",
            Self::ProgramRemoved(_) => "ProgramRemoved",
            Self::ServiceAdded(_) => "ServiceAdded",
            Self::ServiceUpdated(_) => "ServiceUpdated",
            Self::ServiceRemoved(_) => "ServiceRemoved",
            Self::ShellConfigUpdated(_) => "ShellConfigUpdated",
            Self::DesktopConfigUpdated(_) => "DesktopConfigUpdated",
            Self::PackagesAdded(_) => "PackagesAdded",
            Self::PackagesRemoved(_) => "PackagesRemoved",
            Self::ConfigGenerated(_) => "ConfigGenerated",
            Self::ConfigValidated(_) => "ConfigValidated",
            Self::MigrationStarted(_) => "MigrationStarted",
            Self::MigrationCompleted(_) => "MigrationCompleted",
            Self::MigrationFailed(_) => "MigrationFailed",
        }
    }

    fn subject(&self) -> String {
        format!("home_manager.{}", self.event_type())
    }

    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::ConfigCreated(e) => e.config_id.0,
            Self::DotfilesImported(e) => e.config_id.0,
            Self::ProgramAdded(e) => e.config_id.0,
            Self::ProgramUpdated(e) => e.config_id.0,
            Self::ProgramRemoved(e) => e.config_id.0,
            Self::ServiceAdded(e) => e.config_id.0,
            Self::ServiceUpdated(e) => e.config_id.0,
            Self::ServiceRemoved(e) => e.config_id.0,
            Self::ShellConfigUpdated(e) => e.config_id.0,
            Self::DesktopConfigUpdated(e) => e.config_id.0,
            Self::PackagesAdded(e) => e.config_id.0,
            Self::PackagesRemoved(e) => e.config_id.0,
            Self::ConfigGenerated(e) => e.config_id.0,
            Self::ConfigValidated(e) => e.config_id.0,
            Self::MigrationStarted(e) => e.migration_id,
            Self::MigrationCompleted(e) => e.migration_id,
            Self::MigrationFailed(e) => e.migration_id,
        }
    }
}

/// Event: Home configuration created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeConfigCreated {
    /// Unique identifier for the home configuration
    pub config_id: HomeConfigId,
    /// User profile information for the configuration
    pub user_profile: UserProfile,
    /// Set of packages to be managed
    pub packages: PackageSet,
    /// Optional shell configuration settings
    pub shell: Option<ShellConfig>,
    /// Optional desktop environment configuration
    pub desktop: Option<DesktopConfig>,
    /// Timestamp when the configuration was created
    pub created_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Dotfiles imported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfilesImported {
    /// Configuration ID the dotfiles are imported into
    pub config_id: HomeConfigId,
    /// Path to the dotfiles directory
    pub dotfiles_path: PathBuf,
    /// List of dotfile entries that were imported
    pub imported_files: Vec<DotfileEntry>,
    /// Programs detected from the dotfiles
    pub detected_programs: Vec<String>,
    /// Timestamp when the import occurred
    pub imported_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Program added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAdded {
    /// Configuration ID the program is added to
    pub config_id: HomeConfigId,
    /// Program configuration details
    pub program: ProgramConfig,
    /// Timestamp when the program was added
    pub added_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Program updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramUpdated {
    /// Configuration ID containing the program
    pub config_id: HomeConfigId,
    /// Name of the program being updated
    pub program_name: String,
    /// Previous program configuration
    pub old_config: ProgramConfig,
    /// New program configuration
    pub new_config: ProgramConfig,
    /// Timestamp when the program was updated
    pub updated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Program removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramRemoved {
    /// Configuration ID the program is removed from
    pub config_id: HomeConfigId,
    /// Name of the program being removed
    pub program_name: String,
    /// Timestamp when the program was removed
    pub removed_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Service added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAdded {
    /// Configuration ID the service is added to
    pub config_id: HomeConfigId,
    /// Service configuration details
    pub service: ServiceConfig,
    /// Timestamp when the service was added
    pub added_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Service updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceUpdated {
    /// Configuration ID containing the service
    pub config_id: HomeConfigId,
    /// Name of the service being updated
    pub service_name: String,
    /// Previous service configuration
    pub old_config: ServiceConfig,
    /// New service configuration
    pub new_config: ServiceConfig,
    /// Timestamp when the service was updated
    pub updated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Service removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRemoved {
    /// Configuration ID the service is removed from
    pub config_id: HomeConfigId,
    /// Name of the service being removed
    pub service_name: String,
    /// Timestamp when the service was removed
    pub removed_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Shell configuration updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfigUpdated {
    /// Configuration ID containing the shell config
    pub config_id: HomeConfigId,
    /// Previous shell configuration if any
    pub old_shell: Option<ShellConfig>,
    /// New shell configuration
    pub new_shell: ShellConfig,
    /// Timestamp when the shell config was updated
    pub updated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Desktop configuration updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfigUpdated {
    /// Configuration ID containing the desktop config
    pub config_id: HomeConfigId,
    /// Previous desktop configuration if any
    pub old_desktop: Option<DesktopConfig>,
    /// New desktop configuration
    pub new_desktop: DesktopConfig,
    /// Timestamp when the desktop config was updated
    pub updated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Packages added
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesAdded {
    /// Configuration ID the packages are added to
    pub config_id: HomeConfigId,
    /// Category of packages being added
    pub category: PackageCategory,
    /// List of package names added
    pub packages: Vec<String>,
    /// Timestamp when the packages were added
    pub added_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Packages removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagesRemoved {
    /// Configuration ID the packages are removed from
    pub config_id: HomeConfigId,
    /// Category of packages being removed
    pub category: PackageCategory,
    /// List of package names removed
    pub packages: Vec<String>,
    /// Timestamp when the packages were removed
    pub removed_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Configuration generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGenerated {
    /// Configuration ID that was generated
    pub config_id: HomeConfigId,
    /// Path where configuration was written
    pub output_path: PathBuf,
    /// Whether a flake.nix was included
    pub include_flake: bool,
    /// List of files generated
    pub generated_files: Vec<PathBuf>,
    /// Timestamp when generation completed
    pub generated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Configuration validated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidated {
    /// Configuration ID that was validated
    pub config_id: HomeConfigId,
    /// Path to the validated configuration
    pub config_path: PathBuf,
    /// Whether the configuration is valid
    pub is_valid: bool,
    /// List of validation errors found
    pub errors: Vec<String>,
    /// List of validation warnings found
    pub warnings: Vec<String>,
    /// Timestamp when validation completed
    pub validated_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Migration started
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStarted {
    /// Unique identifier for this migration
    pub migration_id: uuid::Uuid,
    /// Source of the configuration being migrated
    pub source: ConfigSource,
    /// User profile being migrated
    pub user_profile: UserProfile,
    /// Migration options and settings
    pub options: MigrationOptions,
    /// Timestamp when migration started
    pub started_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Migration completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationCompleted {
    /// Migration identifier
    pub migration_id: uuid::Uuid,
    /// Created configuration ID
    pub config_id: HomeConfigId,
    /// List of programs successfully migrated
    pub migrated_programs: Vec<String>,
    /// List of services successfully migrated
    pub migrated_services: Vec<String>,
    /// List of dotfiles successfully migrated
    pub migrated_dotfiles: Vec<DotfileEntry>,
    /// Timestamp when migration completed
    pub completed_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

/// Event: Migration failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFailed {
    /// Migration identifier that failed
    pub migration_id: uuid::Uuid,
    /// Error message describing the failure
    pub error: String,
    /// Timestamp when migration failed
    pub failed_at: DateTime<Utc>,
    /// Correlation ID for tracking related events
    pub correlation_id: CorrelationId,
    /// Causation ID linking to the triggering event
    pub causation_id: CausationId,
}

use super::commands::{PackageCategory, MigrationOptions};