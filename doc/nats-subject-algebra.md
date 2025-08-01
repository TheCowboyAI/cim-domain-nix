# NATS Subject Algebra for Nix Domain

## Subject Structure

The Nix domain uses a hierarchical subject structure following the algebra:

```
Subject = Domain × MessageType × Aggregate × Action
```

Where:
- `Domain` = { nix }
- `MessageType` = { cmd, event, query }
- `Aggregate` = { flake, package, module, overlay, config }
- `Action` = varies by message type and aggregate

## Subject Hierarchy Visualization

```mermaid
graph TD
    subgraph "Subject Root"
        NIX[nix]
    end
    
    subgraph "Message Types"
        CMD[cmd]
        EVENT[event]
        QUERY[query]
    end
    
    subgraph "Aggregates"
        FLAKE[flake]
        PACKAGE[package]
        MODULE[module]
        OVERLAY[overlay]
        CONFIG[config]
    end
    
    subgraph "Command Actions"
        C_CREATE[create]
        C_UPDATE[update]
        C_DELETE[delete]
        C_BUILD[build]
        C_ACTIVATE[activate]
        C_ADD_INPUT[add_input]
        C_REMOVE_INPUT[remove_input]
        C_CACHE[cache]
        C_ROLLBACK[rollback]
    end
    
    subgraph "Event Actions"
        E_CREATED[created]
        E_UPDATED[updated]
        E_DELETED[deleted]
        E_BUILT[built]
        E_ACTIVATED[activated]
        E_INPUT_ADDED[input_added]
        E_INPUT_REMOVED[input_removed]
        E_CACHED[cached]
        E_ROLLED_BACK[rolled_back]
    end
    
    subgraph "Query Actions"
        Q_GET[get]
        Q_LIST[list]
        Q_GET_INPUTS[get_inputs]
        Q_GET_STATUS[get_status]
        Q_GET_CURRENT[get_current]
        Q_GET_HISTORY[get_history]
    end
    
    NIX --> CMD
    NIX --> EVENT
    NIX --> QUERY
    
    CMD --> FLAKE
    CMD --> PACKAGE
    CMD --> MODULE
    CMD --> OVERLAY
    CMD --> CONFIG
    
    EVENT --> FLAKE
    EVENT --> PACKAGE
    EVENT --> MODULE
    EVENT --> OVERLAY
    EVENT --> CONFIG
    
    QUERY --> FLAKE
    QUERY --> PACKAGE
    QUERY --> MODULE
    QUERY --> OVERLAY
    QUERY --> CONFIG
    
    %% CIM Standard Styling
    style NIX fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVENT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style QUERY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Command Subject Tree

```mermaid
graph LR
    subgraph "Command Subjects"
        NIX_CMD[nix.cmd]
        
        subgraph "Flake Commands"
            FCM[nix.cmd.flake]
            FCM --> FC1[nix.cmd.flake.create]
            FCM --> FC2[nix.cmd.flake.update]
            FCM --> FC3[nix.cmd.flake.add_input]
            FCM --> FC4[nix.cmd.flake.remove_input]
        end
        
        subgraph "Package Commands"
            PCM[nix.cmd.package]
            PCM --> PC1[nix.cmd.package.build]
            PCM --> PC2[nix.cmd.package.cache]
        end
        
        subgraph "Module Commands"
            MCM[nix.cmd.module]
            MCM --> MC1[nix.cmd.module.create]
            MCM --> MC2[nix.cmd.module.update]
            MCM --> MC3[nix.cmd.module.delete]
        end
        
        subgraph "Overlay Commands"
            OCM[nix.cmd.overlay]
            OCM --> OC1[nix.cmd.overlay.create]
            OCM --> OC2[nix.cmd.overlay.update]
            OCM --> OC3[nix.cmd.overlay.delete]
        end
        
        subgraph "Config Commands"
            CCM[nix.cmd.config]
            CCM --> CC1[nix.cmd.config.create]
            CCM --> CC2[nix.cmd.config.update]
            CCM --> CC3[nix.cmd.config.activate]
            CCM --> CC4[nix.cmd.config.rollback]
        end
        
        NIX_CMD --> FCM
        NIX_CMD --> PCM
        NIX_CMD --> MCM
        NIX_CMD --> OCM
        NIX_CMD --> CCM
    end
    
    %% CIM Standard Styling
    style NIX_CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style FCM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style PCM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style MCM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style OCM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CCM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Event Subject Tree

```mermaid
graph LR
    subgraph "Event Subjects"
        NIX_EVT[nix.event]
        
        subgraph "Flake Events"
            FEV[nix.event.flake]
            FEV --> FE1[nix.event.flake.created]
            FEV --> FE2[nix.event.flake.updated]
            FEV --> FE3[nix.event.flake.input_added]
            FEV --> FE4[nix.event.flake.input_removed]
        end
        
        subgraph "Package Events"
            PEV[nix.event.package]
            PEV --> PE1[nix.event.package.built]
            PEV --> PE2[nix.event.package.cached]
        end
        
        subgraph "Module Events"
            MEV[nix.event.module]
            MEV --> ME1[nix.event.module.created]
            MEV --> ME2[nix.event.module.updated]
            MEV --> ME3[nix.event.module.deleted]
        end
        
        subgraph "Overlay Events"
            OEV[nix.event.overlay]
            OEV --> OE1[nix.event.overlay.created]
            OEV --> OE2[nix.event.overlay.updated]
            OEV --> OE3[nix.event.overlay.deleted]
        end
        
        subgraph "Config Events"
            CEV[nix.event.config]
            CEV --> CE1[nix.event.config.created]
            CEV --> CE2[nix.event.config.updated]
            CEV --> CE3[nix.event.config.activated]
            CEV --> CE4[nix.event.config.rolled_back]
        end
        
        NIX_EVT --> FEV
        NIX_EVT --> PEV
        NIX_EVT --> MEV
        NIX_EVT --> OEV
        NIX_EVT --> CEV
    end
    
    %% CIM Standard Styling
    style NIX_EVT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FEV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PEV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MEV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style OEV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CEV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Query Subject Tree

```mermaid
graph LR
    subgraph "Query Subjects"
        NIX_QRY[nix.query]
        
        subgraph "Flake Queries"
            FQR[nix.query.flake]
            FQR --> FQ1[nix.query.flake.get]
            FQR --> FQ2[nix.query.flake.list]
            FQR --> FQ3[nix.query.flake.get_inputs]
        end
        
        subgraph "Package Queries"
            PQR[nix.query.package]
            PQR --> PQ1[nix.query.package.get]
            PQR --> PQ2[nix.query.package.list]
            PQR --> PQ3[nix.query.package.get_status]
        end
        
        subgraph "Module Queries"
            MQR[nix.query.module]
            MQR --> MQ1[nix.query.module.get]
            MQR --> MQ2[nix.query.module.list]
        end
        
        subgraph "Overlay Queries"
            OQR[nix.query.overlay]
            OQR --> OQ1[nix.query.overlay.get]
            OQR --> OQ2[nix.query.overlay.list]
        end
        
        subgraph "Config Queries"
            CQR[nix.query.config]
            CQR --> CQ1[nix.query.config.get]
            CQR --> CQ2[nix.query.config.list]
            CQR --> CQ3[nix.query.config.get_current]
            CQR --> CQ4[nix.query.config.get_history]
        end
        
        NIX_QRY --> FQR
        NIX_QRY --> PQR
        NIX_QRY --> MQR
        NIX_QRY --> OQR
        NIX_QRY --> CQR
    end
    
    %% CIM Standard Styling
    style NIX_QRY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style FQR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PQR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style MQR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style OQR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style CQR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Command-Event Mapping

```mermaid
graph LR
    subgraph "Commands"
        CMD1[nix.cmd.flake.create]
        CMD2[nix.cmd.package.build]
        CMD3[nix.cmd.config.activate]
    end
    
    subgraph "Events"
        EVT1[nix.event.flake.created]
        EVT2[nix.event.package.built]
        EVT3[nix.event.config.activated]
    end
    
    CMD1 ==>|produces| EVT1
    CMD2 ==>|produces| EVT2
    CMD3 ==>|produces| EVT3
    
    %% CIM Standard Styling
    style CMD1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CMD2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CMD3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVT1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style EVT2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style EVT3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Wildcard Subscription Patterns

```mermaid
graph TD
    subgraph "Subscription Patterns"
        W1[nix.>]
        W2[nix.cmd.>]
        W3[nix.event.>]
        W4[nix.query.>]
        W5[nix.*.flake.>]
        W6[nix.cmd.*.create]
        W7[nix.event.*.created]
    end
    
    subgraph "Matches"
        M1[All Nix messages]
        M2[All commands]
        M3[All events]
        M4[All queries]
        M5[All flake operations]
        M6[All create commands]
        M7[All created events]
    end
    
    W1 --> M1
    W2 --> M2
    W3 --> M3
    W4 --> M4
    W5 --> M5
    W6 --> M6
    W7 --> M7
    
    %% CIM Standard Styling
    style W1 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style W2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style W3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style W4 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Subject Algebra Set Theory

```mermaid
graph TB
    subgraph "Set Definitions"
        D[Domain = {nix}]
        MT[MessageType = {cmd, event, query}]
        A[Aggregate = {flake, package, module, overlay, config}]
        CA[CommandAction = {create, update, delete, build, ...}]
        EA[EventAction = {created, updated, deleted, built, ...}]
        QA[QueryAction = {get, list, get_inputs, ...}]
    end
    
    subgraph "Cartesian Products"
        CS[CommandSubject = Domain × {cmd} × Aggregate × CommandAction]
        ES[EventSubject = Domain × {event} × Aggregate × EventAction]
        QS[QuerySubject = Domain × {query} × Aggregate × QueryAction]
    end
    
    subgraph "Union"
        S[Subject = CommandSubject ∪ EventSubject ∪ QuerySubject]
    end
    
    D --> CS
    MT --> CS
    A --> CS
    CA --> CS
    
    D --> ES
    MT --> ES
    A --> ES
    EA --> ES
    
    D --> QS
    MT --> QS
    A --> QS
    QA --> QS
    
    CS --> S
    ES --> S
    QS --> S
    
    %% CIM Standard Styling
    style D fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style MT fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style A fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CA fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style QA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style CS fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style ES fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style QS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style S fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Subject Lifecycle Flow

```mermaid
sequenceDiagram
    participant U as User
    participant CMD as Command Subject
    participant H as Handler
    participant EVT as Event Subject
    participant S as Subscribers
    
    U->>CMD: Publish to nix.cmd.flake.create
    Note over CMD: Subject: nix.cmd.flake.create
    
    CMD->>H: Route to FlakeAggregate
    H->>H: Process CreateFlake
    H->>EVT: Emit FlakeCreated
    
    Note over EVT: Subject: nix.event.flake.created
    EVT->>S: Notify subscribers
    
    S->>S: Update projections
    S->>S: Trigger workflows
```

## Subject Validation Rules

```mermaid
graph TD
    subgraph "Validation Pipeline"
        S[Subject String]
        V1{Split by '.'}
        V2{Parts = 4?}
        V3{Domain = 'nix'?}
        V4{Valid MessageType?}
        V5{Valid Aggregate?}
        V6{Valid Action?}
        VS[Valid Subject]
        INV[Invalid Subject]
    end
    
    S --> V1
    V1 --> V2
    V2 -->|Yes| V3
    V2 -->|No| INV
    V3 -->|Yes| V4
    V3 -->|No| INV
    V4 -->|Yes| V5
    V4 -->|No| INV
    V5 -->|Yes| V6
    V5 -->|No| INV
    V6 -->|Yes| VS
    V6 -->|No| INV
    
    %% CIM Standard Styling
    style S fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style VS fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style INV fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Aggregate Subject Distribution

```mermaid
pie title Subject Distribution by Aggregate
    "Flake" : 11
    "Package" : 5
    "Module" : 8
    "Overlay" : 8
    "Configuration" : 14
```

## Subject Naming Consistency

```mermaid
graph LR
    subgraph "Naming Pattern"
        CMD_PATTERN[Command: {action}]
        EVT_PATTERN[Event: {action_past_tense}]
        QRY_PATTERN[Query: get_{resource}]
    end
    
    subgraph "Examples"
        C1[create → created]
        C2[update → updated]
        C3[build → built]
        C4[activate → activated]
        Q1[get, list, get_*]
    end
    
    CMD_PATTERN --> C1
    CMD_PATTERN --> C2
    CMD_PATTERN --> C3
    CMD_PATTERN --> C4
    
    EVT_PATTERN --> C1
    EVT_PATTERN --> C2
    EVT_PATTERN --> C3
    EVT_PATTERN --> C4
    
    QRY_PATTERN --> Q1
    
    %% CIM Standard Styling
    style CMD_PATTERN fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVT_PATTERN fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style QRY_PATTERN fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Subject Security Model

```mermaid
graph TD
    subgraph "Access Control"
        R1[Role: Admin]
        R2[Role: Developer]
        R3[Role: Reader]
    end
    
    subgraph "Subject Permissions"
        P1[nix.cmd.> - Write]
        P2[nix.event.> - Read]
        P3[nix.query.> - Read]
        P4[nix.cmd.config.activate - Restricted]
    end
    
    R1 --> P1
    R1 --> P2
    R1 --> P3
    R1 --> P4
    
    R2 --> P1
    R2 --> P2
    R2 --> P3
    
    R3 --> P2
    R3 --> P3
    
    %% CIM Standard Styling
    style R1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style R2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style R3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Summary

The Nix domain subject algebra provides:

1. **Structured Hierarchy**: Clear 4-part naming convention
2. **Type Safety**: Compile-time validation of subjects
3. **Discoverability**: Predictable patterns for all operations
4. **Flexibility**: Wildcard subscriptions for various use cases
5. **Consistency**: Command→Event mapping follows naming rules
6. **Security**: Subject-based access control capabilities

Total subjects: 46 (16 commands + 16 events + 14 queries)