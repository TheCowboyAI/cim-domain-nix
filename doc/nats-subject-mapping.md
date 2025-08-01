# NATS Subject Mapping for Nix Domain

## Overview

The Nix domain uses a comprehensive subject mapping system for NATS messaging that follows CIM conventions. All subjects follow the pattern:

```
{domain}.{message_type}.{aggregate}.{action}
```

Where:
- `domain` = `nix`
- `message_type` = `cmd` | `event` | `query`
- `aggregate` = `flake` | `package` | `module` | `overlay` | `config`
- `action` = specific action name

## Command Subjects

Commands represent actions to be performed:

| Command | NATS Subject |
|---------|--------------|
| `CreateFlake` | `nix.cmd.flake.create` |
| `UpdateFlake` | `nix.cmd.flake.update` |
| `AddFlakeInput` | `nix.cmd.flake.add_input` |
| `RemoveFlakeInput` | `nix.cmd.flake.remove_input` |
| `BuildPackage` | `nix.cmd.package.build` |
| `CachePackage` | `nix.cmd.package.cache` |
| `CreateModule` | `nix.cmd.module.create` |
| `UpdateModule` | `nix.cmd.module.update` |
| `DeleteModule` | `nix.cmd.module.delete` |
| `CreateOverlay` | `nix.cmd.overlay.create` |
| `UpdateOverlay` | `nix.cmd.overlay.update` |
| `DeleteOverlay` | `nix.cmd.overlay.delete` |
| `CreateConfiguration` | `nix.cmd.config.create` |
| `UpdateConfiguration` | `nix.cmd.config.update` |
| `ActivateConfiguration` | `nix.cmd.config.activate` |
| `RollbackConfiguration` | `nix.cmd.config.rollback` |

## Event Subjects

Events represent state changes (past tense):

| Event | NATS Subject |
|-------|--------------|
| `FlakeCreated` | `nix.event.flake.created` |
| `FlakeUpdated` | `nix.event.flake.updated` |
| `FlakeInputAdded` | `nix.event.flake.input_added` |
| `FlakeInputRemoved` | `nix.event.flake.input_removed` |
| `PackageBuilt` | `nix.event.package.built` |
| `PackageCached` | `nix.event.package.cached` |
| `ModuleCreated` | `nix.event.module.created` |
| `ModuleUpdated` | `nix.event.module.updated` |
| `ModuleDeleted` | `nix.event.module.deleted` |
| `OverlayCreated` | `nix.event.overlay.created` |
| `OverlayUpdated` | `nix.event.overlay.updated` |
| `OverlayDeleted` | `nix.event.overlay.deleted` |
| `ConfigurationCreated` | `nix.event.config.created` |
| `ConfigurationUpdated` | `nix.event.config.updated` |
| `ConfigurationActivated` | `nix.event.config.activated` |
| `ConfigurationRolledBack` | `nix.event.config.rolled_back` |

## Query Subjects

Queries for reading data:

| Query | NATS Subject |
|-------|--------------|
| `GetFlake` | `nix.query.flake.get` |
| `ListFlakes` | `nix.query.flake.list` |
| `GetFlakeInputs` | `nix.query.flake.get_inputs` |
| `GetPackage` | `nix.query.package.get` |
| `ListPackages` | `nix.query.package.list` |
| `GetBuildStatus` | `nix.query.package.get_status` |
| `GetModule` | `nix.query.module.get` |
| `ListModules` | `nix.query.module.list` |
| `GetOverlay` | `nix.query.overlay.get` |
| `ListOverlays` | `nix.query.overlay.list` |
| `GetConfiguration` | `nix.query.config.get` |
| `ListConfigurations` | `nix.query.config.list` |
| `GetCurrentConfiguration` | `nix.query.config.get_current` |
| `GetConfigurationHistory` | `nix.query.config.get_history` |

## Special Subjects

System-level subjects:

| Purpose | NATS Subject |
|---------|--------------|
| Health Check | `health.nix` |
| Service Discovery | `discovery.nix` |
| Metrics | `metrics.nix` |

## Wildcard Subscriptions

The subject structure supports wildcard subscriptions:

- `nix.>` - All Nix domain messages
- `nix.cmd.>` - All commands
- `nix.event.>` - All events
- `nix.query.>` - All queries
- `nix.*.flake.>` - All flake-related messages
- `nix.cmd.*.create` - All create commands

## Implementation

The subject mapping is implemented in `src/nats/subject.rs` with:

1. **Type-safe enums** for commands, events, and queries
2. **Automatic subject generation** from enum values
3. **Subject parsing** for incoming messages
4. **Validation** of subject format
5. **Helper methods** for getting all subjects of a type

### Usage Example

```rust
use cim_domain_nix::nats::{NixSubject, SubjectMapper};

// Create a command subject
let subject = NixSubject::command(CommandAction::CreateFlake);
assert_eq!(subject.to_string(), "nix.cmd.flake.create");

// Map from command type to subject
let subject = SubjectMapper::command_subject("CreateFlake").unwrap();
assert_eq!(subject.to_string(), "nix.cmd.flake.create");

// Parse an incoming subject
let parsed = NixSubject::parse("nix.event.flake.created").unwrap();

// Get all command subjects for subscription
let subjects = SubjectMapper::all_command_subjects();
```

## Benefits

1. **Type Safety**: Compile-time checking of subject validity
2. **Consistency**: All subjects follow the same pattern
3. **Discoverability**: Easy to find all subjects for a given type
4. **Flexibility**: Supports wildcard subscriptions
5. **Maintainability**: Adding new commands/events automatically updates subjects

## Visual Documentation

For visual representations of the subject algebra including:
- Hierarchical subject trees
- Command-Event mappings
- Wildcard subscription patterns
- Set theory representation
- Security models

See [NATS Subject Algebra](./nats-subject-algebra.md)