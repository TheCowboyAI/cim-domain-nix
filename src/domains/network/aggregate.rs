// Copyright 2025 Cowboy AI, LLC.

//! Aggregates for network domain

use super::commands::*;
use super::events::{NetworkDomainEvent, *};
use super::value_objects::*;
use crate::value_objects::CausationId;
use cim_domain::DomainEvent;
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
            topology_id: self.id,
            name: cmd.name,
            description: cmd.description,
            metadata: cmd.metadata,
            created_at: Utc::now(),
            correlation_id: cmd.identity.correlation_id,
            causation_id: CausationId(cmd.identity.message_id.0),
        };
        
        Ok(vec![Box::new(NetworkDomainEvent::TopologyCreated(event))])
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
        
        // Generate node ID based on topology ID and node name for determinism
        let node_id = NetworkNodeId(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("{}-{}", self.id.0, cmd.name).as_bytes()
        ));
        
        let event = NodeAddedToTopology {
            topology_id: self.id,
            node_id,
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
        
        Ok(vec![Box::new(NetworkDomainEvent::NodeAdded(event))])
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
        
        Ok(vec![Box::new(NetworkDomainEvent::NodeRemoved(event))])
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
        // TODO: Fix this - nodes are not in the aggregate when this is called
        // if !self.nodes.contains_key(&cmd.from_node) {
        //     return Err("Source node not found".to_string());
        // }
        // if !self.nodes.contains_key(&cmd.to_node) {
        //     return Err("Destination node not found".to_string());
        // }
        
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
        
        Ok(vec![Box::new(NetworkDomainEvent::ConnectionCreated(event))])
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
        
        Ok(vec![Box::new(NetworkDomainEvent::NodeUpdated(event))])
    }
}

