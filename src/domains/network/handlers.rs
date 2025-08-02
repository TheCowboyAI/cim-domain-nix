// Copyright 2025 Cowboy AI, LLC.

//! Command and query handlers for network domain

use super::aggregate::{NetworkTopologyAggregate, NetworkNodeAggregate, NetworkNode};
use super::commands::*;
use super::value_objects::{*, NetworkConnection};
use crate::Result;
use cim_domain::DomainEvent;
use std::collections::HashMap;

/// Command handler for network domain
pub struct NetworkCommandHandler {
    // In a real implementation, this would connect to event store
    pub topologies: HashMap<NetworkTopologyId, NetworkTopologyAggregate>,
    pub nodes: HashMap<NetworkNodeId, NetworkNodeAggregate>,
}

impl NetworkCommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            topologies: HashMap::new(),
            nodes: HashMap::new(),
        }
    }
    
    /// Handle create topology command with predefined ID
    pub async fn handle_create_topology_with_id(
        &mut self,
        topology_id: NetworkTopologyId,
        cmd: CreateNetworkTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let aggregate = NetworkTopologyAggregate::new(topology_id);
        
        let events = aggregate.handle_create(cmd.clone())
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Create a properly initialized aggregate with the event data
        let mut updated_aggregate = aggregate;
        updated_aggregate.exists = true;
        updated_aggregate.name = cmd.name;
        updated_aggregate.description = cmd.description;
        updated_aggregate.metadata = cmd.metadata;
        
        self.topologies.insert(topology_id, updated_aggregate);
        
        Ok(events)
    }
    
    /// Handle create topology command
    pub async fn handle_create_topology(
        &mut self,
        cmd: CreateNetworkTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let topology_id = NetworkTopologyId::new();
        self.handle_create_topology_with_id(topology_id, cmd).await
    }
    
    /// Handle add node command
    pub async fn handle_add_node(
        &mut self,
        cmd: AddNodeToTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let topology_id = cmd.topology_id;
        
        // Clone the command data we need before borrowing the aggregate
        let node_name = cmd.name.clone();
        let node_type = cmd.node_type.clone();
        let tier = cmd.tier;
        let interfaces = cmd.interfaces.clone();
        let services = cmd.services.clone();
        let metadata = cmd.metadata.clone();
        
        let aggregate = self.topologies.get(&topology_id)
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))?;
        
        let events = aggregate.handle_add_node(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Calculate the node ID (same as aggregate will generate)
        let node_id = NetworkNodeId(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("{}-{}", topology_id.0, node_name).as_bytes()
        ));
        
        // Update the topology with the new node
        if let Some(topology) = self.topologies.get_mut(&topology_id) {
            let node = NetworkNode {
                id: node_id,
                name: node_name.clone(),
                node_type: node_type.clone(),
                tier,
                interfaces: interfaces.clone(),
                services: services.clone(),
                metadata: metadata.clone(),
            };
            topology.nodes.insert(node_id, node);
        }
        
        // Also create node aggregate
        let mut node_aggregate = NetworkNodeAggregate::new(node_id);
        node_aggregate.name = node_name;
        node_aggregate.node_type = node_type;
        node_aggregate.tier = tier;
        node_aggregate.interfaces = interfaces;
        node_aggregate.services = services;
        node_aggregate.metadata = metadata;
        node_aggregate.exists = true;
        
        self.nodes.insert(node_id, node_aggregate);
        
        Ok(events)
    }
    
    /// Handle update node configuration command
    pub async fn handle_update_node(
        &mut self,
        cmd: UpdateNodeConfiguration,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let node_id = cmd.node_id;
        let aggregate = self.nodes.get(&node_id)
            .ok_or_else(|| crate::NixDomainError::Other("Node not found".to_string()))?;
        
        let events = aggregate.handle_update_configuration(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        Ok(events)
    }
    
    /// Handle create connection command
    pub async fn handle_create_connection(
        &mut self,
        cmd: CreateNetworkConnection,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let topology_id = cmd.topology_id;
        let from_node = cmd.from_node;
        let to_node = cmd.to_node;
        let connection_type = cmd.connection_type.clone();
        let properties = cmd.properties.clone();
        
        let aggregate = self.topologies.get(&topology_id)
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))?;
        
        let events = aggregate.handle_create_connection(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Update the topology with the new connection
        if let Some(topology) = self.topologies.get_mut(&topology_id) {
            let connection = NetworkConnection {
                from_node,
                to_node,
                connection_type,
                properties,
            };
            topology.connections.push(connection);
        }
        
        Ok(events)
    }
}

/// Query handler for network domain
pub struct NetworkQueryHandler {
    // In a real implementation, this would query event store
    pub topologies: HashMap<NetworkTopologyId, NetworkTopologyAggregate>,
    pub nodes: HashMap<NetworkNodeId, NetworkNodeAggregate>,
}

impl NetworkQueryHandler {
    /// Create a new query handler
    pub fn new() -> Self {
        Self {
            topologies: HashMap::new(),
            nodes: HashMap::new(),
        }
    }
    
    /// Get topology by ID
    pub async fn get_topology(&self, id: NetworkTopologyId) -> Result<Option<NetworkTopologyView>> {
        Ok(self.topologies.get(&id).map(|agg| NetworkTopologyView {
            id: agg.id,
            name: agg.name.clone(),
            description: agg.description.clone(),
            nodes: agg.nodes.values().map(|n| NetworkNodeView {
                id: n.id,
                name: n.name.clone(),
                node_type: n.node_type.clone(),
                tier: n.tier,
                interfaces: n.interfaces.clone(),
                services: n.services.clone(),
                metadata: n.metadata.clone(),
            }).collect(),
            connections: agg.connections.clone(),
            metadata: agg.metadata.clone(),
        }))
    }
    
    /// Get node by ID
    pub async fn get_node(&self, id: NetworkNodeId) -> Result<Option<NetworkNodeView>> {
        Ok(self.nodes.get(&id).map(|agg| NetworkNodeView {
            id: agg.id,
            name: agg.name.clone(),
            node_type: agg.node_type.clone(),
            tier: agg.tier,
            interfaces: agg.interfaces.clone(),
            services: agg.services.clone(),
            metadata: agg.metadata.clone(),
        }))
    }
    
    /// List all topologies
    pub async fn list_topologies(&self) -> Result<Vec<NetworkTopologyView>> {
        Ok(self.topologies.values().map(|agg| NetworkTopologyView {
            id: agg.id,
            name: agg.name.clone(),
            description: agg.description.clone(),
            nodes: agg.nodes.values().map(|n| NetworkNodeView {
                id: n.id,
                name: n.name.clone(),
                node_type: n.node_type.clone(),
                tier: n.tier,
                interfaces: n.interfaces.clone(),
                services: n.services.clone(),
                metadata: n.metadata.clone(),
            }).collect(),
            connections: agg.connections.clone(),
            metadata: agg.metadata.clone(),
        }).collect())
    }
}

/// Read model view of a network topology
#[derive(Debug, Clone)]
pub struct NetworkTopologyView {
    pub id: NetworkTopologyId,
    pub name: String,
    pub description: String,
    pub nodes: Vec<NetworkNodeView>,
    pub connections: Vec<NetworkConnection>,
    pub metadata: HashMap<String, String>,
}

/// Read model view of a network node
#[derive(Debug, Clone)]
pub struct NetworkNodeView {
    pub id: NetworkNodeId,
    pub name: String,
    pub node_type: NodeType,
    pub tier: NodeTier,
    pub interfaces: Vec<NetworkInterface>,
    pub services: Vec<String>,
    pub metadata: HashMap<String, String>,
}