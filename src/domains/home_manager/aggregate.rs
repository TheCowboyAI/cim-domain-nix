// Copyright 2025 Cowboy AI, LLC.

//! Aggregates for Home Manager domain

use super::commands::*;
use super::events::{HomeManagerDomainEvent, *};
use super::value_objects::*;
use crate::value_objects::CausationId;
use cim_domain::DomainEvent;
use std::collections::HashMap;
use chrono::Utc;

/// Home configuration aggregate
#[derive(Debug, Clone)]
pub struct HomeConfigAggregate {
    /// Configuration ID
    pub id: HomeConfigId,
    /// User profile
    pub user_profile: UserProfile,
    /// Programs configuration
    pub programs: HashMap<String, ProgramConfig>,
    /// Services configuration
    pub services: HashMap<String, ServiceConfig>,
    /// Shell configuration
    pub shell: Option<ShellConfig>,
    /// Desktop configuration
    pub desktop: Option<DesktopConfig>,
    /// Package sets
    pub packages: PackageSet,
    /// Dotfiles
    pub dotfiles: Vec<DotfileEntry>,
    /// Whether this aggregate exists
    pub exists: bool,
}

impl HomeConfigAggregate {
    /// Create a new home config aggregate
    pub fn new(id: HomeConfigId) -> Self {
        Self {
            id,
            user_profile: UserProfile {
                username: String::new(),
                full_name: None,
                email: None,
                home_directory: std::path::PathBuf::new(),
                shell: None,
            },
            programs: HashMap::new(),
            services: HashMap::new(),
            shell: None,
            desktop: None,
            packages: PackageSet {
                system: Vec::new(),
                development: Vec::new(),
                desktop: Vec::new(),
                custom: Vec::new(),
            },
            dotfiles: Vec::new(),
            exists: false,
        }
    }
    
    /// Handle create home config command
    pub fn handle_create(
        &self,
        cmd: CreateHomeConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if self.exists {
            return Err("Home configuration already exists".to_string());
        }
        
        let event = HomeConfigCreated {
            config_id: self.id,
            user_profile: cmd.user_profile,
            packages: cmd.packages,
            shell: cmd.shell,
            desktop: cmd.desktop,
            created_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ConfigCreated(event))])
    }
    
    /// Handle import dotfiles command
    pub fn handle_import_dotfiles(
        &self,
        cmd: ImportDotfiles,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        // In a real implementation, this would analyze the dotfiles
        let imported_files = Vec::new(); // TODO: Implement dotfile analysis
        let detected_programs = Vec::new(); // TODO: Implement program detection
        
        let event = DotfilesImported {
            config_id: self.id,
            dotfiles_path: cmd.dotfiles_path,
            imported_files,
            detected_programs,
            imported_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::DotfilesImported(event))])
    }
    
    /// Handle add program command
    pub fn handle_add_program(
        &self,
        cmd: AddProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        if self.programs.contains_key(&cmd.program.name) {
            return Err(format!("Program '{}' already exists", cmd.program.name));
        }
        
        let event = ProgramAdded {
            config_id: self.id,
            program: cmd.program,
            added_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ProgramAdded(event))])
    }
    
    /// Handle update program command
    pub fn handle_update_program(
        &self,
        cmd: UpdateProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        let old_config = self.programs.get(&cmd.program_name)
            .ok_or_else(|| format!("Program '{}' not found", cmd.program_name))?;
        
        let mut new_config = old_config.clone();
        if let Some(enable) = cmd.enable {
            new_config.enable = enable;
        }
        if let Some(settings) = cmd.settings {
            new_config.settings = settings;
        }
        if let Some(extra_packages) = cmd.extra_packages {
            new_config.extra_packages = extra_packages;
        }
        
        let event = ProgramUpdated {
            config_id: self.id,
            program_name: cmd.program_name,
            old_config: old_config.clone(),
            new_config,
            updated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ProgramUpdated(event))])
    }
    
    /// Handle remove program command
    pub fn handle_remove_program(
        &self,
        cmd: RemoveProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        if !self.programs.contains_key(&cmd.program_name) {
            return Err(format!("Program '{}' not found", cmd.program_name));
        }
        
        let event = ProgramRemoved {
            config_id: self.id,
            program_name: cmd.program_name,
            removed_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ProgramRemoved(event))])
    }
    
    /// Handle add service command
    pub fn handle_add_service(
        &self,
        cmd: AddService,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        if self.services.contains_key(&cmd.service.name) {
            return Err(format!("Service '{}' already exists", cmd.service.name));
        }
        
        let event = ServiceAdded {
            config_id: self.id,
            service: cmd.service,
            added_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ServiceAdded(event))])
    }
    
    /// Handle update shell config command
    pub fn handle_update_shell_config(
        &self,
        cmd: UpdateShellConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        let event = ShellConfigUpdated {
            config_id: self.id,
            old_shell: self.shell.clone(),
            new_shell: cmd.shell,
            updated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ShellConfigUpdated(event))])
    }
    
    /// Handle update desktop config command
    pub fn handle_update_desktop_config(
        &self,
        cmd: UpdateDesktopConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        let event = DesktopConfigUpdated {
            config_id: self.id,
            old_desktop: self.desktop.clone(),
            new_desktop: cmd.desktop,
            updated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::DesktopConfigUpdated(event))])
    }
    
    /// Handle add packages command
    pub fn handle_add_packages(
        &self,
        cmd: AddPackages,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        let event = PackagesAdded {
            config_id: self.id,
            category: cmd.category,
            packages: cmd.packages,
            added_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::PackagesAdded(event))])
    }
    
    /// Handle remove packages command
    pub fn handle_remove_packages(
        &self,
        cmd: RemovePackages,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        // Check if packages exist in the category
        let existing_packages = match &cmd.category {
            PackageCategory::System => &self.packages.system,
            PackageCategory::Development => &self.packages.development,
            PackageCategory::Desktop => &self.packages.desktop,
            PackageCategory::Custom => &self.packages.custom,
        };
        
        for pkg in &cmd.packages {
            if !existing_packages.contains(pkg) {
                return Err(format!("Package '{}' not found in {:?} category", pkg, cmd.category));
            }
        }
        
        let event = PackagesRemoved {
            config_id: self.id,
            category: cmd.category,
            packages: cmd.packages,
            removed_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::PackagesRemoved(event))])
    }
    
    /// Handle generate config command
    pub fn handle_generate_config(
        &self,
        cmd: GenerateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        // In a real implementation, this would generate the actual files
        let generated_files = vec![
            cmd.output_path.join("home.nix"),
        ];
        
        let event = ConfigGenerated {
            config_id: self.id,
            output_path: cmd.output_path,
            include_flake: cmd.include_flake,
            generated_files,
            generated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ConfigGenerated(event))])
    }
    
    /// Handle validate config command
    pub fn handle_validate_config(
        &self,
        cmd: ValidateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Home configuration does not exist".to_string());
        }
        
        // In a real implementation, this would actually validate the config
        let is_valid = true;
        let errors = Vec::new();
        let warnings = Vec::new();
        
        let event = ConfigValidated {
            config_id: self.id,
            config_path: cmd.config_path,
            is_valid,
            errors,
            warnings,
            validated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::ConfigValidated(event))])
    }
}

/// Migration aggregate
#[derive(Debug, Clone)]
pub struct MigrationAggregate {
    /// Migration ID
    pub id: uuid::Uuid,
    /// Migration status
    pub status: MigrationStatus,
    /// Source configuration
    pub source: Option<ConfigSource>,
    /// Target config ID (once created)
    pub config_id: Option<HomeConfigId>,
    /// Whether this aggregate exists
    pub exists: bool,
}

impl MigrationAggregate {
    /// Create a new migration aggregate
    pub fn new(id: uuid::Uuid) -> Self {
        Self {
            id,
            status: MigrationStatus::NotStarted,
            source: None,
            config_id: None,
            exists: false,
        }
    }
    
    /// Handle start migration command
    pub fn handle_start_migration(
        &self,
        cmd: MigrateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if self.exists {
            return Err("Migration already exists".to_string());
        }
        
        let event = MigrationStarted {
            migration_id: self.id,
            source: cmd.source,
            user_profile: cmd.user_profile,
            options: cmd.options,
            started_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(HomeManagerDomainEvent::MigrationStarted(event))])
    }
    
    /// Apply event to update aggregate state
    pub fn apply(&mut self, event: &HomeManagerDomainEvent) {
        match event {
            HomeManagerDomainEvent::MigrationStarted(e) => {
                self.exists = true;
                self.status = MigrationStatus::Analyzing;
                self.source = Some(e.source.clone());
            }
            HomeManagerDomainEvent::MigrationCompleted(e) => {
                self.status = MigrationStatus::Completed;
                self.config_id = Some(e.config_id);
            }
            HomeManagerDomainEvent::MigrationFailed(e) => {
                self.status = MigrationStatus::Failed(e.error.clone());
            }
            _ => {} // Other events don't affect migration aggregate
        }
    }
}

impl HomeConfigAggregate {
    /// Apply event to update aggregate state
    pub fn apply(&mut self, event: &HomeManagerDomainEvent) {
        match event {
                HomeManagerDomainEvent::ConfigCreated(e) => {
                    self.exists = true;
                    self.user_profile = e.user_profile.clone();
                    self.packages = e.packages.clone();
                    self.shell = e.shell.clone();
                    self.desktop = e.desktop.clone();
                }
                HomeManagerDomainEvent::DotfilesImported(e) => {
                    for dotfile in &e.imported_files {
                        self.dotfiles.push(dotfile.clone());
                    }
                }
                HomeManagerDomainEvent::ProgramAdded(e) => {
                    self.programs.insert(e.program.name.clone(), e.program.clone());
                }
                HomeManagerDomainEvent::ProgramUpdated(e) => {
                    self.programs.insert(e.program_name.clone(), e.new_config.clone());
                }
                HomeManagerDomainEvent::ProgramRemoved(e) => {
                    self.programs.remove(&e.program_name);
                }
                HomeManagerDomainEvent::ServiceAdded(e) => {
                    self.services.insert(e.service.name.clone(), e.service.clone());
                }
                HomeManagerDomainEvent::ServiceUpdated(e) => {
                    self.services.insert(e.service_name.clone(), e.new_config.clone());
                }
                HomeManagerDomainEvent::ServiceRemoved(e) => {
                    self.services.remove(&e.service_name);
                }
                HomeManagerDomainEvent::ShellConfigUpdated(e) => {
                    self.shell = Some(e.new_shell.clone());
                }
                HomeManagerDomainEvent::DesktopConfigUpdated(e) => {
                    self.desktop = Some(e.new_desktop.clone());
                }
                HomeManagerDomainEvent::PackagesAdded(e) => {
                    let packages = match &e.category {
                        PackageCategory::System => &mut self.packages.system,
                        PackageCategory::Development => &mut self.packages.development,
                        PackageCategory::Desktop => &mut self.packages.desktop,
                        PackageCategory::Custom => &mut self.packages.custom,
                    };
                    for pkg in &e.packages {
                        if !packages.contains(pkg) {
                            packages.push(pkg.clone());
                        }
                    }
                }
                HomeManagerDomainEvent::PackagesRemoved(e) => {
                    let packages = match &e.category {
                        PackageCategory::System => &mut self.packages.system,
                        PackageCategory::Development => &mut self.packages.development,
                        PackageCategory::Desktop => &mut self.packages.desktop,
                        PackageCategory::Custom => &mut self.packages.custom,
                    };
                    packages.retain(|pkg| !e.packages.contains(pkg));
                }
                _ => {} // Other events don't affect home config aggregate
        }
    }
}