// Copyright (c) 2025 - Cowboy AI, Inc.
//! Resource Type Functor: ResourceType ⟷ TopologyNodeType
//!
//! This module implements a Category Theory functor that provides
//! bidirectional mapping between CIM ResourceType and nixos-topology node types.
//!
//! ## Functor F: ResourceType → TopologyNodeType
//!
//! Maps our infrastructure taxonomy (35 types, 9 categories) to nixos-topology types.
//!
//! ## Functor G: TopologyNodeType → ResourceType
//!
//! Reverse mapping for reading existing topology files.
//!
//! ## Category Theory Properties
//!
//! These functors must satisfy:
//! 1. **Identity**: `F(id) = id`
//! 2. **Composition**: `F(g ∘ f) = F(g) ∘ F(f)`
//! 3. **Bijection** (where possible): `G(F(x)) = x`

use cim_infrastructure::ResourceType;

/// nixos-topology node type (simplified representation)
///
/// NOTE: This is a Rust representation of the Nix types from oddlama/nixos-topology.
/// The actual Nix types are defined in the topology module system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TopologyNodeType {
    /// Physical server
    PhysicalServer,
    /// Virtual machine
    VirtualMachine,
    /// Container
    Container,
    /// Network router
    Router,
    /// Network switch
    Switch,
    /// Firewall appliance
    Firewall,
    /// Load balancer
    LoadBalancer,
    /// Storage device
    Storage,
    /// Generic device (catch-all)
    Device,
}

/// Functor F: ResourceType → TopologyNodeType
///
/// Maps CIM infrastructure taxonomy to nixos-topology node types.
///
/// ## Mapping Strategy
///
/// - Direct mappings for common infrastructure (servers, VMs, routers, switches)
/// - Specialized devices (cameras, KVMs, monitors) → Device
/// - Preserves semantic meaning where possible
///
/// ## Examples
///
/// ```rust
/// use cim_domain_nix::functors::resource_type_functor::*;
/// use cim_infrastructure::ResourceType;
///
/// assert_eq!(
///     map_resource_type_to_topology(ResourceType::PhysicalServer),
///     TopologyNodeType::PhysicalServer
/// );
///
/// assert_eq!(
///     map_resource_type_to_topology(ResourceType::Router),
///     TopologyNodeType::Router
/// );
///
/// // Specialized devices map to generic Device
/// assert_eq!(
///     map_resource_type_to_topology(ResourceType::Camera),
///     TopologyNodeType::Device
/// );
/// ```
pub fn map_resource_type_to_topology(resource_type: ResourceType) -> TopologyNodeType {
    match resource_type {
        // Compute Resources
        ResourceType::PhysicalServer => TopologyNodeType::PhysicalServer,
        ResourceType::VirtualMachine => TopologyNodeType::VirtualMachine,
        ResourceType::ContainerHost => TopologyNodeType::Container,
        ResourceType::Hypervisor => TopologyNodeType::PhysicalServer,

        // Network Infrastructure
        ResourceType::Router => TopologyNodeType::Router,
        ResourceType::Switch => TopologyNodeType::Switch,
        ResourceType::Layer3Switch => TopologyNodeType::Switch,
        ResourceType::AccessPoint => TopologyNodeType::Device,
        ResourceType::LoadBalancer => TopologyNodeType::LoadBalancer,

        // Security Appliances
        ResourceType::Firewall => TopologyNodeType::Firewall,
        ResourceType::IDS => TopologyNodeType::Device,
        ResourceType::VPNGateway => TopologyNodeType::Device,
        ResourceType::WAF => TopologyNodeType::Firewall,
        ResourceType::Camera => TopologyNodeType::Device,

        // Storage Devices
        ResourceType::StorageArray => TopologyNodeType::Storage,
        ResourceType::NAS => TopologyNodeType::Storage,
        ResourceType::SANSwitch => TopologyNodeType::Storage,

        // Specialized Appliances
        ResourceType::Appliance => TopologyNodeType::Device,
        ResourceType::BackupAppliance => TopologyNodeType::Device,
        ResourceType::MonitoringAppliance => TopologyNodeType::Device,
        ResourceType::AuthServer => TopologyNodeType::Device,
        ResourceType::KVM => TopologyNodeType::Device,
        ResourceType::Monitor => TopologyNodeType::Device,

        // Edge/IoT Devices
        ResourceType::EdgeDevice => TopologyNodeType::Device,
        ResourceType::IoTGateway => TopologyNodeType::Device,
        ResourceType::Sensor => TopologyNodeType::Device,

        // Power/Environmental
        ResourceType::PDU => TopologyNodeType::Device,
        ResourceType::UPS => TopologyNodeType::Device,
        ResourceType::EnvironmentalMonitor => TopologyNodeType::Device,

        // Telecommunications
        ResourceType::PBX => TopologyNodeType::Device,
        ResourceType::VideoConference => TopologyNodeType::Device,

        // Other/Unknown
        ResourceType::Other => TopologyNodeType::Device,
        ResourceType::Unknown => TopologyNodeType::Device,
    }
}

/// Functor G: TopologyNodeType → ResourceType (reverse mapping)
///
/// Maps nixos-topology node types back to CIM ResourceType.
///
/// ## Limitations
///
/// This is a **lossy** reverse mapping because multiple ResourceTypes map to
/// the same TopologyNodeType. For example:
/// - Camera, KVM, Monitor, PDU, UPS all map to Device
/// - Switch and Layer3Switch both map to Switch
///
/// When reading topology, we use conservative defaults for ambiguous types.
///
/// ## Examples
///
/// ```rust
/// use cim_domain_nix::functors::resource_type_functor::*;
/// use cim_infrastructure::ResourceType;
///
/// assert_eq!(
///     map_topology_to_resource_type(TopologyNodeType::PhysicalServer),
///     ResourceType::PhysicalServer
/// );
///
/// // Device maps to generic Appliance (conservative default)
/// assert_eq!(
///     map_topology_to_resource_type(TopologyNodeType::Device),
///     ResourceType::Appliance
/// );
/// ```
pub fn map_topology_to_resource_type(node_type: TopologyNodeType) -> ResourceType {
    match node_type {
        TopologyNodeType::PhysicalServer => ResourceType::PhysicalServer,
        TopologyNodeType::VirtualMachine => ResourceType::VirtualMachine,
        TopologyNodeType::Container => ResourceType::ContainerHost,
        TopologyNodeType::Router => ResourceType::Router,
        TopologyNodeType::Switch => ResourceType::Switch,
        TopologyNodeType::Firewall => ResourceType::Firewall,
        TopologyNodeType::LoadBalancer => ResourceType::LoadBalancer,
        TopologyNodeType::Storage => ResourceType::StorageArray,
        TopologyNodeType::Device => ResourceType::Appliance, // Conservative default
    }
}

/// Check if a ResourceType can roundtrip through topology mapping
///
/// Returns true if: `G(F(x)) = x` (bijection holds)
///
/// ## Examples
///
/// ```rust
/// use cim_domain_nix::functors::resource_type_functor::*;
/// use cim_infrastructure::ResourceType;
///
/// // Direct mappings roundtrip correctly
/// assert!(can_roundtrip(ResourceType::PhysicalServer));
/// assert!(can_roundtrip(ResourceType::Router));
///
/// // Specialized devices don't roundtrip (map to Device → Appliance)
/// assert!(!can_roundtrip(ResourceType::Camera));
/// assert!(!can_roundtrip(ResourceType::KVM));
/// ```
pub fn can_roundtrip(resource_type: ResourceType) -> bool {
    let topology_type = map_resource_type_to_topology(resource_type);
    let roundtrip = map_topology_to_resource_type(topology_type);
    roundtrip == resource_type
}

/// Get all ResourceTypes that map to a given TopologyNodeType
///
/// Useful for understanding the many-to-one nature of the mapping.
///
/// ## Examples
///
/// ```rust
/// use cim_domain_nix::functors::resource_type_functor::*;
/// use cim_infrastructure::ResourceType;
///
/// let device_types = get_resource_types_for_topology(TopologyNodeType::Device);
/// assert!(device_types.contains(&ResourceType::Camera));
/// assert!(device_types.contains(&ResourceType::KVM));
/// assert!(device_types.contains(&ResourceType::Monitor));
/// ```
pub fn get_resource_types_for_topology(node_type: TopologyNodeType) -> Vec<ResourceType> {
    use ResourceType::*;

    // All ResourceType variants
    let all_types = vec![
        PhysicalServer,
        VirtualMachine,
        ContainerHost,
        Hypervisor,
        Router,
        Switch,
        Layer3Switch,
        AccessPoint,
        LoadBalancer,
        Firewall,
        IDS,
        VPNGateway,
        WAF,
        Camera,
        StorageArray,
        NAS,
        SANSwitch,
        Appliance,
        BackupAppliance,
        MonitoringAppliance,
        AuthServer,
        KVM,
        Monitor,
        EdgeDevice,
        IoTGateway,
        Sensor,
        PDU,
        UPS,
        EnvironmentalMonitor,
        PBX,
        VideoConference,
        Other,
        Unknown,
    ];

    all_types
        .into_iter()
        .filter(|rt| map_resource_type_to_topology(*rt) == node_type)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_resources() {
        assert_eq!(
            map_resource_type_to_topology(ResourceType::PhysicalServer),
            TopologyNodeType::PhysicalServer
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::VirtualMachine),
            TopologyNodeType::VirtualMachine
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::ContainerHost),
            TopologyNodeType::Container
        );
    }

    #[test]
    fn test_network_infrastructure() {
        assert_eq!(
            map_resource_type_to_topology(ResourceType::Router),
            TopologyNodeType::Router
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::Switch),
            TopologyNodeType::Switch
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::Layer3Switch),
            TopologyNodeType::Switch
        );
    }

    #[test]
    fn test_specialized_devices() {
        // All specialized devices map to Device
        assert_eq!(
            map_resource_type_to_topology(ResourceType::Camera),
            TopologyNodeType::Device
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::KVM),
            TopologyNodeType::Device
        );
        assert_eq!(
            map_resource_type_to_topology(ResourceType::Monitor),
            TopologyNodeType::Device
        );
    }

    #[test]
    fn test_reverse_mapping() {
        assert_eq!(
            map_topology_to_resource_type(TopologyNodeType::PhysicalServer),
            ResourceType::PhysicalServer
        );
        assert_eq!(
            map_topology_to_resource_type(TopologyNodeType::Router),
            ResourceType::Router
        );
        // Device maps to conservative default
        assert_eq!(
            map_topology_to_resource_type(TopologyNodeType::Device),
            ResourceType::Appliance
        );
    }

    #[test]
    fn test_roundtrip_direct_mappings() {
        // These should roundtrip correctly
        assert!(can_roundtrip(ResourceType::PhysicalServer));
        assert!(can_roundtrip(ResourceType::VirtualMachine));
        assert!(can_roundtrip(ResourceType::Router));
        assert!(can_roundtrip(ResourceType::Switch));
    }

    #[test]
    fn test_roundtrip_lossy_mappings() {
        // These won't roundtrip (many-to-one mapping)
        assert!(!can_roundtrip(ResourceType::Camera));
        assert!(!can_roundtrip(ResourceType::KVM));
        assert!(!can_roundtrip(ResourceType::Monitor));
        assert!(!can_roundtrip(ResourceType::Layer3Switch)); // Maps to Switch → Switch
    }

    #[test]
    fn test_many_to_one_mapping() {
        let device_types = get_resource_types_for_topology(TopologyNodeType::Device);

        // Should include all specialized devices
        assert!(device_types.contains(&ResourceType::Camera));
        assert!(device_types.contains(&ResourceType::KVM));
        assert!(device_types.contains(&ResourceType::Monitor));
        assert!(device_types.contains(&ResourceType::PDU));
        assert!(device_types.contains(&ResourceType::UPS));
        assert!(device_types.contains(&ResourceType::Appliance));

        // Should have many types
        assert!(device_types.len() > 10);
    }
}
