//! Home Manager configuration support

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;
use crate::value_objects::NixValue;

pub mod analyzer;
pub mod program_converter;
pub mod converter;

/// Represents a Home Manager configuration

pub use analyzer::HomeManagerAnalyzer;
pub use program_converter::ProgramConverter;

/// Configuration for a Home Manager setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeConfiguration {
    /// Configured programs with their settings
    pub programs: HashMap<String, ProgramConfig>,
    /// Configured services with their settings
    pub services: HashMap<String, ServiceConfig>,
    /// File mappings from source to target locations
    pub file_mappings: Vec<FileMapping>,
    /// List of packages to install in the home environment
    pub home_packages: Vec<String>,
    /// Home Manager state version
    pub home_state_version: String,
}

impl HomeConfiguration {
    /// Create a new empty Home Manager configuration
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
            services: HashMap::new(),
            file_mappings: Vec::new(),
            home_packages: Vec::new(),
            home_state_version: "24.05".to_string(),
        }
    }

    /// Add a program configuration
    pub fn add_program(&mut self, name: String, config: ProgramConfig) {
        self.programs.insert(name, config);
    }

    /// Add a service configuration
    pub fn add_service(&mut self, name: String, config: ServiceConfig) {
        self.services.insert(name, config);
    }

    /// Add a file mapping from source to target
    pub fn add_file_mapping(&mut self, source: PathBuf, target: PathBuf) {
        self.file_mappings.push(FileMapping { source, target });
    }
}

/// Configuration for a specific program in Home Manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// Whether the program is enabled
    pub enabled: bool,
    /// Program-specific settings
    pub settings: HashMap<String, NixValue>,
    /// Additional configuration as raw Nix code
    pub extra_config: Option<String>,
}

impl ProgramConfig {
    /// Create a new program configuration
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            settings: HashMap::new(),
            extra_config: None,
        }
    }

    /// Check if the program is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Configuration for a specific service in Home Manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Whether the service is enabled
    pub enabled: bool,
    /// Service-specific settings
    pub settings: HashMap<String, NixValue>,
}

/// Mapping of a file from source to target location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    /// Source file path
    pub source: PathBuf,
    /// Target file path in the home directory
    pub target: PathBuf,
}

/// Analysis results for a Home Manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAnalysis {
    /// Analysis of configured programs
    pub programs: Vec<ProgramAnalysis>,
    /// Analysis of configured services
    pub services: Vec<ServiceAnalysis>,
    /// Information about dotfiles
    pub dotfiles: Vec<DotfileInfo>,
    /// Detected configuration conflicts
    pub conflicts: Vec<ConflictInfo>,
    /// Suggested improvements
    pub suggestions: Vec<Suggestion>,
}

/// Analysis of a specific program configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAnalysis {
    /// Program name
    pub name: String,
    /// Whether the program is enabled
    pub enabled: bool,
    /// Dependencies of this program
    pub dependencies: Vec<String>,
    /// Complexity score of the configuration
    pub configuration_complexity: ComplexityScore,
    /// Security assessment score
    pub security_score: SecurityScore,
}

/// Analysis of a specific service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnalysis {
    /// Service name
    pub name: String,
    /// Whether the service is enabled
    pub enabled: bool,
    /// Estimated resource usage
    pub resource_usage: ResourceUsage,
}

/// Information about a dotfile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfileInfo {
    /// Path to the dotfile
    pub path: PathBuf,
    /// Associated program, if detected
    pub program: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub last_modified: Option<std::time::SystemTime>,
}

/// Information about a configuration conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Type of conflict detected
    pub conflict_type: ConflictType,
    /// Human-readable description of the conflict
    pub description: String,
    /// Items affected by this conflict
    pub affected_items: Vec<String>,
}

/// Types of configuration conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    /// Multiple programs trying to manage the same functionality
    DuplicateProgram,
    /// Settings that contradict each other
    ConflictingSettings,
    /// Required dependency is missing
    MissingDependency,
    /// Version requirements don't match
    VersionMismatch,
}

/// A suggestion for improving the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Human-readable description
    pub description: String,
    /// Priority level of the suggestion
    pub priority: Priority,
}

/// Types of configuration suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    /// Suggest enabling a useful program
    EnableProgram,
    /// Suggest disabling an unused program
    DisableUnusedProgram,
    /// Suggest updating a configuration
    UpdateConfiguration,
    /// Security-related improvement
    SecurityImprovement,
    /// Performance optimization suggestion
    PerformanceOptimization,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    /// Nice to have
    Low,
    /// Should consider
    Medium,
    /// Important to address
    High,
    /// Critical issue
    Critical,
}

/// Score representing configuration complexity (0-100)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComplexityScore(pub u32);

/// Score representing security assessment (0-100)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SecurityScore(pub u32);

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Estimated memory usage in megabytes
    pub memory_mb: Option<u32>,
    /// Estimated CPU usage percentage
    pub cpu_percent: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_configuration() {
        let mut config = HomeConfiguration::new();
        
        let git_config = ProgramConfig::new(true);
        config.add_program("git".to_string(), git_config);
        
        assert_eq!(config.programs.len(), 1);
        assert!(config.programs.get("git").unwrap().is_enabled());
    }
} 