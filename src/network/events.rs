// Copyright 2025 Cowboy AI, LLC.

//! Network event handling

use super::{NetworkTopologyEvent, NetworkEvent, InterfaceChangeEvent};
use crate::{Result, NixDomainError};
use crate::value_objects::CorrelationId;

/// Handler for network domain events
pub struct NetworkEventHandler {
    /// Map of NATS subjects to their corresponding event types
    event_mappings: std::collections::HashMap<String, NetworkEventType>,
}

/// Types of network events we handle
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkEventType {
    /// Network topology has been created
    TopologyCreated,
    /// Network topology has been updated
    TopologyUpdated,
    /// Network interface has been added
    InterfaceAdded,
    /// Network interface has been removed
    InterfaceRemoved,
    /// Network interface has been updated
    InterfaceUpdated,
    /// Network route has been added
    RouteAdded,
    /// Network route has been removed
    RouteRemoved,
    /// Firewall rule has been added
    FirewallRuleAdded,
    /// Firewall rule has been removed
    FirewallRuleRemoved,
}

impl NetworkEventHandler {
    /// Create a new network event handler
    pub fn new() -> Self {
        let mut event_mappings = std::collections::HashMap::new();
        
        // Map NATS subjects to event types
        event_mappings.insert("network.topology.created".to_string(), NetworkEventType::TopologyCreated);
        event_mappings.insert("network.topology.updated".to_string(), NetworkEventType::TopologyUpdated);
        event_mappings.insert("network.interface.added".to_string(), NetworkEventType::InterfaceAdded);
        event_mappings.insert("network.interface.removed".to_string(), NetworkEventType::InterfaceRemoved);
        event_mappings.insert("network.interface.updated".to_string(), NetworkEventType::InterfaceUpdated);
        event_mappings.insert("network.route.added".to_string(), NetworkEventType::RouteAdded);
        event_mappings.insert("network.route.removed".to_string(), NetworkEventType::RouteRemoved);
        event_mappings.insert("network.firewall.rule.added".to_string(), NetworkEventType::FirewallRuleAdded);
        event_mappings.insert("network.firewall.rule.removed".to_string(), NetworkEventType::FirewallRuleRemoved);
        
        Self { event_mappings }
    }
    
    /// Extract topology from a network topology event
    pub fn extract_topology(&self, event: &NetworkTopologyEvent) -> Result<super::NetworkTopology> {
        use super::value_objects::NetworkTopology;
        
        let topology = NetworkTopology::new(
            event.topology_id,
            event.name.clone(),
            event.nodes.clone(),
            event.connections.clone(),
        );
        
        Ok(topology)
    }
    
    /// Parse a NATS message into a network event
    pub async fn parse_nats_event(
        &self,
        subject: &str,
        payload: &[u8],
        headers: &async_nats::header::HeaderMap,
    ) -> Result<NetworkEvent> {
        let event_type = self.event_mappings.get(subject)
            .ok_or_else(|| NixDomainError::Other(format!("Unknown network event subject: {}", subject)))?;
        
        // Extract correlation ID from headers
        let _correlation_id = headers
            .get("X-Correlation-ID")
            .and_then(|value| uuid::Uuid::parse_str(value.as_str()).ok())
            .map(CorrelationId)
            .unwrap_or_else(CorrelationId::new);
        
        match event_type {
            NetworkEventType::TopologyCreated | NetworkEventType::TopologyUpdated => {
                let event: NetworkTopologyEvent = serde_json::from_slice(payload)?;
                Ok(NetworkEvent::TopologyCreated(event))
            }
            NetworkEventType::InterfaceAdded => {
                let event: InterfaceChangeEvent = serde_json::from_slice(payload)?;
                Ok(NetworkEvent::InterfaceAdded(event))
            }
            NetworkEventType::InterfaceRemoved => {
                let event: InterfaceChangeEvent = serde_json::from_slice(payload)?;
                Ok(NetworkEvent::InterfaceRemoved(event))
            }
            NetworkEventType::InterfaceUpdated => {
                let event: InterfaceChangeEvent = serde_json::from_slice(payload)?;
                Ok(NetworkEvent::InterfaceUpdated(event))
            }
            _ => Err(NixDomainError::Other(format!("Unimplemented event type: {:?}", event_type))),
        }
    }
    
    /// Get all network event subjects we should subscribe to
    pub fn get_subscription_subjects(&self) -> Vec<String> {
        self.event_mappings.keys().cloned().collect()
    }
}

impl Default for NetworkEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_handler_creation() {
        let handler = NetworkEventHandler::new();
        let subjects = handler.get_subscription_subjects();
        
        assert!(subjects.contains(&"network.topology.created".to_string()));
        assert!(subjects.contains(&"network.interface.added".to_string()));
        assert_eq!(subjects.len(), 9);
    }
}