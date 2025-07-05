//! Home Manager configuration analyzer

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

use super::{
    HomeConfiguration, ProgramConfig, ServiceConfig,
    HomeAnalysis, ProgramAnalysis, ServiceAnalysis, DotfileInfo,
    ConflictInfo, ConflictType, Suggestion, SuggestionType,
    Priority, ComplexityScore, SecurityScore, ResourceUsage,
};
use crate::parser::{NixParser, ParsedFile, NixExpr};
use crate::value_objects::NixValue;
use crate::NixDomainError;

/// Analyzer for Home Manager configurations
pub struct HomeManagerAnalyzer {
    parser: NixParser,
    config_cache: HashMap<String, HomeConfiguration>,
}

impl Default for HomeManagerAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl HomeManagerAnalyzer {
    /// Create a new Home Manager analyzer
    pub fn new() -> Self {
        Self {
            parser: NixParser::new(),
            config_cache: HashMap::new(),
        }
    }

    /// Analyze a Home Manager configuration file
    pub async fn analyze_home_config(&mut self, path: &Path) -> Result<HomeAnalysis, NixDomainError> {
        let parsed = self.parser.parse_file(path)?;
        let config = self.extract_home_config(&parsed)?;
        
        let analysis = HomeAnalysis {
            programs: self.analyze_programs(&config)?,
            services: self.analyze_services(&config)?,
            dotfiles: self.find_dotfile_mappings(&config)?,
            conflicts: self.detect_conflicts(&config)?,
            suggestions: self.generate_suggestions(&config)?,
        };
        
        Ok(analysis)
    }

    /// Extract Home Manager configuration from a parsed Nix file
    pub fn extract_home_config(&self, parsed: &ParsedFile) -> Result<HomeConfiguration, NixDomainError> {
        let mut config = HomeConfiguration::new();
        
        // Extract programs configuration
        if let Some(programs) = self.find_attribute_path(&parsed.expr, &["programs"]) {
            if let NixExpr::AttrSet(attrs) = programs {
                for (name, expr) in attrs {
                    if let Ok(program_config) = self.extract_program_config(expr) {
                        config.add_program(name.clone(), program_config);
                    }
                }
            }
        }
        
        // Extract services configuration
        if let Some(services) = self.find_attribute_path(&parsed.expr, &["services"]) {
            if let NixExpr::AttrSet(attrs) = services {
                for (name, expr) in attrs {
                    if let Ok(service_config) = self.extract_service_config(expr) {
                        config.add_service(name.clone(), service_config);
                    }
                }
            }
        }
        
        // Extract home.packages
        if let Some(packages) = self.find_attribute_path(&parsed.expr, &["home", "packages"]) {
            if let NixExpr::List(items) = packages {
                for item in items {
                    if let NixExpr::Identifier(pkg) = item {
                        config.home_packages.push(pkg.clone());
                    }
                }
            }
        }
        
        // Extract home.file mappings
        if let Some(files) = self.find_attribute_path(&parsed.expr, &["home", "file"]) {
            if let NixExpr::AttrSet(attrs) = files {
                for (target, expr) in attrs {
                    if let Ok(source) = self.extract_file_source(expr) {
                        config.add_file_mapping(source, PathBuf::from(target));
                    }
                }
            }
        }
        
        Ok(config)
    }

    fn extract_program_config(&self, expr: &NixExpr) -> Result<ProgramConfig, NixDomainError> {
        let mut config = ProgramConfig::new(false);
        
        if let NixExpr::AttrSet(attrs) = expr {
            // Check if enabled
            if let Some(NixExpr::Bool(enabled)) = attrs.get("enable") {
                config.enabled = *enabled;
            }
            
            // Extract settings
            if let Some(NixExpr::AttrSet(settings)) = attrs.get("settings") {
                for (key, value) in settings {
                    if let Ok(nix_value) = self.expr_to_value(value) {
                        config.settings.insert(key.clone(), nix_value);
                    }
                }
            }
            
            // Extract extraConfig
            if let Some(NixExpr::String(extra)) = attrs.get("extraConfig") {
                config.extra_config = Some(extra.clone());
            }
        }
        
        Ok(config)
    }

    fn extract_service_config(&self, expr: &NixExpr) -> Result<ServiceConfig, NixDomainError> {
        let mut enabled = false;
        let mut settings = HashMap::new();
        
        if let NixExpr::AttrSet(attrs) = expr {
            if let Some(NixExpr::Bool(e)) = attrs.get("enable") {
                enabled = *e;
            }
            
            for (key, value) in attrs {
                if key != "enable" {
                    if let Ok(nix_value) = self.expr_to_value(value) {
                        settings.insert(key.clone(), nix_value);
                    }
                }
            }
        }
        
        Ok(ServiceConfig { enabled, settings })
    }

    fn extract_file_source(&self, expr: &NixExpr) -> Result<PathBuf, NixDomainError> {
        if let NixExpr::AttrSet(attrs) = expr {
            if let Some(NixExpr::String(source)) = attrs.get("source") {
                return Ok(PathBuf::from(source));
            }
            if let Some(NixExpr::Path(path)) = attrs.get("source") {
                return Ok(path.clone());
            }
        }
        
        Err(NixDomainError::ParseError("Unable to extract file source".to_string()))
    }

    /// Analyze configured programs for dependencies, complexity, and security
    pub fn analyze_programs(&self, config: &HomeConfiguration) -> Result<Vec<ProgramAnalysis>, NixDomainError> {
        let mut analyses = Vec::new();
        
        for (name, program_config) in &config.programs {
            let analysis = ProgramAnalysis {
                name: name.clone(),
                enabled: program_config.is_enabled(),
                dependencies: self.find_program_dependencies(name, program_config)?,
                configuration_complexity: self.calculate_complexity(program_config),
                security_score: self.assess_security(name, program_config)?,
            };
            analyses.push(analysis);
        }
        
        Ok(analyses)
    }

    /// Analyze configured services for resource usage
    pub fn analyze_services(&self, config: &HomeConfiguration) -> Result<Vec<ServiceAnalysis>, NixDomainError> {
        let mut analyses = Vec::new();
        
        for (name, service_config) in &config.services {
            let analysis = ServiceAnalysis {
                name: name.clone(),
                enabled: service_config.enabled,
                resource_usage: self.estimate_resource_usage(name),
            };
            analyses.push(analysis);
        }
        
        Ok(analyses)
    }

    /// Find and analyze dotfile mappings in the configuration
    pub fn find_dotfile_mappings(&self, config: &HomeConfiguration) -> Result<Vec<DotfileInfo>, NixDomainError> {
        let mut dotfiles = Vec::new();
        
        for mapping in &config.file_mappings {
            if let Ok(metadata) = fs::metadata(&mapping.source) {
                let info = DotfileInfo {
                    path: mapping.source.clone(),
                    program: self.identify_program_from_path(&mapping.source),
                    size: metadata.len(),
                    last_modified: metadata.modified().ok(),
                };
                dotfiles.push(info);
            }
        }
        
        Ok(dotfiles)
    }

    /// Detect configuration conflicts and inconsistencies
    pub fn detect_conflicts(&self, config: &HomeConfiguration) -> Result<Vec<ConflictInfo>, NixDomainError> {
        let mut conflicts = Vec::new();
        
        // Check for conflicting shell configurations
        let shells = ["bash", "zsh", "fish"];
        let enabled_shells: Vec<_> = shells.iter()
            .filter(|&&shell| {
                config.programs.get(shell)
                    .is_some_and(|p| p.enabled)
            })
            .collect();
        
        if enabled_shells.len() > 1 {
            conflicts.push(ConflictInfo {
                conflict_type: ConflictType::ConflictingSettings,
                description: "Multiple shells are enabled".to_string(),
                affected_items: enabled_shells.iter().map(|s| (**s).to_string()).collect(),
            });
        }
        
        // Check for missing dependencies
        for (name, program) in &config.programs {
            if program.enabled {
                let deps = self.find_program_dependencies(name, program)?;
                for dep in deps {
                    if !config.programs.contains_key(&dep) && !config.home_packages.contains(&dep) {
                        conflicts.push(ConflictInfo {
                            conflict_type: ConflictType::MissingDependency,
                            description: format!("{name} requires {dep} but it's not configured"),
                            affected_items: vec![name.clone(), dep],
                        });
                    }
                }
            }
        }
        
        Ok(conflicts)
    }

    /// Generate suggestions for improving the configuration
    pub fn generate_suggestions(&self, config: &HomeConfiguration) -> Result<Vec<Suggestion>, NixDomainError> {
        let mut suggestions = Vec::new();
        
        // Suggest enabling useful programs
        let useful_programs = ["direnv", "starship", "fzf", "ripgrep"];
        for program in useful_programs {
            if !config.programs.contains_key(program) {
                suggestions.push(Suggestion {
                    suggestion_type: SuggestionType::EnableProgram,
                    description: format!("Consider enabling {program} for improved productivity"),
                    priority: Priority::Low,
                });
            }
        }
        
        // Check for security improvements
        if let Some(git) = config.programs.get("git") {
            if git.enabled && !git.settings.contains_key("signing") {
                suggestions.push(Suggestion {
                    suggestion_type: SuggestionType::SecurityImprovement,
                    description: "Enable Git commit signing for better security".to_string(),
                    priority: Priority::Medium,
                });
            }
        }
        
        // Check for unused programs
        for (name, program) in &config.programs {
            if !program.enabled && !program.settings.is_empty() {
                suggestions.push(Suggestion {
                    suggestion_type: SuggestionType::DisableUnusedProgram,
                    description: format!("{name} has configuration but is disabled"),
                    priority: Priority::Low,
                });
            }
        }
        
        Ok(suggestions)
    }

    /// Migrate existing dotfiles to a Home Manager configuration
    pub fn migrate_from_dotfiles(&self, dotfiles_dir: &Path) -> Result<HomeConfiguration, NixDomainError> {
        let mut config = HomeConfiguration::new();
        
        // Scan for common dotfiles
        let entries = fs::read_dir(dotfiles_dir)
            .map_err(|e| NixDomainError::ParseError(format!("Failed to read directory: {e}")))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| NixDomainError::ParseError(format!("Failed to read entry: {e}")))?;
            let path = entry.path();
            
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                match file_name {
                    ".gitconfig" | "gitconfig" => {
                        let git_config = ProgramConfig::new(true);
                        config.add_program("git".to_string(), git_config);
                    }
                    ".vimrc" | "vimrc" => {
                        let mut vim_config = ProgramConfig::new(true);
                        if let Ok(content) = fs::read_to_string(&path) {
                            vim_config.extra_config = Some(content);
                        }
                        config.add_program("vim".to_string(), vim_config);
                    }
                    ".bashrc" | "bashrc" => {
                        let mut bash_config = ProgramConfig::new(true);
                        if let Ok(content) = fs::read_to_string(&path) {
                            bash_config.extra_config = Some(content);
                        }
                        config.add_program("bash".to_string(), bash_config);
                    }
                    _ => {
                        // Add as raw file mapping
                        let target = PathBuf::from(format!(".{file_name}"));
                        config.add_file_mapping(path.clone(), target);
                    }
                }
            }
        }
        
        Ok(config)
    }

    // Helper methods
    fn find_attribute_path<'a>(&self, expr: &'a NixExpr, path: &[&str]) -> Option<&'a NixExpr> {
        let mut current = expr;
        
        for segment in path {
            if let NixExpr::AttrSet(attrs) = current {
                current = attrs.get(*segment)?;
            } else {
                return None;
            }
        }
        
        Some(current)
    }

    fn expr_to_value(&self, expr: &NixExpr) -> Result<NixValue, NixDomainError> {
        match expr {
            NixExpr::String(s) => Ok(NixValue::String(s.clone())),
            NixExpr::Int(i) => Ok(NixValue::Int(*i)),
            NixExpr::Float(f) => Ok(NixValue::Float(*f)),
            NixExpr::Bool(b) => Ok(NixValue::Bool(*b)),
            NixExpr::Path(p) => Ok(NixValue::Path(p.clone())),
            NixExpr::List(items) => {
                let values: Result<Vec<_>, _> = items.iter()
                    .map(|e| self.expr_to_value(e))
                    .collect();
                Ok(NixValue::List(values?))
            }
            _ => Err(NixDomainError::ParseError("Unsupported expression type".to_string())),
        }
    }

    fn find_program_dependencies(&self, name: &str, _config: &ProgramConfig) -> Result<Vec<String>, NixDomainError> {
        // Common program dependencies
        let deps = match name {
            "neovim" => vec!["ripgrep", "fd"],
            "emacs" => vec!["git", "ripgrep"],
            "tmux" => vec!["bash"],
            "starship" => vec!["git"],
            _ => vec![],
        };
        
        Ok(deps.into_iter().map(String::from).collect())
    }

    fn calculate_complexity(&self, config: &ProgramConfig) -> ComplexityScore {
        let mut score = 0;
        
        // Base complexity from number of settings
        score += config.settings.len() as u32;
        
        // Extra complexity for custom config
        if config.extra_config.is_some() {
            score += 10;
        }
        
        ComplexityScore(score)
    }

    fn assess_security(&self, name: &str, config: &ProgramConfig) -> Result<SecurityScore, NixDomainError> {
        let mut score = 100;
        
        // Program-specific security checks
        match name {
            "ssh" => {
                if !config.settings.contains_key("passwordAuthentication") {
                    score -= 10;
                }
            }
            "git" => {
                if !config.settings.contains_key("signing") {
                    score -= 20;
                }
            }
            _ => {}
        }
        
        Ok(SecurityScore(score))
    }

    fn estimate_resource_usage(&self, name: &str) -> ResourceUsage {
        // Rough estimates based on service type
        match name {
            "syncthing" => ResourceUsage {
                memory_mb: Some(200),
                cpu_percent: Some(5.0),
            },
            "gpg-agent" => ResourceUsage {
                memory_mb: Some(50),
                cpu_percent: Some(1.0),
            },
            _ => ResourceUsage {
                memory_mb: None,
                cpu_percent: None,
            },
        }
    }

    fn identify_program_from_path(&self, path: &Path) -> Option<String> {
        let file_name = path.file_name()?.to_str()?;
        
        match file_name {
            ".gitconfig" | "gitconfig" => Some("git".to_string()),
            ".vimrc" | "vimrc" | ".vim" => Some("vim".to_string()),
            ".bashrc" | ".bash_profile" => Some("bash".to_string()),
            ".zshrc" | ".zshenv" => Some("zsh".to_string()),
            ".tmux.conf" => Some("tmux".to_string()),
            _ => None,
        }
    }
} 