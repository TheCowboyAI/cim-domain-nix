//! CQRS adapter for Nix domain

use crate::aggregate::FlakeAggregate;
use crate::commands::{BuildPackage, CreateFlake, UpdateFlake};
use crate::handlers::NixCommandHandler;
use cim_domain::{
    Command, CommandAcknowledgment, CommandEnvelope, CommandHandler, CommandStatus, EntityId,
};
use std::sync::Arc;

/// Wrapper for Nix commands to implement CQRS Command trait
#[derive(Debug, Clone)]
pub enum NixCommandWrapper {
    /// Create a new flake
    CreateFlake(CreateFlake),
    /// Update an existing flake
    UpdateFlake(UpdateFlake),
    /// Build a package from a flake
    BuildPackage(BuildPackage),
}

impl Command for NixCommandWrapper {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<EntityId<Self::Aggregate>> {
        match self {
            NixCommandWrapper::CreateFlake(_) => None, // Creating new aggregate
            NixCommandWrapper::UpdateFlake(_) => None, // Would need flake ID
            NixCommandWrapper::BuildPackage(_) => None, // Query operation
        }
    }
}

/// CQRS handler for `CreateFlake` command
pub struct CreateFlakeHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl CreateFlakeHandler {
    /// Create a new `CreateFlake` handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self { nix_handler }
    }
}

impl CommandHandler<CreateFlake> for CreateFlakeHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateFlake>) -> CommandAcknowledgment {
        // Execute the command through the Nix handler
        let command = envelope.command;

        // Use the nix_handler to actually create the flake
        // For now, we'll use default values for packages and dev_shells
        let packages: Vec<String> = vec![];
        let dev_shells: Vec<String> = vec!["default".to_string()];

        match futures::executor::block_on(self.nix_handler.handle_create_flake(
            &command.path.to_string_lossy(),
            &command.description,
            &packages,
            &dev_shells,
        )) {
            Ok(_) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(e.to_string()),
            },
        }
    }
}

/// CQRS handler for `UpdateFlake` command
pub struct UpdateFlakeHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl UpdateFlakeHandler {
    /// Create a new `UpdateFlake` handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self { nix_handler }
    }
}

impl CommandHandler<UpdateFlake> for UpdateFlakeHandler {
    fn handle(&mut self, envelope: CommandEnvelope<UpdateFlake>) -> CommandAcknowledgment {
        // Execute the command through the Nix handler
        let command = envelope.command;

        // Use the nix_handler to actually update the flake
        let packages: Vec<String> = vec![];
        let dev_shells: Vec<String> = vec![];

        match futures::executor::block_on(self.nix_handler.handle_update_flake(
            &command.path.to_string_lossy(),
            None, // No description update
            &packages,
            &dev_shells,
        )) {
            Ok(_) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(e.to_string()),
            },
        }
    }
}

/// CQRS handler for `BuildPackage` command
pub struct BuildPackageHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl BuildPackageHandler {
    /// Create a new `BuildPackage` handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self { nix_handler }
    }
}

impl CommandHandler<BuildPackage> for BuildPackageHandler {
    fn handle(&mut self, envelope: CommandEnvelope<BuildPackage>) -> CommandAcknowledgment {
        // Execute the command through the Nix handler
        let command = envelope.command;

        // Use the nix_handler to actually build the package
        let attribute_str = command.attribute.segments.join(".");
        let output_path = command
            .output_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string());

        match futures::executor::block_on(self.nix_handler.handle_build_package(
            &command.flake_ref,
            &attribute_str,
            output_path.as_deref(),
        )) {
            Ok(_) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Accepted,
                reason: None,
            },
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.identity.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(e.to_string()),
            },
        }
    }
}
