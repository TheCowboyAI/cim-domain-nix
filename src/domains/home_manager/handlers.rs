// Copyright 2025 Cowboy AI, LLC.

//! CQRS handlers for Home Manager domain

use super::aggregate::*;
use super::commands::*;
use super::events::*;
use super::value_objects::*;
use cim_domain::DomainEvent;
use std::collections::HashMap;

/// Command handler for Home Manager domain
#[derive(Debug, Clone)]
pub struct HomeManagerCommandHandler {
    /// Home configurations mapped by their unique ID
    pub configs: HashMap<HomeConfigId, HomeConfigAggregate>,
    /// Active migrations mapped by migration ID
    pub migrations: HashMap<uuid::Uuid, MigrationAggregate>,
}

impl HomeManagerCommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            migrations: HashMap::new(),
        }
    }

    /// Handle create home config command
    pub fn handle_create_config(
        &mut self,
        cmd: CreateHomeConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = HomeConfigId::new();
        let aggregate = self.configs
            .entry(config_id)
            .or_insert_with(|| HomeConfigAggregate::new(config_id));

        let events = aggregate.handle_create(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.exists = true;
        aggregate.user_profile = cmd.user_profile;
        aggregate.packages = cmd.packages;
        aggregate.shell = cmd.shell;
        aggregate.desktop = cmd.desktop;

        Ok(events)
    }

    /// Handle create home config with specific ID
    pub fn handle_create_config_with_id(
        &mut self,
        config_id: HomeConfigId,
        cmd: CreateHomeConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let aggregate = self.configs
            .entry(config_id)
            .or_insert_with(|| HomeConfigAggregate::new(config_id));

        let events = aggregate.handle_create(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.exists = true;
        aggregate.user_profile = cmd.user_profile;
        aggregate.packages = cmd.packages;
        aggregate.shell = cmd.shell;
        aggregate.desktop = cmd.desktop;

        Ok(events)
    }

    /// Handle import dotfiles command
    pub fn handle_import_dotfiles(
        &mut self,
        cmd: ImportDotfiles,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let aggregate = self.configs
            .get_mut(&cmd.config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_import_dotfiles(cmd)?;
        
        // Note: In a real implementation, we would update dotfiles state here
        // For now, the dotfiles import doesn't change the aggregate state significantly

        Ok(events)
    }

    /// Handle add program command
    pub fn handle_add_program(
        &mut self,
        cmd: AddProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_add_program(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.programs.insert(cmd.program.name.clone(), cmd.program);

        Ok(events)
    }

    /// Handle update program command
    pub fn handle_update_program(
        &mut self,
        cmd: UpdateProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_update_program(cmd.clone())?;
        
        // Update aggregate state directly
        if let Some(program) = aggregate.programs.get_mut(&cmd.program_name) {
            if let Some(enable) = cmd.enable {
                program.enable = enable;
            }
            if let Some(settings) = cmd.settings {
                program.settings = settings;
            }
            if let Some(packages) = cmd.extra_packages {
                program.extra_packages = packages;
            }
        }

        Ok(events)
    }

    /// Handle remove program command
    pub fn handle_remove_program(
        &mut self,
        cmd: RemoveProgram,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_remove_program(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.programs.remove(&cmd.program_name);

        Ok(events)
    }

    /// Handle add service command
    pub fn handle_add_service(
        &mut self,
        cmd: AddService,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_add_service(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.services.insert(cmd.service.name.clone(), cmd.service);

        Ok(events)
    }

    /// Handle update shell config command
    pub fn handle_update_shell_config(
        &mut self,
        cmd: UpdateShellConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_update_shell_config(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.shell = Some(cmd.shell);

        Ok(events)
    }

    /// Handle update desktop config command
    pub fn handle_update_desktop_config(
        &mut self,
        cmd: UpdateDesktopConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_update_desktop_config(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.desktop = Some(cmd.desktop);

        Ok(events)
    }

    /// Handle add packages command
    pub fn handle_add_packages(
        &mut self,
        cmd: AddPackages,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_add_packages(cmd.clone())?;
        
        // Update aggregate state directly
        let packages = match &cmd.category {
            PackageCategory::System => &mut aggregate.packages.system,
            PackageCategory::Development => &mut aggregate.packages.development,
            PackageCategory::Desktop => &mut aggregate.packages.desktop,
            PackageCategory::Custom => &mut aggregate.packages.custom,
        };
        for pkg in cmd.packages {
            if !packages.contains(&pkg) {
                packages.push(pkg);
            }
        }

        Ok(events)
    }

    /// Handle remove packages command
    pub fn handle_remove_packages(
        &mut self,
        cmd: RemovePackages,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let config_id = cmd.config_id;
        let aggregate = self.configs
            .get_mut(&config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_remove_packages(cmd.clone())?;
        
        // Update aggregate state directly
        let packages = match &cmd.category {
            PackageCategory::System => &mut aggregate.packages.system,
            PackageCategory::Development => &mut aggregate.packages.development,
            PackageCategory::Desktop => &mut aggregate.packages.desktop,
            PackageCategory::Custom => &mut aggregate.packages.custom,
        };
        packages.retain(|pkg| !cmd.packages.contains(pkg));

        Ok(events)
    }

    /// Handle generate config command
    pub fn handle_generate_config(
        &mut self,
        cmd: GenerateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let aggregate = self.configs
            .get_mut(&cmd.config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_generate_config(cmd)?;
        
        // Note: Generate config doesn't change aggregate state

        Ok(events)
    }

    /// Handle validate config command
    pub fn handle_validate_config(
        &mut self,
        cmd: ValidateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let aggregate = self.configs
            .get_mut(&cmd.config_id)
            .ok_or_else(|| "Home configuration not found".to_string())?;

        let events = aggregate.handle_validate_config(cmd)?;
        
        // Note: Validate config doesn't change aggregate state

        Ok(events)
    }

    /// Handle start migration command
    pub fn handle_start_migration(
        &mut self,
        cmd: MigrateConfig,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        let migration_id = uuid::Uuid::new_v4();
        let aggregate = self.migrations
            .entry(migration_id)
            .or_insert_with(|| MigrationAggregate::new(migration_id));

        let events = aggregate.handle_start_migration(cmd.clone())?;
        
        // Update aggregate state directly
        aggregate.exists = true;
        aggregate.status = MigrationStatus::Analyzing;
        aggregate.source = Some(cmd.source);

        Ok(events)
    }
}

impl Default for HomeManagerCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Query handler for Home Manager domain
#[derive(Debug, Clone)]
pub struct HomeManagerQueryHandler {
    /// Home configurations stored as read models
    pub configs: HashMap<HomeConfigId, HomeConfigReadModel>,
    /// Index mapping program names to configs that contain them
    pub programs_by_name: HashMap<String, Vec<HomeConfigId>>,
    /// Index mapping service names to configs that contain them
    pub services_by_name: HashMap<String, Vec<HomeConfigId>>,
}

/// Read model for home configuration
#[derive(Debug, Clone)]
pub struct HomeConfigReadModel {
    /// Unique configuration identifier
    pub id: HomeConfigId,
    /// User profile information
    pub user_profile: UserProfile,
    /// Configured programs by name
    pub programs: HashMap<String, ProgramConfig>,
    /// Configured services by name
    pub services: HashMap<String, ServiceConfig>,
    /// Shell configuration if set
    pub shell: Option<ShellConfig>,
    /// Desktop environment configuration if set
    pub desktop: Option<DesktopConfig>,
    /// Package sets organized by category
    pub packages: PackageSet,
    /// Managed dotfile entries
    pub dotfiles: Vec<DotfileEntry>,
}

impl HomeManagerQueryHandler {
    /// Create a new query handler
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
            programs_by_name: HashMap::new(),
            services_by_name: HashMap::new(),
        }
    }

    /// Get home configuration by ID
    pub fn get_config(&self, id: &HomeConfigId) -> Option<&HomeConfigReadModel> {
        self.configs.get(id)
    }

    /// Get all configurations
    pub fn get_all_configs(&self) -> Vec<&HomeConfigReadModel> {
        self.configs.values().collect()
    }

    /// Find configurations by program
    pub fn find_configs_by_program(&self, program_name: &str) -> Vec<&HomeConfigReadModel> {
        self.programs_by_name
            .get(program_name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.configs.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find configurations by service
    pub fn find_configs_by_service(&self, service_name: &str) -> Vec<&HomeConfigReadModel> {
        self.services_by_name
            .get(service_name)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.configs.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Apply event to update read models
    pub fn apply_event(&mut self, event: &HomeManagerDomainEvent) {
            match event {
                HomeManagerDomainEvent::ConfigCreated(e) => {
                    let read_model = HomeConfigReadModel {
                        id: e.config_id,
                        user_profile: e.user_profile.clone(),
                        programs: HashMap::new(),
                        services: HashMap::new(),
                        shell: e.shell.clone(),
                        desktop: e.desktop.clone(),
                        packages: e.packages.clone(),
                        dotfiles: Vec::new(),
                    };
                    self.configs.insert(e.config_id, read_model);
                }
                HomeManagerDomainEvent::DotfilesImported(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        for dotfile in &e.imported_files {
                            config.dotfiles.push(dotfile.clone());
                        }
                    }
                }
                HomeManagerDomainEvent::ProgramAdded(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.programs.insert(e.program.name.clone(), e.program.clone());
                        
                        // Update program index
                        self.programs_by_name
                            .entry(e.program.name.clone())
                            .or_insert_with(Vec::new)
                            .push(e.config_id);
                    }
                }
                HomeManagerDomainEvent::ProgramUpdated(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.programs.insert(e.program_name.clone(), e.new_config.clone());
                    }
                }
                HomeManagerDomainEvent::ProgramRemoved(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.programs.remove(&e.program_name);
                        
                        // Update program index
                        if let Some(configs) = self.programs_by_name.get_mut(&e.program_name) {
                            configs.retain(|id| id != &e.config_id);
                        }
                    }
                }
                HomeManagerDomainEvent::ServiceAdded(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.services.insert(e.service.name.clone(), e.service.clone());
                        
                        // Update service index
                        self.services_by_name
                            .entry(e.service.name.clone())
                            .or_insert_with(Vec::new)
                            .push(e.config_id);
                    }
                }
                HomeManagerDomainEvent::ServiceUpdated(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.services.insert(e.service_name.clone(), e.new_config.clone());
                    }
                }
                HomeManagerDomainEvent::ServiceRemoved(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.services.remove(&e.service_name);
                        
                        // Update service index
                        if let Some(configs) = self.services_by_name.get_mut(&e.service_name) {
                            configs.retain(|id| id != &e.config_id);
                        }
                    }
                }
                HomeManagerDomainEvent::ShellConfigUpdated(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.shell = Some(e.new_shell.clone());
                    }
                }
                HomeManagerDomainEvent::DesktopConfigUpdated(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        config.desktop = Some(e.new_desktop.clone());
                    }
                }
                HomeManagerDomainEvent::PackagesAdded(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        let packages = match &e.category {
                            PackageCategory::System => &mut config.packages.system,
                            PackageCategory::Development => &mut config.packages.development,
                            PackageCategory::Desktop => &mut config.packages.desktop,
                            PackageCategory::Custom => &mut config.packages.custom,
                        };
                        for pkg in &e.packages {
                            if !packages.contains(pkg) {
                                packages.push(pkg.clone());
                            }
                        }
                    }
                }
                HomeManagerDomainEvent::PackagesRemoved(e) => {
                    if let Some(config) = self.configs.get_mut(&e.config_id) {
                        let packages = match &e.category {
                            PackageCategory::System => &mut config.packages.system,
                            PackageCategory::Development => &mut config.packages.development,
                            PackageCategory::Desktop => &mut config.packages.desktop,
                            PackageCategory::Custom => &mut config.packages.custom,
                        };
                        packages.retain(|pkg| !e.packages.contains(pkg));
                    }
                }
                _ => {} // Other events don't affect query handler
            }
    }
}

impl Default for HomeManagerQueryHandler {
    fn default() -> Self {
        Self::new()
    }
}