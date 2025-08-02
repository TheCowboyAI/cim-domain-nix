// Copyright 2025 Cowboy AI, LLC.

//! Events for network domain

use super::value_objects::*;
use crate::value_objects::{CorrelationId, CausationId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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

impl cim_domain::DomainEvent for NetworkTopologyCreated {
    fn event_type(&self) -> &'static str {
        "NetworkTopologyCreated"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
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

impl cim_domain::DomainEvent for NodeAddedToTopology {
    fn event_type(&self) -> &'static str {
        "NodeAddedToTopology"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
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

impl cim_domain::DomainEvent for NodeRemovedFromTopology {
    fn event_type(&self) -> &'static str {
        "NodeRemovedFromTopology"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
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

impl cim_domain::DomainEvent for NodeConfigurationUpdated {
    fn event_type(&self) -> &'static str {
        "NodeConfigurationUpdated"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
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

impl cim_domain::DomainEvent for NetworkConnectionCreated {
    fn event_type(&self) -> &'static str {
        "NetworkConnectionCreated"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
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

impl cim_domain::DomainEvent for NetworkConnectionRemoved {
    fn event_type(&self) -> &'static str {
        "NetworkConnectionRemoved"
    }
    
    fn event_version(&self) -> &'static str {
        "1.0"
    }
}