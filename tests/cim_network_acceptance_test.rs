// Copyright 2025 Cowboy AI, LLC.

//! Acceptance test for CIM Network Domain with Starlink topology

#[cfg(test)]
mod tests {
    use cim_domain_nix::domains::network::{
        NetworkTopologyService,
        ConnectionType,
        ConnectionProperties,
        CreateNetworkConnection,
        NodeTier,
    };
    use cim_domain_nix::value_objects::MessageIdentity;

    #[tokio::test]
    async fn test_starlink_topology_acceptance() -> anyhow::Result<()> {
        // Create service
        let mut service = NetworkTopologyService::new();
        
        // Create Starlink topology
        let topology = service.create_starlink_topology(
            "test-starlink".to_string(),
            "192.168.100".to_string(),
            "10.0.0".to_string(),
        ).await?;
        
        assert_eq!(topology.name, "test-starlink");
        assert_eq!(topology.nodes.len(), 1); // Just Starlink router initially
        
        // Add UDM Pro
        let udm_id = service.add_udm_pro(
            topology.id,
            "192.168.100.2".to_string(),
            "10.0.0".to_string(),
        ).await?;
        
        // Add Mac Studio
        let mac_id = service.add_mac_studio_leaf(
            topology.id,
            "10.0.0.100".to_string(),
        ).await?;
        
        // Generate NixOS configs
        let configs = service.generate_nixos_configs(topology.id).await?;
        
        // Verify we have 3 configurations
        assert_eq!(configs.len(), 3);
        
        // Verify Starlink config
        let starlink_config = configs.iter()
            .find(|c| c.hostname == "starlink-router")
            .expect("Should have Starlink config");
        assert!(starlink_config.networking.contains_key("hostName"));
        
        // Verify UDM Pro config
        let udm_config = configs.iter()
            .find(|c| c.hostname == "udm-pro")
            .expect("Should have UDM Pro config");
        assert!(udm_config.services.contains_key("dhcpd4"));
        assert!(udm_config.services.contains_key("unbound"));
        assert!(udm_config.networking.contains_key("nat"));
        
        // Verify Mac Studio config
        let mac_config = configs.iter()
            .find(|c| c.hostname == "mac-studio-leaf")
            .expect("Should have Mac Studio config");
        assert!(mac_config.services.contains_key("nats"));
        assert!(mac_config.packages.contains(&"cim-leaf".to_string()));
        assert!(mac_config.extra_config.contains("services.cim-leaf"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_node_tier_hierarchy() -> anyhow::Result<()> {
        let mut service = NetworkTopologyService::new();
        
        // Create topology with all tiers
        let topology = service.create_starlink_topology(
            "tier-test".to_string(),
            "192.168.1".to_string(),
            "10.0.0".to_string(),
        ).await?;
        
        // Starlink should be SuperCluster tier
        let starlink = topology.nodes.first().expect("Should have Starlink");
        assert_eq!(starlink.tier, NodeTier::SuperCluster);
        
        // Add UDM (Cluster tier)
        let udm_id = service.add_udm_pro(
            topology.id,
            "192.168.1.2".to_string(),
            "10.0.0".to_string(),
        ).await?;
        
        // Add Mac (Leaf tier)
        let mac_id = service.add_mac_studio_leaf(
            topology.id,
            "10.0.0.100".to_string(),
        ).await?;
        
        // Verify tier hierarchy
        assert!(NodeTier::SuperCluster.can_serve(&NodeTier::Cluster));
        assert!(NodeTier::Cluster.can_serve(&NodeTier::Leaf));
        assert!(NodeTier::Leaf.can_serve(&NodeTier::Client));
        assert!(!NodeTier::Client.can_serve(&NodeTier::Leaf));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_nixos_config_generation() -> anyhow::Result<()> {
        let mut service = NetworkTopologyService::new();
        
        // Create simple topology
        let topology = service.create_starlink_topology(
            "config-test".to_string(),
            "192.168.1".to_string(),
            "10.0.0".to_string(),
        ).await?;
        
        // Generate configs
        let configs = service.generate_nixos_configs(topology.id).await?;
        
        assert_eq!(configs.len(), 1);
        
        let config = &configs[0];
        
        // Verify basic configuration
        assert_eq!(config.hostname, "starlink-router");
        assert_eq!(config.system, "x86_64-linux");
        assert!(!config.packages.is_empty());
        
        // Verify networking configuration
        assert_eq!(config.networking.get("hostName"), Some(&"starlink-router".to_string()));
        assert_eq!(config.networking.get("domain"), Some(&"local".to_string()));
        
        // Verify firewall is configured for gateway
        assert!(config.services.contains_key("firewall"));
        
        Ok(())
    }
}