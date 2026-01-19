// Copyright (c) 2025 - Cowboy AI, Inc.
//! Topology Writer Adapter: Infrastructure Resources → nixos-topology
//!
//! This adapter takes Infrastructure domain resources and generates
//! nixos-topology configuration files.
//!
//! ## Architecture
//!
//! ```text
//! ComputeResource entities
//!     │
//!     ▼ (functors)
//! TopologyWriter
//!     │
//!     ▼ (Nix codegen)
//! topology.nix files
//!     │
//!     ▼ (git commit)
//! Version Control
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cim_domain_nix::adapters::topology_writer::TopologyWriter;
//! use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut writer = TopologyWriter::new(Path::new("topology.nix"));
//!
//! // Add a resource
//! let hostname = Hostname::new("router01")?;
//! let resource = ComputeResource::new(hostname, ResourceType::Router)?;
//! writer.add_node(&resource)?;
//!
//! // Generate Nix code
//! let nix_code = writer.generate_topology()?;
//! println!("{}", nix_code);
//!
//! // Write to file
//! writer.write_to_file().await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use cim_infrastructure::{ComputeResource, ResourceType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::functors::resource_type_functor::*;

/// Topology Writer - Generates nixos-topology files from Infrastructure resources
///
/// ## Responsibilities
///
/// 1. **Collect Resources**: Maintain in-memory collection of resources
/// 2. **Map Types**: Use ResourceType functor for type mapping
/// 3. **Generate Nix**: Produce valid nixos-topology Nix code
/// 4. **Write Files**: Update topology.nix and related files
///
/// ## Example
///
/// ```rust,no_run
/// use cim_domain_nix::adapters::topology_writer::TopologyWriter;
/// use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let mut writer = TopologyWriter::new(Path::new("topology.nix"));
///
/// // Add multiple resources
/// let router = ComputeResource::new(
///     Hostname::new("router01")?,
///     ResourceType::Router
/// )?;
/// writer.add_node(&router)?;
///
/// let switch = ComputeResource::new(
///     Hostname::new("switch01")?,
///     ResourceType::Switch
/// )?;
/// writer.add_node(&switch)?;
///
/// // Generate and write
/// writer.write_to_file().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TopologyWriter {
    /// Path to topology.nix file
    output_path: PathBuf,

    /// Collection of nodes to write
    nodes: HashMap<String, ComputeResource>,

    /// Topology name
    topology_name: String,
}

impl TopologyWriter {
    /// Create a new topology writer
    ///
    /// ## Arguments
    ///
    /// * `output_path` - Path where topology.nix will be written
    ///
    /// ## Returns
    ///
    /// New `TopologyWriter` instance
    pub fn new(output_path: impl Into<PathBuf>) -> Self {
        Self {
            output_path: output_path.into(),
            nodes: HashMap::new(),
            topology_name: "infrastructure".to_string(),
        }
    }

    /// Create a topology writer with a custom topology name
    ///
    /// ## Arguments
    ///
    /// * `output_path` - Path where topology.nix will be written
    /// * `topology_name` - Name for the topology
    pub fn with_name(output_path: impl Into<PathBuf>, topology_name: impl Into<String>) -> Self {
        Self {
            output_path: output_path.into(),
            nodes: HashMap::new(),
            topology_name: topology_name.into(),
        }
    }

    /// Add a node to the topology
    ///
    /// ## Arguments
    ///
    /// * `resource` - ComputeResource to add as a topology node
    ///
    /// ## Returns
    ///
    /// Result indicating success or error
    ///
    /// ## Example
    ///
    /// ```rust
    /// use cim_domain_nix::adapters::topology_writer::TopologyWriter;
    /// use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
    /// use std::path::Path;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let mut writer = TopologyWriter::new(Path::new("topology.nix"));
    ///
    /// let router = ComputeResource::new(
    ///     Hostname::new("router01")?,
    ///     ResourceType::Router
    /// )?;
    /// writer.add_node(&router)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_node(&mut self, resource: &ComputeResource) -> Result<()> {
        let node_name = resource.hostname.short_name().to_string();
        self.nodes.insert(node_name, resource.clone());
        Ok(())
    }

    /// Remove a node from the topology
    ///
    /// ## Arguments
    ///
    /// * `hostname` - Short hostname of the node to remove
    pub fn remove_node(&mut self, hostname: &str) -> Option<ComputeResource> {
        self.nodes.remove(hostname)
    }

    /// Update an existing node
    ///
    /// ## Arguments
    ///
    /// * `resource` - Updated ComputeResource
    pub fn update_node(&mut self, resource: &ComputeResource) -> Result<()> {
        self.add_node(resource)
    }

    /// Generate nixos-topology Nix code
    ///
    /// ## Returns
    ///
    /// String containing valid Nix code for topology.nix
    ///
    /// ## Example
    ///
    /// ```rust
    /// use cim_domain_nix::adapters::topology_writer::TopologyWriter;
    /// use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
    /// use std::path::Path;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// let mut writer = TopologyWriter::new(Path::new("topology.nix"));
    ///
    /// let router = ComputeResource::new(
    ///     Hostname::new("router01")?,
    ///     ResourceType::Router
    /// )?;
    /// writer.add_node(&router)?;
    ///
    /// let nix_code = writer.generate_topology()?;
    /// assert!(nix_code.contains("router01"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_topology(&self) -> Result<String> {
        let mut output = String::new();

        // File header
        output.push_str("# nixos-topology configuration\n");
        output.push_str("# Generated by cim-domain-nix\n");
        output.push_str("# DO NOT EDIT MANUALLY - Changes will be overwritten\n\n");

        output.push_str("{\n");
        output.push_str(&format!("  # Topology: {}\n\n", self.topology_name));

        // Nodes section
        output.push_str("  nodes = {\n");

        let mut node_names: Vec<_> = self.nodes.keys().collect();
        node_names.sort();

        for node_name in node_names {
            let resource = &self.nodes[node_name];
            let node_nix = self.generate_node_nix(resource)?;
            output.push_str(&node_nix);
            output.push('\n');
        }

        output.push_str("  };\n");

        // Networks section (placeholder)
        output.push_str("\n  networks = {\n");
        output.push_str("    # Networks will be added here\n");
        output.push_str("  };\n");

        // Connections section (placeholder)
        output.push_str("\n  connections = [\n");
        output.push_str("    # Connections will be added here\n");
        output.push_str("  ];\n");

        output.push_str("}\n");

        Ok(output)
    }

    /// Generate Nix code for a single node
    ///
    /// ## Arguments
    ///
    /// * `resource` - ComputeResource to convert to Nix
    ///
    /// ## Returns
    ///
    /// String containing Nix code for this node
    fn generate_node_nix(&self, resource: &ComputeResource) -> Result<String> {
        let node_name = resource.hostname.short_name();

        // Map ResourceType to topology type string using functor
        let topology_type = map_resource_type_to_topology(resource.resource_type);
        let type_str = self.topology_type_to_nix_string(topology_type);

        let mut output = String::new();
        output.push_str(&format!("    {} = {{\n", node_name));
        output.push_str(&format!("      type = \"{}\";\n", type_str));
        output.push_str(&format!("      hostname = \"{}\";\n", resource.hostname.as_str()));

        // Add hardware info if available
        if let Some(ref manufacturer) = resource.manufacturer {
            output.push_str(&format!("      manufacturer = \"{}\";\n", manufacturer));
        }
        if let Some(ref model) = resource.model {
            output.push_str(&format!("      model = \"{}\";\n", model));
        }
        if let Some(ref serial) = resource.serial_number {
            output.push_str(&format!("      serialNumber = \"{}\";\n", serial));
        }

        // Add metadata
        if !resource.metadata.is_empty() {
            output.push_str("      metadata = {\n");
            let mut keys: Vec<_> = resource.metadata.keys().collect();
            keys.sort();
            for key in keys {
                let value = &resource.metadata[key];
                output.push_str(&format!("        {} = \"{}\";\n", key, value));
            }
            output.push_str("      };\n");
        }

        output.push_str("    };");

        Ok(output)
    }

    /// Convert TopologyNodeType to Nix string
    fn topology_type_to_nix_string(&self, node_type: TopologyNodeType) -> &'static str {
        match node_type {
            TopologyNodeType::PhysicalServer => "physical-server",
            TopologyNodeType::VirtualMachine => "virtual-machine",
            TopologyNodeType::Container => "container",
            TopologyNodeType::Router => "router",
            TopologyNodeType::Switch => "switch",
            TopologyNodeType::Firewall => "firewall",
            TopologyNodeType::LoadBalancer => "load-balancer",
            TopologyNodeType::Storage => "storage",
            TopologyNodeType::Device => "device",
        }
    }

    /// Write topology to file
    ///
    /// ## Returns
    ///
    /// Result indicating success or error
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use cim_domain_nix::adapters::topology_writer::TopologyWriter;
    /// use cim_infrastructure::{ComputeResource, Hostname, ResourceType};
    /// use std::path::Path;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let mut writer = TopologyWriter::new(Path::new("topology.nix"));
    ///
    /// let router = ComputeResource::new(
    ///     Hostname::new("router01")?,
    ///     ResourceType::Router
    /// )?;
    /// writer.add_node(&router)?;
    ///
    /// writer.write_to_file().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write_to_file(&self) -> Result<()> {
        let nix_code = self.generate_topology()?;

        // Ensure parent directory exists
        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        // Write file
        fs::write(&self.output_path, nix_code)
            .await
            .context(format!(
                "Failed to write topology file: {}",
                self.output_path.display()
            ))?;

        Ok(())
    }

    /// Get the number of nodes in the topology
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a node exists
    pub fn has_node(&self, hostname: &str) -> bool {
        self.nodes.contains_key(hostname)
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cim_infrastructure::Hostname;

    #[test]
    fn test_create_writer() {
        let writer = TopologyWriter::new("test.nix");
        assert_eq!(writer.node_count(), 0);
        assert_eq!(writer.topology_name, "infrastructure");
    }

    #[test]
    fn test_add_node() {
        let mut writer = TopologyWriter::new("test.nix");

        let hostname = Hostname::new("router01").unwrap();
        let resource = ComputeResource::new(hostname, ResourceType::Router).unwrap();

        writer.add_node(&resource).unwrap();
        assert_eq!(writer.node_count(), 1);
        assert!(writer.has_node("router01"));
    }

    #[test]
    fn test_remove_node() {
        let mut writer = TopologyWriter::new("test.nix");

        let hostname = Hostname::new("router01").unwrap();
        let resource = ComputeResource::new(hostname, ResourceType::Router).unwrap();

        writer.add_node(&resource).unwrap();
        assert_eq!(writer.node_count(), 1);

        let removed = writer.remove_node("router01");
        assert!(removed.is_some());
        assert_eq!(writer.node_count(), 0);
    }

    #[test]
    fn test_generate_topology_empty() {
        let writer = TopologyWriter::new("test.nix");
        let nix_code = writer.generate_topology().unwrap();

        assert!(nix_code.contains("nodes = {"));
        assert!(nix_code.contains("networks = {"));
        assert!(nix_code.contains("connections = ["));
    }

    #[test]
    fn test_generate_topology_with_router() {
        let mut writer = TopologyWriter::new("test.nix");

        let hostname = Hostname::new("router01").unwrap();
        let resource = ComputeResource::new(hostname, ResourceType::Router).unwrap();
        writer.add_node(&resource).unwrap();

        let nix_code = writer.generate_topology().unwrap();

        assert!(nix_code.contains("router01"));
        assert!(nix_code.contains("type = \"router\""));
        assert!(nix_code.contains("hostname = \"router01\""));
    }

    #[test]
    fn test_generate_topology_with_multiple_nodes() {
        let mut writer = TopologyWriter::new("test.nix");

        // Add router
        let router = ComputeResource::new(
            Hostname::new("router01").unwrap(),
            ResourceType::Router,
        )
        .unwrap();
        writer.add_node(&router).unwrap();

        // Add switch
        let switch = ComputeResource::new(
            Hostname::new("switch01").unwrap(),
            ResourceType::Switch,
        )
        .unwrap();
        writer.add_node(&switch).unwrap();

        // Add camera (maps to Device)
        let camera = ComputeResource::new(
            Hostname::new("camera01").unwrap(),
            ResourceType::Camera,
        )
        .unwrap();
        writer.add_node(&camera).unwrap();

        let nix_code = writer.generate_topology().unwrap();

        assert!(nix_code.contains("router01"));
        assert!(nix_code.contains("switch01"));
        assert!(nix_code.contains("camera01"));
        assert!(nix_code.contains("type = \"router\""));
        assert!(nix_code.contains("type = \"switch\""));
        assert!(nix_code.contains("type = \"device\"")); // Camera maps to Device
    }

    #[test]
    fn test_generate_node_with_hardware_info() {
        let mut writer = TopologyWriter::new("test.nix");

        let hostname = Hostname::new("router01").unwrap();
        let mut resource = ComputeResource::new(hostname, ResourceType::Router).unwrap();
        resource.set_hardware(
            Some("Cisco".to_string()),
            Some("ASR 1001-X".to_string()),
            Some("ABC123".to_string()),
        );

        writer.add_node(&resource).unwrap();

        let nix_code = writer.generate_topology().unwrap();

        assert!(nix_code.contains("manufacturer = \"Cisco\""));
        assert!(nix_code.contains("model = \"ASR 1001-X\""));
        assert!(nix_code.contains("serialNumber = \"ABC123\""));
    }

    #[test]
    fn test_generate_node_with_metadata() {
        let mut writer = TopologyWriter::new("test.nix");

        let hostname = Hostname::new("router01").unwrap();
        let mut resource = ComputeResource::new(hostname, ResourceType::Router).unwrap();
        resource.add_metadata("rack", "A01").unwrap();
        resource.add_metadata("row", "1").unwrap();

        writer.add_node(&resource).unwrap();

        let nix_code = writer.generate_topology().unwrap();

        assert!(nix_code.contains("metadata = {"));
        assert!(nix_code.contains("rack = \"A01\""));
        assert!(nix_code.contains("row = \"1\""));
    }

    #[test]
    fn test_functor_integration() {
        let mut writer = TopologyWriter::new("test.nix");

        // Test all major resource types
        let types = vec![
            (ResourceType::PhysicalServer, "physical-server"),
            (ResourceType::Router, "router"),
            (ResourceType::Switch, "switch"),
            (ResourceType::Camera, "device"), // Maps to Device
            (ResourceType::KVM, "device"),    // Maps to Device
        ];

        for (i, (resource_type, expected_nix_type)) in types.iter().enumerate() {
            let hostname = Hostname::new(&format!("node{:02}", i)).unwrap();
            let resource = ComputeResource::new(hostname, *resource_type).unwrap();
            writer.add_node(&resource).unwrap();

            let nix_code = writer.generate_topology().unwrap();
            assert!(
                nix_code.contains(&format!("type = \"{}\"", expected_nix_type)),
                "Failed for {:?} -> {}",
                resource_type,
                expected_nix_type
            );

            writer.clear();
        }
    }
}
