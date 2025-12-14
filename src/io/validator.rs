// Copyright 2025 Cowboy AI, LLC.

//! Nix File Validator
//!
//! Validates Nix files for structural and semantic correctness.

use super::Result;
use crate::nix::*;
use crate::nix::topology::*;

// ============================================================================
// Validation Result
// ============================================================================

/// Result of validation
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    /// Whether the input is valid
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }

    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: ValidationResult) {
        self.valid = self.valid && other.valid;
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Validator
// ============================================================================

/// Nix file validator
///
/// Validates Nix files for:
/// - Structural correctness (valid Nix syntax)
/// - Semantic correctness (valid topology structure)
/// - Consistency (no broken references)
pub struct NixValidator {
    parser: NixParser,
}

impl NixValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            parser: NixParser::new(),
        }
    }

    /// Validate topology content string
    pub fn validate_topology_content(&self, content: &str) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Parse to AST
        let ast = match self.parser.parse_str(content) {
            Ok(ast) => ast,
            Err(e) => {
                result.add_error(format!("Parse error: {}", e));
                return Ok(result);
            }
        };

        // Convert AST to NixValue
        let value = match crate::nix::ast_to_value(&ast) {
            Ok(value) => value,
            Err(e) => {
                result.add_error(format!("AST conversion error: {}", e));
                return Ok(result);
            }
        };

        // Validate the value
        self.validate_topology_ast(&value, &mut result);

        Ok(result)
    }

    /// Validate topology from a NixValue
    pub fn validate_topology_value(&self, value: &NixValue) -> ValidationResult {
        let mut result = ValidationResult::new();
        self.validate_topology_ast(value, &mut result);
        result
    }

    /// Validate topology AST
    fn validate_topology_ast(&self, ast: &NixValue, result: &mut ValidationResult) {
        // Must be an attrset
        let attrs = match ast {
            NixValue::Attrset(attrs) => attrs,
            _ => {
                result.add_error("Root must be an attribute set".to_string());
                return;
            }
        };

        // Validate nodes section
        if let Some(nodes_value) = attrs.get("nodes") {
            self.validate_nodes(nodes_value, result);
        }

        // Validate networks section
        if let Some(networks_value) = attrs.get("networks") {
            self.validate_networks(networks_value, result);
        }

        // Validate connections section
        if let Some(connections_value) = attrs.get("connections") {
            self.validate_connections(connections_value, result);
        }
    }

    /// Validate nodes section
    fn validate_nodes(&self, value: &NixValue, result: &mut ValidationResult) {
        let nodes = match value {
            NixValue::Attrset(nodes) => nodes,
            _ => {
                result.add_error("nodes must be an attribute set".to_string());
                return;
            }
        };

        for (node_name, node_value) in nodes.attributes.iter() {
            self.validate_node(node_name, node_value, result);
        }
    }

    /// Validate a single node
    fn validate_node(&self, name: &str, value: &NixValue, result: &mut ValidationResult) {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => {
                result.add_error(format!("Node '{}' must be an attribute set", name));
                return;
            }
        };

        // Check for required fields
        if !attrs.attributes.contains_key("type") {
            result.add_warning(format!("Node '{}' missing type field", name));
        }

        if !attrs.attributes.contains_key("system") {
            result.add_warning(format!("Node '{}' missing system field", name));
        }

        // Validate type
        if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "physical" | "vm" | "container" | "network-device" => {},
                _ => result.add_warning(format!(
                    "Node '{}' has unknown type '{}'", name, s.value
                )),
            }
        }

        // Validate hardware if present
        if let Some(hardware) = attrs.get("hardware") {
            self.validate_hardware(name, hardware, result);
        }

        // Validate interfaces if present
        if let Some(interfaces) = attrs.get("interfaces") {
            self.validate_interfaces(name, interfaces, result);
        }
    }

    /// Validate hardware config
    fn validate_hardware(&self, node_name: &str, value: &NixValue, result: &mut ValidationResult) {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => {
                result.add_error(format!(
                    "Node '{}' hardware must be an attribute set", node_name
                ));
                return;
            }
        };

        // Validate numeric fields
        if let Some(cores) = attrs.get("cpu_cores") {
            if !matches!(cores, NixValue::Integer(_)) {
                result.add_error(format!(
                    "Node '{}' cpu_cores must be an integer", node_name
                ));
            }
        }

        if let Some(mem) = attrs.get("memory_mb") {
            if !matches!(mem, NixValue::Integer(_)) {
                result.add_error(format!(
                    "Node '{}' memory_mb must be an integer", node_name
                ));
            }
        }

        if let Some(storage) = attrs.get("storage_gb") {
            if !matches!(storage, NixValue::Integer(_)) {
                result.add_error(format!(
                    "Node '{}' storage_gb must be an integer", node_name
                ));
            }
        }
    }

    /// Validate interfaces
    fn validate_interfaces(&self, node_name: &str, value: &NixValue, result: &mut ValidationResult) {
        let interfaces = match value {
            NixValue::List(interfaces) => interfaces,
            _ => {
                result.add_error(format!(
                    "Node '{}' interfaces must be a list", node_name
                ));
                return;
            }
        };

        for (i, iface) in interfaces.elements.iter().enumerate() {
            let attrs = match iface {
                NixValue::Attrset(attrs) => attrs,
                _ => {
                    result.add_error(format!(
                        "Node '{}' interface {} must be an attribute set", node_name, i
                    ));
                    continue;
                }
            };

            if !attrs.attributes.contains_key("name") {
                result.add_error(format!(
                    "Node '{}' interface {} missing name", node_name, i
                ));
            }
        }
    }

    /// Validate networks section
    fn validate_networks(&self, value: &NixValue, result: &mut ValidationResult) {
        let networks = match value {
            NixValue::Attrset(networks) => networks,
            _ => {
                result.add_error("networks must be an attribute set".to_string());
                return;
            }
        };

        for (network_name, network_value) in networks.attributes.iter() {
            self.validate_network(network_name, network_value, result);
        }
    }

    /// Validate a single network
    fn validate_network(&self, name: &str, value: &NixValue, result: &mut ValidationResult) {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => {
                result.add_error(format!("Network '{}' must be an attribute set", name));
                return;
            }
        };

        // Validate type
        if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "lan" | "wan" | "vlan" | "vpn" | "management" => {},
                _ => result.add_warning(format!(
                    "Network '{}' has unknown type '{}'", name, s.value
                )),
            }
        }

        // Validate CIDR if present
        if let Some(NixValue::String(cidr)) = attrs.get("cidr_v4") {
            if !cidr.value.contains('/') {
                result.add_error(format!(
                    "Network '{}' cidr_v4 must be in CIDR notation (x.x.x.x/y)", name
                ));
            }
        }

        if let Some(NixValue::String(cidr)) = attrs.get("cidr_v6") {
            if !cidr.value.contains('/') {
                result.add_error(format!(
                    "Network '{}' cidr_v6 must be in CIDR notation", name
                ));
            }
        }
    }

    /// Validate connections section
    fn validate_connections(&self, value: &NixValue, result: &mut ValidationResult) {
        let connections = match value {
            NixValue::List(connections) => connections,
            _ => {
                result.add_error("connections must be a list".to_string());
                return;
            }
        };

        for (i, conn) in connections.elements.iter().enumerate() {
            self.validate_connection(i, conn, result);
        }
    }

    /// Validate a single connection
    fn validate_connection(&self, index: usize, value: &NixValue, result: &mut ValidationResult) {
        let attrs = match value {
            NixValue::Attrset(attrs) => attrs,
            _ => {
                result.add_error(format!("Connection {} must be an attribute set", index));
                return;
            }
        };

        // Check required fields
        let required = ["from_node", "from_interface", "to_node", "to_interface"];
        for field in &required {
            if !attrs.attributes.contains_key(*field) {
                result.add_error(format!(
                    "Connection {} missing required field '{}'", index, field
                ));
            }
        }

        // Validate type
        if let Some(NixValue::String(s)) = attrs.get("type") {
            match s.value.as_str() {
                "ethernet" | "fiber" | "wireless" | "virtual" => {},
                _ => result.add_warning(format!(
                    "Connection {} has unknown type '{}'", index, s.value
                )),
            }
        }
    }

    /// Validate a topology object
    pub fn validate_topology(&self, topology: &NixTopology) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for node name conflicts
        for (name, _) in &topology.nodes {
            if topology.networks.contains_key(name) {
                result.add_error(format!(
                    "Name conflict: '{}' used for both node and network", name
                ));
            }
        }

        // Validate connection references
        for conn in &topology.connections {
            if !topology.nodes.contains_key(&conn.from_node) {
                result.add_error(format!(
                    "Connection references non-existent node '{}'", conn.from_node
                ));
            }
            if !topology.nodes.contains_key(&conn.to_node) {
                result.add_error(format!(
                    "Connection references non-existent node '{}'", conn.to_node
                ));
            }
        }

        result
    }
}

impl Default for NixValidator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_topology() {
        let validator = NixValidator::new();
        let value = NixValue::Attrset(NixAttrset::new());
        let result = validator.validate_topology_value(&value);
        assert!(result.is_valid());
    }

    #[test]
    fn test_string_validation_empty() {
        let validator = NixValidator::new();
        let result = validator.validate_topology_content("{ }");
        assert!(result.is_ok());

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid());
    }

    #[test]
    fn test_string_validation_valid_node() {
        let validator = NixValidator::new();
        let content = r#"{
            nodes = {
                server01 = {
                    type = "physical";
                    system = "x86_64-linux";
                };
            };
        }"#;

        let result = validator.validate_topology_content(content);
        assert!(result.is_ok());

        let validation_result = result.unwrap();
        assert!(validation_result.is_valid());
    }

    #[test]
    fn test_string_validation_invalid_syntax() {
        let validator = NixValidator::new();
        let content = "{ invalid syntax";

        let result = validator.validate_topology_content(content);
        assert!(result.is_ok());

        let validation_result = result.unwrap();
        assert!(!validation_result.is_valid());
        assert!(!validation_result.errors.is_empty());
    }

    #[test]
    fn test_validate_topology_object() {
        let validator = NixValidator::new();
        let mut topology = NixTopology::new("test".to_string());

        let node = TopologyNode::new(
            "server01".to_string(),
            TopologyNodeType::PhysicalServer,
            "x86_64-linux".to_string(),
        );
        topology.add_node(node);

        let result = validator.validate_topology(&topology);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_broken_connection_reference() {
        let validator = NixValidator::new();
        let mut topology = NixTopology::new("test".to_string());

        // Add connection without corresponding nodes
        let conn = TopologyConnection::new(
            "server01".to_string(),
            "eth0".to_string(),
            "server02".to_string(),
            "eth0".to_string(),
            ConnectionType::Ethernet,
        );
        topology.add_connection(conn);

        let result = validator.validate_topology(&topology);
        assert!(!result.is_valid());
        assert!(result.errors.iter().any(|e| e.contains("non-existent")));
    }
}
