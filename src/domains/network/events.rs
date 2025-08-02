// Copyright 2025 Cowboy AI, LLC.

//! Events for network domain

use super::value_objects::*;
use crate::value_objects::{CorrelationId, CausationId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::any::Any;

/// Wrapper enum for all network events that implements DomainEvent
#[derive(Debug, Clone)]
pub enum NetworkDomainEvent {
    TopologyCreated(NetworkTopologyCreated),
    NodeAdded(NodeAddedToTopology),
    NodeRemoved(NodeRemovedFromTopology),
    NodeUpdated(NodeConfigurationUpdated),
    ConnectionCreated(NetworkConnectionCreated),
    ConnectionRemoved(NetworkConnectionRemoved),
}

impl NetworkDomainEvent {
    /// Get the inner event as Any for downcasting
    pub fn as_any(&self) -> &dyn Any {
        match self {
            Self::TopologyCreated(e) => e,
            Self::NodeAdded(e) => e,
            Self::NodeRemoved(e) => e,
            Self::NodeUpdated(e) => e,
            Self::ConnectionCreated(e) => e,
            Self::ConnectionRemoved(e) => e,
        }
    }
}


impl cim_domain::DomainEvent for NetworkDomainEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::TopologyCreated(_) => "NetworkTopologyCreated",
            Self::NodeAdded(_) => "NodeAddedToTopology",
            Self::NodeRemoved(_) => "NodeRemovedFromTopology",
            Self::NodeUpdated(_) => "NodeConfigurationUpdated",
            Self::ConnectionCreated(_) => "NetworkConnectionCreated",
            Self::ConnectionRemoved(_) => "NetworkConnectionRemoved",
        }
    }
    
    fn subject(&self) -> String {
        match self {
            Self::TopologyCreated(e) => format!("cim.network.topology.{}.created", e.topology_id.0),
            Self::NodeAdded(e) => format!("cim.network.topology.{}.node.added", e.topology_id.0),
            Self::NodeRemoved(e) => format!("cim.network.topology.{}.node.removed", e.topology_id.0),
            Self::NodeUpdated(e) => format!("cim.network.node.{}.configuration.updated", e.node_id.0),
            Self::ConnectionCreated(e) => format!("cim.network.topology.{}.connection.created", e.topology_id.0),
            Self::ConnectionRemoved(e) => format!("cim.network.topology.{}.connection.removed", e.topology_id.0),
        }
    }
    
    fn aggregate_id(&self) -> uuid::Uuid {
        match self {
            Self::TopologyCreated(e) => e.topology_id.0,
            Self::NodeAdded(e) => e.topology_id.0,
            Self::NodeRemoved(e) => e.topology_id.0,
            Self::NodeUpdated(e) => e.node_id.0,
            Self::ConnectionCreated(e) => e.topology_id.0,
            Self::ConnectionRemoved(e) => e.topology_id.0,
        }
    }
}

/// Event: Network topology was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopologyCreated {
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Topology name
    pub name: String,
    /// Description
    pub description: String,
    /// Initial metadata
    pub metadata: HashMap<String, String>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}

/// Event: Node was added to topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAddedToTopology {
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// New node ID
    pub node_id: NetworkNodeId,
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
    /// When added
    pub added_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}

/// Event: Node was removed from topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRemovedFromTopology {
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Removed node ID
    pub node_id: NetworkNodeId,
    /// When removed
    pub removed_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}

/// Event: Node configuration was updated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfigurationUpdated {
    /// Node ID
    pub node_id: NetworkNodeId,
    /// Previous interfaces
    pub old_interfaces: Option<Vec<NetworkInterface>>,
    /// New interfaces
    pub new_interfaces: Option<Vec<NetworkInterface>>,
    /// Previous services
    pub old_services: Option<Vec<String>>,
    /// New services
    pub new_services: Option<Vec<String>>,
    /// Previous metadata
    pub old_metadata: Option<HashMap<String, String>>,
    /// New metadata
    pub new_metadata: Option<HashMap<String, String>>,
    /// When updated
    pub updated_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}

/// Event: Network connection was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectionCreated {
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Connection details
    pub connection: NetworkConnection,
    /// When created
    pub created_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}

/// Event: Network connection was removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnectionRemoved {
    /// Topology ID
    pub topology_id: NetworkTopologyId,
    /// Source node
    pub from_node: NetworkNodeId,
    /// Destination node
    pub to_node: NetworkNodeId,
    /// When removed
    pub removed_at: DateTime<Utc>,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Causation ID
    pub causation_id: CausationId,
}