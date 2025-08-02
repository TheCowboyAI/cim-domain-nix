//! Command and event handlers for the Nix domain
//!
//! This module will contain handlers that process commands
//! and react to events in the Nix ecosystem.

pub mod cqrs_adapter;

use crate::parser::{FlakeParser, NixFile};
use crate::{
    commands::{
        ActivateConfiguration, AddFlakeInput, BuildPackage, CheckFlake, CreateConfiguration,
        CreateFlake, CreateModule, CreateOverlay, DevelopFlake, EvaluateExpression, NixCommand,
        RunGarbageCollection, UpdateFlake,
    },
    events::{
        ConfigurationActivated, ConfigurationCreated, ExpressionEvaluated, FlakeCreated,
        FlakeInputAdded, FlakeUpdated, GarbageCollected, ModuleCreated, OverlayCreated,
        PackageBuilt,
    },
    templates::FlakeTemplate,
    value_objects::MessageIdentity,
    NixDomainError, Result,
};
use chrono::Utc;
use std::path::PathBuf;
use std::process::Command;
use tokio::fs;
use uuid::Uuid;

/// Handler for flake-related commands
pub struct FlakeCommandHandler;

impl Default for FlakeCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl FlakeCommandHandler {
    /// Create a new flake command handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Handle create flake command
    pub async fn create_flake(
        &self,
        cmd: CreateFlake,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        // Create directory if it doesn't exist
        fs::create_dir_all(&cmd.path).await?;

        // Determine template
        let template = if let Some(template_name) = &cmd.template {
            match template_name.as_str() {
                "rust" => FlakeTemplate::Rust,
                "python" => FlakeTemplate::Python,
                "nodejs" => FlakeTemplate::NodeJs,
                "go" => FlakeTemplate::Go,
                "cpp" => FlakeTemplate::Cpp,
                "haskell" => FlakeTemplate::Haskell,
                "polyglot" => FlakeTemplate::Polyglot,
                "nixos" => FlakeTemplate::NixOSSystem,
                "home-manager" => FlakeTemplate::HomeManager,
                "devshell" => FlakeTemplate::DevShell,
                _ => FlakeTemplate::Custom(template_name.clone()),
            }
        } else {
            FlakeTemplate::DevShell
        };

        // Generate flake.nix content
        let flake_content = template.generate_flake_nix();

        // Write flake.nix
        let flake_path = cmd.path.join("flake.nix");
        fs::write(&flake_path, flake_content).await?;

        // Write additional files for the template
        for (filename, content) in template.additional_files() {
            let file_path = cmd.path.join(&filename);

            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent).await?;
            }

            fs::write(file_path, content).await?;
        }

        // Initialize git repo if not already initialized
        if !cmd.path.join(".git").exists() {
            Command::new("git")
                .arg("init")
                .current_dir(&cmd.path)
                .output()
                .map_err(|e| NixDomainError::CommandError(format!("Failed to init git: {e}")))?;
        }

        // Add flake.nix to git
        Command::new("git")
            .args(["add", "flake.nix"])
            .current_dir(&cmd.path)
            .output()
            .map_err(|e| {
                NixDomainError::CommandError(format!("Failed to add flake.nix to git: {e}"))
            })?;

        // Run nix flake init if template was not provided
        if cmd.template.is_none() {
            Command::new("nix")
                .args(["flake", "init"])
                .current_dir(&cmd.path)
                .output()
                .map_err(|e| NixDomainError::CommandError(format!("Failed to init flake: {e}")))?;
        }

        let event = FlakeCreated {
            flake_id: Uuid::new_v4(),
            identity: cmd.identity,
            path: cmd.path,
            description: cmd.description,
            template: cmd.template,
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }

    /// Handle update flake command
    pub async fn update_flake(
        &self,
        cmd: UpdateFlake,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        // Run nix flake update
        let output = Command::new("nix")
            .args(["flake", "update"])
            .current_dir(&cmd.path)
            .output()
            .map_err(|e| NixDomainError::CommandError(format!("Failed to update flake: {e}")))?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let event = FlakeUpdated {
            flake_id: Uuid::new_v4(),
            identity: cmd.identity,
            path: cmd.path,
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }

    /// Handle add flake input command
    pub async fn add_flake_input(
        &self,
        cmd: AddFlakeInput,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        // Parse the flake using the new parser
        let flake_path = cmd.path.join("flake.nix");
        let file = NixFile::parse_file(&flake_path)?;
        let mut flake = FlakeParser::parse(&file)?;

        // Add the input using AST manipulation
        flake.add_input(&cmd.name, &cmd.url)?;

        // Write back the formatted result
        fs::write(&flake_path, flake.to_string()).await?;

        // Run nix flake update for the new input
        let output = Command::new("nix")
            .args(["flake", "update", &cmd.name])
            .current_dir(&cmd.path)
            .output()
            .map_err(|e| {
                NixDomainError::CommandError(format!("Failed to update flake input: {e}"))
            })?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let event = FlakeInputAdded {
            flake_id: Uuid::new_v4(),
            identity: cmd.identity,
            path: cmd.path.clone(),
            input_name: cmd.name,
            input_url: cmd.url,
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }

    /// Handle check flake command
    pub async fn check_flake(&self, cmd: CheckFlake) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let output = Command::new("nix")
            .args(["flake", "check"])
            .current_dir(&cmd.path)
            .output()
            .map_err(|e| NixDomainError::CommandError(format!("Failed to check flake: {e}")))?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // No event for check, it's a query operation
        Ok(vec![])
    }

    /// Handle develop flake command
    pub async fn develop_flake(
        &self,
        cmd: DevelopFlake,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let mut args = vec!["develop"];

        if let Some(command) = &cmd.command {
            args.push("-c");
            args.push(command);
        }

        let output = Command::new("nix")
            .args(&args)
            .current_dir(&cmd.path)
            .output()
            .map_err(|e| NixDomainError::CommandError(format!("Failed to enter dev shell: {e}")))?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // No event for develop, it's an interactive operation
        Ok(vec![])
    }
}

/// Handler for package build commands
pub struct PackageCommandHandler;

impl Default for PackageCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageCommandHandler {
    /// Create a new package command handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Handle build package command
    pub async fn build_package(
        &self,
        cmd: BuildPackage,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let start_time = std::time::Instant::now();

        // Build the package
        let mut args = vec!["build"];

        // Create the flake URI with attribute
        let flake_uri = format!("{}#{}", cmd.flake_ref, cmd.attribute);
        args.push(&flake_uri);

        // Add output path if specified
        let out_link_arg;
        if let Some(ref output_path) = cmd.output_path {
            out_link_arg = format!("--out-link={}", output_path.display());
            args.push(&out_link_arg);
        }

        let output = Command::new("nix")
            .args(&args)
            .output()
            .map_err(|e| NixDomainError::BuildError(format!("Failed to run nix build: {e}")))?;

        if !output.status.success() {
            return Err(NixDomainError::BuildError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let build_time = start_time.elapsed();

        // Get the output path
        let output_path = cmd.output_path.unwrap_or_else(|| PathBuf::from("./result"));

        let event = PackageBuilt {
            package_id: Uuid::new_v4(),
            identity: cmd.identity,
            flake_ref: cmd.flake_ref,
            attribute: cmd.attribute,
            output_path,
            build_time,
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }
}

/// Handler for expression evaluation
pub struct ExpressionCommandHandler;

impl Default for ExpressionCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpressionCommandHandler {
    /// Create a new expression command handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Handle evaluate expression command
    pub async fn evaluate_expression(
        &self,
        cmd: EvaluateExpression,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let output = Command::new("nix")
            .args(["eval", "--expr", &cmd.expression])
            .output()
            .map_err(|e| {
                NixDomainError::ExecutionError(format!("Failed to evaluate expression: {e}"))
            })?;

        if !output.status.success() {
            return Err(NixDomainError::ExecutionError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let event = ExpressionEvaluated {
            expression_id: Uuid::new_v4(),
            identity: cmd.identity,
            expression: cmd.expression,
            result,
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }
}

/// Handler for garbage collection
pub struct GarbageCollectionHandler;

impl Default for GarbageCollectionHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GarbageCollectionHandler {
    /// Create a new garbage collection handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Handle run garbage collection command
    pub async fn run_garbage_collection(
        &self,
        cmd: RunGarbageCollection,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let mut args = vec!["store", "gc"];

        // Add older-than flag if specified
        let days_str;
        if let Some(days) = cmd.older_than_days {
            days_str = format!("{days}d");
            args.push("--min-free");
            args.push(&days_str);
        }

        let output = Command::new("nix").args(&args).output().map_err(|e| {
            NixDomainError::CommandError(format!("Failed to run garbage collection: {e}"))
        })?;

        if !output.status.success() {
            return Err(NixDomainError::CommandError(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Parse output to get freed bytes
        let output_str = String::from_utf8_lossy(&output.stdout);
        let freed_bytes = parse_freed_bytes(&output_str).unwrap_or(0);

        let event = GarbageCollected {
            collection_id: Uuid::new_v4(),
            identity: cmd.identity,
            freed_bytes,
            removed_paths: vec![], // In a real implementation, we'd parse these from output
            timestamp: Utc::now(),
        };

        Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
    }
}

/// Main command handler that delegates to specific handlers
pub struct NixCommandHandler {
    /// Handler for flake operations
    flake_handler: FlakeCommandHandler,
    /// Handler for package operations
    package_handler: PackageCommandHandler,
    /// Handler for expression evaluation
    expression_handler: ExpressionCommandHandler,
    /// Handler for garbage collection
    gc_handler: GarbageCollectionHandler,
}

impl Default for NixCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl NixCommandHandler {
    /// Create a new Nix command handler
    #[must_use]
    pub fn new() -> Self {
        Self {
            flake_handler: FlakeCommandHandler::new(),
            package_handler: PackageCommandHandler::new(),
            expression_handler: ExpressionCommandHandler::new(),
            gc_handler: GarbageCollectionHandler::new(),
        }
    }

    /// Handle create flake command
    pub async fn handle_create_flake(
        &self,
        name: &str,
        description: &str,
        _packages: &[String],
        _dev_shells: &[String],
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let cmd = CreateFlake {
            identity: MessageIdentity::new_root(),
            path: PathBuf::from(name),
            description: description.to_string(),
            template: None,
        };
        self.flake_handler.create_flake(cmd).await
    }

    /// Handle update flake command
    pub async fn handle_update_flake(
        &self,
        path: &str,
        _description: Option<&str>,
        _packages: &[String],
        _dev_shells: &[String],
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let cmd = UpdateFlake {
            identity: MessageIdentity::new_root(),
            path: PathBuf::from(path),
        };
        self.flake_handler.update_flake(cmd).await
    }

    /// Handle build package command
    pub async fn handle_build_package(
        &self,
        flake_ref: &str,
        attribute: &str,
        output_path: Option<&str>,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        let cmd = BuildPackage {
            identity: MessageIdentity::new_root(),
            flake_ref: flake_ref.to_string(),
            attribute: crate::value_objects::AttributePath {
                segments: attribute.split('.').map(String::from).collect(),
            },
            output_path: output_path.map(PathBuf::from),
        };
        self.package_handler.build_package(cmd).await
    }

    /// Handle any Nix command by delegating to the appropriate handler
    pub async fn handle_command(
        &self,
        cmd: Box<dyn NixCommand>,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>> {
        // Downcast to specific command types
        if let Some(create_flake) = cmd.as_any().downcast_ref::<CreateFlake>() {
            self.flake_handler.create_flake(create_flake.clone()).await
        } else if let Some(update_flake) = cmd.as_any().downcast_ref::<UpdateFlake>() {
            self.flake_handler.update_flake(update_flake.clone()).await
        } else if let Some(add_input) = cmd.as_any().downcast_ref::<AddFlakeInput>() {
            self.flake_handler.add_flake_input(add_input.clone()).await
        } else if let Some(build_pkg) = cmd.as_any().downcast_ref::<BuildPackage>() {
            self.package_handler.build_package(build_pkg.clone()).await
        } else if let Some(eval_expr) = cmd.as_any().downcast_ref::<EvaluateExpression>() {
            self.expression_handler
                .evaluate_expression(eval_expr.clone())
                .await
        } else if let Some(run_gc) = cmd.as_any().downcast_ref::<RunGarbageCollection>() {
            self.gc_handler.run_garbage_collection(run_gc.clone()).await
        } else if let Some(check_flake) = cmd.as_any().downcast_ref::<CheckFlake>() {
            self.flake_handler.check_flake(check_flake.clone()).await
        } else if let Some(develop_flake) = cmd.as_any().downcast_ref::<DevelopFlake>() {
            self.flake_handler
                .develop_flake(develop_flake.clone())
                .await
        } else if let Some(create_module) = cmd.as_any().downcast_ref::<CreateModule>() {
            // For now, just return the event
            let event = ModuleCreated {
                event_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
                identity: create_module.identity.clone(),
                module: create_module.module.clone(),
            };
            Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
        } else if let Some(create_overlay) = cmd.as_any().downcast_ref::<CreateOverlay>() {
            // For now, just return the event
            let event = OverlayCreated {
                event_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
                identity: create_overlay.identity.clone(),
                overlay: create_overlay.overlay.clone(),
            };
            Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
        } else if let Some(create_config) = cmd.as_any().downcast_ref::<CreateConfiguration>() {
            // For now, just return the event
            let event = ConfigurationCreated {
                event_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
                identity: create_config.identity.clone(),
                configuration: create_config.configuration.clone(),
            };
            Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
        } else if let Some(activate_config) = cmd.as_any().downcast_ref::<ActivateConfiguration>() {
            // For now, just return the event
            let event = ConfigurationActivated {
                event_id: Uuid::new_v4(),
                occurred_at: Utc::now(),
                identity: activate_config.identity.clone(),
                configuration_id: Uuid::new_v4(), // In real impl, would look up the config
                generation: 1,                    // In real impl, would increment
                activation_type: activate_config.activation_type.clone(),
            };
            Ok(vec![Box::new(event) as Box<dyn std::any::Any + Send>])
        } else {
            Err(NixDomainError::Other("Unknown command type".to_string()))
        }
    }
}

/// Parse freed bytes from nix gc output
fn parse_freed_bytes(output: &str) -> Option<u64> {
    // Look for patterns like "1234 MiB freed"
    for line in output.lines() {
        if line.contains("freed") {
            // Extract number from line
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if let Ok(num) = part.parse::<f64>() {
                    // Check if next part is a unit
                    if i + 1 < parts.len() {
                        let unit = parts[i + 1];
                        return Some(match unit {
                            "B" => num as u64,
                            "KiB" => (num * 1024.0) as u64,
                            "MiB" => (num * 1024.0 * 1024.0) as u64,
                            "GiB" => (num * 1024.0 * 1024.0 * 1024.0) as u64,
                            _ => 0,
                        });
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_freed_bytes() {
        assert_eq!(parse_freed_bytes("100 MiB freed"), Some(104_857_600));
        assert_eq!(parse_freed_bytes("1.5 GiB freed"), Some(1_610_612_736));
        assert_eq!(parse_freed_bytes("no match"), None);
    }
}
