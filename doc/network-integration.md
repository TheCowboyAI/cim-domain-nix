# Network Integration Guide

This guide describes how cim-domain-nix integrates with network topology events to automatically generate NixOS system configurations following a hierarchical network architecture.

## Overview

The network integration module processes events from the `nix-network` domain to automatically generate and manage NixOS system configurations based on network topology. This enables infrastructure-as-code workflows where network design drives system configuration.

## Hierarchical Network Architecture

The system supports a four-tier hierarchical network model:

```
┌─────────────────┐
│ Super-Cluster   │ (Regional/Global)
└────────┬────────┘
         │
    ┌────┴────┐
    │ Cluster │ (Data Center/Site)
    └────┬────┘
         │
    ┌────┴────┐
    │  Leaf   │ (Rack/Zone)
    └────┬────┘
         │
    ┌────┴────┐
    │ Client  │ (End Device)
    └─────────┘
```

### Network Tiers

#### 1. Client Tier
- **Purpose**: End-user devices and edge compute nodes
- **Node Types**: Workstation, IoTDevice, Container
- **Characteristics**:
  - DHCP clients by default
  - Minimal services
  - Firewall allows only essential ports
  - No routing capabilities

#### 2. Leaf Tier
- **Purpose**: First aggregation point, typically rack-level
- **Node Types**: Router, AccessPoint
- **Characteristics**:
  - Provides DHCP to clients
  - Local DNS caching
  - VLAN termination
  - Basic firewall/NAT
  - Uplinks to cluster tier

#### 3. Cluster Tier
- **Purpose**: Site or data center core
- **Node Types**: Gateway, Server
- **Characteristics**:
  - Inter-VLAN routing
  - Advanced services (DNS, DHCP relay)
  - Load balancing
  - Site-to-site VPN termination
  - Redundant paths

#### 4. Super-Cluster Tier
- **Purpose**: Regional or global connectivity
- **Node Types**: Gateway (BGP-enabled)
- **Characteristics**:
  - BGP routing
  - Multi-site coordination
  - Global load balancing
  - WAN optimization
  - DDoS protection

## Event Processing

### Supported Events

The system listens for these NATS subjects:

```
network.topology.created    # New network topology
network.topology.updated    # Topology changes
network.interface.added     # New interface on node
network.interface.removed   # Interface removed
network.interface.updated   # Interface reconfigured
network.route.added        # Routing changes
network.route.removed      # Route removal
```

### Event Structure

```rust
pub struct NetworkTopologyEvent {
    pub topology_id: Uuid,
    pub name: String,
    pub nodes: Vec<NetworkNode>,
    pub connections: Vec<NetworkConnection>,
    pub timestamp: DateTime<Utc>,
}

pub struct NetworkNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub tier: NetworkTier,  // Client, Leaf, Cluster, SuperCluster
    pub interfaces: Vec<NetworkInterface>,
    pub services: Vec<String>,
    pub metadata: HashMap<String, String>,
}
```

## System Generation

### Automatic Configuration

Based on node type and tier, the system automatically configures:

#### Client Tier Configuration
```nix
{
  networking = {
    useDHCP = true;
    firewall = {
      enable = true;
      allowedTCPPorts = [ 22 ];  # SSH only
    };
  };
  
  services.openssh.enable = true;
}
```

#### Leaf Tier Configuration
```nix
{
  networking = {
    interfaces = {
      eth0 = { /* uplink to cluster */ };
      eth1 = { /* client network */ };
    };
    nat.enable = true;
    firewall.enable = true;
  };
  
  services = {
    dhcpd4.enable = true;
    unbound.enable = true;  # DNS caching
    hostapd.enable = true;  # If wireless
  };
}
```

#### Cluster Tier Configuration
```nix
{
  boot.kernel.sysctl = {
    "net.ipv4.ip_forward" = true;
    "net.ipv6.conf.all.forwarding" = true;
  };
  
  networking = {
    vlans = { /* VLAN interfaces */ };
    bridges = { /* Bridge configuration */ };
  };
  
  services = {
    keepalived.enable = true;  # HA
    haproxy.enable = true;     # Load balancing
    strongswan.enable = true;  # VPN
  };
}
```

#### Super-Cluster Tier Configuration
```nix
{
  services = {
    bird2 = {  # BGP routing
      enable = true;
      config = /* BGP configuration */;
    };
    
    coredns = {  # Global DNS
      enable = true;
      config = /* GeoDNS configuration */;
    };
  };
  
  networking.nftables = {  # Advanced firewall
    enable = true;
    ruleset = /* DDoS protection rules */;
  };
}
```

## Usage Example

### Processing Network Events

```rust
use cim_domain_nix::network::{
    NetworkIntegrationService, 
    NetworkTopologyEvent,
    NetworkTier,
};

// Initialize service
let mut service = NetworkIntegrationService::new();

// Create hierarchical network topology
let topology = create_hierarchical_network();

// Process topology event
let systems = service
    .process_topology_event(topology)
    .await?;

// Generated systems follow hierarchy
for system in systems {
    match system.tier {
        NetworkTier::Client => {
            // Minimal configuration
        }
        NetworkTier::Leaf => {
            // Edge services
        }
        NetworkTier::Cluster => {
            // Core services
        }
        NetworkTier::SuperCluster => {
            // Global services
        }
    }
}
```

### Subscribing to Network Events

```rust
use cim_domain_nix::nats::NatsSubscriber;

let subscriber = NatsSubscriber::new(config).await?;

// Subscribe to all network events
subscriber.subscribe("network.>").await?;

// Process events as they arrive
while let Some(msg) = subscription.next().await {
    let event = parse_network_event(&msg)?;
    service.handle_network_event(event).await?;
}
```

## Service Mapping

Services are automatically configured based on node tier and type:

| Tier | Node Type | Services |
|------|-----------|----------|
| Client | Workstation | SSH |
| Client | IoTDevice | Monitoring agent |
| Leaf | Router | DHCP, DNS, NAT |
| Leaf | AccessPoint | hostapd, DHCP |
| Cluster | Gateway | HAProxy, Keepalived, VPN |
| Cluster | Server | Application-specific |
| SuperCluster | Gateway | BGP, GeoDNS, WAF |

## Network Patterns

### Spine-Leaf Architecture
```
[Super-Cluster: Core Switches]
       |         |
[Cluster: Spine Switches]
   |    |    |    |
[Leaf: Top-of-Rack]
   |    |    |    |
[Clients: Servers]
```

### Hub-and-Spoke
```
    [Super-Cluster: HQ]
           |
    [Cluster: Regional]
     /     |     \
[Leaf]   [Leaf]  [Leaf]
  |        |       |
[Clients][Clients][Clients]
```

### Mesh Network
```
[Cluster] --- [Cluster]
   |   \    /    |
   |     \/      |
   |     /\      |
   |   /    \    |
[Leaf] ------ [Leaf]
```

## Dynamic Updates

The system handles dynamic network changes:

### Interface Changes
- Automatically updates system configuration
- Preserves service availability
- Handles IP address changes

### Topology Changes
- Node additions/removals
- Connection changes
- Tier migrations

### Example: Adding a Node

```rust
let new_node = NetworkNode {
    id: Uuid::new_v4(),
    name: "leaf-sw-03".to_string(),
    node_type: NodeType::Router,
    tier: NetworkTier::Leaf,
    interfaces: vec![/* interfaces */],
    services: vec!["dhcp".to_string()],
    metadata: HashMap::new(),
};

// Emit event
publish_event(NetworkEvent::NodeAdded(new_node)).await?;

// System automatically:
// 1. Generates NixOS configuration
// 2. Configures DHCP for client subnet
// 3. Sets up uplink to cluster tier
// 4. Enables routing and NAT
```

## Best Practices

### 1. Tier Assignment
- Assign nodes to appropriate tiers based on function
- Don't skip tiers unless necessary
- Maintain clear boundaries between tiers

### 2. Service Placement
- Run services at the lowest appropriate tier
- Use caching at each tier (DNS, content)
- Implement redundancy within tiers

### 3. Security
- Firewall rules become more restrictive toward clients
- Implement zero-trust between tiers
- Use VLANs to segment traffic

### 4. Monitoring
- Each tier monitors the tier below
- Aggregate metrics upward
- Alert on tier-wide issues

## Integration with Other CIM Domains

### With cim-domain-git
- Track generated configurations in Git
- Version control for infrastructure
- Rollback capabilities

### With cim-domain-k8s
- Generate Kubernetes node configurations
- Network policy enforcement
- Service mesh integration

### With cim-domain-monitoring
- Automatic monitoring configuration
- Tier-based alerting rules
- Performance baselines

## Troubleshooting

### Common Issues

1. **Missing Routes Between Tiers**
   - Check tier assignments
   - Verify connection definitions
   - Ensure routing is enabled

2. **Service Not Available**
   - Verify node tier and type
   - Check service prerequisites
   - Review firewall rules

3. **Configuration Not Generated**
   - Check event subscription
   - Verify event format
   - Review logs for errors

### Debug Mode

Enable detailed logging:

```rust
std::env::set_var("RUST_LOG", "cim_domain_nix::network=debug");
```

## Future Enhancements

- **IPv6 Support**: Full dual-stack configuration
- **SDN Integration**: OpenFlow/EVPN support
- **Cloud Integration**: AWS/Azure/GCP VPC mapping
- **Service Mesh**: Automatic Istio/Linkerd configuration
- **Network Policies**: Kubernetes NetworkPolicy generation