# CIM Domain Person Module Context

## Purpose
The `cim-domain-person` module is a core component of the Composable Information Machine (CIM) ecosystem, responsible for managing person entities and their associated data using an Entity Component System (ECS) architecture.

## Architecture Overview
- **Pattern**: ECS (Entity Component System) using Bevy
- **Core Design**: Minimal Person aggregate with composable components
- **Event Sourcing**: CQRS pattern with event-driven state management
- **Cross-Domain**: Integrates with Location, Organization, Identity, Git, and Agent domains

## Key Concepts
1. **Person Aggregate**: Minimal core containing only ID, name, and lifecycle state
2. **Components**: All other person data (contact, skills, preferences, social) managed as separate components
3. **Projections**: Multiple read models (Network, Search, Skills, Summary, Timeline)
4. **Events**: Domain events for state changes and cross-domain integration

## Module Structure
```
src/
├── aggregate/       # Core person entity and lifecycle
├── commands/        # Command handlers and validation
├── components/      # Person data components (contact, skills, etc.)
├── cross_domain/    # Integration with other CIM domains
├── events/          # Domain events and handlers
├── infrastructure/  # Storage, messaging, persistence
├── projections/     # Read models and views
├── queries/         # Query handlers
└── services/        # Business logic services
```

## Integration Points
- **Identity Domain**: Authentication and identity management
- **Location Domain**: Person-location associations
- **Organization Domain**: Employment and organizational relationships
- **Git Domain**: Development activity tracking
- **Agent Domain**: AI agent capabilities and interactions

## Development Focus
When working on this module:
1. Maintain clean separation between aggregate and components
2. Use event sourcing for all state changes
3. Ensure cross-domain events are properly handled
4. Follow ECS patterns for new features
5. Keep projections updated with domain changes