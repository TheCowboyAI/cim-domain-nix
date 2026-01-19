// Copyright 2025 Cowboy AI, LLC.

//! Nix File Reader
//!
//! Reads Nix files and parses them into domain objects.

use std::path::Path;
use std::fs;
use super::{IoError, Result};
use crate::nix::*;
use crate::nix::topology::*;

// ============================================================================
// Reader Traits
// ============================================================================

/// Generic Nix file reader
pub trait NixReader {
    /// The type of object this reader produces
    type Output;

    /// Read from a file path
    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::Output>;

    /// Read from a string
    fn read_string(&self, content: &str) -> Result<Self::Output>;
}

// ============================================================================
// Topology Reader
// ============================================================================

/// Reader for nix-topology files
///
/// Parses nix-topology format files into `NixTopology` objects.
///
/// # Format
///
/// The reader expects nix-topology format:
///
/// ```nix
/// {
///   nodes = {
///     server01 = {
///       type = "physical";
///       system = "x86_64-linux";
///       hardware = {
///         cpu_cores = 8;
///         memory_mb = 16384;
///       };
///     };
///   };
///   networks = {
///     lan = {
///       type = "lan";
///       cidr_v4 = "192.168.1.0/24";
///     };
///   };
/// }
/// ```
pub struct TopologyReader {
    parser: NixParser,
}

impl TopologyReader {
    /// Create a new topology reader
    pub fn new() -> Self {
        Self {
            parser: NixParser::new(),
        }
    }

    /// Parse a topology from Nix AST
    fn parse_topology_from_ast(&self, ast: &NixValue, name: String) -> Result<NixTopology> {
        let mut topology = NixTopology::new(name);

        // Extract attrset
        let attrs = match ast {
            NixValue::Attrset(attrs) => attrs,
            _ => return Err(IoError::ParseError("Expected attrset at root".to_string())),
        };

        // Parse nodes
        if let Some(NixValue::Attrset(nodes)) = attrs.get("nodes") {
            for (node_name, node_value) in nodes.attributes.iter() {
                if let Some(node) = self.parse_node(node_name, node_value)? {
                    topology.add_node(node);
                }
            }
        }

        // Parse networks
        if let Some(NixValue::Attrset(networks)) = attrs.get("networks") {
            for (network_name, network_value) in networks.attributes.iter() {
                if let Some(network) = self.parse_network(network_name, network_value)? {
                    topology.add_network(network);
                }
            }
        }

        // Parse connections
        if let Some(NixValue::List(connections)) = attrs.get("connections") {
            for conn_value in connections.elements.iter() {
                if let Some(connection) = self.parse_connection(conn_value)? {
                    topology.add_connection(connection);
                }
            }
        }

        Ok(topology)
    }

    /// Parse a node from Nix value
    fn parse_node(&self, name: &str, value: &NixValue) -> Result<Option<TopologyNode>> {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => return Ok(None),
        };

        // Get node type
        let node_type = if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "physical" => TopologyNodeType::PhysicalServer,
                "vm" => TopologyNodeType::VirtualMachine,
                "container" => TopologyNodeType::Container,
                "network-device" => TopologyNodeType::NetworkDevice,
                _ => TopologyNodeType::PhysicalServer, // Default
            }
        } else {
            TopologyNodeType::PhysicalServer
        };

        // Get system
        let system = if let Some(NixValue::String(s)) = attrs.get("system") {
            s.value.clone()
        } else {
            "x86_64-linux".to_string() // Default
        };

        let mut node = TopologyNode::new(name.to_string(), node_type, system);

        // Parse hardware config
        if let Some(NixValue::Attrset(hw)) = attrs.get("hardware") {
            let mut hw_config = HardwareConfig::new();

            if let Some(NixValue::Integer(cores)) = hw.get("cpu_cores") {
                hw_config.cpu_cores = Some(cores.value as u32);
            }

            if let Some(NixValue::Integer(mem)) = hw.get("memory_mb") {
                hw_config.memory_mb = Some(mem.value as u64);
            }

            if let Some(NixValue::Integer(storage)) = hw.get("storage_gb") {
                hw_config.storage_gb = Some(storage.value as u64);
            }

            node.hardware = Some(hw_config);
        }

        // Parse interfaces
        if let Some(NixValue::List(interfaces)) = attrs.get("interfaces") {
            for iface_value in interfaces.elements.iter() {
                if let Some(iface) = self.parse_interface(iface_value)? {
                    node.add_interface(iface);
                }
            }
        }

        Ok(Some(node))
    }

    /// Parse a network from Nix value
    fn parse_network(&self, name: &str, value: &NixValue) -> Result<Option<TopologyNetwork>> {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => return Ok(None),
        };

        // Get network type
        let network_type = if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "lan" => NetworkType::LAN,
                "wan" => NetworkType::WAN,
                "vlan" => NetworkType::VLAN,
                "vpn" => NetworkType::VPN,
                "management" => NetworkType::Management,
                _ => NetworkType::LAN,
            }
        } else {
            NetworkType::LAN
        };

        let mut network = TopologyNetwork::new(name.to_string(), network_type);

        // Get CIDR ranges
        if let Some(NixValue::String(s)) = attrs.get("cidr_v4") {
            network = network.with_cidr_v4(s.value.clone());
        }

        if let Some(NixValue::String(s)) = attrs.get("cidr_v6") {
            network = network.with_cidr_v6(s.value.clone());
        }

        Ok(Some(network))
    }

    /// Parse an interface from Nix value
    fn parse_interface(&self, value: &NixValue) -> Result<Option<NodeInterface>> {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => return Ok(None),
        };

        let name = if let Some(NixValue::String(s)) = attrs.get("name") {
            s.value.clone()
        } else {
            return Ok(None);
        };

        let mut interface = NodeInterface::new(name);

        if let Some(NixValue::String(s)) = attrs.get("network") {
            interface.network = Some(s.value.clone());
        }

        if let Some(NixValue::String(s)) = attrs.get("ip_address") {
            interface.ip_address = Some(s.value.clone());
        }

        if let Some(NixValue::String(s)) = attrs.get("mac_address") {
            interface.mac_address = Some(s.value.clone());
        }

        Ok(Some(interface))
    }

    /// Parse a connection from Nix value
    fn parse_connection(&self, value: &NixValue) -> Result<Option<TopologyConnection>> {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => return Ok(None),
        };

        let from_node = if let Some(NixValue::String(s)) = attrs.get("from_node") {
            s.value.clone()
        } else {
            return Ok(None);
        };

        let from_interface = if let Some(NixValue::String(s)) = attrs.get("from_interface") {
            s.value.clone()
        } else {
            return Ok(None);
        };

        let to_node = if let Some(NixValue::String(s)) = attrs.get("to_node") {
            s.value.clone()
        } else {
            return Ok(None);
        };

        let to_interface = if let Some(NixValue::String(s)) = attrs.get("to_interface") {
            s.value.clone()
        } else {
            return Ok(None);
        };

        let connection_type = if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "ethernet" => ConnectionType::Ethernet,
                "bridge" => ConnectionType::Bridge,
                "wireless" => ConnectionType::Wireless,
                "vpn" => ConnectionType::VPN,
                _ => ConnectionType::Ethernet,
            }
        } else {
            ConnectionType::Ethernet
        };

        let connection = TopologyConnection::new(
            from_node,
            from_interface,
            to_node,
            to_interface,
            connection_type,
        );

        Ok(Some(connection))
    }
}

impl Default for TopologyReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TopologyReader {
    /// Read topology from a NixValue
    pub fn read_from_value(&self, value: &NixValue, name: String) -> Result<NixTopology> {
        self.parse_topology_from_ast(value, name)
    }
}

impl NixReader for TopologyReader {
    type Output = NixTopology;

    fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Self::Output> {
        let path_ref = path.as_ref();

        if !path_ref.exists() {
            return Err(IoError::FileNotFound(
                path_ref.display().to_string()
            ));
        }

        let content = fs::read_to_string(path_ref)?;
        self.read_string(&content)
    }

    fn read_string(&self, content: &str) -> Result<Self::Output> {
        // Parse to AST
        let ast = self.parser.parse_str(content)
            .map_err(|e| IoError::ParseError(format!("Failed to parse Nix: {}", e)))?;

        // Convert AST to NixValue
        let value = crate::nix::ast_to_value(&ast)
            .map_err(|e| IoError::ParseError(format!("Failed to convert AST: {}", e)))?;

        // Extract topology name from file or use default
        let name = "imported-topology".to_string();

        // Parse NixValue to topology
        self.parse_topology_from_ast(&value, name)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_empty_topology() {
        let reader = TopologyReader::new();
        let value = NixValue::Attrset(NixAttrset::new());
        let result = reader.read_from_value(&value, "test".to_string());
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.nodes.len(), 0);
        assert_eq!(topology.networks.len(), 0);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let reader = TopologyReader::new();
        let result = reader.read_file("/nonexistent/file.nix");
        assert!(matches!(result, Err(IoError::FileNotFound(_))));
    }

    #[test]
    fn test_string_parsing_empty() {
        let reader = TopologyReader::new();
        let result = reader.read_string("{ }");
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.nodes.len(), 0);
        assert_eq!(topology.networks.len(), 0);
    }

    #[test]
    fn test_string_parsing_with_node() {
        let reader = TopologyReader::new();
        let content = r#"{
            nodes = {
                server01 = {
                    type = "physical";
                    system = "x86_64-linux";
                };
            };
        }"#;

        let result = reader.read_string(content);
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.nodes.len(), 1);
        assert!(topology.nodes.contains_key("server01"));
    }

    #[test]
    fn test_string_parsing_with_network() {
        let reader = TopologyReader::new();
        let content = r#"{
            networks = {
                lan = {
                    type = "lan";
                    cidr_v4 = "192.168.1.0/24";
                };
            };
        }"#;

        let result = reader.read_string(content);
        assert!(result.is_ok());

        let topology = result.unwrap();
        assert_eq!(topology.networks.len(), 1);
        assert!(topology.networks.contains_key("lan"));
    }
}
