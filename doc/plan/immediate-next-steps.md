# Immediate Next Steps - Week 1

## Priority Order

### Day 1: Fix Current Build Issues
1. **Resolve dependency issues**
   ```bash
   # Check and fix cim-domain-git dependency
   cd ../cim-domain-git
   cargo check
   
   # Update Cargo.toml if needed
   # Consider using git dependencies or publishing to crates.io
   ```

2. **Run full test suite**
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

3. **Update progress.json with current git hash**

### Day 2-3: Start NATS Integration
1. **Create NATS module structure**
   ```bash
   mkdir -p src/nats
   touch src/nats/{mod.rs,client.rs,publisher.rs,config.rs,error.rs}
   ```

2. **Add NATS dependencies to Cargo.toml**

3. **Implement basic NATS client wrapper**

4. **Write unit tests for NATS client**

### Day 4-5: Event Publishing
1. **Implement EventPublisher**
   - Proper header mapping
   - Correlation/causation propagation
   - Error handling

2. **Update aggregates to use publisher**

3. **Create integration tests**

### Day 6-7: Command Subscription
1. **Implement CommandSubscriber**
   - Subject routing
   - Command deserialization
   - Response handling

2. **Update command handlers**

3. **Test command flow end-to-end**

## Checklist for Week 1

### Code Tasks
- [ ] Fix build issues
- [ ] Create NATS module structure
- [ ] Implement NATS client
- [ ] Implement event publisher
- [ ] Add NATS configuration
- [ ] Update aggregates for NATS
- [ ] Write 20+ NATS tests

### Documentation Tasks
- [ ] Update CLAUDE.md with NATS info
- [ ] Create NATS usage examples
- [ ] Update architecture diagrams
- [ ] Document configuration options

### Testing Tasks
- [ ] Unit tests for each NATS component
- [ ] Integration tests with test container
- [ ] Performance baseline tests
- [ ] Error scenario tests

## Definition of Done - Week 1

1. **NATS module compiles and passes tests**
2. **Events can be published to NATS**
3. **Commands can be received from NATS**
4. **All existing tests still pass**
5. **Documentation is updated**
6. **Progress tracked in progress.json**

## Quick Start Commands

```bash
# Fix dependencies
cargo update

# Add NATS deps
cargo add async-nats tokio-stream

# Run tests continuously
cargo watch -x test

# Check everything
cargo check && cargo test && cargo clippy && cargo fmt --check

# Update progress
./scripts/update-progress.sh  # Create if needed
```

## Risk Mitigation

1. **If NATS is complex**: Start with simple publish only
2. **If tests fail**: Fix incrementally, don't break existing
3. **If performance is poor**: Profile early, optimize later
4. **If integration is hard**: Use adapter pattern

## Daily Standup Questions

1. What did I complete yesterday?
2. What will I work on today?
3. What blockers do I have?
4. Is the timeline still realistic?

## Communication

- Update progress.json daily
- Commit working code frequently
- Document decisions in ADRs
- Ask for help when blocked