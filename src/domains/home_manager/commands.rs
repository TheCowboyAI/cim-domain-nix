// Copyright 2025 Cowboy AI, LLC.

//! Commands for Home Manager domain operations

use super::value_objects::*;
use crate::value_objects::MessageIdentity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Command to create a new home configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHomeConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// User profile information
    pub user_profile: UserProfile,
    /// Initial package set
    pub packages: PackageSet,
    /// Shell configuration
    pub shell: Option<ShellConfig>,
    /// Desktop configuration
    pub desktop: Option<DesktopConfig>,
}

/// Command to import dotfiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportDotfiles {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Path to dotfiles directory
    pub dotfiles_path: PathBuf,
    /// Patterns to include (glob patterns)
    pub include_patterns: Vec<String>,
    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Whether to analyze and convert to programs
    pub analyze_programs: bool,
}

/// Command to add a program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddProgram {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Program configuration
    pub program: ProgramConfig,
}

/// Command to update a program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProgram {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Program name
    pub program_name: String,
    /// Whether to enable or disable the program (None keeps current state)
    pub enable: Option<bool>,
    /// Updated program-specific settings (None keeps current settings)
    pub settings: Option<serde_json::Value>,
    /// Updated list of extra packages (None keeps current packages)
    pub extra_packages: Option<Vec<String>>,
}

/// Command to remove a program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveProgram {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Program name
    pub program_name: String,
}

/// Command to add a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddService {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Service configuration
    pub service: ServiceConfig,
}

/// Command to update a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateService {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Service name
    pub service_name: String,
    /// Whether to enable or disable the service (None keeps current state)
    pub enable: Option<bool>,
    /// Updated service-specific settings (None keeps current settings)
    pub settings: Option<serde_json::Value>,
    /// Updated environment variables (None keeps current environment)
    pub environment: Option<HashMap<String, String>>,
}

/// Command to remove a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveService {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Service name
    pub service_name: String,
}

/// Command to update shell configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShellConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Shell configuration
    pub shell: ShellConfig,
}

/// Command to update desktop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDesktopConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Desktop configuration
    pub desktop: DesktopConfig,
}

/// Command to add packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPackages {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Package category
    pub category: PackageCategory,
    /// Packages to add
    pub packages: Vec<String>,
}

/// Command to remove packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovePackages {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Package category
    pub category: PackageCategory,
    /// Packages to remove
    pub packages: Vec<String>,
}

/// Package categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PackageCategory {
    /// Core system utilities and tools
    System,
    /// Development tools and libraries
    Development,
    /// Desktop applications and utilities
    Desktop,
    /// User-defined custom packages
    Custom,
}

/// Command to generate Home Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Output path
    pub output_path: PathBuf,
    /// Whether to include flake configuration
    pub include_flake: bool,
}

/// Command to validate configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Home configuration ID
    pub config_id: HomeConfigId,
    /// Path to configuration to validate
    pub config_path: PathBuf,
}

/// Command to migrate from existing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateConfig {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Source configuration
    pub source: ConfigSource,
    /// User profile
    pub user_profile: UserProfile,
    /// Migration options
    pub options: MigrationOptions,
}

/// Migration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOptions {
    /// Whether to preserve comments from source configuration
    pub preserve_comments: bool,
    /// Whether to organize configuration by category
    pub organize_by_category: bool,
    /// Whether to detect and convert programs automatically
    pub detect_programs: bool,
    /// Whether to create backup of original configuration
    pub create_backup: bool,
}