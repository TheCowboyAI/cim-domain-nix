# Converting Existing cim-domain-* Projects to Pure CT/FRP

**This guide helps you convert existing cim-domain-* projects to the standardized pure Category Theory and Functional Reactive Programming architecture with container deployment support.**

## Overview

The conversion process transforms your domain from any current architecture to:
- ✅ Pure functional event sourcing
- ✅ Category Theory compliance
- ✅ 100% FRP (zero side effects in domain logic)
- ✅ NATS microservice with JetStream
- ✅ Scalable container deployment (Proxmox LXC, NixOS, nix-darwin)

## Prerequisites

Before starting:
1. Backup your existing project
2. Create a new branch: `git checkout -b ct-frp-conversion`
3. Review `.claude/CIM_DOMAIN_TEMPLATE.md` for target architecture
4. Ensure you have `cim-domain` library available at `../cim-domain`

## Conversion Checklist

### Phase 1: Architecture Assessment

- [ ] Identify all domain aggregates
- [ ] List all commands (user intentions)
- [ ] List all events (things that happened)
- [ ] Identify value objects
- [ ] Map current state management approach
- [ ] Identify any CRUD operations (to be removed)
- [ ] Identify any side effects in domain logic (to be moved)

### Phase 2: Domain Model Conversion

#### Step 1: Remove CRUD Operations

**Before (CRUD):**
```rust
pub struct PersonRepository {
    db: Database,
}

impl PersonRepository {
    pub fn update(&self, person: &Person) -> Result<()> {
        self.db.update(person)?; // ❌ CRUD operation
        Ok(())
    }
}
```

**After (Event Sourcing):**
```rust
pub struct PersonRepository {
    event_store: Arc<dyn EventStore>,
}

impl PersonRepository {
    pub async fn save_events(
        &self,
        aggregate_id: PersonId,
        events: Vec<PersonEvent>,
    ) -> Result<()> {
        self.event_store.append_events(aggregate_id, events).await
    }
}
```

#### Step 2: Convert Aggregates to MealyStateMachine

**Before:**
```rust
pub struct Order {
    id: OrderId,
    status: OrderStatus,
    items: Vec<OrderItem>,
}

impl Order {
    pub fn add_item(&mut self, item: OrderItem) {
        self.items.push(item); // ❌ Mutable operation
    }
}
```

**After:**
```rust
use cim_domain::MealyStateMachine;

pub struct Order {
    id: OrderId,
    status: OrderStatus,
    items: Vec<OrderItem>,
    version: u64,
}

impl MealyStateMachine for Order {
    type State = OrderState;
    type Input = OrderCommand;
    type Output = Vec<OrderEvent>;

    fn output(&self, state: Self::State, input: Self::Input) -> Self::Output {
        match (state, input) {
            (OrderState::Open, OrderCommand::AddItem(cmd)) => {
                vec![OrderEvent::ItemAdded {
                    order_id: self.id,
                    item: cmd.item,
                    added_at: Utc::now(),
                }]
            }
            _ => vec![],
        }
    }

    fn transition(&self, state: Self::State, input: Self::Input) -> Self::State {
        match (state, input) {
            (OrderState::Open, OrderCommand::AddItem(_)) => OrderState::Open,
            _ => state,
        }
    }
}

impl Order {
    pub fn apply_event_pure(&self, event: &OrderEvent) -> Result<Self> {
        let mut new_order = self.clone();
        match event {
            OrderEvent::ItemAdded { item, .. } => {
                new_order.items.push(item.clone());
                new_order.version += 1;
            }
            // ... other events
        }
        Ok(new_order)
    }
}
```

#### Step 3: Define Pure Commands and Events

**Commands** (user intentions):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderCommand {
    CreateOrder(CreateOrder),
    AddItem(AddItem),
    RemoveItem(RemoveItem),
    PlaceOrder(PlaceOrder),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddItem {
    pub order_id: OrderId,
    pub item: OrderItem,
}
```

**Events** (immutable facts):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEvent {
    OrderCreated {
        order_id: OrderId,
        created_at: DateTime<Utc>,
    },
    ItemAdded {
        order_id: OrderId,
        item: OrderItem,
        added_at: DateTime<Utc>,
    },
    ItemRemoved {
        order_id: OrderId,
        item_id: ItemId,
        removed_at: DateTime<Utc>,
    },
    OrderPlaced {
        order_id: OrderId,
        placed_at: DateTime<Utc>,
    },
}
```

#### Step 4: Implement Category Theory Traits

```rust
use cim_domain::category_theory::{Functor, Monad};

// Functor: Structure-preserving transformation
impl Functor for OrderItem {
    type Morphism<A, B> = fn(A) -> B;

    fn fmap<A, B>(self, f: Self::Morphism<A, B>) -> Self
    where
        A: Clone,
        B: Clone,
    {
        // Preserve structure while transforming
        self
    }
}

// Monad: Compositional operations
impl Monad for Order {
    fn pure(value: Self) -> Self {
        value
    }

    fn bind<F, B>(self, f: F) -> B
    where
        F: FnOnce(Self) -> B,
    {
        f(self)
    }
}
```

### Phase 3: NATS Integration

#### Step 1: Add NATS Event Store

Create `src/infrastructure/nats_integration.rs`:

```rust
use async_nats::{Client, jetstream};
use cim_domain::DomainResult;

pub struct NatsOrderStore {
    jetstream: jetstream::Context,
    stream_name: String,
}

impl NatsOrderStore {
    pub async fn new(client: Client, stream_name: String) -> DomainResult<Self> {
        let jetstream = jetstream::new(client.clone());

        let stream_config = jetstream::stream::Config {
            name: stream_name.clone(),
            subjects: vec!["order.events.>".to_string()],
            retention: jetstream::stream::RetentionPolicy::Limits,
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        };

        // Create or get existing stream
        match jetstream.get_stream(&stream_name).await {
            Ok(_) => {},
            Err(_) => {
                jetstream.create_stream(stream_config).await.ok();
            }
        }

        Ok(Self { jetstream, stream_name })
    }

    pub async fn append_events(
        &self,
        aggregate_id: OrderId,
        events: Vec<OrderEvent>,
    ) -> DomainResult<()> {
        for event in events {
            let subject = format!("order.events.{}.{}", aggregate_id, event.event_type());
            let payload = serde_json::to_vec(&event)?;
            self.jetstream.publish(subject, payload.into()).await?;
        }
        Ok(())
    }
}
```

#### Step 2: Create Service Binary

Create `src/bin/order-service.rs`:

```rust
use std::sync::Arc;
use std::env;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let nats_url = env::var("NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());

    info!("Connecting to NATS at {}", nats_url);
    let client = async_nats::connect(&nats_url).await?;
    info!("✓ Connected to NATS");

    // Initialize domain components
    let event_store = Arc::new(NatsOrderStore::new(client.clone(), "ORDER_EVENTS".to_string()).await?);
    let repository = Arc::new(OrderRepository::new(event_store));

    // Start command handler
    info!("Service ready - listening on order.commands.>");

    // Command handling loop...

    Ok(())
}
```

### Phase 4: Container Deployment

#### Step 1: Update flake.nix

Replace entire `flake.nix` with template from `.claude/CIM_DOMAIN_TEMPLATE.md` section "Standard Flake Template", replacing `{name}` with your domain name.

#### Step 2: Create Container Module

Create `deployment/nix/container.nix` from template.

#### Step 3: Update Cargo.toml

Add service binary:
```toml
[[bin]]
name = "your-domain-service"
path = "src/bin/your-domain-service.rs"
```

### Phase 5: Testing Migration

#### Test 1: Pure Functions
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pure_event_application() {
        let order = Order::empty();
        let event = OrderEvent::ItemAdded { /* ... */ };

        // Pure function - no side effects
        let new_order = order.apply_event_pure(&event).unwrap();

        assert_eq!(new_order.items.len(), 1);
        assert_eq!(order.items.len(), 0); // Original unchanged
    }
}
```

#### Test 2: NATS Integration
```bash
# Start local NATS with JetStream
nats-server -js

# Run service
NATS_URL=nats://localhost:4222 cargo run --bin your-domain-service

# Test with NATS CLI
nats pub order.commands.create '{"CreateOrder": {...}}'
nats sub "order.events.>"
```

#### Test 3: Container Build
```bash
# Build service binary
nix build .#your-domain-service

# Build LXC container
nix build .#your-domain-lxc
```

## Common Conversion Patterns

### Pattern 1: Mutable State → Immutable Events

**Before:**
```rust
pub struct Account {
    balance: Decimal,
}

impl Account {
    pub fn deposit(&mut self, amount: Decimal) {
        self.balance += amount; // ❌ Mutable
    }
}
```

**After:**
```rust
pub struct Account {
    balance: Decimal,
    version: u64,
}

pub enum AccountEvent {
    MoneyDeposited {
        account_id: AccountId,
        amount: Decimal,
        deposited_at: DateTime<Utc>,
    },
}

impl Account {
    pub fn apply_event_pure(&self, event: &AccountEvent) -> Result<Self> {
        let mut new_account = self.clone();
        match event {
            AccountEvent::MoneyDeposited { amount, .. } => {
                new_account.balance += amount;
                new_account.version += 1;
            }
        }
        Ok(new_account)
    }
}
```

### Pattern 2: Database Operations → Event Store

**Before:**
```rust
pub fn save_order(&self, order: &Order) -> Result<()> {
    let query = "UPDATE orders SET status = $1 WHERE id = $2";
    self.db.execute(query, &[&order.status, &order.id])?; // ❌ CRUD
    Ok(())
}
```

**After:**
```rust
pub async fn save_order(
    &self,
    order: &Order,
    events: Vec<OrderEvent>,
) -> Result<()> {
    self.event_store.append_events(order.id, events).await?; // ✅ Event sourcing
    Ok(())
}
```

### Pattern 3: Side Effects → Infrastructure Layer

**Before:**
```rust
pub fn place_order(&mut self) -> Result<()> {
    self.status = OrderStatus::Placed;

    // ❌ Side effect in domain logic
    send_email(&self.customer_email, "Order placed")?;

    Ok(())
}
```

**After:**
```rust
// Pure domain logic
pub fn place_order(&self) -> Vec<OrderEvent> {
    vec![
        OrderEvent::OrderPlaced {
            order_id: self.id,
            placed_at: Utc::now(),
        },
        // Domain event that triggers infrastructure
        OrderEvent::OrderPlacedNotificationRequested {
            order_id: self.id,
            customer_email: self.customer_email.clone(),
        },
    ]
}

// Infrastructure layer (separate)
async fn handle_notification_requested(event: &OrderEvent) {
    if let OrderEvent::OrderPlacedNotificationRequested { customer_email, .. } = event {
        send_email(customer_email, "Order placed").await.ok();
    }
}
```

### Pattern 4: Direct Validation → Domain Events

**Before:**
```rust
pub fn set_quantity(&mut self, qty: u32) -> Result<()> {
    if qty > 100 {
        return Err(Error::QuantityTooHigh); // ❌ Throws error
    }
    self.quantity = qty;
    Ok(())
}
```

**After:**
```rust
pub fn set_quantity(&self, qty: u32) -> Vec<OrderEvent> {
    if qty > 100 {
        vec![OrderEvent::QuantityRejected {
            order_id: self.id,
            requested_qty: qty,
            reason: "Exceeds maximum of 100".to_string(),
            rejected_at: Utc::now(),
        }]
    } else {
        vec![OrderEvent::QuantitySet {
            order_id: self.id,
            quantity: qty,
            set_at: Utc::now(),
        }]
    }
}
```

## Validation Checklist

After conversion, verify:

### Domain Layer
- [ ] No mutable operations (`&mut self`)
- [ ] No CRUD operations
- [ ] No I/O operations (database, HTTP, file)
- [ ] All state changes via events
- [ ] Pure functions only
- [ ] MealyStateMachine implemented
- [ ] Category Theory traits implemented

### Events
- [ ] All events immutable
- [ ] Past tense naming (e.g., `OrderPlaced` not `PlaceOrder`)
- [ ] Include timestamp and relevant IDs
- [ ] Serializable (derive Serialize/Deserialize)

### NATS Integration
- [ ] Service binary created
- [ ] Event store implementation
- [ ] Command handler
- [ ] Subscribes to `{domain}.commands.>`
- [ ] Publishes to `{domain}.events.>`

### Deployment
- [ ] `flake.nix` updated with all outputs
- [ ] Container module created
- [ ] Can build: `nix build .#{domain}-lxc`
- [ ] Can run: `nix run .#{domain}-service`

### Testing
- [ ] Unit tests for pure functions
- [ ] Integration tests with NATS
- [ ] Examples demonstrate usage

## Migration Timeline

### Week 1: Assessment & Planning
- Day 1-2: Review current architecture
- Day 3-4: Identify all aggregates, commands, events
- Day 5: Create migration branch and backup

### Week 2: Core Domain Conversion
- Day 1-2: Convert main aggregate to pure functions
- Day 3-4: Define all commands and events
- Day 5: Implement MealyStateMachine

### Week 3: Infrastructure
- Day 1-2: Add NATS integration
- Day 3-4: Create service binary
- Day 5: Test with local NATS

### Week 4: Deployment & Testing
- Day 1-2: Add container configuration
- Day 3-4: Test container builds
- Day 5: Deploy to staging

## Troubleshooting

### Issue: "Cannot borrow as mutable"
**Cause**: Trying to use `&mut self`
**Fix**: Use pure functions returning new instances

### Issue: "Side effects in domain logic"
**Cause**: I/O operations in aggregate
**Fix**: Move to infrastructure layer, emit domain event instead

### Issue: "Complex state transitions"
**Cause**: Too many responsibilities in one aggregate
**Fix**: Split into multiple aggregates or use saga pattern

### Issue: "Lost data after conversion"
**Cause**: Missing event types
**Fix**: Ensure all state changes have corresponding events

## Support & Resources

- **Template**: `.claude/CIM_DOMAIN_TEMPLATE.md`
- **Reference Implementation**: `cim-domain-person`
- **Bootstrap Script**: `.claude/scripts/new-cim-domain.sh`
- **CIM Framework**: `../cim-domain`

## After Conversion

Once converted:

1. **Update documentation**:
   - README with new architecture
   - API documentation
   - Deployment guides

2. **Test thoroughly**:
   - Unit tests for pure functions
   - Integration tests with NATS
   - Container deployment tests

3. **Deploy incrementally**:
   - Start with single container
   - Scale to multiple replicas
   - Monitor and tune

4. **Share learnings**:
   - Document any custom patterns
   - Update this guide if needed
   - Help other domains convert

## Success Criteria

Your conversion is complete when:

- ✅ All domain logic is pure functions
- ✅ No CRUD operations remain
- ✅ All state changes via immutable events
- ✅ NATS service runs successfully
- ✅ Can build LXC container
- ✅ Can deploy to Proxmox
- ✅ Can scale horizontally
- ✅ All tests pass
- ✅ Documentation updated
- ✅ Zero compiler warnings

Congratulations! Your domain is now a pure CT/FRP NATS microservice ready for production scaling! 🎉
