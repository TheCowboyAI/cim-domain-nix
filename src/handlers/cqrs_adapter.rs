//! CQRS adapter for Nix domain

use crate::commands::{CreateFlake, UpdateFlake, BuildPackage};
use crate::handlers::NixCommandHandler;
use crate::aggregate::FlakeAggregate;
use cim_domain::{
    Command, CommandHandler, CommandAcknowledgment, CommandStatus,
    EntityId, CommandEnvelope
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

/// CQRS handler for CreateFlake command
pub struct CreateFlakeHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl CreateFlakeHandler {
    /// Create a new CreateFlake handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self {
            nix_handler,
        }
    }
}

impl CommandHandler<CreateFlake> for CreateFlakeHandler {
    fn handle(&mut self, envelope: CommandEnvelope<CreateFlake>) -> CommandAcknowledgment {
        // For now, just acknowledge the command
        // In a real implementation, this would execute the command through the Nix handler
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS handler for UpdateFlake command
pub struct UpdateFlakeHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl UpdateFlakeHandler {
    /// Create a new UpdateFlake handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self {
            nix_handler,
        }
    }
}

impl CommandHandler<UpdateFlake> for UpdateFlakeHandler {
    fn handle(&mut self, envelope: CommandEnvelope<UpdateFlake>) -> CommandAcknowledgment {
        // For now, just acknowledge the command
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
}

/// CQRS handler for BuildPackage command
pub struct BuildPackageHandler {
    nix_handler: Arc<NixCommandHandler>,
}

impl BuildPackageHandler {
    /// Create a new BuildPackage handler
    pub fn new(nix_handler: Arc<NixCommandHandler>) -> Self {
        Self {
            nix_handler,
        }
    }
}

impl CommandHandler<BuildPackage> for BuildPackageHandler {
    fn handle(&mut self, envelope: CommandEnvelope<BuildPackage>) -> CommandAcknowledgment {
        // For now, just acknowledge the command
        CommandAcknowledgment {
            command_id: envelope.id,
            correlation_id: envelope.identity.correlation_id,
            status: CommandStatus::Accepted,
            reason: None,
        }
    }
} 