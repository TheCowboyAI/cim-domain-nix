// Copyright 2025 Cowboy AI, LLC.

//! Network topology acceptance tests

mod acceptance;

#[cfg(test)]
mod network_topology_acceptance_tests {
    use cim_domain_nix::{
        network::{
            NetworkIntegrationService,
            value_objects::{
                NetworkTopology, NetworkNode, NodeType, NodeTier,
                NetworkInterface, InterfaceType, IpAddress,
            },
            NetworkTopologyEvent,
        },
        value_objects::MessageIdentity,
    };
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_minimal_topology() -> anyhow::Result<()> {
        let mut service = NetworkIntegrationService::new();
        
        // Create a minimal topology
        let topology = NetworkTopology {
            id: Uuid::new_v4(),
            name: "minimal".to_string(),
            nodes: vec![
                NetworkNode {
                    id: Uuid::new_v4(),
                    name: "router".to_string(),
                    node_type: NodeType::Router,
                    tier: NodeTier::Cluster,
                    interfaces: vec![
                        NetworkInterface {
                            name: "eth0".to_string(),
                            mac_address: None,
                            interface_type: InterfaceType::Ethernet,
                            addresses: vec![
                                IpAddress {
                                    address: "10.0.0.1".to_string(),
                                    prefix_length: 24,
                                    dhcp: false,
                                }
                            ],
                            mtu: Some(1500),
                            vlan_id: None,
                            bridge_members: vec![],
                        }
                    ],
                    services: vec!["dhcp".to_string()],
                    metadata: HashMap::new(),
                }
            ],
            connections: vec![],
            metadata: HashMap::new(),
        };
        
        let systems = service.process_topology_event(NetworkTopologyEvent {
            topology,
            correlation_id: MessageIdentity::new_root().correlation_id,
            timestamp: chrono::Utc::now(),
        }).await?;
        
        assert_eq!(systems.len(), 1);
        assert_eq!(systems[0].hostname, "router");
        
        Ok(())
    }

    #[tokio::test] 
    async fn test_hierarchical_tiers() -> anyhow::Result<()> {
        let mut service = NetworkIntegrationService::new();
        
        // Create topology with all tier types
        let mut nodes = Vec::new();
        
        // Super-cluster node
        nodes.push(NetworkNode {
            id: Uuid::new_v4(),
            name: "super-cluster".to_string(),
            node_type: NodeType::Gateway,
            tier: NodeTier::SuperCluster,
            interfaces: vec![],
            services: vec!["wan".to_string()],
            metadata: HashMap::new(),
        });
        
        // Cluster node
        nodes.push(NetworkNode {
            id: Uuid::new_v4(),
            name: "cluster".to_string(),
            node_type: NodeType::Router,
            tier: NodeTier::Cluster,
            interfaces: vec![],
            services: vec!["dhcp".to_string(), "dns".to_string()],
            metadata: HashMap::new(),
        });
        
        // Leaf node
        nodes.push(NetworkNode {
            id: Uuid::new_v4(),
            name: "leaf".to_string(),
            node_type: NodeType::Server,
            tier: NodeTier::Leaf,
            interfaces: vec![],
            services: vec!["nats".to_string()],
            metadata: HashMap::new(),
        });
        
        // Client node
        nodes.push(NetworkNode {
            id: Uuid::new_v4(),
            name: "client".to_string(),
            node_type: NodeType::Workstation,
            tier: NodeTier::Client,
            interfaces: vec![],
            services: vec![],
            metadata: HashMap::new(),
        });
        
        let topology = NetworkTopology {
            id: Uuid::new_v4(),
            name: "hierarchical".to_string(),
            nodes,
            connections: vec![
                ("super-cluster".to_string(), "cluster".to_string()),
                ("cluster".to_string(), "leaf".to_string()),
                ("leaf".to_string(), "client".to_string()),
            ],
            metadata: HashMap::new(),
        };
        
        let systems = service.process_topology_event(NetworkTopologyEvent {
            topology,
            correlation_id: MessageIdentity::new_root().correlation_id,
            timestamp: chrono::Utc::now(),
        }).await?;
        
        assert_eq!(systems.len(), 4);
        
        // Verify tier-based service configuration
        let cluster_system = systems.iter()
            .find(|s| s.hostname == "cluster")
            .expect("Should have cluster system");
        assert!(cluster_system.services.contains(&"dhcp".to_string()));
        assert!(cluster_system.services.contains(&"dns".to_string()));
        
        let leaf_system = systems.iter()
            .find(|s| s.hostname == "leaf")
            .expect("Should have leaf system");
        assert!(leaf_system.services.contains(&"nats".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_interface_updates() -> anyhow::Result<()> {
        use cim_domain_nix::network::InterfaceChangeEvent;
        use cim_domain_nix::network::value_objects::InterfaceChangeType;
        
        let mut service = NetworkIntegrationService::new();
        
        // Create initial topology
        let node_id = Uuid::new_v4();
        let topology = NetworkTopology {
            id: Uuid::new_v4(),
            name: "test".to_string(),
            nodes: vec![
                NetworkNode {
                    id: node_id,
                    name: "test-node".to_string(),
                    node_type: NodeType::Router,
                    tier: NodeTier::Leaf,
                    interfaces: vec![
                        NetworkInterface {
                            name: "eth0".to_string(),
                            mac_address: None,
                            interface_type: InterfaceType::Ethernet,
                            addresses: vec![
                                IpAddress {
                                    address: "10.0.0.1".to_string(),
                                    prefix_length: 24,
                                    dhcp: false,
                                }
                            ],
                            mtu: Some(1500),
                            vlan_id: None,
                            bridge_members: vec![],
                        }
                    ],
                    services: vec![],
                    metadata: HashMap::new(),
                }
            ],
            connections: vec![],
            metadata: HashMap::new(),
        };
        
        // Process initial topology
        let _systems = service.process_topology_event(NetworkTopologyEvent {
            topology,
            correlation_id: MessageIdentity::new_root().correlation_id,
            timestamp: chrono::Utc::now(),
        }).await?;
        
        // Update interface
        let change_event = InterfaceChangeEvent {
            node_id,
            interface_name: "eth0".to_string(),
            change_type: InterfaceChangeType::AddressChange,
            old_config: Some(serde_json::json!({
                "address": "10.0.0.1/24"
            })),
            new_config: serde_json::json!({
                "address": "10.0.0.2/24"
            }),
            correlation_id: MessageIdentity::new_root().correlation_id,
            timestamp: chrono::Utc::now(),
        };
        
        let updated_config = service.process_interface_change(change_event).await?;
        assert_eq!(updated_config.hostname, "test-node");
        assert!(updated_config.networking_config.contains("10.0.0.2"));
        
        Ok(())
    }
}