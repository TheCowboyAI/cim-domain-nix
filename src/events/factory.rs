//! Event factory for creating domain events with proper correlation/causation
//!
//! This module provides factory methods for creating events according to CIM patterns.

use crate::events::*;
use crate::value_objects::{MessageFactory, MessageIdentity};
use chrono::Utc;
use std::path::PathBuf;
use uuid::Uuid;

/// Factory for creating Nix domain events
pub struct NixEventFactory;

impl NixEventFactory {
    /// Create a FlakeCreated event as a root event
    pub fn create_flake_created_root(
        flake_id: Uuid,
        path: PathBuf,
        description: String,
        template: Option<String>,
    ) -> FlakeCreated {
        FlakeCreated {
            flake_id,
            timestamp: Utc::now(),
            identity: MessageFactory::create_root_identity(),
            path,
            description,
            template,
        }
    }

    /// Create a FlakeCreated event caused by another message
    pub fn create_flake_created_caused_by(
        flake_id: Uuid,
        path: PathBuf,
        description: String,
        template: Option<String>,
        parent_identity: &MessageIdentity,
    ) -> FlakeCreated {
        FlakeCreated {
            flake_id,
            timestamp: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            path,
            description,
            template,
        }
    }

    /// Create a FlakeUpdated event caused by another message
    pub fn create_flake_updated(
        flake_id: Uuid,
        path: PathBuf,
        parent_identity: &MessageIdentity,
    ) -> FlakeUpdated {
        FlakeUpdated {
            flake_id,
            timestamp: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            path,
        }
    }

    /// Create a FlakeInputAdded event caused by another message
    pub fn create_flake_input_added(
        flake_id: Uuid,
        path: PathBuf,
        input_name: String,
        input_url: String,
        parent_identity: &MessageIdentity,
    ) -> FlakeInputAdded {
        FlakeInputAdded {
            flake_id,
            timestamp: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            path,
            input_name,
            input_url,
        }
    }

    /// Create a ModuleCreated event
    pub fn create_module_created(
        module: crate::value_objects::NixModule,
        parent_identity: &MessageIdentity,
    ) -> ModuleCreated {
        ModuleCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            module,
        }
    }

    /// Create an OverlayCreated event
    pub fn create_overlay_created(
        overlay: crate::value_objects::Overlay,
        parent_identity: &MessageIdentity,
    ) -> OverlayCreated {
        OverlayCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            overlay,
        }
    }

    /// Create a ConfigurationCreated event
    pub fn create_configuration_created(
        configuration: crate::value_objects::NixOSConfiguration,
        parent_identity: &MessageIdentity,
    ) -> ConfigurationCreated {
        ConfigurationCreated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            configuration,
        }
    }

    /// Create a ConfigurationActivated event
    pub fn create_configuration_activated(
        configuration_id: Uuid,
        generation: u32,
        activation_type: ActivationType,
        parent_identity: &MessageIdentity,
    ) -> ConfigurationActivated {
        ConfigurationActivated {
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            identity: MessageFactory::create_caused_identity(parent_identity),
            configuration_id,
            generation,
            activation_type,
        }
    }
}
