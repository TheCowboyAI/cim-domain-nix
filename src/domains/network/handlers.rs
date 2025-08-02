// Copyright 2025 Cowboy AI, LLC.

//! Command and query handlers for network domain

use super::aggregate::{NetworkTopologyAggregate, NetworkNodeAggregate};
use super::commands::*;
use super::events::*;
use super::value_objects::*;
use crate::Result;
use cim_domain::{Aggregate, DomainEvent};
use std::collections::HashMap;

/// Command handler for network domain
pub struct NetworkCommandHandler {
    // In a real implementation, this would connect to event store
    topologies: HashMap<NetworkTopologyId, NetworkTopologyAggregate>,
    nodes: HashMap<NetworkNodeId, NetworkNodeAggregate>,
}

impl NetworkCommandHandler {
    /// Create a new command handler
    pub fn new() -> Self {
        Self {
            topologies: HashMap::new(),
            nodes: HashMap::new(),
        }
    }
    
    /// Handle create topology command
    pub async fn handle_create_topology(
        &mut self,
        cmd: CreateNetworkTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let topology_id = NetworkTopologyId::new();
        let aggregate = NetworkTopologyAggregate::new(topology_id);
        
        let events = aggregate.handle_create(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Apply events to aggregate
        let mut updated_aggregate = aggregate;
        for event in &events {
            updated_aggregate.apply_event(event.clone())
                .map_err(|e| crate::NixDomainError::Other(e))?;
        }
        
        self.topologies.insert(topology_id, updated_aggregate);
        
        Ok(events)
    }
    
    /// Handle add node command
    pub async fn handle_add_node(
        &mut self,
        cmd: AddNodeToTopology,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let aggregate = self.topologies.get(&cmd.topology_id)
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))?;
        
        let events = aggregate.handle_add_node(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Apply events to aggregate
        if let Some(aggregate) = self.topologies.get_mut(&aggregate.id) {
            for event in &events {
                aggregate.apply_event(event.clone())
                    .map_err(|e| crate::NixDomainError::Other(e))?;
                
                // Also create/update node aggregate
                if let Ok(node_event) = event.as_any().downcast_ref::<NodeAddedToTopology>() {
                    let mut node_aggregate = NetworkNodeAggregate::new(node_event.node_id);
                    node_aggregate.apply_event(event.clone())
                        .map_err(|e| crate::NixDomainError::Other(e))?;
                    self.nodes.insert(node_event.node_id, node_aggregate);
                }
            }
        }
        
        Ok(events)
    }
    
    /// Handle update node configuration command
    pub async fn handle_update_node(
        &mut self,
        cmd: UpdateNodeConfiguration,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let aggregate = self.nodes.get(&cmd.node_id)
            .ok_or_else(|| crate::NixDomainError::Other("Node not found".to_string()))?;
        
        let events = aggregate.handle_update_configuration(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Apply events to aggregate
        if let Some(aggregate) = self.nodes.get_mut(&aggregate.id) {
            for event in &events {
                aggregate.apply_event(event.clone())
                    .map_err(|e| crate::NixDomainError::Other(e))?;
            }
        }
        
        Ok(events)
    }
    
    /// Handle create connection command
    pub async fn handle_create_connection(
        &mut self,
        cmd: CreateNetworkConnection,
    ) -> Result<Vec<Box<dyn DomainEvent>>> {
        let aggregate = self.topologies.get(&cmd.topology_id)
            .ok_or_else(|| crate::NixDomainError::Other("Topology not found".to_string()))?;
        
        let events = aggregate.handle_create_connection(cmd)
            .map_err(|e| crate::NixDomainError::Other(e))?;
        
        // Apply events to aggregate
        if let Some(aggregate) = self.topologies.get_mut(&aggregate.id) {
            for event in &events {
                aggregate.apply_event(event.clone())
                    .map_err(|e| crate::NixDomainError::Other(e))?;
            }
        }
        
        Ok(events)
    }
}

/// Query handler for network domain
pub struct NetworkQueryHandler {
    // In a real implementation, this would query event store
    topologies: HashMap<NetworkTopologyId, NetworkTopologyAggregate>,
    nodes: HashMap<NetworkNodeId, NetworkNodeAggregate>,
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