// Copyright 2025 Cowboy AI, LLC.

//! Aggregates for the Nix domain
//!
//! This module contains aggregate roots that maintain consistency
//! for Nix domain entities.

use crate::{
    commands::{
        ActivateConfiguration, AddFlakeInput, CreateConfiguration, CreateFlake, CreateModule,
        CreateOverlay,
    },
    events::{
        ConfigurationActivated, ConfigurationCreated, FlakeCreated, FlakeInputAdded, ModuleCreated,
        NixEventFactory, OverlayCreated,
    },
    value_objects::{Flake, NixModule, NixOSConfiguration, Overlay},
    NixDomainError, Result,
};
use chrono::Utc;
use cim_domain::AggregateRoot;
use uuid::Uuid;

/// Represents a Nix flake aggregate root
#[derive(Debug, Clone)]
pub struct FlakeAggregate {
    /// Unique identifier for the flake aggregate
    pub id: Uuid,
    /// The flake entity, if created
    pub flake: Option<Flake>,
    /// List of modules associated with this flake
    pub modules: Vec<NixModule>,
    /// List of overlays associated with this flake
    pub overlays: Vec<Overlay>,
    /// Timestamp when this aggregate was created
    pub created_at: chrono::DateTime<Utc>,
    /// Timestamp when this aggregate was last updated
    pub updated_at: chrono::DateTime<Utc>,
    /// Version number for optimistic concurrency control
    version: u64,
}

impl FlakeAggregate {
    /// Create a new flake aggregate
    pub fn new(id: Uuid) -> Self {
        let now = Utc::now();
        Self {
            id,
            flake: None,
            modules: Vec::new(),
            overlays: Vec::new(),
            created_at: now,
            updated_at: now,
            version: 0,
        }
    }

    /// Handle a command
    ///
    /// # Errors
    ///
    /// Returns an error if the command type is unknown
    pub fn handle_command(
        &mut self,
        cmd: Box<dyn std::any::Any>,
    ) -> Result<Vec<Box<dyn std::any::Any>>> {
        let mut events = Vec::new();

        if let Some(create_flake) = cmd.downcast_ref::<CreateFlake>() {
            // Handle create flake command
            let event = NixEventFactory::create_flake_created_caused_by(
                self.id,
                create_flake.path.clone(),
                create_flake.description.clone(),
                create_flake.template.clone(),
                &create_flake.identity,
            );
            events.push(Box::new(event) as Box<dyn std::any::Any>);
        } else if let Some(add_input) = cmd.downcast_ref::<AddFlakeInput>() {
            // Handle add input command
            let event = NixEventFactory::create_flake_input_added(
                self.id,
                add_input.path.clone(),
                add_input.name.clone(),
                add_input.url.clone(),
                &add_input.identity,
            );
            events.push(Box::new(event) as Box<dyn std::any::Any>);
        } else {
            return Err(NixDomainError::Other("Unknown command".to_string()));
        }

        Ok(events)
    }

    /// Apply an event to the aggregate
    ///
    /// # Errors
    ///
    /// Currently always returns Ok, but may return errors in the future
    pub fn apply_event(&mut self, event: &dyn std::any::Any) -> Result<()> {
        if let Some(flake_created) = event.downcast_ref::<FlakeCreated>() {
            // Apply flake created event
            self.id = flake_created.flake_id;
            self.updated_at = flake_created.timestamp;
        } else if event.downcast_ref::<FlakeInputAdded>().is_some() {
            // Apply input added event
            self.updated_at = Utc::now();
        }

        Ok(())
    }
}

impl AggregateRoot for FlakeAggregate {
    type Id = uuid::Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }
}

/// Represents a Nix module aggregate
#[derive(Debug, Clone)]
pub struct ModuleAggregate {
    /// Unique identifier for the module aggregate
    pub id: Uuid,
    /// The module entity
    pub module: NixModule,
}

impl ModuleAggregate {
    /// Create a module
    ///
    /// # Errors
    ///
    /// Returns an error if module creation fails
    pub fn handle_create_module(cmd: CreateModule) -> Result<(Self, Vec<ModuleCreated>)> {
        let aggregate = Self {
            id: cmd.module.id,
            module: cmd.module.clone(),
        };
        let event = ModuleCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: cmd.identity,
            module: cmd.module,
        };
        Ok((aggregate, vec![event]))
    }
}

/// Represents a Nix overlay aggregate
#[derive(Debug, Clone)]
pub struct OverlayAggregate {
    /// Unique identifier for the overlay aggregate
    pub id: Uuid,
    /// The overlay entity
    pub overlay: Overlay,
}

impl OverlayAggregate {
    /// Create an overlay
    ///
    /// # Errors
    ///
    /// Returns an error if overlay creation fails
    pub fn handle_create_overlay(cmd: CreateOverlay) -> Result<(Self, Vec<OverlayCreated>)> {
        let aggregate = Self {
            id: cmd.overlay.id,
            overlay: cmd.overlay.clone(),
        };
        let event = OverlayCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: cmd.identity,
            overlay: cmd.overlay,
        };
        Ok((aggregate, vec![event]))
    }
}

/// Represents a `NixOS` configuration aggregate
#[derive(Debug, Clone)]
pub struct ConfigurationAggregate {
    /// Unique identifier for the configuration aggregate
    pub id: Uuid,
    /// Name of the configuration
    pub name: String,
    /// The `NixOS` configuration entity
    pub configuration: NixOSConfiguration,
    /// Whether this configuration is currently active
    pub is_active: bool,
    /// Current generation number (incremented on each activation)
    pub current_generation: u32,
}

impl ConfigurationAggregate {
    /// Create a configuration
    ///
    /// # Errors
    ///
    /// Returns an error if configuration creation fails
    pub fn handle_create_configuration(
        cmd: CreateConfiguration,
    ) -> Result<(Self, Vec<ConfigurationCreated>)> {
        let id = Uuid::new_v4();
        let aggregate = Self {
            id,
            name: cmd.name.clone(),
            configuration: cmd.configuration.clone(),
            is_active: false,
            current_generation: 0,
        };
        let event = ConfigurationCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: cmd.identity,
            configuration: cmd.configuration,
        };
        Ok((aggregate, vec![event]))
    }

    /// Activate a configuration
    ///
    /// # Errors
    ///
    /// Returns an error if configuration activation fails
    pub fn handle_activate_configuration(
        &mut self,
        cmd: ActivateConfiguration,
    ) -> Result<Vec<ConfigurationActivated>> {
        self.is_active = true;
        self.current_generation += 1;
        let event = ConfigurationActivated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: cmd.identity,
            configuration_id: self.id,
            generation: self.current_generation,
            activation_type: cmd.activation_type,
        };
        Ok(vec![event])
    }
}
