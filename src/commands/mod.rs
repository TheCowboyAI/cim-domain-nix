//! Commands for Nix domain operations

use std::any::Any;
use std::path::PathBuf;
use uuid::Uuid;
use crate::value_objects::{AttributePath, NixModule, Overlay, NixOSConfiguration, MessageIdentity};
use crate::events::ActivationType;
use crate::aggregate::FlakeAggregate;

/// Trait for all Nix commands
pub trait NixCommand: Send + Sync {
    /// Get the command ID
    fn command_id(&self) -> Uuid;
    
    /// Get the message identity for correlation/causation
    fn identity(&self) -> &MessageIdentity;
    
    /// Get the command as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Command to create a new flake
#[derive(Debug, Clone)]
pub struct CreateFlake {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Path where the flake will be created
    pub path: PathBuf,
    /// Human-readable description for the flake
    pub description: String,
    /// Optional template to use for initialization
    pub template: Option<String>,
}

impl NixCommand for CreateFlake {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for CreateFlake {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Creating new aggregate
    }
}

/// Command to update a flake
#[derive(Debug, Clone)]
pub struct UpdateFlake {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Path to the flake to update
    pub path: PathBuf,
}

impl NixCommand for UpdateFlake {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for UpdateFlake {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Would need flake ID to properly identify
    }
}

/// Command to add an input to a flake
#[derive(Debug, Clone)]
pub struct AddFlakeInput {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Path to the flake to modify
    pub path: PathBuf,
    /// Name of the input to add
    pub name: String,
    /// URL of the input flake
    pub url: String,
}

impl NixCommand for AddFlakeInput {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for AddFlakeInput {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Would need flake ID to properly identify
    }
}

/// Command to build a package
#[derive(Debug, Clone)]
pub struct BuildPackage {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Flake reference to build from
    pub flake_ref: String,
    /// Attribute path to the package
    pub attribute: AttributePath,
    /// Optional output path override
    pub output_path: Option<PathBuf>,
}

impl NixCommand for BuildPackage {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for BuildPackage {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Build is a query operation on a flake
    }
}

/// Command to create a module
#[derive(Debug, Clone)]
pub struct CreateModule {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Module name
    pub name: String,
    /// The module to create
    pub module: NixModule,
}

impl NixCommand for CreateModule {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for CreateModule {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Modules are created within flakes
    }
}

/// Command to create an overlay
#[derive(Debug, Clone)]
pub struct CreateOverlay {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Overlay name
    pub name: String,
    /// The overlay to create
    pub overlay: Overlay,
}

impl NixCommand for CreateOverlay {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for CreateOverlay {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Overlays are created within flakes
    }
}

/// Command to create a configuration
#[derive(Debug, Clone)]
pub struct CreateConfiguration {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Configuration name
    pub name: String,
    /// The configuration to create
    pub configuration: NixOSConfiguration,
}

impl NixCommand for CreateConfiguration {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for CreateConfiguration {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Configurations may be standalone
    }
}

/// Command to activate a configuration
#[derive(Debug, Clone)]
pub struct ActivateConfiguration {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Configuration name to activate
    pub name: String,
    /// Type of activation to perform
    pub activation_type: ActivationType,
}

impl NixCommand for ActivateConfiguration {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for ActivateConfiguration {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Activation affects system state
    }
}

/// Command to evaluate a Nix expression
#[derive(Debug, Clone)]
pub struct EvaluateExpression {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Nix expression to evaluate
    pub expression: String,
}

impl NixCommand for EvaluateExpression {
    fn command_id(&self) -> Uuid {
        self.identity.message_id.0
    }
    
    fn identity(&self) -> &MessageIdentity {
        &self.identity
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for EvaluateExpression {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Expression evaluation is stateless
    }
}

/// Command to run garbage collection
#[derive(Debug, Clone)]
pub struct RunGarbageCollection {
    /// Optional: only collect items older than this many days
    pub older_than_days: Option<u32>,
}

impl NixCommand for RunGarbageCollection {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for RunGarbageCollection {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // GC is a system-wide operation
    }
}

/// Command to check a flake
#[derive(Debug, Clone)]
pub struct CheckFlake {
    /// Path to the flake to check
    pub path: PathBuf,
}

impl NixCommand for CheckFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for CheckFlake {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Check is a query operation
    }
}

/// Command to enter a development shell
#[derive(Debug, Clone)]
pub struct DevelopFlake {
    /// Path to the flake with devShell
    pub path: PathBuf,
    /// Optional command to run in the shell
    pub command: Option<String>,
}

impl NixCommand for DevelopFlake {
    fn command_id(&self) -> Uuid {
        Uuid::new_v4()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl cim_domain::Command for DevelopFlake {
    type Aggregate = FlakeAggregate;

    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Develop is an interactive operation
    }
} 