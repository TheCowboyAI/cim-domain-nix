// Copyright (c) 2025 - Cowboy AI, Inc.
//! Topology Reader Adapter: nixos-topology → Infrastructure Events
//!
//! This adapter reads nixos-topology configuration files and generates
//! Infrastructure domain events that can be published to NATS.
//!
//! ## Architecture
//!
//! ```text
//! topology.nix files
//!     │
//!     ▼ (rnix parser)
//! TopologyReader
//!     │
//!     ▼ (functors)
//! Vec<InfrastructureEvent>
//!     │
//!     ▼ (NATS publish)
//! Event Store (JetStream)
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use cim_domain_nix::adapters::topology_reader::TopologyReader;
//! use std::path::Path;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let reader = TopologyReader::new();
//! let events = reader.read_topology_file(Path::new("topology.nix")).await?;
//!
//! println!("Discovered {} resources", events.len());
//! for event in events {
//!     // Publish to NATS, apply to projections, etc.
//!     println!("Event: {:?}", event);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{bail, Context, Result};
use cim_infrastructure::{
    ComputeResource, Hostname, ResourceType,
};
use rnix::{Root, SyntaxKind, SyntaxNode};
use std::path::Path;
use tokio::fs;

use crate::functors::resource_type_functor::*;

/// Topology Reader - Reads nixos-topology files and generates Infrastructure resources
///
/// ## Responsibilities
///
/// 1. **Parse Nix Files**: Use rnix to parse topology.nix and related files
/// 2. **Extract Nodes**: Find all topology nodes (machines, routers, etc.)
/// 3. **Map Types**: Use ResourceType functor to map topology types
/// 4. **Generate Resources**: Create ComputeResource entities
///
/// ## Example
///
/// ```rust,no_run
/// use cim_domain_nix::adapters::topology_reader::TopologyReader;
/// use std::path::Path;
///
/// # async fn example() -> anyhow::Result<()> {
/// let reader = TopologyReader::new();
/// let resources = reader.read_topology_file(Path::new("./topology.nix")).await?;
///
/// for resource in resources {
///     println!("Found machine: {}", resource.hostname);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TopologyReader {
    /// Whether to strictly validate topology (fail on unknown types)
    strict_mode: bool,
}

impl TopologyReader {
    /// Create a new topology reader
    ///
    /// ## Arguments
    ///
    /// * None
    ///
    /// ## Returns
    ///
    /// New `TopologyReader` instance
    pub fn new() -> Self {
        Self {
            strict_mode: false,
        }
    }

    /// Create a topology reader with strict validation
    ///
    /// In strict mode, unknown topology node types will cause errors
    /// rather than mapping to generic `Appliance` type.
    pub fn new_strict() -> Self {
        Self {
            strict_mode: true,
        }
    }

    /// Read a topology file and generate Infrastructure resources
    ///
    /// ## Arguments
    ///
    /// * `path` - Path to topology.nix file
    ///
    /// ## Returns
    ///
    /// Vector of `ComputeResource` discovered from topology
    ///
    /// ## Errors
    ///
    /// - File not found
    /// - Parse errors (invalid Nix syntax)
    /// - Missing required topology attributes
    ///
    /// ## Example
    ///
    /// ```rust,no_run
    /// use cim_domain_nix::adapters::topology_reader::TopologyReader;
    /// use std::path::Path;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let reader = TopologyReader::new();
    /// let resources = reader.read_topology_file(Path::new("topology.nix")).await?;
    /// println!("Discovered {} resources", resources.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read_topology_file(&self, path: &Path) -> Result<Vec<ComputeResource>> {
        // Read file contents
        let content = fs::read_to_string(path)
            .await
            .context(format!("Failed to read topology file: {}", path.display()))?;

        self.parse_topology(&content)
            .context("Failed to parse topology")
    }

    /// Parse topology Nix content and generate resources
    ///
    /// ## Arguments
    ///
    /// * `content` - Nix file content as string
    ///
    /// ## Returns
    ///
    /// Vector of `ComputeResource`
    ///
    /// ## Implementation Notes
    ///
    /// Uses rnix to parse the Nix AST and extract topology nodes.
    /// Expected structure:
    /// ```nix
    /// {
    ///   nodes = {
    ///     router01 = {
    ///       type = "router";
    ///       hostname = "router01";
    ///       manufacturer = "Ubiquiti";
    ///       model = "UniFi Dream Machine Pro";
    ///       metadata = { ... };
    ///     };
    ///   };
    /// }
    /// ```
    pub fn parse_topology(&self, content: &str) -> Result<Vec<ComputeResource>> {
        // Parse Nix content with rnix
        let parsed = Root::parse(content);

        // Check for parse errors
        if !parsed.errors().is_empty() {
            let error_msgs: Vec<String> = parsed
                .errors()
                .iter()
                .map(|e| format!("{:?}", e))
                .collect();
            bail!("Nix parse errors: {}", error_msgs.join(", "));
        }

        let syntax = parsed.syntax();

        // Find the nodes attribute set
        let nodes_attrset = self.find_nodes_attrset(&syntax)
            .context("Failed to find 'nodes' attribute set in topology")?;

        // Extract all node entries
        let mut resources = Vec::new();

        for entry in self.extract_attrset_entries(&nodes_attrset) {
            match self.parse_node_entry(&entry) {
                Ok(resource) => resources.push(resource),
                Err(e) => {
                    if self.strict_mode {
                        return Err(e).context("Failed to parse node in strict mode");
                    } else {
                        // In lenient mode, log and skip
                        tracing::warn!("Skipping node due to parse error: {}", e);
                    }
                }
            }
        }

        Ok(resources)
    }

    /// Find the 'nodes' attribute set in the topology
    fn find_nodes_attrset(&self, syntax: &SyntaxNode) -> Result<SyntaxNode> {
        // Walk the AST to find: { nodes = { ... }; }
        for child in syntax.descendants() {
            if child.kind() == SyntaxKind::NODE_ATTR_SET {
                // Look for an attribute named "nodes"
                for entry_child in child.children() {
                    if entry_child.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                        // Check if this is the "nodes" key
                        if let Some(key_node) = entry_child.first_child() {
                            let key_text = key_node.text().to_string();
                            if key_text.trim() == "nodes" {
                                // Found it! Return the value (the attrset)
                                if let Some(value) = entry_child.last_child() {
                                    if value.kind() == SyntaxKind::NODE_ATTR_SET {
                                        return Ok(value);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        bail!("Could not find 'nodes' attribute set in topology file")
    }

    /// Extract all entries from an attribute set
    fn extract_attrset_entries(&self, attrset: &SyntaxNode) -> Vec<SyntaxNode> {
        attrset
            .children()
            .filter(|n| n.kind() == SyntaxKind::NODE_ATTRPATH_VALUE)
            .collect()
    }

    /// Parse a single node entry from the topology
    fn parse_node_entry(&self, entry: &SyntaxNode) -> Result<ComputeResource> {
        // Extract node name (key)
        let node_name = entry
            .first_child()
            .context("Node entry missing key")?
            .text()
            .to_string()
            .trim()
            .to_string();

        // Extract node attributes (value attrset)
        let node_attrs = entry
            .last_child()
            .context("Node entry missing value")?;

        if node_attrs.kind() != SyntaxKind::NODE_ATTR_SET {
            bail!("Node value is not an attribute set");
        }

        // Extract required attributes
        let node_type = self.extract_string_attr(&node_attrs, "type")
            .context("Missing required 'type' attribute")?;

        // Hostname defaults to node name if not specified
        let hostname_str = self.extract_string_attr(&node_attrs, "hostname")
            .unwrap_or_else(|_| node_name.clone());

        // Parse using existing parse_node method
        let mut resource = self.parse_node(&node_name, &node_type, "x86_64-linux")?;

        // Override hostname if explicitly specified
        if let Ok(explicit_hostname) = Hostname::new(&hostname_str) {
            resource.hostname = explicit_hostname;
        }

        // Extract optional hardware info
        if let Ok(manufacturer) = self.extract_string_attr(&node_attrs, "manufacturer") {
            let model = self.extract_string_attr(&node_attrs, "model").ok();
            let serial = self.extract_string_attr(&node_attrs, "serialNumber").ok();
            resource.set_hardware(Some(manufacturer), model, serial);
        }

        // Extract metadata if present
        if let Ok(metadata_node) = self.find_attr(&node_attrs, "metadata") {
            if let Some(metadata_attrset) = metadata_node.last_child() {
                if metadata_attrset.kind() == SyntaxKind::NODE_ATTR_SET {
                    for meta_entry in self.extract_attrset_entries(&metadata_attrset) {
                        if let (Ok(key), Ok(value)) = (
                            self.extract_key(&meta_entry),
                            self.extract_value_string(&meta_entry),
                        ) {
                            let _ = resource.add_metadata(&key, &value);
                        }
                    }
                }
            }
        }

        Ok(resource)
    }

    /// Find an attribute by name in an attribute set
    fn find_attr(&self, attrset: &SyntaxNode, name: &str) -> Result<SyntaxNode> {
        for entry in attrset.children() {
            if entry.kind() == SyntaxKind::NODE_ATTRPATH_VALUE {
                if let Some(key) = entry.first_child() {
                    let key_text = key.text().to_string();
                    if key_text.trim() == name {
                        return Ok(entry);
                    }
                }
            }
        }
        bail!("Attribute '{}' not found", name)
    }

    /// Extract a string attribute value
    fn extract_string_attr(&self, attrset: &SyntaxNode, name: &str) -> Result<String> {
        let entry = self.find_attr(attrset, name)?;
        self.extract_value_string(&entry)
    }

    /// Extract key from a key-value entry
    fn extract_key(&self, entry: &SyntaxNode) -> Result<String> {
        Ok(entry
            .first_child()
            .context("Missing key")?
            .text()
            .to_string()
            .trim()
            .to_string())
    }

    /// Extract string value from a key-value entry
    fn extract_value_string(&self, entry: &SyntaxNode) -> Result<String> {
        let value_node = entry.last_child().context("Missing value")?;

        // Handle different value types
        let text = value_node.text().to_string();

        // Remove quotes if present
        let trimmed = text.trim();
        if trimmed.starts_with('"') && trimmed.ends_with('"') {
            Ok(trimmed[1..trimmed.len() - 1].to_string())
        } else {
            Ok(trimmed.to_string())
        }
    }

    /// Parse a topology node and generate ComputeResource
    ///
    /// ## Arguments
    ///
    /// * `node_name` - Node name (hostname)
    /// * `node_type_str` - Node type from topology (e.g., "nixosConfigurations.router01")
    /// * `system` - System architecture (e.g., "x86_64-linux")
    ///
    /// ## Returns
    ///
    /// `ComputeResource` entity
    ///
    /// ## Example Mapping
    ///
    /// ```nix
    /// # In topology.nix:
    /// nodes.router01 = {
    ///   type = "router";
    ///   system = "x86_64-linux";
    /// };
    /// ```
    ///
    /// Maps to:
    /// ```rust,ignore
    /// ComputeResource {
    ///     hostname: Hostname::new("router01"),
    ///     resource_type: ResourceType::Router,
    ///     // ...
    /// }
    /// ```
    pub fn parse_node(
        &self,
        node_name: &str,
        node_type_str: &str,
        _system: &str,
    ) -> Result<ComputeResource> {
        // Parse node type string to TopologyNodeType
        let topology_type = self.parse_topology_type(node_type_str)?;

        // Map to ResourceType using functor
        let resource_type = map_topology_to_resource_type(topology_type);

        // Create hostname
        let hostname = Hostname::new(node_name)
            .context(format!("Invalid hostname: {}", node_name))?;

        // Create ComputeResource
        let resource = ComputeResource::new(hostname, resource_type)
            .context("Failed to create ComputeResource")?;

        Ok(resource)
    }

    /// Parse topology node type string to TopologyNodeType
    ///
    /// Maps common nixos-topology type strings to our enum.
    fn parse_topology_type(&self, type_str: &str) -> Result<TopologyNodeType> {
        let type_lower = type_str.to_lowercase();

        let topology_type = if type_lower.contains("server") || type_lower.contains("host") {
            TopologyNodeType::PhysicalServer
        } else if type_lower.contains("vm") || type_lower.contains("virtual") {
            TopologyNodeType::VirtualMachine
        } else if type_lower.contains("container") {
            TopologyNodeType::Container
        } else if type_lower.contains("router") {
            TopologyNodeType::Router
        } else if type_lower.contains("switch") {
            TopologyNodeType::Switch
        } else if type_lower.contains("firewall") {
            TopologyNodeType::Firewall
        } else if type_lower.contains("loadbalancer") || type_lower.contains("lb") {
            TopologyNodeType::LoadBalancer
        } else if type_lower.contains("storage") || type_lower.contains("nas") {
            TopologyNodeType::Storage
        } else if self.strict_mode {
            anyhow::bail!("Unknown topology type in strict mode: {}", type_str);
        } else {
            TopologyNodeType::Device
        };

        Ok(topology_type)
    }
}

impl Default for TopologyReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_topology_type_router() {
        let reader = TopologyReader::new();
        let topology_type = reader.parse_topology_type("router").unwrap();
        assert_eq!(topology_type, TopologyNodeType::Router);
    }

    #[test]
    fn test_parse_topology_type_server() {
        let reader = TopologyReader::new();
        let topology_type = reader.parse_topology_type("physicalServer").unwrap();
        assert_eq!(topology_type, TopologyNodeType::PhysicalServer);
    }

    #[test]
    fn test_parse_topology_type_unknown_lenient() {
        let reader = TopologyReader::new();
        let topology_type = reader.parse_topology_type("unknown_thing").unwrap();
        assert_eq!(topology_type, TopologyNodeType::Device);
    }

    #[test]
    fn test_parse_topology_type_unknown_strict() {
        let reader = TopologyReader::new_strict();
        let result = reader.parse_topology_type("unknown_thing");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_node() {
        let reader = TopologyReader::new();
        let resource = reader
            .parse_node("router01", "router", "x86_64-linux")
            .unwrap();

        assert_eq!(resource.hostname.as_str(), "router01");
        assert_eq!(resource.resource_type, ResourceType::Router);
    }

    #[test]
    fn test_functor_integration() {
        // Verify that our topology types map correctly through the functor
        let reader = TopologyReader::new();

        let topology_type = reader.parse_topology_type("router").unwrap();
        let resource_type = map_topology_to_resource_type(topology_type);
        assert_eq!(resource_type, ResourceType::Router);

        let topology_type = reader.parse_topology_type("switch").unwrap();
        let resource_type = map_topology_to_resource_type(topology_type);
        assert_eq!(resource_type, ResourceType::Switch);
    }

    #[test]
    fn test_parse_topology_with_rnix() {
        // Test the full rnix parser integration
        let reader = TopologyReader::new();

        let nix_content = r#"
        {
          nodes = {
            router01 = {
              type = "router";
              hostname = "router01";
              manufacturer = "Ubiquiti";
              model = "UniFi Dream Machine Pro";
            };
            switch01 = {
              type = "switch";
              hostname = "switch01";
              metadata = {
                poe_capable = "true";
                rack = "rack01";
              };
            };
            camera01 = {
              type = "device";
              hostname = "camera01";
            };
          };
          networks = { };
          connections = [ ];
        }
        "#;

        let resources = reader.parse_topology(nix_content).unwrap();

        assert_eq!(resources.len(), 3);

        // Find router01
        let router = resources.iter().find(|r| r.hostname.short_name() == "router01").unwrap();
        assert_eq!(router.resource_type, ResourceType::Router);
        assert_eq!(router.manufacturer.as_ref().unwrap(), "Ubiquiti");
        assert_eq!(router.model.as_ref().unwrap(), "UniFi Dream Machine Pro");

        // Find switch01
        let switch = resources.iter().find(|r| r.hostname.short_name() == "switch01").unwrap();
        assert_eq!(switch.resource_type, ResourceType::Switch);
        assert_eq!(switch.metadata.get("poe_capable").unwrap(), "true");
        assert_eq!(switch.metadata.get("rack").unwrap(), "rack01");

        // Find camera01
        let camera = resources.iter().find(|r| r.hostname.short_name() == "camera01").unwrap();
        assert_eq!(camera.resource_type, ResourceType::Appliance); // Device maps to Appliance
    }

    #[test]
    fn test_parse_topology_strict_mode() {
        // Test strict mode rejects unknown types
        let reader = TopologyReader::new_strict();

        let nix_content = r#"
        {
          nodes = {
            unknown01 = {
              type = "totally_unknown_type";
              hostname = "unknown01";
            };
          };
        }
        "#;

        let result = reader.parse_topology(nix_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_topology_lenient_mode() {
        // Test lenient mode allows unknown types
        let reader = TopologyReader::new();

        let nix_content = r#"
        {
          nodes = {
            unknown01 = {
              type = "totally_unknown_type";
              hostname = "unknown01";
            };
          };
        }
        "#;

        let resources = reader.parse_topology(nix_content).unwrap();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].resource_type, ResourceType::Appliance); // Unknown maps to Appliance
    }
}
