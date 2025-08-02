// Copyright 2025 Cowboy AI, LLC.

//! CIM Network Domain Implementation
//! 
//! This module implements the network domain for CIM, providing:
//! - Network topology modeling
//! - Node hierarchy (Client -> Leaf -> Cluster -> Super-cluster)
//! - Event-driven network configuration
//! - Automatic NixOS system generation from network topology

pub mod aggregate;
pub mod commands;
pub mod events;
pub mod value_objects;
pub mod handlers;
pub mod services;

pub use aggregate::{NetworkTopologyAggregate, NetworkNodeAggregate};
pub use commands::{
    CreateNetworkTopology, AddNodeToTopology, RemoveNodeFromTopology,
    UpdateNodeConfiguration, CreateNetworkConnection, RemoveNetworkConnection,
};
pub use events::{
    NetworkTopologyCreated, NodeAddedToTopology, NodeRemovedFromTopology,
    NodeConfigurationUpdated, NetworkConnectionCreated, NetworkConnectionRemoved,
};
pub use value_objects::{
    NetworkTopologyId, NetworkNodeId, NodeTier, NodeType,
    NetworkInterface, IpAddress, InterfaceType, NetworkService,
};
pub use services::NetworkTopologyService;

use crate::Result;