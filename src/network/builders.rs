// Copyright 2025 Cowboy AI, LLC.

//! System builders for network-based NixOS configurations

use super::{
    NetworkNode, NetworkTopology, SystemConfiguration, NetworkConfig, ServiceConfig,
    FirewallConfig, InterfaceConfig, RouteConfig, DnsConfig,
    value_objects::{NodeType, InterfaceType},
};
use crate::Result;
use crate::value_objects::FlakeRef;
use std::collections::HashMap;

/// Configuration for system building
#[derive(Debug, Clone)]
pub struct SystemBuildConfig {
    /// Base flake to use for system configurations
    pub base_flake: Option<FlakeRef>,
    /// Default timezone
    pub timezone: String,
    /// Default locale
    pub locale: String,
    /// Enable SSH by default
    pub enable_ssh: bool,
    /// Custom nixpkgs configuration
    pub nixpkgs_config: HashMap<String, serde_json::Value>,
}

impl Default for SystemBuildConfig {
    fn default() -> Self {
        Self {
            base_flake: None,
            timezone: "UTC".to_string(),
            locale: "en_US.UTF-8".to_string(),
            enable_ssh: true,
            nixpkgs_config: HashMap::new(),
        }
    }
}

/// Builder for creating NixOS systems from network topology
pub struct NetworkSystemBuilder {
    config: SystemBuildConfig,
}

impl NetworkSystemBuilder {
    /// Create a new system builder
    pub fn new() -> Self {
        Self {
            config: SystemBuildConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: SystemBuildConfig) -> Self {
        Self {
            config,
        }
    }
    
    /// Build a system configuration for a network node
    pub async fn build_system_for_node(
        &self,
        node: &NetworkNode,
        topology: &NetworkTopology,
    ) -> Result<SystemConfiguration> {
        let system_id = uuid::Uuid::new_v4();
        
        // Build network configuration
        let network_config = self.build_network_config(node, topology)?;
        
        // Build service configurations based on node type and services
        let services = self.build_services_for_node(node, topology)?;
        
        // Build firewall configuration
        let firewall_config = self.build_firewall_config(node, &services)?;
        
        Ok(SystemConfiguration {
            system_id,
            node_id: node.id,
            hostname: node.name.clone(),
            network_config,
            services,
            firewall_config,
        })
    }
    
    /// Build network configuration for a node
    fn build_network_config(
        &self,
        node: &NetworkNode,
        topology: &NetworkTopology,
    ) -> Result<NetworkConfig> {
        let mut interfaces = Vec::new();
        let mut routes = Vec::new();
        
        // Configure interfaces
        for interface in &node.interfaces {
            let interface_config = InterfaceConfig {
                name: interface.name.clone(),
                mac_address: interface.mac_address.clone(),
                addresses: interface.static_addresses()
                    .iter()
                    .filter_map(|a| a.to_cidr())
                    .collect(),
                dhcp: interface.uses_dhcp(),
                mtu: interface.mtu,
            };
            interfaces.push(interface_config);
        }
        
        // Add default routes based on node type and connections
        if !node.is_router() {
            // Find gateway in topology
            if let Some(gateway) = self.find_gateway_for_node(node, topology) {
                routes.push(RouteConfig {
                    destination: "0.0.0.0/0".to_string(),
                    gateway: Some(gateway),
                    interface: node.primary_interface()
                        .map(|i| i.name.clone())
                        .unwrap_or_else(|| "eth0".to_string()),
                });
            }
        }
        
        // Configure DNS
        let dns = if node.should_run_dns() {
            // If this node runs DNS, use localhost
            Some(DnsConfig {
                nameservers: vec!["127.0.0.1".to_string()],
                search_domains: vec![],
            })
        } else {
            // Find DNS servers in topology
            self.find_dns_servers(topology).map(|servers| DnsConfig {
                nameservers: servers,
                search_domains: vec![],
            })
        };
        
        Ok(NetworkConfig {
            interfaces,
            routes,
            dns,
        })
    }
    
    /// Build services for a node based on its type and role
    fn build_services_for_node(
        &self,
        node: &NetworkNode,
        topology: &NetworkTopology,
    ) -> Result<Vec<ServiceConfig>> {
        let mut services = Vec::new();
        
        // Common services
        if self.config.enable_ssh {
            services.push(ServiceConfig {
                name: "openssh".to_string(),
                enabled: true,
                settings: serde_json::json!({
                    "enable": true,
                    "settings": {
                        "PermitRootLogin": "prohibit-password",
                        "PasswordAuthentication": false
                    }
                }),
            });
        }
        
        // Node type specific services
        match &node.node_type {
            NodeType::Router | NodeType::Gateway => {
                services.extend(self.build_router_services(node, topology)?);
            }
            NodeType::Server => {
                services.extend(self.build_server_services(node)?);
            }
            NodeType::AccessPoint => {
                services.extend(self.build_access_point_services(node)?);
            }
            _ => {}
        }
        
        // Services specified in node metadata
        for service_name in &node.services {
            if let Some(service) = self.build_service_by_name(service_name, node)? {
                services.push(service);
            }
        }
        
        Ok(services)
    }
    
    /// Build router-specific services
    fn build_router_services(
        &self,
        node: &NetworkNode,
        topology: &NetworkTopology,
    ) -> Result<Vec<ServiceConfig>> {
        let mut services = Vec::new();
        
        // Enable IP forwarding
        services.push(ServiceConfig {
            name: "ip-forwarding".to_string(),
            enabled: true,
            settings: serde_json::json!({
                "boot.kernel.sysctl": {
                    "net.ipv4.ip_forward": true,
                    "net.ipv6.conf.all.forwarding": true
                }
            }),
        });
        
        // DHCP server
        if node.should_run_dhcp() {
            if let Some(dhcp_config) = self.build_dhcp_config(node, topology)? {
                services.push(dhcp_config);
            }
        }
        
        // DNS server
        if node.should_run_dns() {
            services.push(ServiceConfig {
                name: "unbound".to_string(),
                enabled: true,
                settings: serde_json::json!({
                    "enable": true,
                    "settings": {
                        "server": {
                            "interface": ["0.0.0.0", "::0"],
                            "access-control": [
                                "127.0.0.0/8 allow",
                                "192.168.0.0/16 allow",
                                "10.0.0.0/8 allow"
                            ]
                        }
                    }
                }),
            });
        }
        
        // NAT/Masquerading
        if self.should_enable_nat(node, topology) {
            services.push(ServiceConfig {
                name: "nat".to_string(),
                enabled: true,
                settings: serde_json::json!({
                    "networking.nat": {
                        "enable": true,
                        "externalInterface": self.find_external_interface(node),
                        "internalInterfaces": self.find_internal_interfaces(node)
                    }
                }),
            });
        }
        
        Ok(services)
    }
    
    /// Build server-specific services
    fn build_server_services(&self, node: &NetworkNode) -> Result<Vec<ServiceConfig>> {
        let mut services = Vec::new();
        
        // Check metadata for service hints
        if let Some(role) = node.metadata.get("role") {
            match role.as_str() {
                "web" => {
                    services.push(ServiceConfig {
                        name: "nginx".to_string(),
                        enabled: true,
                        settings: serde_json::json!({
                            "enable": true,
                            "virtualHosts": {
                                "default": {
                                    "root": "/var/www"
                                }
                            }
                        }),
                    });
                }
                "database" => {
                    services.push(ServiceConfig {
                        name: "postgresql".to_string(),
                        enabled: true,
                        settings: serde_json::json!({
                            "enable": true,
                            "enableTCPIP": true
                        }),
                    });
                }
                _ => {}
            }
        }
        
        Ok(services)
    }
    
    /// Build access point services
    fn build_access_point_services(&self, node: &NetworkNode) -> Result<Vec<ServiceConfig>> {
        let mut services = Vec::new();
        
        // Find WiFi interface
        if let Some(wifi_interface) = node.interfaces.iter()
            .find(|i| matches!(i.interface_type, InterfaceType::Wifi)) 
        {
            services.push(ServiceConfig {
                name: "hostapd".to_string(),
                enabled: true,
                settings: serde_json::json!({
                    "enable": true,
                    "interface": wifi_interface.name,
                    "ssid": node.metadata.get("ssid").cloned().unwrap_or_else(|| format!("{}-wifi", node.name)),
                    "channel": node.metadata.get("channel").and_then(|c| c.parse::<u8>().ok()).unwrap_or(6),
                    "hw_mode": "g",
                    "ieee80211n": true
                }),
            });
        }
        
        Ok(services)
    }
    
    /// Build service configuration by name
    fn build_service_by_name(&self, name: &str, node: &NetworkNode) -> Result<Option<ServiceConfig>> {
        let service = match name {
            "wireguard" => {
                Some(ServiceConfig {
                    name: "wireguard".to_string(),
                    enabled: true,
                    settings: serde_json::json!({
                        "networking.wireguard.interfaces": self.build_wireguard_config(node)
                    }),
                })
            }
            "monitoring" => {
                Some(ServiceConfig {
                    name: "prometheus".to_string(),
                    enabled: true,
                    settings: serde_json::json!({
                        "enable": true,
                        "port": 9090
                    }),
                })
            }
            _ => None,
        };
        
        Ok(service)
    }
    
    /// Build DHCP configuration
    fn build_dhcp_config(&self, node: &NetworkNode, _topology: &NetworkTopology) -> Result<Option<ServiceConfig>> {
        // Simple DHCP config - in production would use topology info
        Ok(Some(ServiceConfig {
            name: "dhcpd4".to_string(),
            enabled: true,
            settings: serde_json::json!({
                "enable": true,
                "interfaces": node.interfaces.iter()
                    .filter(|i| !matches!(i.interface_type, InterfaceType::Loopback))
                    .map(|i| &i.name)
                    .collect::<Vec<_>>(),
                "extraConfig": format!(
                    "subnet {} netmask {} {{\n  range {} {};\n  option routers {};\n  option domain-name-servers {};\n}}",
                    "192.168.1.0",
                    "255.255.255.0",
                    "192.168.1.100",
                    "192.168.1.200",
                    self.get_router_ip(node),
                    self.get_dns_servers(node)
                )
            }),
        }))
    }
    
    /// Build firewall configuration
    fn build_firewall_config(
        &self,
        _node: &NetworkNode,
        services: &[ServiceConfig],
    ) -> Result<Option<FirewallConfig>> {
        let mut allowed_tcp_ports = Vec::new();
        let mut allowed_udp_ports = Vec::new();
        
        // SSH
        if self.config.enable_ssh {
            allowed_tcp_ports.push(22);
        }
        
        // Service-specific ports
        for service in services {
            match service.name.as_str() {
                "nginx" => allowed_tcp_ports.extend(&[80, 443]),
                "postgresql" => allowed_tcp_ports.push(5432),
                "prometheus" => allowed_tcp_ports.push(9090),
                "unbound" => {
                    allowed_tcp_ports.push(53);
                    allowed_udp_ports.push(53);
                }
                "dhcpd4" => allowed_udp_ports.extend(&[67, 68]),
                _ => {}
            }
        }
        
        // WireGuard
        if services.iter().any(|s| s.name == "wireguard") {
            allowed_udp_ports.push(51820);
        }
        
        Ok(Some(FirewallConfig {
            enable: true,
            allowed_tcp_ports,
            allowed_udp_ports,
            rules: vec![],
        }))
    }
    
    /// Helper: Find gateway for a node
    fn find_gateway_for_node(&self, node: &NetworkNode, topology: &NetworkTopology) -> Option<String> {
        // Find connected router/gateway nodes
        for conn in topology.node_connections(node.id) {
            let other_node_id = if conn.from_node == node.id {
                conn.to_node
            } else {
                conn.from_node
            };
            
            if let Some(other_node) = topology.find_node(other_node_id) {
                if other_node.is_router() {
                    // Get the IP of the router's interface connected to this node
                    if let Some(interface) = other_node.get_interface(&conn.from_interface) {
                        if let Some(addr) = interface.static_addresses().first() {
                            return addr.address.clone().into();
                        }
                    }
                }
            }
        }
        
        None
    }
    
    /// Helper: Find DNS servers in topology
    fn find_dns_servers(&self, topology: &NetworkTopology) -> Option<Vec<String>> {
        let mut servers = Vec::new();
        
        for node in topology.nodes() {
            if node.should_run_dns() {
                if let Some(interface) = node.primary_interface() {
                    if let Some(addr) = interface.static_addresses().first() {
                        servers.push(addr.address.clone());
                    }
                }
            }
        }
        
        if servers.is_empty() {
            // Fallback to public DNS
            Some(vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()])
        } else {
            Some(servers)
        }
    }
    
    /// Helper: Check if NAT should be enabled
    fn should_enable_nat(&self, node: &NetworkNode, _topology: &NetworkTopology) -> bool {
        // Enable NAT if node is a gateway with both internal and external interfaces
        node.node_type == NodeType::Gateway && node.interfaces.len() > 1
    }
    
    /// Helper: Find external interface (typically the one with public IP or default route)
    fn find_external_interface(&self, node: &NetworkNode) -> String {
        // Simple heuristic: interface with name containing "wan" or "external"
        node.interfaces.iter()
            .find(|i| i.name.contains("wan") || i.name.contains("external"))
            .or_else(|| node.interfaces.first())
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "eth0".to_string())
    }
    
    /// Helper: Find internal interfaces
    fn find_internal_interfaces(&self, node: &NetworkNode) -> Vec<String> {
        node.interfaces.iter()
            .filter(|i| !i.name.contains("wan") && !i.name.contains("external"))
            .map(|i| i.name.clone())
            .collect()
    }
    
    /// Helper: Build WireGuard configuration
    fn build_wireguard_config(&self, _node: &NetworkNode) -> serde_json::Value {
        // This would be more sophisticated in a real implementation
        serde_json::json!({
            "wg0": {
                "ips": ["10.0.0.1/24"],
                "listenPort": 51820,
                "privateKeyFile": "/etc/wireguard/private.key"
            }
        })
    }
    
    // Network calculation helpers
    fn extract_network<'a>(&self, cidr: &'a str) -> &'a str {
        cidr.split('/').next().unwrap_or("192.168.1.0")
    }
    
    fn extract_netmask(&self, cidr: &str) -> &'static str {
        match cidr.split('/').nth(1).and_then(|p| p.parse::<u8>().ok()) {
            Some(24) => "255.255.255.0",
            Some(16) => "255.255.0.0",
            Some(8) => "255.0.0.0",
            _ => "255.255.255.0",
        }
    }
    
    fn calculate_dhcp_start(&self, cidr: &str) -> String {
        // Simple implementation - would be more sophisticated in production
        let network = self.extract_network(cidr);
        let parts: Vec<&str> = network.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}.100", parts[0], parts[1], parts[2])
        } else {
            "192.168.1.100".to_string()
        }
    }
    
    fn calculate_dhcp_end(&self, cidr: &str) -> String {
        let network = self.extract_network(cidr);
        let parts: Vec<&str> = network.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}.200", parts[0], parts[1], parts[2])
        } else {
            "192.168.1.200".to_string()
        }
    }
    
    fn get_router_ip(&self, node: &NetworkNode) -> String {
        node.primary_interface()
            .and_then(|i| {
                let addresses = i.static_addresses();
                addresses.first().map(|a| a.address.clone())
            })
            .unwrap_or_else(|| "192.168.1.1".to_string())
    }
    
    fn get_dns_servers(&self, node: &NetworkNode) -> String {
        if node.should_run_dns() {
            self.get_router_ip(node)
        } else {
            "8.8.8.8, 8.8.4.4".to_string()
        }
    }
}

impl Default for NetworkSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::value_objects::{IpAddress, NodeType};
    
    #[tokio::test]
    async fn test_system_builder() {
        let builder = NetworkSystemBuilder::new();
        
        // Create a test node
        let node = NetworkNode {
            id: uuid::Uuid::new_v4(),
            name: "gateway".to_string(),
            node_type: NodeType::Gateway,
            interfaces: vec![],
            services: vec!["dhcp".to_string()],
            metadata: HashMap::new(),
        };
        
        // Create a simple topology
        let topology = NetworkTopology::new(
            uuid::Uuid::new_v4(),
            "test-network".to_string(),
            vec![node.clone()],
            vec![],
        );
        
        let config = builder.build_system_for_node(&node, &topology).await.unwrap();
        
        assert_eq!(config.hostname, "gateway");
        assert_eq!(config.node_id, node.id);
        assert!(config.firewall_config.is_some());
    }
}