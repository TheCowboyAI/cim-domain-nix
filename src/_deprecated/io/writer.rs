// Copyright 2025 Cowboy AI, LLC.

//! Nix File Writer
//!
//! Serializes domain objects to Nix files.

use std::path::Path;
use std::fs;
use super::Result;
use crate::nix::topology::*;

// ============================================================================
// Writer Traits
// ============================================================================

/// Generic Nix file writer
pub trait NixWriter {
    /// The type of object this writer accepts
    type Input;

    /// Write to a file path
    fn write_file<P: AsRef<Path>>(&self, input: &Self::Input, path: P) -> Result<()>;

    /// Serialize to string
    fn write_string(&self, input: &Self::Input) -> Result<String>;
}

// ============================================================================
// Topology Writer
// ============================================================================

/// Writer for nix-topology files
///
/// Serializes `NixTopology` objects to nix-topology format.
///
/// # Output Format
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
pub struct TopologyWriter {
    indent_size: usize,
}

impl TopologyWriter {
    /// Create a new topology writer with default 2-space indentation
    pub fn new() -> Self {
        Self { indent_size: 2 }
    }

    /// Create a writer with custom indentation
    pub fn with_indent(indent_size: usize) -> Self {
        Self { indent_size }
    }

    /// Generate indent string for given level
    fn indent(&self, level: usize) -> String {
        " ".repeat(self.indent_size * level)
    }

    /// Serialize a node to Nix string
    fn serialize_node(&self, node: &TopologyNode, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = self.indent(indent_level);
        let inner_indent = self.indent(indent_level + 1);

        output.push_str(&format!("{}{} = {{\n", indent, node.name));

        // Node type
        let type_str = match node.node_type {
            TopologyNodeType::PhysicalServer => "physical",
            TopologyNodeType::VirtualMachine => "vm",
            TopologyNodeType::Container => "container",
            TopologyNodeType::NetworkDevice => "network-device",
        };
        output.push_str(&format!("{}type = \"{}\";\n", inner_indent, type_str));

        // System
        output.push_str(&format!("{}system = \"{}\";\n", inner_indent, node.system));

        // Hardware config
        if let Some(ref hw) = node.hardware {
            output.push_str(&format!("{}hardware = {{\n", inner_indent));
            let hw_indent = self.indent(indent_level + 2);

            if let Some(cores) = hw.cpu_cores {
                output.push_str(&format!("{}cpu_cores = {};\n", hw_indent, cores));
            }
            if let Some(mem) = hw.memory_mb {
                output.push_str(&format!("{}memory_mb = {};\n", hw_indent, mem));
            }
            if let Some(storage) = hw.storage_gb {
                output.push_str(&format!("{}storage_gb = {};\n", hw_indent, storage));
            }

            output.push_str(&format!("{}}};\n", inner_indent));
        }

        // Interfaces
        if !node.interfaces.is_empty() {
            output.push_str(&format!("{}interfaces = [\n", inner_indent));
            for interface in &node.interfaces {
                output.push_str(&self.serialize_interface(interface, indent_level + 2));
            }
            output.push_str(&format!("{}];\n", inner_indent));
        }

        // Services
        if !node.services.is_empty() {
            output.push_str(&format!("{}services = [\n", inner_indent));
            for service in &node.services {
                let service_indent = self.indent(indent_level + 2);
                output.push_str(&format!("{}\"{}\"\n", service_indent, service));
            }
            output.push_str(&format!("{}];\n", inner_indent));
        }

        output.push_str(&format!("{}}};\n", indent));
        output
    }

    /// Serialize a network to Nix string
    fn serialize_network(&self, network: &TopologyNetwork, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = self.indent(indent_level);
        let inner_indent = self.indent(indent_level + 1);

        output.push_str(&format!("{}{} = {{\n", indent, network.name));

        // Network type
        let type_str = match network.network_type {
            NetworkType::LAN => "lan",
            NetworkType::WAN => "wan",
            NetworkType::VLAN => "vlan",
            NetworkType::VPN => "vpn",
            NetworkType::Management => "management",
        };
        output.push_str(&format!("{}type = \"{}\";\n", inner_indent, type_str));

        // CIDR ranges
        if let Some(ref cidr_v4) = network.cidr_v4 {
            output.push_str(&format!("{}cidr_v4 = \"{}\";\n", inner_indent, cidr_v4));
        }

        if let Some(ref cidr_v6) = network.cidr_v6 {
            output.push_str(&format!("{}cidr_v6 = \"{}\";\n", inner_indent, cidr_v6));
        }

        output.push_str(&format!("{}}};\n", indent));
        output
    }

    /// Serialize an interface to Nix string
    fn serialize_interface(&self, interface: &NodeInterface, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = self.indent(indent_level);
        let inner_indent = self.indent(indent_level + 1);

        output.push_str(&format!("{}{{\n", indent));
        output.push_str(&format!("{}name = \"{}\";\n", inner_indent, interface.name));

        if let Some(ref network) = interface.network {
            output.push_str(&format!("{}network = \"{}\";\n", inner_indent, network));
        }

        if let Some(ref ip) = interface.ip_address {
            output.push_str(&format!("{}ip_address = \"{}\";\n", inner_indent, ip));
        }

        if let Some(ref mac) = interface.mac_address {
            output.push_str(&format!("{}mac_address = \"{}\";\n", inner_indent, mac));
        }

        output.push_str(&format!("{}}}\n", indent));
        output
    }

    /// Serialize a connection to Nix string
    fn serialize_connection(&self, connection: &TopologyConnection, indent_level: usize) -> String {
        let mut output = String::new();
        let indent = self.indent(indent_level);
        let inner_indent = self.indent(indent_level + 1);

        output.push_str(&format!("{}{{\n", indent));
        output.push_str(&format!("{}from_node = \"{}\";\n", inner_indent, connection.from_node));
        output.push_str(&format!("{}from_interface = \"{}\";\n", inner_indent, connection.from_interface));
        output.push_str(&format!("{}to_node = \"{}\";\n", inner_indent, connection.to_node));
        output.push_str(&format!("{}to_interface = \"{}\";\n", inner_indent, connection.to_interface));

        let type_str = match connection.connection_type {
            ConnectionType::Ethernet => "ethernet",
            ConnectionType::Bridge => "bridge",
            ConnectionType::Wireless => "wireless",
            ConnectionType::VPN => "vpn",
        };
        output.push_str(&format!("{}type = \"{}\";\n", inner_indent, type_str));

        output.push_str(&format!("{}}}\n", indent));
        output
    }
}

impl Default for TopologyWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl NixWriter for TopologyWriter {
    type Input = NixTopology;

    fn write_file<P: AsRef<Path>>(&self, topology: &Self::Input, path: P) -> Result<()> {
        let content = self.write_string(topology)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn write_string(&self, topology: &Self::Input) -> Result<String> {
        let mut output = String::from("{\n");

        // Write nodes
        if !topology.nodes.is_empty() {
            output.push_str("  nodes = {\n");
            for (_name, node) in &topology.nodes {
                output.push_str(&self.serialize_node(node, 2));
            }
            output.push_str("  };\n");
        }

        // Write networks
        if !topology.networks.is_empty() {
            output.push_str("  networks = {\n");
            for (_name, network) in &topology.networks {
                output.push_str(&self.serialize_network(network, 2));
            }
            output.push_str("  };\n");
        }

        // Write connections
        if !topology.connections.is_empty() {
            output.push_str("  connections = [\n");
            for connection in &topology.connections {
                output.push_str(&self.serialize_connection(connection, 2));
            }
            output.push_str("  ];\n");
        }

        output.push_str("}\n");
        Ok(output)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_empty_topology() {
        let writer = TopologyWriter::new();
        let topology = NixTopology::new("test".to_string());

        let result = writer.write_string(&topology);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("{"));
        assert!(output.contains("}"));
    }

    #[test]
    fn test_write_topology_with_node() {
        let writer = TopologyWriter::new();
        let mut topology = NixTopology::new("test".to_string());

        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = writer.write_string(&topology);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("nodes"));
        assert!(output.contains("server01"));
        assert!(output.contains("type = \"physical\""));
        assert!(output.contains("system = \"x86_64-linux\""));
    }

    #[test]
    fn test_write_topology_with_network() {
        let writer = TopologyWriter::new();
        let mut topology = NixTopology::new("test".to_string());

        let network = TopologyNetwork::new("lan".to_string(), NetworkType::LAN)
            .with_cidr_v4("192.168.1.0/24".to_string());
        topology.add_network(network);

        let result = writer.write_string(&topology);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("networks"));
        assert!(output.contains("lan"));
        assert!(output.contains("type = \"lan\""));
        assert!(output.contains("cidr_v4 = \"192.168.1.0/24\""));
    }

    #[test]
    fn test_write_node_with_hardware() {
        let writer = TopologyWriter::new();
        let mut topology = NixTopology::new("test".to_string());

        let mut node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );

        let mut hw = HardwareConfig::new();
        hw.cpu_cores = Some(8);
        hw.memory_mb = Some(16384);
        hw.storage_gb = Some(1000);
        node.hardware = Some(hw);

        topology.add_node(node);

        let output = writer.write_string(&topology).unwrap();
        assert!(output.contains("hardware"));
        assert!(output.contains("cpu_cores = 8"));
        assert!(output.contains("memory_mb = 16384"));
        assert!(output.contains("storage_gb = 1000"));
    }

    #[test]
    fn test_custom_indent() {
        let writer = TopologyWriter::with_indent(4);
        let mut topology = NixTopology::new("test".to_string());

        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let output = writer.write_string(&topology).unwrap();
        // Should have 4-space indents
        assert!(output.contains("    server01"));
    }

    #[test]
    fn test_write_connection() {
        let writer = TopologyWriter::new();
        let mut topology = NixTopology::new("test".to_string());

        let connection = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        );
        topology.add_connection(connection);

        let output = writer.write_string(&topology).unwrap();
        assert!(output.contains("connections"));
        assert!(output.contains("from_node = \"server01\""));
        assert!(output.contains("to_node = \"server02\""));
        assert!(output.contains("type = \"ethernet\""));
    }
}
