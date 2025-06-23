//! Home Manager integration for analyzing and converting configurations

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

pub mod analyzer;
pub mod converter;
pub mod program_converter;

pub use analyzer::HomeManagerAnalyzer;
pub use converter::DotfileConverter;
pub use program_converter::ProgramConverter;

use crate::parser::{NixParser, ParsedFile};
use crate::value_objects::NixValue;
use crate::NixDomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeConfiguration {
    pub programs: HashMap<String, ProgramConfig>,
    pub services: HashMap<String, ServiceConfig>,
    pub file_mappings: Vec<FileMapping>,
    pub home_packages: Vec<String>,
    pub home_state_version: String,
}

impl HomeConfiguration {
    pub fn new() -> Self {
        Self {
            programs: HashMap::new(),
            services: HashMap::new(),
            file_mappings: Vec::new(),
            home_packages: Vec::new(),
            home_state_version: "24.05".to_string(),
        }
    }

    pub fn add_program(&mut self, name: String, config: ProgramConfig) {
        self.programs.insert(name, config);
    }

    pub fn add_service(&mut self, name: String, config: ServiceConfig) {
        self.services.insert(name, config);
    }

    pub fn add_file_mapping(&mut self, source: PathBuf, target: PathBuf) {
        self.file_mappings.push(FileMapping { source, target });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    pub enabled: bool,
    pub settings: HashMap<String, NixValue>,
    pub extra_config: Option<String>,
}

impl ProgramConfig {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            settings: HashMap::new(),
            extra_config: None,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub enabled: bool,
    pub settings: HashMap<String, NixValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAnalysis {
    pub programs: Vec<ProgramAnalysis>,
    pub services: Vec<ServiceAnalysis>,
    pub dotfiles: Vec<DotfileInfo>,
    pub conflicts: Vec<ConflictInfo>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramAnalysis {
    pub name: String,
    pub enabled: bool,
    pub dependencies: Vec<String>,
    pub configuration_complexity: ComplexityScore,
    pub security_score: SecurityScore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAnalysis {
    pub name: String,
    pub enabled: bool,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DotfileInfo {
    pub path: PathBuf,
    pub program: Option<String>,
    pub size: u64,
    pub last_modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub conflict_type: ConflictType,
    pub description: String,
    pub affected_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictType {
    DuplicateProgram,
    ConflictingSettings,
    MissingDependency,
    VersionMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    EnableProgram,
    DisableUnusedProgram,
    UpdateConfiguration,
    SecurityImprovement,
    PerformanceOptimization,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ComplexityScore(pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SecurityScore(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: Option<u32>,
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