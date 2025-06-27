# Nix API Documentation

## Overview

The Nix domain API provides commands, queries, and events for {domain purpose}.

## Commands

### CreateNix

Creates a new nix in the system.

```rust
use cim_domain_nix::commands::CreateNix;

let command = CreateNix {
    id: NixId::new(),
    // ... fields
};
```

**Fields:**
- `id`: Unique identifier for the nix
- `field1`: Description
- `field2`: Description

**Validation:**
- Field1 must be non-empty
- Field2 must be valid

**Events Emitted:**
- `NixCreated`

### UpdateNix

Updates an existing nix.

```rust
use cim_domain_nix::commands::UpdateNix;

let command = UpdateNix {
    id: entity_id,
    // ... fields to update
};
```

**Fields:**
- `id`: Identifier of the nix to update
- `field1`: New value (optional)

**Events Emitted:**
- `NixUpdated`

## Queries

### GetNixById

Retrieves a nix by its identifier.

```rust
use cim_domain_nix::queries::GetNixById;

let query = GetNixById {
    id: entity_id,
};
```

**Returns:** `Option<NixView>`

### List{Entities}

Lists all {entities} with optional filtering.

```rust
use cim_domain_nix::queries::List{Entities};

let query = List{Entities} {
    filter: Some(Filter {
        // ... filter criteria
    }),
    pagination: Some(Pagination {
        page: 1,
        per_page: 20,
    }),
};
```

**Returns:** `Vec<NixView>`

## Events

### NixCreated

Emitted when a new nix is created.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixCreated {
    pub id: NixId,
    pub timestamp: SystemTime,
    // ... other fields
}
```

### NixUpdated

Emitted when a nix is updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixUpdated {
    pub id: NixId,
    pub changes: Vec<FieldChange>,
    pub timestamp: SystemTime,
}
```

## Value Objects

### NixId

Unique identifier for {entities}.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NixId(Uuid);

impl NixId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### {ValueObject}

Represents {description}.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {ValueObject} {
    pub field1: String,
    pub field2: i32,
}
```

## Error Handling

The domain uses the following error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum NixError {
    #[error("nix not found: {id}")]
    NotFound { id: NixId },
    
    #[error("Invalid {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Operation not allowed: {reason}")]
    Forbidden { reason: String },
}
```

## Usage Examples

### Creating a New Nix

```rust
use cim_domain_nix::{
    commands::CreateNix,
    handlers::handle_create_nix,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = CreateNix {
        id: NixId::new(),
        name: "Example".to_string(),
        // ... other fields
    };
    
    let events = handle_create_nix(command).await?;
    
    for event in events {
        println!("Event emitted: {:?}", event);
    }
    
    Ok(())
}
```

### Querying {Entities}

```rust
use cim_domain_nix::{
    queries::{List{Entities}, execute_query},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = List{Entities} {
        filter: None,
        pagination: Some(Pagination {
            page: 1,
            per_page: 10,
        }),
    };
    
    let results = execute_query(query).await?;
    
    for item in results {
        println!("{:?}", item);
    }
    
    Ok(())
}
```

## Integration with Other Domains

This domain integrates with:

- **{Other Domain}**: Description of integration
- **{Other Domain}**: Description of integration

## Performance Considerations

- Commands are processed asynchronously
- Queries use indexed projections for fast retrieval
- Events are published to NATS for distribution

## Security Considerations

- All commands require authentication
- Authorization is enforced at the aggregate level
- Sensitive data is encrypted in events 