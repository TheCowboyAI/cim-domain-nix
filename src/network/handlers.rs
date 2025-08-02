//! Handlers for network-related system operations

use super::InterfaceChangeEvent;
use crate::{Result, NixDomainError};
use crate::value_objects::MessageIdentity;
use crate::commands::ActivateConfiguration;
use crate::events::ActivationType;
use std::collections::HashMap;

/// Handler for network-based system operations
pub struct NetworkSystemHandler {
    /// Mapping of node IDs to system configuration IDs
    node_to_system: HashMap<uuid::Uuid, uuid::Uuid>,
    /// Cache of active configurations
    active_configs: HashMap<uuid::Uuid, super::SystemConfiguration>,
}

impl NetworkSystemHandler {
    /// Create a new network system handler
    pub fn new() -> Self {
        Self {
            node_to_system: HashMap::new(),
            active_configs: HashMap::new(),
        }
    }
    
    /// Find systems affected by an interface change
    pub fn find_affected_systems(&self, event: &InterfaceChangeEvent) -> Result<Vec<uuid::Uuid>> {
        let mut affected = Vec::new();
        
        // Find system for the node with the changed interface
        if let Some(&system_id) = self.node_to_system.get(&event.node_id) {
            affected.push(system_id);
        }
        
        // In a real implementation, we might also find systems that depend on this interface
        // For example, if this is a gateway interface, all clients might be affected
        
        Ok(affected)
    }
    
    /// Update network configuration for a system
    pub async fn update_network_config(
        &mut self,
        system_id: uuid::Uuid,
        event: &InterfaceChangeEvent,
    ) -> Result<()> {
        let config = self.active_configs.get_mut(&system_id)
            .ok_or_else(|| NixDomainError::Other(format!("System {} not found", system_id)))?;
        
        match event.change_type {
            super::InterfaceChangeType::Added => {
                // Add new interface configuration
                let interface_config = super::InterfaceConfig {
                    name: event.interface.name.clone(),
                    mac_address: event.interface.mac_address.clone(),
                    addresses: event.interface.static_addresses()
                        .iter()
                        .filter_map(|a| a.to_cidr())
                        .collect(),
                    dhcp: event.interface.uses_dhcp(),
                    mtu: event.interface.mtu,
                };
                
                config.network_config.interfaces.push(interface_config);
            }
            super::InterfaceChangeType::Removed => {
                // Remove interface configuration
                config.network_config.interfaces.retain(|i| i.name != event.interface.name);
            }
            super::InterfaceChangeType::Updated => {
                // Update existing interface
                if let Some(interface) = config.network_config.interfaces
                    .iter_mut()
                    .find(|i| i.name == event.interface.name) {
                    
                    interface.addresses = event.interface.static_addresses()
                        .iter()
                        .filter_map(|a| a.to_cidr())
                        .collect();
                    interface.dhcp = event.interface.uses_dhcp();
                    interface.mtu = event.interface.mtu;
                }
            }
        }
        
        Ok(())
    }
    
    /// Register a new system configuration
    pub fn register_system(&mut self, node_id: uuid::Uuid, config: super::SystemConfiguration) {
        self.node_to_system.insert(node_id, config.system_id);
        self.active_configs.insert(config.system_id, config);
    }
    
    /// Get configuration for a node
    pub fn get_node_config(&self, node_id: uuid::Uuid) -> Option<&super::SystemConfiguration> {
        self.node_to_system.get(&node_id)
            .and_then(|system_id| self.active_configs.get(system_id))
    }
    
    /// Generate NixOS module for network configuration
    pub fn generate_network_module(&self, config: &super::NetworkConfig) -> String {
        let mut module = String::from("{ config, pkgs, ... }:\n\n{\n");
        
        // Network interfaces
        module.push_str("  networking = {\n");
        
        // Generate interface configurations
        if !config.interfaces.is_empty() {
            module.push_str("    interfaces = {\n");
            
            for interface in &config.interfaces {
                module.push_str(&format!("      {} = {{\n", interface.name));
                
                if interface.dhcp {
                    module.push_str("        useDHCP = true;\n");
                } else if !interface.addresses.is_empty() {
                    module.push_str("        ipv4.addresses = [\n");
                    for addr in &interface.addresses {
                        // Parse address and prefix
                        if let Some((ip, prefix)) = addr.split_once('/') {
                            module.push_str(&format!(
                                "          {{ address = \"{}\"; prefixLength = {}; }}\n",
                                ip, prefix
                            ));
                        }
                    }
                    module.push_str("        ];\n");
                }
                
                if let Some(mtu) = interface.mtu {
                    module.push_str(&format!("        mtu = {};\n", mtu));
                }
                
                module.push_str("      };\n");
            }
            
            module.push_str("    };\n");
        }
        
        // Routes
        if !config.routes.is_empty() {
            module.push_str("    routes = [\n");
            
            for route in &config.routes {
                module.push_str("      {\n");
                module.push_str(&format!("        address = \"{}\";\n", route.destination));
                if let Some(gw) = &route.gateway {
                    module.push_str(&format!("        via = \"{}\";\n", gw));
                }
                module.push_str(&format!("        interface = \"{}\";\n", route.interface));
                module.push_str("      }\n");
            }
            
            module.push_str("    ];\n");
        }
        
        // DNS
        if let Some(dns) = &config.dns {
            if !dns.nameservers.is_empty() {
                module.push_str(&format!("    nameservers = [ {} ];\n",
                    dns.nameservers.iter()
                        .map(|ns| format!("\"{}\"", ns))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
            
            if !dns.search_domains.is_empty() {
                module.push_str(&format!("    search = [ {} ];\n",
                    dns.search_domains.iter()
                        .map(|d| format!("\"{}\"", d))
                        .collect::<Vec<_>>()
                        .join(" ")
                ));
            }
        }
        
        module.push_str("  };\n");
        module.push_str("}\n");
        
        module
    }
    
    /// Create activation command for a system
    pub fn create_activation_command(
        &self,
        system_id: uuid::Uuid,
        identity: MessageIdentity,
    ) -> Result<ActivateConfiguration> {
        let config = self.active_configs.get(&system_id)
            .ok_or_else(|| NixDomainError::Other(format!("System {} not found", system_id)))?;
        
        Ok(ActivateConfiguration {
            identity,
            name: config.hostname.clone(),
            activation_type: ActivationType::Switch,
        })
    }
}

impl Default for NetworkSystemHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_module_generation() {
        let config = super::super::NetworkConfig {
            interfaces: vec![
                super::super::InterfaceConfig {
                    name: "eth0".to_string(),
                    mac_address: None,
                    addresses: vec!["192.168.1.10/24".to_string()],
                    dhcp: false,
                    mtu: Some(1500),
                },
            ],
            routes: vec![
                super::super::RouteConfig {
                    destination: "0.0.0.0/0".to_string(),
                    gateway: Some("192.168.1.1".to_string()),
                    interface: "eth0".to_string(),
                },
            ],
            dns: Some(super::super::DnsConfig {
                nameservers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
                search_domains: vec!["example.com".to_string()],
            }),
        };
        
        let handler = NetworkSystemHandler::new();
        let module = handler.generate_network_module(&config);
        
        assert!(module.contains("interfaces"));
        assert!(module.contains("eth0"));
        assert!(module.contains("192.168.1.10"));
        assert!(module.contains("routes"));
        assert!(module.contains("nameservers"));
    }
}