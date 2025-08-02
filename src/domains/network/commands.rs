// Copyright 2025 Cowboy AI, LLC.

//! Commands for network domain operations

use super::value_objects::*;
use crate::value_objects::MessageIdentity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Command to create a new network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkTopology {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Name of the topology
    pub name: String,
    /// Description
    pub description: String,
    /// Initial metadata
    pub metadata: HashMap<String, String>,
}

impl cim_domain::Command for CreateNetworkTopology {
    type Aggregate = super::aggregate::NetworkTopologyAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        None // Creating new aggregate
    }
}

/// Command to add a node to the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNodeToTopology {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Topology to add to
    pub topology_id: NetworkTopologyId,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// Node tier in hierarchy
    pub tier: NodeTier,
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
    /// Services this node provides
    pub services: Vec<String>,
    /// Node metadata
    pub metadata: HashMap<String, String>,
}

impl cim_domain::Command for AddNodeToTopology {
    type Aggregate = super::aggregate::NetworkTopologyAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_type_and_id(
            "NetworkTopology",
            self.topology_id.0,
        ))
    }
}

/// Command to remove a node from topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveNodeFromTopology {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Node to remove
    pub node_id: NetworkNodeId,
}

impl cim_domain::Command for RemoveNodeFromTopology {
    type Aggregate = super::aggregate::NetworkTopologyAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_type_and_id(
            "NetworkTopology",
            self.topology_id.0,
        ))
    }
}

/// Command to update node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNodeConfiguration {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Node to update
    pub node_id: NetworkNodeId,
    /// New interfaces (if changed)
    pub interfaces: Option<Vec<NetworkInterface>>,
    /// New services (if changed)
    pub services: Option<Vec<String>>,
    /// Updated metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl cim_domain::Command for UpdateNodeConfiguration {
    type Aggregate = super::aggregate::NetworkNodeAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_type_and_id(
            "NetworkNode",
            self.node_id.0,
        ))
    }
}

/// Command to create a network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNetworkConnection {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Source node
    pub from_node: NetworkNodeId,
    /// Destination node
    pub to_node: NetworkNodeId,
    /// Connection type
    pub connection_type: ConnectionType,
    /// Connection properties
    pub properties: ConnectionProperties,
}

impl cim_domain::Command for CreateNetworkConnection {
    type Aggregate = super::aggregate::NetworkTopologyAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_type_and_id(
            "NetworkTopology",
            self.topology_id.0,
        ))
    }
}

/// Command to remove a network connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveNetworkConnection {
    /// Message identity for correlation/causation
    pub identity: MessageIdentity,
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Source node
    pub from_node: NetworkNodeId,
    /// Destination node
    pub to_node: NetworkNodeId,
}

impl cim_domain::Command for RemoveNetworkConnection {
    type Aggregate = super::aggregate::NetworkTopologyAggregate;
    
    fn aggregate_id(&self) -> Option<cim_domain::EntityId<Self::Aggregate>> {
        Some(cim_domain::EntityId::from_type_and_id(
            "NetworkTopology",
            self.topology_id.0,
        ))
    }
}