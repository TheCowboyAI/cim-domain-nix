// Copyright 2025 Cowboy AI, LLC.

//! Commands for network domain operations

use super::value_objects::*;
use crate::value_objects::MessageIdentity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

