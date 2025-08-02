// Copyright 2025 Cowboy AI, LLC.

//! Service layer for Home Manager domain

use super::commands::*;
use super::handlers::*;
use super::value_objects::*;
use crate::value_objects::MessageIdentity;
use cim_domain::DomainEvent;
use std::sync::{Arc, Mutex};

/// Home Manager service
#[derive(Clone)]
pub struct HomeManagerService {
    /// Command handler
    pub command_handler: Arc<Mutex<HomeManagerCommandHandler>>,
    /// Query handler
    pub query_handler: Arc<Mutex<HomeManagerQueryHandler>>,
}

impl HomeManagerService {
    /// Create a new Home Manager service
    pub fn new() -> Self {
        Self {
            command_handler: Arc::new(Mutex::new(HomeManagerCommandHandler::new())),
            query_handler: Arc::new(Mutex::new(HomeManagerQueryHandler::new())),
        }
    }

    /// Create a new home configuration
    pub fn create_config(
        &self,
        user_profile: UserProfile,
        packages: PackageSet,
        shell: Option<ShellConfig>,
        desktop: Option<DesktopConfig>,
    ) -> Result<HomeConfigId, String> {
        let config_id = HomeConfigId::new();
        let cmd = CreateHomeConfig {
            identity: MessageIdentity::new_root(),
            user_profile,
            packages,
            shell,
            desktop,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let _events = command_handler.handle_create_config_with_id(config_id, cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(config_id)
    }

    /// Import dotfiles into configuration
    pub fn import_dotfiles(
        &self,
        config_id: HomeConfigId,
        dotfiles_path: std::path::PathBuf,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        analyze_programs: bool,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = ImportDotfiles {
            identity: MessageIdentity::new_root(),
            config_id,
            dotfiles_path,
            include_patterns,
            exclude_patterns,
            analyze_programs,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_import_dotfiles(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Add a program to configuration
    pub fn add_program(
        &self,
        config_id: HomeConfigId,
        program: ProgramConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = AddProgram {
            identity: MessageIdentity::new_root(),
            config_id,
            program,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_add_program(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Update a program configuration
    pub fn update_program(
        &self,
        config_id: HomeConfigId,
        program_name: String,
        enable: Option<bool>,
        settings: Option<serde_json::Value>,
        extra_packages: Option<Vec<String>>,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = UpdateProgram {
            identity: MessageIdentity::new_root(),
            config_id,
            program_name,
            enable,
            settings,
            extra_packages,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_update_program(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Remove a program from configuration
    pub fn remove_program(
        &self,
        config_id: HomeConfigId,
        program_name: String,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = RemoveProgram {
            identity: MessageIdentity::new_root(),
            config_id,
            program_name,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_remove_program(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Add a service to configuration
    pub fn add_service(
        &self,
        config_id: HomeConfigId,
        service: ServiceConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = AddService {
            identity: MessageIdentity::new_root(),
            config_id,
            service,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_add_service(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Update shell configuration
    pub fn update_shell_config(
        &self,
        config_id: HomeConfigId,
        shell: ShellConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = UpdateShellConfig {
            identity: MessageIdentity::new_root(),
            config_id,
            shell,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_update_shell_config(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Update desktop configuration
    pub fn update_desktop_config(
        &self,
        config_id: HomeConfigId,
        desktop: DesktopConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = UpdateDesktopConfig {
            identity: MessageIdentity::new_root(),
            config_id,
            desktop,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_update_desktop_config(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Add packages to configuration
    pub fn add_packages(
        &self,
        config_id: HomeConfigId,
        category: PackageCategory,
        packages: Vec<String>,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = AddPackages {
            identity: MessageIdentity::new_root(),
            config_id,
            category,
            packages,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_add_packages(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Remove packages from configuration
    pub fn remove_packages(
        &self,
        config_id: HomeConfigId,
        category: PackageCategory,
        packages: Vec<String>,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = RemovePackages {
            identity: MessageIdentity::new_root(),
            config_id,
            category,
            packages,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        let events = command_handler.handle_remove_packages(cmd)?;
        
        // Sync state to query handler
        drop(command_handler);
        self.sync_handlers();

        Ok(events)
    }

    /// Generate Home Manager configuration files
    pub fn generate_config(
        &self,
        config_id: HomeConfigId,
        output_path: std::path::PathBuf,
        include_flake: bool,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = GenerateConfig {
            identity: MessageIdentity::new_root(),
            config_id,
            output_path,
            include_flake,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        command_handler.handle_generate_config(cmd)
    }

    /// Validate Home Manager configuration
    pub fn validate_config(
        &self,
        config_id: HomeConfigId,
        config_path: std::path::PathBuf,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = ValidateConfig {
            identity: MessageIdentity::new_root(),
            config_id,
            config_path,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        command_handler.handle_validate_config(cmd)
    }

    /// Start configuration migration
    pub fn start_migration(
        &self,
        source: ConfigSource,
        user_profile: UserProfile,
        options: MigrationOptions,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let cmd = MigrateConfig {
            identity: MessageIdentity::new_root(),
            source,
            user_profile,
            options,
        };

        let mut command_handler = self.command_handler.lock().unwrap();
        command_handler.handle_start_migration(cmd)
    }

    /// Get configuration by ID
    pub fn get_config(&self, id: &HomeConfigId) -> Option<HomeConfigReadModel> {
        let query_handler = self.query_handler.lock().unwrap();
        query_handler.get_config(id).cloned()
    }

    /// Get all configurations
    pub fn get_all_configs(&self) -> Vec<HomeConfigReadModel> {
        let query_handler = self.query_handler.lock().unwrap();
        query_handler.get_all_configs().into_iter().cloned().collect()
    }

    /// Find configurations by program
    pub fn find_configs_by_program(&self, program_name: &str) -> Vec<HomeConfigReadModel> {
        let query_handler = self.query_handler.lock().unwrap();
        query_handler.find_configs_by_program(program_name)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Find configurations by service
    pub fn find_configs_by_service(&self, service_name: &str) -> Vec<HomeConfigReadModel> {
        let query_handler = self.query_handler.lock().unwrap();
        query_handler.find_configs_by_service(service_name)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Sync handlers
    fn sync_handlers(&self) {
        let command_handler = self.command_handler.lock().unwrap();
        let mut query_handler = self.query_handler.lock().unwrap();
        
        // Clear and rebuild query handler state from command handler
        query_handler.configs.clear();
        query_handler.programs_by_name.clear();
        query_handler.services_by_name.clear();
        
        for (id, aggregate) in &command_handler.configs {
            // Don't check exists flag - we want all configs
            if true {
                let read_model = HomeConfigReadModel {
                    id: *id,
                    user_profile: aggregate.user_profile.clone(),
                    programs: aggregate.programs.clone(),
                    services: aggregate.services.clone(),
                    shell: aggregate.shell.clone(),
                    desktop: aggregate.desktop.clone(),
                    packages: aggregate.packages.clone(),
                    dotfiles: aggregate.dotfiles.clone(),
                };
                
                // Update program index
                for program_name in aggregate.programs.keys() {
                    query_handler.programs_by_name
                        .entry(program_name.clone())
                        .or_insert_with(Vec::new)
                        .push(*id);
                }
                
                // Update service index
                for service_name in aggregate.services.keys() {
                    query_handler.services_by_name
                        .entry(service_name.clone())
                        .or_insert_with(Vec::new)
                        .push(*id);
                }
                
                query_handler.configs.insert(*id, read_model);
            }
        }
    }
}

impl Default for HomeManagerService {
    fn default() -> Self {
        Self::new()
    }
}