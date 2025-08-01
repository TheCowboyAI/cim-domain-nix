//! Subject mapping for the Nix domain
//!
//! Defines all NATS subjects used by the Nix domain following CIM conventions:
//! - Commands: nix.cmd.{aggregate}.{action}
//! - Events: nix.event.{aggregate}.{action}
//! - Queries: nix.query.{aggregate}.{action}

use std::fmt;

/// The Nix domain identifier
pub const DOMAIN: &str = "nix";

/// Aggregate types in the Nix domain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aggregate {
    Flake,
    Package,
    Module,
    Overlay,
    Configuration,
}

impl fmt::Display for Aggregate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Aggregate::Flake => write!(f, "flake"),
            Aggregate::Package => write!(f, "package"),
            Aggregate::Module => write!(f, "module"),
            Aggregate::Overlay => write!(f, "overlay"),
            Aggregate::Configuration => write!(f, "config"),
        }
    }
}

/// Message types following CIM conventions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Command,
    Event,
    Query,
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Command => write!(f, "cmd"),
            MessageType::Event => write!(f, "event"),
            MessageType::Query => write!(f, "query"),
        }
    }
}

/// Command actions for each aggregate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandAction {
    // Flake commands
    CreateFlake,
    UpdateFlake,
    AddFlakeInput,
    RemoveFlakeInput,
    
    // Package commands
    BuildPackage,
    CachePackage,
    
    // Module commands
    CreateModule,
    UpdateModule,
    DeleteModule,
    
    // Overlay commands
    CreateOverlay,
    UpdateOverlay,
    DeleteOverlay,
    
    // Configuration commands
    CreateConfiguration,
    UpdateConfiguration,
    ActivateConfiguration,
    RollbackConfiguration,
}

impl CommandAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Flake commands
            CommandAction::CreateFlake => "create",
            CommandAction::UpdateFlake => "update",
            CommandAction::AddFlakeInput => "add_input",
            CommandAction::RemoveFlakeInput => "remove_input",
            
            // Package commands
            CommandAction::BuildPackage => "build",
            CommandAction::CachePackage => "cache",
            
            // Module commands
            CommandAction::CreateModule => "create",
            CommandAction::UpdateModule => "update",
            CommandAction::DeleteModule => "delete",
            
            // Overlay commands
            CommandAction::CreateOverlay => "create",
            CommandAction::UpdateOverlay => "update",
            CommandAction::DeleteOverlay => "delete",
            
            // Configuration commands
            CommandAction::CreateConfiguration => "create",
            CommandAction::UpdateConfiguration => "update",
            CommandAction::ActivateConfiguration => "activate",
            CommandAction::RollbackConfiguration => "rollback",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            CommandAction::CreateFlake
            | CommandAction::UpdateFlake
            | CommandAction::AddFlakeInput
            | CommandAction::RemoveFlakeInput => Aggregate::Flake,
            
            CommandAction::BuildPackage
            | CommandAction::CachePackage => Aggregate::Package,
            
            CommandAction::CreateModule
            | CommandAction::UpdateModule
            | CommandAction::DeleteModule => Aggregate::Module,
            
            CommandAction::CreateOverlay
            | CommandAction::UpdateOverlay
            | CommandAction::DeleteOverlay => Aggregate::Overlay,
            
            CommandAction::CreateConfiguration
            | CommandAction::UpdateConfiguration
            | CommandAction::ActivateConfiguration
            | CommandAction::RollbackConfiguration => Aggregate::Configuration,
        }
    }
}

/// Event actions (past tense of commands)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAction {
    // Flake events
    FlakeCreated,
    FlakeUpdated,
    FlakeInputAdded,
    FlakeInputRemoved,
    
    // Package events
    PackageBuilt,
    PackageCached,
    
    // Module events
    ModuleCreated,
    ModuleUpdated,
    ModuleDeleted,
    
    // Overlay events
    OverlayCreated,
    OverlayUpdated,
    OverlayDeleted,
    
    // Configuration events
    ConfigurationCreated,
    ConfigurationUpdated,
    ConfigurationActivated,
    ConfigurationRolledBack,
}

impl EventAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Flake events
            EventAction::FlakeCreated => "created",
            EventAction::FlakeUpdated => "updated",
            EventAction::FlakeInputAdded => "input_added",
            EventAction::FlakeInputRemoved => "input_removed",
            
            // Package events
            EventAction::PackageBuilt => "built",
            EventAction::PackageCached => "cached",
            
            // Module events
            EventAction::ModuleCreated => "created",
            EventAction::ModuleUpdated => "updated",
            EventAction::ModuleDeleted => "deleted",
            
            // Overlay events
            EventAction::OverlayCreated => "created",
            EventAction::OverlayUpdated => "updated",
            EventAction::OverlayDeleted => "deleted",
            
            // Configuration events
            EventAction::ConfigurationCreated => "created",
            EventAction::ConfigurationUpdated => "updated",
            EventAction::ConfigurationActivated => "activated",
            EventAction::ConfigurationRolledBack => "rolled_back",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            EventAction::FlakeCreated
            | EventAction::FlakeUpdated
            | EventAction::FlakeInputAdded
            | EventAction::FlakeInputRemoved => Aggregate::Flake,
            
            EventAction::PackageBuilt
            | EventAction::PackageCached => Aggregate::Package,
            
            EventAction::ModuleCreated
            | EventAction::ModuleUpdated
            | EventAction::ModuleDeleted => Aggregate::Module,
            
            EventAction::OverlayCreated
            | EventAction::OverlayUpdated
            | EventAction::OverlayDeleted => Aggregate::Overlay,
            
            EventAction::ConfigurationCreated
            | EventAction::ConfigurationUpdated
            | EventAction::ConfigurationActivated
            | EventAction::ConfigurationRolledBack => Aggregate::Configuration,
        }
    }
}

/// Query actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryAction {
    // Flake queries
    GetFlake,
    ListFlakes,
    GetFlakeInputs,
    
    // Package queries
    GetPackage,
    ListPackages,
    GetBuildStatus,
    
    // Module queries
    GetModule,
    ListModules,
    
    // Overlay queries
    GetOverlay,
    ListOverlays,
    
    // Configuration queries
    GetConfiguration,
    ListConfigurations,
    GetCurrentConfiguration,
    GetConfigurationHistory,
}

impl QueryAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            // Flake queries
            QueryAction::GetFlake => "get",
            QueryAction::ListFlakes => "list",
            QueryAction::GetFlakeInputs => "get_inputs",
            
            // Package queries
            QueryAction::GetPackage => "get",
            QueryAction::ListPackages => "list",
            QueryAction::GetBuildStatus => "get_status",
            
            // Module queries
            QueryAction::GetModule => "get",
            QueryAction::ListModules => "list",
            
            // Overlay queries
            QueryAction::GetOverlay => "get",
            QueryAction::ListOverlays => "list",
            
            // Configuration queries
            QueryAction::GetConfiguration => "get",
            QueryAction::ListConfigurations => "list",
            QueryAction::GetCurrentConfiguration => "get_current",
            QueryAction::GetConfigurationHistory => "get_history",
        }
    }
    
    pub fn aggregate(&self) -> Aggregate {
        match self {
            QueryAction::GetFlake
            | QueryAction::ListFlakes
            | QueryAction::GetFlakeInputs => Aggregate::Flake,
            
            QueryAction::GetPackage
            | QueryAction::ListPackages
            | QueryAction::GetBuildStatus => Aggregate::Package,
            
            QueryAction::GetModule
            | QueryAction::ListModules => Aggregate::Module,
            
            QueryAction::GetOverlay
            | QueryAction::ListOverlays => Aggregate::Overlay,
            
            QueryAction::GetConfiguration
            | QueryAction::ListConfigurations
            | QueryAction::GetCurrentConfiguration
            | QueryAction::GetConfigurationHistory => Aggregate::Configuration,
        }
    }
}

/// A NATS subject for the Nix domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NixSubject {
    pub(crate) message_type: MessageType,
    pub(crate) aggregate: Aggregate,
    pub(crate) action: String,
}

impl NixSubject {
    /// Create a command subject
    pub fn command(action: CommandAction) -> Self {
        Self {
            message_type: MessageType::Command,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Create an event subject
    pub fn event(action: EventAction) -> Self {
        Self {
            message_type: MessageType::Event,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Create a query subject
    pub fn query(action: QueryAction) -> Self {
        Self {
            message_type: MessageType::Query,
            aggregate: action.aggregate(),
            action: action.as_str().to_string(),
        }
    }
    
    /// Parse a subject string into a NixSubject
    pub fn parse(subject: &str) -> Option<Self> {
        let parts: Vec<&str> = subject.split('.').collect();
        if parts.len() != 4 || parts[0] != DOMAIN {
            return None;
        }
        
        let message_type = match parts[1] {
            "cmd" => MessageType::Command,
            "event" => MessageType::Event,
            "query" => MessageType::Query,
            _ => return None,
        };
        
        let aggregate = match parts[2] {
            "flake" => Aggregate::Flake,
            "package" => Aggregate::Package,
            "module" => Aggregate::Module,
            "overlay" => Aggregate::Overlay,
            "config" => Aggregate::Configuration,
            _ => return None,
        };
        
        Some(Self {
            message_type,
            aggregate,
            action: parts[3].to_string(),
        })
    }
}

impl fmt::Display for NixSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", DOMAIN, self.message_type, self.aggregate, self.action)
    }
}

/// Maps between domain types and NATS subjects
pub struct SubjectMapper;

impl SubjectMapper {
    /// Get the subject for a command
    pub fn command_subject(command_type: &str) -> Option<NixSubject> {
        match command_type {
            "CreateFlake" => Some(NixSubject::command(CommandAction::CreateFlake)),
            "UpdateFlake" => Some(NixSubject::command(CommandAction::UpdateFlake)),
            "AddFlakeInput" => Some(NixSubject::command(CommandAction::AddFlakeInput)),
            "BuildPackage" => Some(NixSubject::command(CommandAction::BuildPackage)),
            "CreateModule" => Some(NixSubject::command(CommandAction::CreateModule)),
            "CreateOverlay" => Some(NixSubject::command(CommandAction::CreateOverlay)),
            "CreateConfiguration" => Some(NixSubject::command(CommandAction::CreateConfiguration)),
            "ActivateConfiguration" => Some(NixSubject::command(CommandAction::ActivateConfiguration)),
            _ => None,
        }
    }
    
    /// Get the subject for an event
    pub fn event_subject(event_type: &str) -> Option<NixSubject> {
        match event_type {
            "FlakeCreated" => Some(NixSubject::event(EventAction::FlakeCreated)),
            "FlakeUpdated" => Some(NixSubject::event(EventAction::FlakeUpdated)),
            "FlakeInputAdded" => Some(NixSubject::event(EventAction::FlakeInputAdded)),
            "PackageBuilt" => Some(NixSubject::event(EventAction::PackageBuilt)),
            "ModuleCreated" => Some(NixSubject::event(EventAction::ModuleCreated)),
            "OverlayCreated" => Some(NixSubject::event(EventAction::OverlayCreated)),
            "ConfigurationCreated" => Some(NixSubject::event(EventAction::ConfigurationCreated)),
            "ConfigurationActivated" => Some(NixSubject::event(EventAction::ConfigurationActivated)),
            _ => None,
        }
    }
    
    /// Get all command subjects for subscription
    pub fn all_command_subjects() -> Vec<NixSubject> {
        vec![
            NixSubject::command(CommandAction::CreateFlake),
            NixSubject::command(CommandAction::UpdateFlake),
            NixSubject::command(CommandAction::AddFlakeInput),
            NixSubject::command(CommandAction::RemoveFlakeInput),
            NixSubject::command(CommandAction::BuildPackage),
            NixSubject::command(CommandAction::CachePackage),
            NixSubject::command(CommandAction::CreateModule),
            NixSubject::command(CommandAction::UpdateModule),
            NixSubject::command(CommandAction::DeleteModule),
            NixSubject::command(CommandAction::CreateOverlay),
            NixSubject::command(CommandAction::UpdateOverlay),
            NixSubject::command(CommandAction::DeleteOverlay),
            NixSubject::command(CommandAction::CreateConfiguration),
            NixSubject::command(CommandAction::UpdateConfiguration),
            NixSubject::command(CommandAction::ActivateConfiguration),
            NixSubject::command(CommandAction::RollbackConfiguration),
        ]
    }
    
    /// Get all event subjects for subscription
    pub fn all_event_subjects() -> Vec<NixSubject> {
        vec![
            NixSubject::event(EventAction::FlakeCreated),
            NixSubject::event(EventAction::FlakeUpdated),
            NixSubject::event(EventAction::FlakeInputAdded),
            NixSubject::event(EventAction::FlakeInputRemoved),
            NixSubject::event(EventAction::PackageBuilt),
            NixSubject::event(EventAction::PackageCached),
            NixSubject::event(EventAction::ModuleCreated),
            NixSubject::event(EventAction::ModuleUpdated),
            NixSubject::event(EventAction::ModuleDeleted),
            NixSubject::event(EventAction::OverlayCreated),
            NixSubject::event(EventAction::OverlayUpdated),
            NixSubject::event(EventAction::OverlayDeleted),
            NixSubject::event(EventAction::ConfigurationCreated),
            NixSubject::event(EventAction::ConfigurationUpdated),
            NixSubject::event(EventAction::ConfigurationActivated),
            NixSubject::event(EventAction::ConfigurationRolledBack),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_subject_format() {
        let subject = NixSubject::command(CommandAction::CreateFlake);
        assert_eq!(subject.to_string(), "nix.cmd.flake.create");
        
        let subject = NixSubject::command(CommandAction::BuildPackage);
        assert_eq!(subject.to_string(), "nix.cmd.package.build");
        
        let subject = NixSubject::command(CommandAction::ActivateConfiguration);
        assert_eq!(subject.to_string(), "nix.cmd.config.activate");
    }
    
    #[test]
    fn test_event_subject_format() {
        let subject = NixSubject::event(EventAction::FlakeCreated);
        assert_eq!(subject.to_string(), "nix.event.flake.created");
        
        let subject = NixSubject::event(EventAction::PackageBuilt);
        assert_eq!(subject.to_string(), "nix.event.package.built");
        
        let subject = NixSubject::event(EventAction::ConfigurationActivated);
        assert_eq!(subject.to_string(), "nix.event.config.activated");
    }
    
    #[test]
    fn test_query_subject_format() {
        let subject = NixSubject::query(QueryAction::GetFlake);
        assert_eq!(subject.to_string(), "nix.query.flake.get");
        
        let subject = NixSubject::query(QueryAction::ListPackages);
        assert_eq!(subject.to_string(), "nix.query.package.list");
        
        let subject = NixSubject::query(QueryAction::GetCurrentConfiguration);
        assert_eq!(subject.to_string(), "nix.query.config.get_current");
    }
    
    #[test]
    fn test_parse_subject() {
        let subject = NixSubject::parse("nix.cmd.flake.create").unwrap();
        assert_eq!(subject.message_type, MessageType::Command);
        assert_eq!(subject.aggregate, Aggregate::Flake);
        assert_eq!(subject.action, "create");
        
        let subject = NixSubject::parse("nix.event.package.built").unwrap();
        assert_eq!(subject.message_type, MessageType::Event);
        assert_eq!(subject.aggregate, Aggregate::Package);
        assert_eq!(subject.action, "built");
        
        // Invalid subjects
        assert!(NixSubject::parse("invalid.subject").is_none());
        assert!(NixSubject::parse("other.cmd.flake.create").is_none());
        assert!(NixSubject::parse("nix.invalid.flake.create").is_none());
    }
    
    #[test]
    fn test_subject_mapper() {
        let subject = SubjectMapper::command_subject("CreateFlake").unwrap();
        assert_eq!(subject.to_string(), "nix.cmd.flake.create");
        
        let subject = SubjectMapper::event_subject("FlakeCreated").unwrap();
        assert_eq!(subject.to_string(), "nix.event.flake.created");
        
        assert!(SubjectMapper::command_subject("InvalidCommand").is_none());
    }
}