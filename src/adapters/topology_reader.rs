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

use anyhow::{Context, Result};
use cim_infrastructure::{
    ComputeResource, Hostname, ResourceType,
};
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
    /// This is a simplified implementation that demonstrates the concept.
    /// A full implementation would:
    /// 1. Use rnix to parse Nix AST
    /// 2. Walk the topology attribute set
    /// 3. Extract nodes, networks, connections
    /// 4. Generate ComputeResource entities
    ///
    /// For now, this is a placeholder that shows the interface.
    fn parse_topology(&self, _content: &str) -> Result<Vec<ComputeResource>> {
        // TODO: Implement full Nix parsing with rnix
        //
        // Steps:
        // 1. Parse with rnix::Root::parse(content)
        // 2. Walk AST to find topology nodes
        // 3. For each node:
        //    - Extract name, type, system
        //    - Map type using resource_type_functor
        //    - Create ComputeResource
        // 4. Return all discovered resources

        // Placeholder: Return empty vec for now
        Ok(Vec::new())
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
}
