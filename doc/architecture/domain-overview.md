# Nix Domain Architecture Overview

## Domain Structure

```mermaid
graph TB
    subgraph "Nix Domain"
        subgraph "Value Objects"
            FR[FlakeRef]
            AP[AttributePath]
            SP[StorePath]
            DV[Derivation]
            NM[NixModule]
            OV[Overlay]
            NC[NixOSConfiguration]
        end

        subgraph "Aggregates"
            FA[FlakeAggregate]
            MA[ModuleAggregate]
            OA[OverlayAggregate]
            CA[ConfigurationAggregate]
        end

        subgraph "Events"
            FC[FlakeCreated]
            FU[FlakeUpdated]
            FI[FlakeInputAdded]
            PB[PackageBuilt]
            MC[ModuleCreated]
            OC[OverlayCreated]
            CC[ConfigurationCreated]
            CAC[ConfigurationActivated]
        end

        subgraph "Commands"
            CFC[CreateFlake]
            UFC[UpdateFlake]
            AIC[AddInput]
            BPC[BuildPackage]
            CMC[CreateModule]
            COC[CreateOverlay]
            CCC[CreateConfiguration]
            ACC[ActivateConfiguration]
        end

        subgraph "Services"
            FS[FlakeService]
            BS[BuildService]
            CS[ConfigurationService]
        end

        subgraph "Infrastructure"
            NP[Nix Parser<br/>rnix]
            NF[Nix Formatters<br/>nixpkgs-fmt, alejandra]
            NA[Nix Analyzers<br/>Security, Performance]
            GI[Git Integration<br/>flake.lock tracking]
        end
    end

    %% Value Object relationships
    FA --> FR
    FA --> AP
    MA --> NM
    OA --> OV
    CA --> NC
    
    %% Command to Aggregate flow
    CFC --> FA
    UFC --> FA
    AIC --> FA
    BPC --> FA
    CMC --> MA
    COC --> OA
    CCC --> CA
    ACC --> CA
    
    %% Aggregate to Event flow
    FA --> FC
    FA --> FU
    FA --> FI
    FA --> PB
    MA --> MC
    OA --> OC
    CA --> CC
    CA --> CAC
    
    %% Services orchestration
    FS --> FA
    BS --> FA
    CS --> CA
    
    %% Infrastructure support
    FS --> NP
    FS --> NF
    BS --> NA
    CS --> GI

    %% CIM Standard High-Contrast Styling
    %% Value Objects - Results/Outcomes (Light Green)
    style FR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style AP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style SP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NM fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style OV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Aggregates - Primary/Core (Red)
    style FA fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style MA fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style OA fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CA fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Events - Secondary/Storage (Teal)
    style FC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FU fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FI fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PB fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style OC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CAC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Commands - Primary/Core (Red)
    style CFC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style UFC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AIC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style BPC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CMC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style COC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CCC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style ACC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Services - Choice/Query (Yellow)
    style FS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style BS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style CS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Infrastructure - Start/Root (Dark Gray)
    style NP fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style NF fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style NA fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style GI fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
```

## Event Flow

```mermaid
sequenceDiagram
    participant C as Client
    participant H as Command Handler
    participant A as Aggregate
    participant E as Event Store
    participant N as NATS

    C->>H: Command (with correlation_id)
    H->>A: Handle Command
    A->>A: Validate Business Rules
    A->>E: Generate Event
    Note over E: Event includes:<br/>- correlation_id<br/>- causation_id<br/>- aggregate_id
    E->>N: Publish Event
    N-->>C: Event Notification
```

## CQRS Pattern

```mermaid
graph LR
    subgraph "Write Side"
        CMD[Commands] --> CH[Command Handlers]
        CH --> AGG[Aggregates]
        AGG --> EVT[Events]
    end
    
    subgraph "Read Side"
        EVT --> PROJ[Projections]
        PROJ --> QH[Query Handlers]
        QH --> QRY[Queries]
    end
    
    EVT --> ES[Event Store]
    ES --> NATS[NATS Messaging]
    
    %% CIM Standard High-Contrast Styling
    %% Commands - Primary (Red)
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CH fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Aggregates - Primary Core (Red)
    style AGG fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Events - Secondary (Teal)
    style EVT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style ES fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Projections - Results (Light Green)
    style PROJ fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Queries - Choice/Query (Yellow)
    style QH fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style QRY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% NATS - Infrastructure (Dark Gray)
    style NATS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
```

## Parser Architecture

```mermaid
graph TB
    subgraph "Parser Module"
        NE[Nix Expression<br/>String] --> RN[rnix Parser]
        RN --> AST[Abstract Syntax Tree]
        AST --> MA[Manipulation API]
        MA --> NE2[Modified Expression]
        
        subgraph "Analysis"
            AST --> SA[Security Analyzer]
            AST --> PA[Performance Analyzer]
            AST --> DA[Dead Code Analyzer]
            
            SA --> SR[Security Report]
            PA --> PR[Performance Report]
            DA --> DR[Dead Code Report]
        end
    end
    
    %% CIM Standard High-Contrast Styling
    %% Input - Start/Root (Dark Gray)
    style NE fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    
    %% Parser - Primary Core (Red)
    style RN fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% AST - Secondary Processing (Teal)
    style AST fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Output - Results (Light Green)
    style NE2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Analyzers - Choice/Query (Yellow)
    style SA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style DA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Reports - Results (Light Green)
    style SR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Integration with CIM Ecosystem

```mermaid
graph TB
    subgraph "CIM Ecosystem"
        CD[cim-domain<br/>Core Types] --> NDX[cim-domain-nix]
        CS[cim-subject<br/>Event Routing] --> NDX
        CG[cim-domain-git<br/>Git Operations] --> NDX
        
        NDX --> NATS[NATS Messaging]
        
        subgraph "Event Subjects"
            NATS --> NS1[nix.flake.*]
            NATS --> NS2[nix.package.*]
            NATS --> NS3[nix.module.*]
            NATS --> NS4[nix.config.*]
        end
    end
    
    %% CIM Standard High-Contrast Styling
    %% Dependencies - Secondary (Teal)
    style CD fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CG fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Main Module - Primary Core (Red)
    style NDX fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% NATS - Infrastructure (Dark Gray)
    style NATS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    
    %% Event Subjects - Results (Light Green)
    style NS1 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NS2 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NS3 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NS4 fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Key Design Principles

1. **Event-Driven**: All state changes are events with correlation/causation IDs
2. **CQRS**: Separate command and query models
3. **Domain Isolation**: No shared state with other domains
4. **Parser Integration**: Full AST manipulation capabilities
5. **CIM Standards**: Follows all CIM patterns and conventions