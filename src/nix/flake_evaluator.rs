// Copyright 2025 Cowboy AI, LLC.

//! Flake Evaluator
//!
//! Evaluates Nix flakes using the Nix CLI to extract packages, devShells,
//! checks, and apps from the outputs function.
//!
//! This complements FlakeAnalyzer by providing evaluated (not just static) data.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

// ============================================================================
// Evaluated Flake Structure
// ============================================================================

/// Complete evaluated flake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedFlake {
    /// Flake description
    pub description: Option<String>,

    /// Packages per system
    pub packages: HashMap<String, HashMap<String, PackageInfo>>,

    /// Development shells per system
    pub dev_shells: HashMap<String, HashMap<String, DevShellInfo>>,

    /// Checks per system
    pub checks: HashMap<String, HashMap<String, CheckInfo>>,

    /// Apps per system
    pub apps: HashMap<String, HashMap<String, AppInfo>>,
}

/// Package information from evaluated flake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,

    /// Package type (from nix flake show)
    #[serde(rename = "type")]
    pub pkg_type: String,

    /// Description if available
    pub description: Option<String>,
}

/// Development shell information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevShellInfo {
    /// Shell name
    pub name: String,

    /// Shell type
    #[serde(rename = "type")]
    pub shell_type: String,

    /// Description if available
    pub description: Option<String>,
}

/// Check information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInfo {
    /// Check name
    pub name: String,

    /// Check type
    #[serde(rename = "type")]
    pub check_type: String,

    /// Description if available
    pub description: Option<String>,
}

/// App information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// App name
    pub name: String,

    /// App type
    #[serde(rename = "type")]
    pub app_type: String,

    /// Description if available
    pub description: Option<String>,
}

// ============================================================================
// Flake Evaluator
// ============================================================================

/// Evaluates flakes using Nix CLI
pub struct FlakeEvaluator {
    /// Nix command to use (default: "nix")
    nix_command: String,
}

impl FlakeEvaluator {
    /// Create a new flake evaluator
    pub fn new() -> Self {
        Self {
            nix_command: "nix".to_string(),
        }
    }

    /// Create evaluator with custom nix command
    pub fn with_command(command: String) -> Self {
        Self {
            nix_command: command,
        }
    }

    /// Evaluate a flake at the given path
    pub fn evaluate<P: AsRef<Path>>(&self, flake_path: P) -> Result<EvaluatedFlake, EvaluationError> {
        let path_str = flake_path.as_ref().to_string_lossy();

        // Run nix flake show --json
        let output = Command::new(&self.nix_command)
            .args(["flake", "show", "--json"])
            .arg(path_str.as_ref())
            .output()
            .map_err(|e| EvaluationError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(EvaluationError::NixError(stderr.to_string()));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| EvaluationError::ParseError(e.to_string()))?;

        // Parse JSON output
        self.parse_flake_show(&stdout)
    }

    /// Parse the output of `nix flake show --json`
    fn parse_flake_show(&self, json_str: &str) -> Result<EvaluatedFlake, EvaluationError> {
        let json: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| EvaluationError::ParseError(e.to_string()))?;

        let mut flake = EvaluatedFlake {
            description: None,
            packages: HashMap::new(),
            dev_shells: HashMap::new(),
            checks: HashMap::new(),
            apps: HashMap::new(),
        };

        // Extract description
        if let Some(desc) = json.get("description").and_then(|d| d.as_str()) {
            flake.description = Some(desc.to_string());
        }

        // Process each system
        if let Some(obj) = json.as_object() {
            for (key, value) in obj {
                match key.as_str() {
                    "packages" => {
                        if let Some(packages_obj) = value.as_object() {
                            for (system, system_packages) in packages_obj {
                                let mut system_map = HashMap::new();
                                if let Some(pkgs) = system_packages.as_object() {
                                    for (pkg_name, pkg_info) in pkgs {
                                        let info = self.parse_package_info(pkg_name, pkg_info);
                                        system_map.insert(pkg_name.clone(), info);
                                    }
                                }
                                flake.packages.insert(system.clone(), system_map);
                            }
                        }
                    }
                    "devShells" => {
                        if let Some(shells_obj) = value.as_object() {
                            for (system, system_shells) in shells_obj {
                                let mut system_map = HashMap::new();
                                if let Some(shells) = system_shells.as_object() {
                                    for (shell_name, shell_info) in shells {
                                        let info = self.parse_devshell_info(shell_name, shell_info);
                                        system_map.insert(shell_name.clone(), info);
                                    }
                                }
                                flake.dev_shells.insert(system.clone(), system_map);
                            }
                        }
                    }
                    "checks" => {
                        if let Some(checks_obj) = value.as_object() {
                            for (system, system_checks) in checks_obj {
                                let mut system_map = HashMap::new();
                                if let Some(checks) = system_checks.as_object() {
                                    for (check_name, check_info) in checks {
                                        let info = self.parse_check_info(check_name, check_info);
                                        system_map.insert(check_name.clone(), info);
                                    }
                                }
                                flake.checks.insert(system.clone(), system_map);
                            }
                        }
                    }
                    "apps" => {
                        if let Some(apps_obj) = value.as_object() {
                            for (system, system_apps) in apps_obj {
                                let mut system_map = HashMap::new();
                                if let Some(apps) = system_apps.as_object() {
                                    for (app_name, app_info) in apps {
                                        let info = self.parse_app_info(app_name, app_info);
                                        system_map.insert(app_name.clone(), info);
                                    }
                                }
                                flake.apps.insert(system.clone(), system_map);
                            }
                        }
                    }
                    _ => {} // Ignore other fields
                }
            }
        }

        Ok(flake)
    }

    fn parse_package_info(&self, name: &str, value: &serde_json::Value) -> PackageInfo {
        PackageInfo {
            name: name.to_string(),
            pkg_type: value.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("derivation")
                .to_string(),
            description: value.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }

    fn parse_devshell_info(&self, name: &str, value: &serde_json::Value) -> DevShellInfo {
        DevShellInfo {
            name: name.to_string(),
            shell_type: value.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("derivation")
                .to_string(),
            description: value.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }

    fn parse_check_info(&self, name: &str, value: &serde_json::Value) -> CheckInfo {
        CheckInfo {
            name: name.to_string(),
            check_type: value.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("derivation")
                .to_string(),
            description: value.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }

    fn parse_app_info(&self, name: &str, value: &serde_json::Value) -> AppInfo {
        AppInfo {
            name: name.to_string(),
            app_type: value.get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("app")
                .to_string(),
            description: value.get("description")
                .and_then(|d| d.as_str())
                .map(|s| s.to_string()),
        }
    }

    /// Check if nix command is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.nix_command)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for FlakeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Evaluation errors
#[derive(Debug, Clone)]
pub enum EvaluationError {
    /// Nix command failed to execute
    CommandFailed(String),

    /// Nix evaluation error
    NixError(String),

    /// Failed to parse JSON output
    ParseError(String),

    /// Nix not available
    NixNotAvailable,
}

impl std::fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::NixError(msg) => write!(f, "Nix error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NixNotAvailable => write!(f, "Nix command not available"),
        }
    }
}

impl std::error::Error for EvaluationError {}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Evaluate a flake at the given path
pub fn evaluate_flake<P: AsRef<Path>>(path: P) -> Result<EvaluatedFlake, EvaluationError> {
    let evaluator = FlakeEvaluator::new();
    evaluator.evaluate(path)
}

/// Check if Nix is available
pub fn nix_available() -> bool {
    FlakeEvaluator::new().is_available()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = FlakeEvaluator::new();
        assert_eq!(evaluator.nix_command, "nix");
    }

    #[test]
    fn test_nix_availability() {
        let evaluator = FlakeEvaluator::new();
        // This will pass if nix is installed
        let available = evaluator.is_available();
        println!("Nix available: {}", available);
    }

    #[test]
    #[ignore] // Only run when explicitly requested (requires nix)
    fn test_evaluate_current_flake() {
        let evaluator = FlakeEvaluator::new();
        if !evaluator.is_available() {
            println!("Skipping: Nix not available");
            return;
        }

        let result = evaluator.evaluate(".");
        match result {
            Ok(flake) => {
                println!("Description: {:?}", flake.description);
                println!("Packages: {} systems", flake.packages.len());
                println!("DevShells: {} systems", flake.dev_shells.len());
            }
            Err(e) => {
                println!("Evaluation failed: {}", e);
            }
        }
    }
}
