# CIM Domain Standardization - Complete Package

**This document describes the complete standardization package for all cim-domain-* projects to ensure consistency, scalability, and pure CT/FRP compliance.**

## 📦 What's Included

This standardization package provides everything needed to create or convert cim-domain-* services:

### 1. **CIM_DOMAIN_TEMPLATE.md** (Main Template)
Complete template instructions for any cim-domain-* project including:
- Pure CT/FRP architecture requirements
- Standard project structure
- Required dependencies
- Flake template for Nix builds
- NATS service binary template
- Container deployment configurations
- Implementation checklist
- Best practices and anti-patterns

### 2. **new-cim-domain.sh** (Bootstrap Script)
Automated script to create a new cim-domain-* project from scratch:
```bash
./.claude/scripts/new-cim-domain.sh order "Order management domain"
```

Creates complete project with:
- Directory structure
- Cargo.toml
- flake.nix with all outputs
- Service binary
- Container configurations
- Git repository initialized
- CLAUDE instructions

### 3. **CONVERSION_GUIDE.md** (Migration Guide)
Step-by-step guide for converting existing cim-domain-* projects:
- Assessment checklist
- Phase-by-phase conversion
- Code transformation patterns
- Common conversion scenarios
- Troubleshooting
- Validation checklist

### 4. **This Document** (Rollout Plan)
Overview and rollout strategy for the standardization effort.

## 🎯 Goals

### Primary Goals
1. **Consistency**: All cim-domain-* services follow same architecture
2. **Scalability**: Every service can scale horizontally in containers
3. **Purity**: 100% pure functional CT/FRP implementation
4. **Deployability**: Standard deployment across Proxmox LXC, NixOS, nix-darwin

### Technical Goals
- Pure functional event sourcing (no CRUD)
- Category Theory compliance
- NATS-based microservices
- JetStream event store
- Container-native deployment
- Horizontal scaling support

## 🏗️ Standard Architecture

Every cim-domain-* service will have:

```
┌──────────────────────────────────────────┐
│     NATS Cluster (Shared)                │
│     JetStream Event Storage              │
└──────────────────────────────────────────┘
           ▲ ▲ ▲
           │ │ │
     ┌─────┴─┴─┴─────┐
     │               │
  Container-1   Container-2   Container-3
  (Replica)     (Replica)     (Replica)

Each Container:
- Pure domain logic (CT/FRP)
- NATS command handler
- Event publisher
- Stateless (events in JetStream)
- Independently scalable
```

### Layers

1. **Domain Layer** (Pure)
   - Aggregates with MealyStateMachine
   - Commands (intentions)
   - Events (immutable facts)
   - Value objects
   - Category Theory traits

2. **Application Layer** (CQRS)
   - Command handlers
   - Query specifications
   - Pure projections
   - Service interfaces

3. **Infrastructure Layer** (I/O)
   - NATS integration
   - Event store
   - Persistence
   - External services

## 📋 Standardization Package Files

### Location
All standard files are in `.claude/` directory of `cim-domain-person`:

```
.claude/
├── CIM_DOMAIN_TEMPLATE.md         # Main template (copy to new projects)
├── CONVERSION_GUIDE.md            # Migration guide for existing projects
├── CIM_DOMAIN_STANDARDIZATION.md  # This document
└── scripts/
    └── new-cim-domain.sh          # Bootstrap script for new projects
```

### How to Use

#### For New Projects
```bash
# Copy bootstrap script
cp /path/to/cim-domain-person/.claude/scripts/new-cim-domain.sh .

# Run it
./new-cim-domain.sh invoice "Invoice generation and tracking"

# Result: Complete cim-domain-invoice project ready to implement
```

#### For Existing Projects
```bash
# Copy template and guide
cp /path/to/cim-domain-person/.claude/CIM_DOMAIN_TEMPLATE.md .claude/
cp /path/to/cim-domain-person/.claude/CONVERSION_GUIDE.md .claude/

# Follow conversion guide step by step
# Create branch first!
git checkout -b ct-frp-conversion
```

## 🚀 Rollout Strategy

### Phase 1: Template Finalization (✅ Complete)
- [x] Create CIM_DOMAIN_TEMPLATE.md
- [x] Create bootstrap script
- [x] Create conversion guide
- [x] Test with cim-domain-person
- [x] Document rollout strategy

### Phase 2: New Projects (Weeks 1-2)
Create new domains using bootstrap script:

**Priority 1: Core Business Domains**
1. cim-domain-order - Order management
2. cim-domain-invoice - Invoicing
3. cim-domain-payment - Payment processing
4. cim-domain-product - Product catalog

**Priority 2: Supporting Domains**
5. cim-domain-inventory - Inventory tracking
6. cim-domain-shipping - Shipping and logistics
7. cim-domain-customer - Customer management
8. cim-domain-supplier - Supplier management

**For each:**
```bash
./new-cim-domain.sh <domain-name> "<description>"
cd cim-domain-<domain-name>
# Implement domain logic
# Test with NATS
# Deploy container
```

### Phase 3: Existing Project Conversion (Weeks 3-6)
Convert existing cim-domain-* projects:

**Conversion Order (by dependency)**:
1. Lowest dependencies first (leaf nodes)
2. Core domains second
3. Integration domains last

**For each project:**
```bash
cd cim-domain-existing
git checkout -b ct-frp-conversion

# Follow CONVERSION_GUIDE.md
# Week 1: Assessment
# Week 2: Domain conversion
# Week 3: NATS integration
# Week 4: Testing & deployment

# Merge when complete
git checkout main
git merge ct-frp-conversion
```

### Phase 4: Validation & Refinement (Week 7-8)
- Validate all domains follow standard
- Ensure all can build containers
- Test horizontal scaling
- Document any custom patterns
- Update templates based on learnings

### Phase 5: Production Deployment (Week 9+)
- Deploy to staging environment
- Scale each domain to 3 replicas
- Monitor performance
- Roll to production incrementally
- Document operational procedures

## ✅ Compliance Checklist

Each cim-domain-* project must pass this checklist:

### Architecture
- [ ] Pure functional event sourcing (no CRUD)
- [ ] MealyStateMachine implemented for main aggregate
- [ ] All state changes via immutable events
- [ ] Category Theory traits (Functor, Monad)
- [ ] 100% FRP (no side effects in domain)
- [ ] Uses `Uuid::now_v7()` for all IDs

### Structure
- [ ] Standard directory layout
- [ ] Service binary in `src/bin/{domain}-service.rs`
- [ ] Commands in `src/commands/`
- [ ] Events in `src/events/`
- [ ] Aggregates in `src/aggregate/`
- [ ] Infrastructure in `src/infrastructure/`

### Dependencies
- [ ] `cim-domain` from `../cim-domain`
- [ ] `async-nats` for NATS integration
- [ ] Standard dependencies as per template

### NATS Integration
- [ ] Event store using JetStream
- [ ] Subscribes to `{domain}.commands.>`
- [ ] Publishes to `{domain}.events.>`
- [ ] Command handler implementation
- [ ] Repository pattern with snapshots

### Deployment
- [ ] `flake.nix` with all required outputs
- [ ] Container module (`deployment/nix/container.nix`)
- [ ] NixOS module (`deployment/nix/module.nix`)
- [ ] Can build: `nix build .#{domain}-service`
- [ ] Can build: `nix build .#{domain}-lxc`
- [ ] Systemd service configuration
- [ ] nix-darwin support

### Testing
- [ ] Unit tests for pure functions
- [ ] Integration tests with NATS
- [ ] Example usage in `examples/`
- [ ] All tests passing
- [ ] Zero compiler warnings

### Documentation
- [ ] README.md with architecture overview
- [ ] CHANGELOG.md following semantic versioning
- [ ] API documentation
- [ ] Deployment guides
- [ ] `.claude/CLAUDE.md` with project-specific instructions

### Quality
- [ ] Compiles with zero warnings
- [ ] All tests pass
- [ ] NATS service runs successfully
- [ ] Container builds successfully
- [ ] Can deploy to Proxmox
- [ ] Can scale horizontally

## 📊 Progress Tracking

Create a tracking spreadsheet for all cim-domain-* projects:

| Project | Status | Phase | Container Builds | Deployed | Notes |
|---------|--------|-------|-----------------|----------|-------|
| cim-domain-person | ✅ Complete | Reference | ✅ Yes | ✅ Yes | Template project |
| cim-domain-order | 🔄 Converting | 2 | ⏳ Pending | ❌ No | In progress |
| cim-domain-product | 📋 Planned | 1 | ❌ No | ❌ No | New project |
| ... | | | | | |

**Status Values:**
- ✅ Complete - Fully compliant
- 🔄 Converting - In conversion process
- 📋 Planned - Not started
- ⚠️ Blocked - Issues to resolve

## 🛠️ Tools & Resources

### Bootstrap New Project
```bash
./.claude/scripts/new-cim-domain.sh <name> "<description>"
```

### Build Service
```bash
nix build .#{domain}-service
```

### Run Locally
```bash
NATS_URL=nats://10.0.0.41:4222 cargo run --bin {domain}-service
```

### Build Container
```bash
nix build .#{domain}-lxc
```

### Deploy to Proxmox
```bash
scp result/*.tar.xz root@proxmox:/var/lib/vz/template/cache/
pct create <id> /var/lib/vz/template/cache/*.tar.xz \
  --hostname {domain}-service \
  --net0 name=eth0,bridge=vmbr0,ip=10.0.64.{id}/19,gw=10.0.64.1
```

## 🎓 Training & Onboarding

### For Developers

1. **Read Template**
   - Study CIM_DOMAIN_TEMPLATE.md
   - Understand CT/FRP principles
   - Review cim-domain-person as example

2. **Create Test Project**
   - Use bootstrap script
   - Implement simple domain (e.g., Todo)
   - Deploy locally

3. **Convert Existing Project**
   - Follow CONVERSION_GUIDE.md
   - Start with small domain
   - Get code review

### For Operations

1. **Understand Deployment**
   - Read CONTAINER_DEPLOYMENT.md
   - Practice building containers
   - Deploy to test Proxmox

2. **Learn Scaling**
   - Deploy 3 replicas
   - Test failover
   - Monitor NATS JetStream

3. **Operational Procedures**
   - Health checking
   - Log aggregation
   - Backup strategies

## 📈 Success Metrics

Track these metrics to measure standardization success:

### Technical Metrics
- Number of domains following standard
- Container build success rate
- Deployment success rate
- Test coverage percentage
- Zero-warning builds percentage

### Operational Metrics
- Service uptime
- NATS message throughput
- Event processing latency
- Container resource usage
- Scaling response time

### Development Metrics
- Time to create new domain
- Time to convert existing domain
- Developer satisfaction
- Code reuse percentage

## 🔄 Continuous Improvement

This standardization is living documentation:

### Review Cadence
- Monthly: Review compliance across all domains
- Quarterly: Update templates based on learnings
- Annually: Major architecture review

### Feedback Loop
1. Developers encounter issues
2. Document in template or guide
3. Update bootstrap script if needed
4. Share learnings across team
5. Update this documentation

### Version Control
- Track template versions
- Document breaking changes
- Provide migration paths
- Maintain changelog

## 🎯 End State Vision

When standardization is complete:

### All cim-domain-* Services
- ✅ Pure CT/FRP architecture
- ✅ NATS-based microservices
- ✅ Container-native deployment
- ✅ Horizontally scalable
- ✅ Declaratively deployable
- ✅ Fully tested
- ✅ Well documented

### Infrastructure
- Multiple NATS clusters (regional)
- JetStream replication
- Automated container deployment
- Monitoring and alerting
- Log aggregation
- Backup and disaster recovery

### Development Process
- Fast domain creation (< 1 hour)
- Smooth conversions (< 1 week)
- Consistent code reviews
- Automated testing
- CI/CD pipelines
- Production deployments

### Operational Excellence
- 99.9% uptime
- Sub-second event processing
- Elastic scaling
- Self-healing systems
- Observable infrastructure
- Rapid incident response

## 📞 Support

### Questions About
- **Template**: See CIM_DOMAIN_TEMPLATE.md
- **New Projects**: See bootstrap script comments
- **Conversion**: See CONVERSION_GUIDE.md
- **Deployment**: See deployment/CONTAINER_DEPLOYMENT.md

### Issues & Improvements
- Create issue in appropriate cim-domain-* repo
- For template issues: Use cim-domain-person repo
- Share learnings via team chat
- Update docs with solutions

## 🎉 Getting Started

Ready to standardize your domain?

### Option 1: New Domain
```bash
./.claude/scripts/new-cim-domain.sh your-domain "Your description"
```

### Option 2: Convert Existing
```bash
cp .claude/CONVERSION_GUIDE.md /path/to/your-domain/.claude/
# Follow the guide
```

### Option 3: Study Example
```bash
cd cim-domain-person
# Review the reference implementation
```

---

**This standardization package represents the culmination of best practices, lessons learned, and architectural patterns for building scalable, maintainable, pure functional microservices in the CIM ecosystem.**

Let's build something amazing! 🚀
