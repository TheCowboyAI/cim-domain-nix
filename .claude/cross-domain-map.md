# Cross-Domain Integration Map

## Overview
The Person domain acts as a central hub for human-related data, integrating with multiple CIM domains to provide a complete view of individuals within the system.

## Domain Relationships

### Identity Domain
**Purpose**: Authentication and identity management

**Inbound Events**:
- `IdentityCreated` → Create/link person
- `IdentityVerified` → Update person verification status
- `IdentityRevoked` → Suspend person

**Outbound Events**:
- `PersonCreated` → Notify identity system
- `PersonSuspended` → Update identity status
- `PersonArchived` → Handle identity cleanup

**Commands**:
```rust
LinkPersonIdentity { person_id: PersonId, identity_id: IdentityId }
UnlinkPersonIdentity { person_id: PersonId, identity_id: IdentityId }
```

### Location Domain
**Purpose**: Track person-location associations

**Inbound Events**:
- `LocationCreated` → Available for assignment
- `LocationDeactivated` → Remove associations

**Outbound Events**:
- `PersonLocationAssigned` → Update location occupancy
- `PersonLocationRemoved` → Free location

**Commands**:
```rust
AssignPersonLocation { person_id: PersonId, location_id: LocationId }
RemovePersonLocation { person_id: PersonId, location_id: LocationId }
UpdatePersonLocation { person_id: PersonId, old_location: LocationId, new_location: LocationId }
```

### Organization Domain
**Purpose**: Employment and organizational relationships

**Inbound Events**:
- `OrganizationCreated` → Available for employment
- `RoleCreated` → New position available
- `OrganizationDissolved` → End all employments

**Outbound Events**:
- `PersonEmploymentAdded` → New employee
- `PersonEmploymentEnded` → Employee departed
- `PersonRoleChanged` → Position update

**Commands**:
```rust
AddPersonEmployment { person_id: PersonId, org_id: OrganizationId, role: String }
EndPersonEmployment { person_id: PersonId, org_id: OrganizationId }
UpdatePersonRole { person_id: PersonId, org_id: OrganizationId, new_role: String }
```

### Git Domain
**Purpose**: Development activity tracking

**Inbound Events**:
- `CommitCreated` → Track contribution
- `PullRequestMerged` → Update skills
- `RepositoryForked` → Note interest

**Outbound Events**:
- `DeveloperProfileUpdated` → Skill changes
- `ContributionRecorded` → Activity logged

**Components**:
```rust
GitProfile {
    username: String,
    repositories: Vec<RepositoryId>,
    contribution_stats: ContributionStats,
}
```

### Agent Domain
**Purpose**: AI agent capabilities and interactions

**Inbound Events**:
- `AgentAssigned` → Link to person
- `AgentTaskCompleted` → Update person's task history
- `AgentLearned` → Sync knowledge

**Outbound Events**:
- `PersonPreferencesUpdated` → Adjust agent behavior
- `PersonSkillsUpdated` → Update agent capabilities

**Commands**:
```rust
AssignAgentToPerson { person_id: PersonId, agent_id: AgentId }
UpdateAgentPermissions { person_id: PersonId, agent_id: AgentId, permissions: Permissions }
```

## Integration Patterns

### Event Flow Example
```
Organization Domain              Person Domain                Location Domain
        |                             |                              |
        |--RoleCreated--------------->|                              |
        |                             |                              |
        |<--PersonEmploymentAdded-----|                              |
        |                             |                              |
        |                             |--PersonLocationAssigned----->|
        |                             |                              |
```

### Data Synchronization
1. **Event-Driven**: Primary mechanism for cross-domain updates
2. **Query Federation**: Read models can aggregate data from multiple domains
3. **Saga Patterns**: Complex workflows spanning domains

### Consistency Boundaries
- Each domain maintains its own consistency
- Use eventual consistency between domains
- Implement compensating actions for failures

## Implementation Guidelines

### Handler Structure
```rust
// In cross_domain/handlers/organization.rs
pub async fn handle_organization_events(
    event: OrganizationEvent,
    person_repo: &PersonRepository,
    command_bus: &CommandBus,
) -> Result<()> {
    match event {
        OrganizationEvent::RoleCreated { org_id, role } => {
            // Handle new role availability
        }
        // ...
    }
}
```

### Testing Cross-Domain Flows
- Use integration tests for complete flows
- Mock external domain events
- Verify both inbound and outbound events
- Test failure scenarios and compensations