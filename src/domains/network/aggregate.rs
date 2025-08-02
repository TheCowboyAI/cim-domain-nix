// Copyright 2025 Cowboy AI, LLC.

//! Aggregates for network domain

use super::commands::*;
use super::events::*;
use super::value_objects::*;
use crate::value_objects::{CorrelationId, CausationId};
use cim_domain::{Aggregate, DomainEvent, EntityId};
use std::collections::HashMap;
use chrono::Utc;

/// Network topology aggregate
#[derive(Debug, Clone)]
pub struct NetworkTopologyAggregate {
    /// Topology ID
    pub id: NetworkTopologyId,
    /// Topology name
    pub name: String,
    /// Description
    pub description: String,
    /// Nodes in the topology
    pub nodes: HashMap<NetworkNodeId, NetworkNode>,
    /// Connections between nodes
    pub connections: Vec<NetworkConnection>,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Whether this aggregate exists
    pub exists: bool,
}

/// Network node data
#[derive(Debug, Clone)]
pub struct NetworkNode {
    /// Node ID
    pub id: NetworkNodeId,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Node tier
    pub tier: NodeTier,
    /// Interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Services
    pub services: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl NetworkTopologyAggregate {
    /// Create a new topology aggregate
    pub fn new(id: NetworkTopologyId) -> Self {
        Self {
            id,
            name: String::new(),
            description: String::new(),
            nodes: HashMap::new(),
            connections: Vec::new(),
            metadata: HashMap::new(),
            exists: false,
        }
    }
    
    /// Handle create topology command
    pub fn handle_create(
        &self,
        cmd: CreateNetworkTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if self.exists {
            return Err("Topology already exists".to_string());
        }
        
        let event = NetworkTopologyCreated {
            topology_id: NetworkTopologyId::new(),
            name: cmd.name,
            description: cmd.description,
            metadata: cmd.metadata,
            created_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Handle add node command
    pub fn handle_add_node(
        &self,
        cmd: AddNodeToTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Topology does not exist".to_string());
        }
        
        // Check for duplicate node names
        if self.nodes.values().any(|n| n.name == cmd.name) {
            return Err(format!("Node with name '{}' already exists", cmd.name));
        }
        
        let event = NodeAddedToTopology {
            topology_id: self.id,
            node_id: NetworkNodeId::new(),
            name: cmd.name,
            node_type: cmd.node_type,
            tier: cmd.tier,
            interfaces: cmd.interfaces,
            services: cmd.services,
            metadata: cmd.metadata,
            added_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Handle remove node command
    pub fn handle_remove_node(
        &self,
        cmd: RemoveNodeFromTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Topology does not exist".to_string());
        }
        
        if !self.nodes.contains_key(&cmd.node_id) {
            return Err("Node not found in topology".to_string());
        }
        
        // Check if node has connections
        let has_connections = self.connections.iter().any(|c| {
            c.from_node == cmd.node_id || c.to_node == cmd.node_id
        });
        
        if has_connections {
            return Err("Cannot remove node with active connections".to_string());
        }
        
        let event = NodeRemovedFromTopology {
            topology_id: self.id,
            node_id: cmd.node_id,
            removed_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(event)])
    }
    
    /// Handle create connection command
    pub fn handle_create_connection(
        &self,
        cmd: CreateNetworkConnection,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Topology does not exist".to_string());
        }
        
        // Verify both nodes exist
        if !self.nodes.contains_key(&cmd.from_node) {
            return Err("Source node not found".to_string());
        }
        if !self.nodes.contains_key(&cmd.to_node) {
            return Err("Destination node not found".to_string());
        }
        
        // Check for duplicate connection
        let duplicate = self.connections.iter().any(|c| {
            (c.from_node == cmd.from_node && c.to_node == cmd.to_node) ||
            (c.from_node == cmd.to_node && c.to_node == cmd.from_node)
        });
        
        if duplicate {
            return Err("Connection already exists between these nodes".to_string());
        }
        
        let connection = NetworkConnection {
            from_node: cmd.from_node,
            to_node: cmd.to_node,
            connection_type: cmd.connection_type,
            properties: cmd.properties,
        };
        
        let event = NetworkConnectionCreated {
            topology_id: self.id,
            connection,
            created_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(event)])
    }
}

impl Aggregate for NetworkTopologyAggregate {
    type Command = ();  // We handle commands directly
    type Event = Box<dyn DomainEvent>;
    
    fn aggregate_type() -> &'static str {
        "NetworkTopology"
    }
    
    fn handle_command(&self, _command: Self::Command) -> Result<Vec<Self::Event>, String> {
        unreachable!("Commands are handled directly")
    }
    
    fn apply_event(&mut self, event: Self::Event) -> Result<(), String> {
        match event.event_type() {
            "NetworkTopologyCreated" => {
                if let Ok(e) = event.as_any().downcast_ref::<NetworkTopologyCreated>() {
                    self.id = e.topology_id;
                    self.name = e.name.clone();
                    self.description = e.description.clone();
                    self.metadata = e.metadata.clone();
                    self.exists = true;
                }
            }
            "NodeAddedToTopology" => {
                if let Ok(e) = event.as_any().downcast_ref::<NodeAddedToTopology>() {
                    let node = NetworkNode {
                        id: e.node_id,
                        name: e.name.clone(),
                        node_type: e.node_type.clone(),
                        tier: e.tier,
                        interfaces: e.interfaces.clone(),
                        services: e.services.clone(),
                        metadata: e.metadata.clone(),
                    };
                    self.nodes.insert(e.node_id, node);
                }
            }
            "NodeRemovedFromTopology" => {
                if let Ok(e) = event.as_any().downcast_ref::<NodeRemovedFromTopology>() {
                    self.nodes.remove(&e.node_id);
                }
            }
            "NetworkConnectionCreated" => {
                if let Ok(e) = event.as_any().downcast_ref::<NetworkConnectionCreated>() {
                    self.connections.push(e.connection.clone());
                }
            }
            "NetworkConnectionRemoved" => {
                if let Ok(e) = event.as_any().downcast_ref::<NetworkConnectionRemoved>() {
                    self.connections.retain(|c| {
                        !(c.from_node == e.from_node && c.to_node == e.to_node)
                    });
                }
            }
            _ => {}
        }
        Ok(())
    }
}

/// Network node aggregate (for individual node operations)
#[derive(Debug, Clone)]
pub struct NetworkNodeAggregate {
    /// Node ID
    pub id: NetworkNodeId,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Node tier
    pub tier: NodeTier,
    /// Interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Services
    pub services: Vec<String>,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Whether this aggregate exists
    pub exists: bool,
}

impl NetworkNodeAggregate {
    /// Create a new node aggregate
    pub fn new(id: NetworkNodeId) -> Self {
        Self {
            id,
            name: String::new(),
            node_type: NodeType::Server,
            tier: NodeTier::Leaf,
            interfaces: Vec::new(),
            services: Vec::new(),
            metadata: HashMap::new(),
            exists: false,
        }
    }
    
    /// Handle update configuration command
    pub fn handle_update_configuration(
        &self,
        cmd: UpdateNodeConfiguration,
    ) -> Result<Vec<Box<dyn DomainEvent>>, String> {
        if !self.exists {
            return Err("Node does not exist".to_string());
        }
        
        let event = NodeConfigurationUpdated {
            node_id: self.id,
            old_interfaces: if cmd.interfaces.is_some() {
                Some(self.interfaces.clone())
            } else {
                None
            },
            new_interfaces: cmd.interfaces,
            old_services: if cmd.services.is_some() {
                Some(self.services.clone())
            } else {
                None
            },
            new_services: cmd.services,
            old_metadata: if cmd.metadata.is_some() {
                Some(self.metadata.clone())
            } else {
                None
            },
            new_metadata: cmd.metadata,
            updated_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(event)])
    }
}

impl Aggregate for NetworkNodeAggregate {
    type Command = ();
    type Event = Box<dyn DomainEvent>;
    
    fn aggregate_type() -> &'static str {
        "NetworkNode"
    }
    
    fn handle_command(&self, _command: Self::Command) -> Result<Vec<Self::Event>, String> {
        unreachable!("Commands are handled directly")
    }
    
    fn apply_event(&mut self, event: Self::Event) -> Result<(), String> {
        match event.event_type() {
            "NodeAddedToTopology" => {
                if let Ok(e) = event.as_any().downcast_ref::<NodeAddedToTopology>() {
                    if e.node_id == self.id {
                        self.name = e.name.clone();
                        self.node_type = e.node_type.clone();
                        self.tier = e.tier;
                        self.interfaces = e.interfaces.clone();
                        self.services = e.services.clone();
                        self.metadata = e.metadata.clone();
                        self.exists = true;
                    }
                }
            }
            "NodeConfigurationUpdated" => {
                if let Ok(e) = event.as_any().downcast_ref::<NodeConfigurationUpdated>() {
                    if let Some(interfaces) = &e.new_interfaces {
                        self.interfaces = interfaces.clone();
                    }
                    if let Some(services) = &e.new_services {
                        self.services = services.clone();
                    }
                    if let Some(metadata) = &e.new_metadata {
                        self.metadata = metadata.clone();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}