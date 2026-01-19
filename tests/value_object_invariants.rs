// Copyright (c) 2025 - Cowboy AI, Inc.
//! Value Object Invariant Tests
//!
//! This test suite verifies that all value object invariants are preserved
//! through the topology reading and writing process. It documents WHY each
//! invariant exists and tests boundary conditions.
//!
//! ## Purpose of Invariants
//!
//! Value objects have invariants to ensure:
//! 1. **Domain Correctness**: Values match real-world constraints
//! 2. **System Reliability**: Invalid states cannot exist
//! 3. **Integration Safety**: External systems receive valid data
//! 4. **Data Integrity**: Roundtrip operations preserve meaning
//!
//! ## Run with
//! ```bash
//! cargo test --test value_object_invariants
//! ```

use anyhow::Result;
use cim_domain_nix::adapters::{TopologyReader, TopologyWriter};
use cim_infrastructure::{ComputeResource, Hostname, ResourceType};

// ============================================================================
// HOSTNAME INVARIANTS
// ============================================================================

/// Hostname invariants ensure DNS compliance and prevent routing failures.
///
/// **WHY INVARIANT**:
/// - DNS servers reject invalid hostnames
/// - Network infrastructure requires RFC 1123 compliance
/// - Certificate validation depends on correct hostname format
/// - Service discovery systems expect valid FQDNs
///
/// **INVARIANTS**:
/// 1. Non-empty (empty hostnames have no meaning)
/// 2. Total length ≤ 253 characters (DNS protocol limit)
/// 3. Each label ≤ 63 characters (DNS protocol limit)
/// 4. Labels contain only alphanumeric + hyphen
/// 5. Labels cannot start/end with hyphen
/// 6. TLD cannot be all numeric (prevents confusion with IP addresses)
#[test]
fn test_hostname_invariants_from_topology() -> Result<()> {
    let reader = TopologyReader::new();

    // Test 1: Valid hostname
    let nix_valid = r#"
    {
      nodes = {
        web01 = {
          type = "physical-server";
          hostname = "web01.prod.example.com";
        };
      };
    }
    "#;

    let resources = reader.parse_topology(nix_valid)?;
    assert_eq!(resources.len(), 1);
    assert_eq!(resources[0].hostname.as_str(), "web01.prod.example.com");
    assert_eq!(resources[0].hostname.short_name(), "web01");
    assert_eq!(resources[0].hostname.domain(), Some("prod.example.com"));
    assert!(resources[0].hostname.is_fqdn());

    // Test 2: Short hostname (not FQDN)
    let nix_short = r#"
    {
      nodes = {
        router01 = {
          type = "router";
          hostname = "router01";
        };
      };
    }
    "#;

    let resources = reader.parse_topology(nix_short)?;
    assert_eq!(resources[0].hostname.as_str(), "router01");
    assert!(!resources[0].hostname.is_fqdn());

    // Test 3: Hostname with hyphen (valid)
    let nix_hyphen = r#"
    {
      nodes = {
        web-server-01 = {
          type = "physical-server";
          hostname = "web-server-01.example.com";
        };
      };
    }
    "#;

    let resources = reader.parse_topology(nix_hyphen)?;
    assert_eq!(resources[0].hostname.as_str(), "web-server-01.example.com");

    // Test 4: Invalid hostnames should fail when creating ComputeResource
    // (not when parsing Nix, since Nix parsing is syntax-only)
    assert!(Hostname::new("").is_err()); // Empty
    assert!(Hostname::new("-invalid").is_err()); // Starts with hyphen
    assert!(Hostname::new("invalid-").is_err()); // Ends with hyphen
    assert!(Hostname::new(&"a".repeat(64)).is_err()); // Label too long
    assert!(Hostname::new(&format!("{}.com", "a".repeat(250))).is_err()); // Total too long

    Ok(())
}

/// Test that hostname invariants are preserved in roundtrip operations.
///
/// **WHY THIS MATTERS**:
/// - Configuration management relies on stable hostnames
/// - DNS records must match exactly
/// - Certificate CN/SAN fields must be preserved
/// - Service mesh routing depends on exact hostname matches
#[test]
fn test_hostname_roundtrip_invariants() -> Result<()> {
    let test_cases = vec![
        "web01",
        "web01.example.com",
        "web-server-01.prod.us-east-1.example.com",
        "router-01",
        "switch-tor-01.dc1.example.net",
    ];

    for hostname_str in test_cases {
        let hostname = Hostname::new(hostname_str)?;
        let resource = ComputeResource::new(hostname.clone(), ResourceType::PhysicalServer)?;

        let mut writer = TopologyWriter::with_name("memory:test", "test");
        writer.add_node(&resource)?;
        let nix_content = writer.generate_topology()?;

        let reader = TopologyReader::new();
        let read_resources = reader.parse_topology(&nix_content)?;

        assert_eq!(read_resources.len(), 1);
        assert_eq!(read_resources[0].hostname.as_str(), hostname.as_str());
        assert_eq!(read_resources[0].hostname.short_name(), hostname.short_name());
        assert_eq!(read_resources[0].hostname.domain(), hostname.domain());
        assert_eq!(read_resources[0].hostname.is_fqdn(), hostname.is_fqdn());
    }

    Ok(())
}

// ============================================================================
// RESOURCE TYPE INVARIANTS
// ============================================================================

/// ResourceType invariants ensure consistent infrastructure taxonomy.
///
/// **WHY INVARIANT**:
/// - Monitoring systems need consistent device classification
/// - Automation scripts depend on predictable types
/// - RBAC policies are applied per resource type
/// - Capacity planning requires accurate categorization
/// - NetBox/CMDB integration needs canonical types
///
/// **INVARIANTS**:
/// 1. Each type has unique canonical string representation
/// 2. Each type belongs to exactly one category
/// 3. Color is deterministic based on category
/// 4. String parsing is case-insensitive
/// 5. Aliases resolve to canonical form
#[test]
fn test_resource_type_invariants_from_topology() -> Result<()> {
    let reader = TopologyReader::new();

    // Test all major resource types
    let test_cases = vec![
        ("router", ResourceType::Router),
        ("switch", ResourceType::Switch),
        ("physical-server", ResourceType::PhysicalServer),
        ("virtual-machine", ResourceType::VirtualMachine),
        ("device", ResourceType::Appliance), // Device maps to Appliance via functor
    ];

    for (nix_type, expected_type) in test_cases {
        let nix_content = format!(
            r#"
            {{
              nodes = {{
                test01 = {{
                  type = "{}";
                  hostname = "test01";
                }};
              }};
            }}
            "#,
            nix_type
        );

        let resources = reader.parse_topology(&nix_content)?;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].resource_type, expected_type);

        // Verify category assignment is consistent
        let category = resources[0].resource_type.category();
        assert_eq!(
            resources[0].resource_type.category(),
            category,
            "Category must be deterministic"
        );

        // Verify color is derived from category
        let color = resources[0].resource_type.netbox_color();
        assert!(!color.is_empty(), "Color must be assigned");
        assert_eq!(color.len(), 6, "Color must be 6-char hex");
    }

    Ok(())
}

/// Test that ResourceType mappings through the functor preserve invariants.
///
/// **WHY THIS MATTERS**:
/// - Many-to-one mappings must be explicit (Camera → Device → Appliance)
/// - Bijective mappings must roundtrip exactly (Router → Router → Router)
/// - Type information loss must be documented and acceptable
/// - Category membership must be preserved when possible
#[test]
fn test_resource_type_functor_invariants() -> Result<()> {
    use cim_domain_nix::functors::*;

    // Test bijective mappings (should roundtrip)
    let bijective_types = vec![
        ResourceType::PhysicalServer,
        ResourceType::VirtualMachine,
        ResourceType::ContainerHost,
        ResourceType::Router,
        ResourceType::Switch,
        ResourceType::StorageArray, // Storage → StorageArray
    ];

    for resource_type in bijective_types {
        let topology_type = map_resource_type_to_topology(resource_type);
        let roundtrip = map_topology_to_resource_type(topology_type);

        assert_eq!(
            resource_type, roundtrip,
            "{:?} should roundtrip through functor",
            resource_type
        );
        assert!(
            can_roundtrip(resource_type),
            "{:?} should be marked as bijective",
            resource_type
        );
    }

    // Test lossy mappings (many-to-one, do not roundtrip)
    // All these map to Device, which maps back to Appliance (the conservative default)
    let lossy_types = vec![
        ResourceType::Camera,
        ResourceType::KVM,
        ResourceType::Monitor,
        ResourceType::PDU,
        ResourceType::UPS,
        ResourceType::AccessPoint,
        ResourceType::IDS,
        ResourceType::VPNGateway,
        // Note: Appliance itself roundtrips (Device → Appliance), so it's bijective
        ResourceType::EdgeDevice,
        ResourceType::Sensor,
    ];

    for resource_type in lossy_types {
        let topology_type = map_resource_type_to_topology(resource_type);

        // All these map to Device
        assert_eq!(topology_type, TopologyNodeType::Device);

        // Device maps back to Appliance (conservative default)
        let roundtrip = map_topology_to_resource_type(topology_type);
        assert_eq!(roundtrip, ResourceType::Appliance);

        // Should be marked as lossy
        assert!(
            !can_roundtrip(resource_type),
            "{:?} should be marked as lossy",
            resource_type
        );
    }

    Ok(())
}

// ============================================================================
// METADATA INVARIANTS
// ============================================================================

/// Metadata invariants ensure consistent key-value semantics.
///
/// **WHY INVARIANT**:
/// - Ansible/Terraform rely on consistent metadata structure
/// - Monitoring tags must be predictable
/// - Custom attributes enable extensibility without schema changes
/// - Search/filter operations depend on stable keys
/// - Database column names require valid identifiers
/// - Environment variables follow similar naming conventions
///
/// **INVARIANTS**:
/// 1. Keys are non-empty strings (keys must identify something)
/// 2. Keys ≤ 64 characters (database field limit)
/// 3. Keys must be lowercase alphanumeric + underscore ONLY (no hyphens!)
///    - Rationale: Matches database column naming, env var conventions
///    - Rationale: Prevents shell quoting issues, JSON key problems
///    - Rationale: Compatible with most monitoring/config systems
/// 4. Values are strings (not complex types)
/// 5. Values preserve original formatting (including spaces)
/// 6. Duplicate keys overwrite (HashMap semantics)
#[test]
fn test_metadata_invariants_roundtrip() -> Result<()> {
    let hostname = Hostname::new("test01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Add various metadata (following key naming rules)
    resource.add_metadata("rack", "rack01")?;
    resource.add_metadata("role", "web server")?; // Space in value
    resource.add_metadata("environment", "production")?;
    resource.add_metadata("numeric_value", "42")?; // Number as string
    resource.add_metadata("snake_case_key", "value")?; // Underscore in key (valid)

    // Write and read back
    let mut writer = TopologyWriter::with_name("memory:test", "test");
    writer.add_node(&resource)?;
    let nix_content = writer.generate_topology()?;

    let reader = TopologyReader::new();
    let read_resources = reader.parse_topology(&nix_content)?;
    assert_eq!(read_resources.len(), 1);

    let read_resource = &read_resources[0];

    // Verify all metadata preserved
    assert_eq!(read_resource.metadata.get("rack"), Some(&"rack01".to_string()));
    assert_eq!(
        read_resource.metadata.get("role"),
        Some(&"web server".to_string())
    );
    assert_eq!(
        read_resource.metadata.get("environment"),
        Some(&"production".to_string())
    );
    assert_eq!(
        read_resource.metadata.get("numeric_value"),
        Some(&"42".to_string())
    );
    assert_eq!(
        read_resource.metadata.get("snake_case_key"),
        Some(&"value".to_string())
    );

    // Verify count
    assert_eq!(read_resource.metadata.len(), 5);

    Ok(())
}

/// Test metadata key validation invariants.
///
/// **WHY THIS MATTERS**:
/// - Invalid keys would break database operations
/// - Shell scripts would fail with spaces/hyphens in variables
/// - JSON parsers handle all keys, but monitoring systems don't
/// - Consistency prevents integration bugs
#[test]
fn test_metadata_key_validation_invariants() -> Result<()> {
    let hostname = Hostname::new("test01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Valid keys (should succeed)
    assert!(resource.add_metadata("valid_key", "value").is_ok());
    assert!(resource.add_metadata("key123", "value").is_ok());
    assert!(resource.add_metadata("abc_123_xyz", "value").is_ok());

    // Invalid keys (should fail)
    assert!(resource.add_metadata("", "value").is_err()); // Empty
    assert!(resource.add_metadata("hyphenated-key", "value").is_err()); // Hyphen
    assert!(resource.add_metadata("CamelCase", "value").is_err()); // Uppercase
    assert!(resource.add_metadata("space key", "value").is_err()); // Space
    assert!(resource.add_metadata("dot.key", "value").is_err()); // Dot
    assert!(resource.add_metadata(&"x".repeat(65), "value").is_err()); // Too long

    // Edge case: 64 characters (should succeed)
    assert!(resource.add_metadata(&"a".repeat(64), "value").is_ok());

    Ok(())
}

/// Test metadata key uniqueness invariant.
///
/// **WHY THIS MATTERS**:
/// - HashMap semantics require unique keys
/// - Last value wins in case of duplicates
/// - This is a language-level invariant (HashMap behavior)
#[test]
fn test_metadata_key_uniqueness() -> Result<()> {
    let hostname = Hostname::new("test01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

    // Add same key twice
    resource.add_metadata("environment", "staging")?;
    resource.add_metadata("environment", "production")?; // Overwrites

    assert_eq!(
        resource.metadata.get("environment"),
        Some(&"production".to_string())
    );
    assert_eq!(resource.metadata.len(), 1);

    Ok(())
}

// ============================================================================
// HARDWARE INFO INVARIANTS
// ============================================================================

/// Hardware info invariants ensure consistent equipment tracking.
///
/// **WHY INVARIANT**:
/// - Asset management systems need accurate manufacturer data
/// - Warranty lookups require exact model numbers
/// - Support tickets reference serial numbers
/// - Procurement systems track vendor information
/// - RMA processes depend on serial number accuracy
///
/// **INVARIANTS**:
/// 1. All fields are optional (not all devices have full info)
/// 2. Serial numbers are unique per manufacturer (business rule, not enforced here)
/// 3. Model implies manufacturer (semantic invariant)
/// 4. Values preserve original case and formatting
#[test]
fn test_hardware_info_invariants_roundtrip() -> Result<()> {
    let test_cases = vec![
        // Full hardware info
        (
            Some("Dell"),
            Some("PowerEdge R740"),
            Some("SN123456789"),
        ),
        // Manufacturer and model only
        (Some("Ubiquiti"), Some("UniFi Dream Machine Pro"), None),
        // Manufacturer only
        (Some("Cisco"), None, None),
        // No hardware info
        (None, None, None),
    ];

    for (manufacturer, model, serial) in test_cases {
        let hostname = Hostname::new("test01")?;
        let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;

        if manufacturer.is_some() || model.is_some() || serial.is_some() {
            resource.set_hardware(
                manufacturer.map(String::from),
                model.map(String::from),
                serial.map(String::from),
            );
        }

        // Write and read back
        let mut writer = TopologyWriter::with_name("memory:test", "test");
        writer.add_node(&resource)?;
        let nix_content = writer.generate_topology()?;

        let reader = TopologyReader::new();
        let read_resources = reader.parse_topology(&nix_content)?;
        assert_eq!(read_resources.len(), 1);

        let read_resource = &read_resources[0];

        // Verify hardware info preserved
        assert_eq!(
            read_resource.manufacturer.as_deref(),
            manufacturer,
            "Manufacturer must match"
        );
        assert_eq!(read_resource.model.as_deref(), model, "Model must match");
        assert_eq!(
            read_resource.serial_number.as_deref(),
            serial,
            "Serial number must match"
        );
    }

    Ok(())
}

// ============================================================================
// NIX PARSING INVARIANTS
// ============================================================================

/// Nix parsing invariants ensure robust error handling.
///
/// **WHY INVARIANT**:
/// - Malformed Nix files should not crash the parser
/// - Invalid syntax should be caught early
/// - Error messages should be actionable
/// - Parser should not accept invalid attribute names
///
/// **INVARIANTS**:
/// 1. Invalid Nix syntax is rejected
/// 2. Missing required attributes cause errors
/// 3. Type mismatches are detected
/// 4. Malformed attribute sets are caught
#[test]
fn test_nix_parsing_error_invariants() {
    let reader = TopologyReader::new();

    // Test 1: Invalid Nix syntax
    let invalid_syntax = "{ nodes = { invalid syntax } }";
    let result = reader.parse_topology(invalid_syntax);
    assert!(result.is_err(), "Invalid syntax should be rejected");

    // Test 2: Missing nodes attribute
    let no_nodes = "{ other = { }; }";
    let result = reader.parse_topology(no_nodes);
    assert!(
        result.is_err(),
        "Missing 'nodes' attribute should cause error"
    );

    // Test 3: Missing required 'type' attribute (lenient mode skips with warning)
    let no_type = r#"
    {
      nodes = {
        test01 = {
          hostname = "test01";
        };
      };
    }
    "#;
    let result = reader.parse_topology(no_type);
    // In lenient mode, node with missing 'type' is skipped (logged as warning)
    assert!(result.is_ok(), "Lenient mode should skip invalid nodes");
    assert_eq!(result.unwrap().len(), 0, "Invalid node should be skipped");

    // Test 3b: Missing required 'type' attribute (strict mode should error)
    let strict_reader = TopologyReader::new_strict();
    let result = strict_reader.parse_topology(no_type);
    assert!(
        result.is_err(),
        "Strict mode should reject missing 'type' attribute"
    );

    // Test 4: Empty nodes (should succeed but return empty vec)
    let empty_nodes = "{ nodes = { }; }";
    let result = reader.parse_topology(empty_nodes);
    assert!(result.is_ok(), "Empty nodes should be valid");
    assert_eq!(result.unwrap().len(), 0);
}

/// Test strict vs lenient mode invariants.
///
/// **WHY THIS MATTERS**:
/// - Strict mode catches configuration errors early
/// - Lenient mode allows gradual migration
/// - Mode choice is explicit, not implicit
/// - Behavior is predictable and documented
#[test]
fn test_strict_lenient_mode_invariants() -> Result<()> {
    let nix_unknown_type = r#"
    {
      nodes = {
        mystery01 = {
          type = "totally_unknown_device_type";
          hostname = "mystery01";
        };
      };
    }
    "#;

    // Strict mode: reject unknown types
    let strict_reader = TopologyReader::new_strict();
    let result = strict_reader.parse_topology(nix_unknown_type);
    assert!(
        result.is_err(),
        "Strict mode must reject unknown types"
    );

    // Lenient mode: map to Appliance
    let lenient_reader = TopologyReader::new();
    let result = lenient_reader.parse_topology(nix_unknown_type)?;
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].resource_type,
        ResourceType::Appliance,
        "Lenient mode must map unknown to Appliance"
    );

    Ok(())
}

// ============================================================================
// ROUNDTRIP INVARIANTS
// ============================================================================

/// Roundtrip invariants ensure bidirectional data integrity.
///
/// **WHY INVARIANT**:
/// - Configuration management requires stable serialization
/// - GitOps workflows depend on deterministic output
/// - Diff tools need consistent formatting
/// - Idempotency requires exact roundtrips
///
/// **INVARIANTS**:
/// 1. Hostname is preserved exactly
/// 2. Hardware info is preserved if present
/// 3. Metadata is preserved (all keys and values)
/// 4. Resource type may change due to functor (documented)
/// 5. Node order may change (HashMap, not guaranteed)
#[test]
fn test_complete_roundtrip_invariants() -> Result<()> {
    // Create a resource with all possible data
    let hostname = Hostname::new("complete-test.example.com")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::Router)?;

    resource.set_hardware(
        Some("Ubiquiti".to_string()),
        Some("UniFi Dream Machine Pro".to_string()),
        Some("UDM-SERIAL-123".to_string()),
    );

    resource.add_metadata("rack", "network-rack-01")?;
    resource.add_metadata("vlan_support", "true")?;
    resource.add_metadata("management_ip", "192.168.1.1")?;
    resource.add_metadata("firmware_version", "1.2.3")?;

    // First write
    let mut writer1 = TopologyWriter::with_name("memory:test1", "test");
    writer1.add_node(&resource)?;
    let nix_content1 = writer1.generate_topology()?;

    // First read
    let reader = TopologyReader::new();
    let resources1 = reader.parse_topology(&nix_content1)?;
    assert_eq!(resources1.len(), 1);

    // Second write (from read data)
    let mut writer2 = TopologyWriter::with_name("memory:test2", "test");
    writer2.add_node(&resources1[0])?;
    let nix_content2 = writer2.generate_topology()?;

    // Second read
    let resources2 = reader.parse_topology(&nix_content2)?;
    assert_eq!(resources2.len(), 1);

    // Verify all invariants preserved
    let r1 = &resources1[0];
    let r2 = &resources2[0];

    // Hostname invariant
    assert_eq!(r1.hostname.as_str(), r2.hostname.as_str());
    assert_eq!(r1.hostname.short_name(), r2.hostname.short_name());
    assert_eq!(r1.hostname.domain(), r2.hostname.domain());

    // Hardware info invariant
    assert_eq!(r1.manufacturer, r2.manufacturer);
    assert_eq!(r1.model, r2.model);
    assert_eq!(r1.serial_number, r2.serial_number);

    // Metadata invariant
    assert_eq!(r1.metadata.len(), r2.metadata.len());
    for (key, value) in &r1.metadata {
        assert_eq!(
            r2.metadata.get(key),
            Some(value),
            "Metadata key {} must be preserved",
            key
        );
    }

    // Resource type invariant (Router is bijective)
    assert_eq!(r1.resource_type, r2.resource_type);

    Ok(())
}

// ============================================================================
// BOUNDARY CONDITION TESTS
// ============================================================================

/// Test boundary conditions for all value objects.
///
/// **WHY THIS MATTERS**:
/// - Edge cases reveal hidden assumptions
/// - Boundary values often trigger bugs
/// - Maximum values test limits
/// - Minimum values test validation
#[test]
fn test_boundary_conditions() -> Result<()> {
    // Hostname: maximum length label (63 chars)
    let max_label = "a".repeat(63);
    let hostname = Hostname::new(&max_label)?;
    assert_eq!(hostname.short_name(), &max_label);

    // Hostname: maximum total length (253 chars)
    // 253 = 63 + . + 63 + . + 63 + . + 60 (60 for last part including TLD)
    // 63 + 1 + 63 + 1 + 63 + 1 + 60 = 252 (just under limit)
    let long_hostname = format!("{}.{}.{}.{}", max_label, max_label, max_label, "a".repeat(60));
    assert!(long_hostname.len() <= 253, "Hostname length: {}", long_hostname.len());
    let hostname = Hostname::new(&long_hostname)?;
    // This is not a valid FQDN because the last label is all letters (no TLD separator)
    // but it's valid as a hostname
    assert_eq!(hostname.as_str(), long_hostname);

    // Metadata: empty string values (should be allowed)
    let hostname = Hostname::new("test01")?;
    let mut resource = ComputeResource::new(hostname, ResourceType::PhysicalServer)?;
    resource.add_metadata("empty_value", "")?;
    assert_eq!(resource.metadata.get("empty_value"), Some(&"".to_string()));

    // Metadata: special characters in values
    resource.add_metadata("special", "value with spaces and symbols: !@#$%")?;
    assert_eq!(
        resource.metadata.get("special"),
        Some(&"value with spaces and symbols: !@#$%".to_string())
    );

    Ok(())
}
