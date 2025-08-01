# Nix Domain API Diagrams

## Complete API Overview

```mermaid
graph TB
    subgraph "API Entry Points"
        CMD[Commands]
        QRY[Queries]
        SVC[Services]
        PARS[Parser]
        ANLZ[Analyzer]
    end
    
    subgraph "Core Components"
        AGG[Aggregates]
        EVT[Events]
        VO[Value Objects]
        MI[MessageIdentity]
    end
    
    subgraph "Infrastructure"
        NATS[NATS Messaging]
        FS[File System]
        NIX[Nix CLI]
    end
    
    CMD --> MI
    CMD --> AGG
    AGG --> EVT
    EVT --> MI
    EVT --> NATS
    
    QRY --> AGG
    SVC --> CMD
    SVC --> QRY
    
    PARS --> FS
    ANLZ --> PARS
    
    AGG --> VO
    EVT --> VO
    
    %% CIM Standard Styling
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AGG fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style QRY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style SVC fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style VO fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style MI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NATS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style FS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style NIX fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style PARS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style ANLZ fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Command Hierarchy

```mermaid
graph LR
    subgraph "Base"
        NC[NixCommand trait]
        MI[MessageIdentity]
    end
    
    subgraph "Flake Commands"
        CF[CreateFlake]
        UF[UpdateFlake]
        AIF[AddFlakeInput]
    end
    
    subgraph "Build Commands"
        BP[BuildPackage]
    end
    
    subgraph "Module Commands"
        CM[CreateModule]
        CO[CreateOverlay]
    end
    
    subgraph "Config Commands"
        CC[CreateConfiguration]
        AC[ActivateConfiguration]
    end
    
    NC --> CF
    NC --> UF
    NC --> AIF
    NC --> BP
    NC --> CM
    NC --> CO
    NC --> CC
    NC --> AC
    
    MI --> CF
    MI --> UF
    MI --> AIF
    MI --> BP
    MI --> CM
    MI --> CO
    MI --> CC
    MI --> AC
    
    %% CIM Standard Styling
    style NC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style MI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CF fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style UF fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AIF fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style BP fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CM fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CO fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Event Hierarchy

```mermaid
graph LR
    subgraph "Base"
        NDE[NixDomainEvent trait]
        MI[MessageIdentity]
        EF[NixEventFactory]
    end
    
    subgraph "Flake Events"
        FC[FlakeCreated]
        FU[FlakeUpdated]
        FIA[FlakeInputAdded]
    end
    
    subgraph "Build Events"
        PB[PackageBuilt]
    end
    
    subgraph "Module Events"
        MC[ModuleCreated]
        OC[OverlayCreated]
    end
    
    subgraph "Config Events"
        CC[ConfigurationCreated]
        CA[ConfigurationActivated]
    end
    
    NDE --> FC
    NDE --> FU
    NDE --> FIA
    NDE --> PB
    NDE --> MC
    NDE --> OC
    NDE --> CC
    NDE --> CA
    
    MI --> FC
    MI --> FU
    MI --> FIA
    MI --> PB
    MI --> MC
    MI --> OC
    MI --> CC
    MI --> CA
    
    EF -.creates.-> FC
    EF -.creates.-> FU
    EF -.creates.-> FIA
    EF -.creates.-> PB
    EF -.creates.-> MC
    EF -.creates.-> OC
    EF -.creates.-> CC
    EF -.creates.-> CA
    
    %% CIM Standard Styling
    style NDE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style EF fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style FC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FU fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FIA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PB fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style OC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Value Objects

```mermaid
graph TB
    subgraph "Message Identity"
        MI[MessageIdentity]
        MID[MessageId]
        CID[CorrelationId]
        CAID[CausationId]
        MF[MessageFactory]
    end
    
    subgraph "Flake Types"
        FR[FlakeRef]
        FI[FlakeInputs]
        FO[FlakeOutputs]
        F[Flake]
    end
    
    subgraph "Path Types"
        AP[AttributePath]
        SP[StorePath]
    end
    
    subgraph "Config Types"
        NM[NixModule]
        OV[Overlay]
        NC[NixOSConfiguration]
        DRV[Derivation]
    end
    
    subgraph "Expression Types"
        NE[NixExpression]
        NV[NixValue]
    end
    
    MI --> MID
    MI --> CID
    MI --> CAID
    MF -.creates.-> MI
    
    F --> FI
    F --> FO
    F --> FR
    
    NC --> NM
    NC --> OV
    
    %% CIM Standard Styling
    %% Message Identity - Results (Light Green)
    style MI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style MID fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CID fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style CAID fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style MF fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Flake Types - Results (Light Green)
    style FR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style FI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style FO fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style F fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Path Types - Results (Light Green)
    style AP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style SP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Config Types - Results (Light Green)
    style NM fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style OV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DRV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Expression Types - Results (Light Green)
    style NE fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Service Layer

```mermaid
graph TB
    subgraph "Services"
        FS[FlakeService]
        BS[BuildService]
        CS[ConfigurationService]
    end
    
    subgraph "Service Operations"
        subgraph "FlakeService Methods"
            CF[create_flake]
            UF[update_flake]
            AI[add_input]
        end
        
        subgraph "BuildService Methods"
            BP[build_package]
            BC[build_configuration]
        end
        
        subgraph "ConfigurationService Methods"
            CC[create_configuration]
            AC[activate_configuration]
            VC[validate_configuration]
        end
    end
    
    subgraph "Dependencies"
        CMD[Commands]
        EVT[Events]
        AGG[Aggregates]
    end
    
    FS --> CF
    FS --> UF
    FS --> AI
    
    BS --> BP
    BS --> BC
    
    CS --> CC
    CS --> AC
    CS --> VC
    
    CF --> CMD
    UF --> CMD
    AI --> CMD
    BP --> CMD
    CC --> CMD
    AC --> CMD
    
    CMD --> AGG
    AGG --> EVT
    
    %% CIM Standard Styling
    %% Services - Choice/Query (Yellow)
    style FS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style BS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style CS fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Methods - Secondary (Teal)
    style CF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style UF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AI fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style BP fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style BC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style VC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Dependencies - Primary (Red)
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AGG fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Parser and Analyzer API

```mermaid
graph TB
    subgraph "Parser API"
        PF[parse_nix_file]
        PE[parse_nix_expr]
        AM[AstManipulator]
        subgraph "AST Operations"
            AA[add_attribute]
            MV[modify_value]
            RA[remove_attribute]
            TS[to_string]
        end
    end
    
    subgraph "Analyzer API"
        SA[SecurityAnalyzer]
        PA[PerformanceAnalyzer]
        DA[DeadCodeAnalyzer]
        DEP[DependencyAnalyzer]
        
        subgraph "Analysis Results"
            SI[SecurityIssue]
            PS[PerformanceSuggestion]
            DC[DeadCode]
            DG[DependencyGraph]
        end
    end
    
    subgraph "Infrastructure"
        RNIX[rnix Parser]
        AST[Syntax Tree]
        FS[File System]
    end
    
    PF --> RNIX
    PE --> RNIX
    RNIX --> AST
    AST --> AM
    
    AM --> AA
    AM --> MV
    AM --> RA
    AM --> TS
    
    SA --> AST
    PA --> AST
    DA --> AST
    DEP --> AST
    
    SA --> SI
    PA --> PS
    DA --> DC
    DEP --> DG
    
    PF --> FS
    
    %% CIM Standard Styling
    %% Parser - Secondary (Teal)
    style PF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AM fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% AST Operations - Secondary (Teal)
    style AA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MV fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style RA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style TS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Analyzers - Choice/Query (Yellow)
    style SA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style DA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style DEP fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    
    %% Results - Results (Light Green)
    style SI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PS fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DG fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    
    %% Infrastructure - Dark Gray
    style RNIX fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style AST fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style FS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
```

## Message Flow with Correlation/Causation

```mermaid
sequenceDiagram
    participant U as User
    participant C as Command
    participant H as Handler
    participant A as Aggregate
    participant E as Event
    participant N as NATS
    
    Note over U,N: Root Command (Self-Correlated)
    U->>C: CreateFlake<br/>correlation_id: ABC<br/>causation_id: ABC
    C->>H: Process Command
    H->>A: Validate & Execute
    A->>E: FlakeCreated<br/>correlation_id: ABC<br/>causation_id: ABC
    E->>N: Publish Event
    
    Note over U,N: Caused Command (Inherits Correlation)
    U->>C: BuildPackage<br/>correlation_id: ABC<br/>causation_id: FlakeCreated.id
    C->>H: Process Command
    H->>A: Validate & Execute
    A->>E: PackageBuilt<br/>correlation_id: ABC<br/>causation_id: BuildPackage.id
    E->>N: Publish Event
```

## Complete Workflow Example

```mermaid
graph TD
    subgraph "User Actions"
        U1[User creates flake]
        U2[User builds package]
        U3[User activates config]
    end
    
    subgraph "Commands"
        CF[CreateFlake<br/>correlation: A<br/>causation: A]
        BP[BuildPackage<br/>correlation: A<br/>causation: CF]
        AC[ActivateConfig<br/>correlation: A<br/>causation: BP]
    end
    
    subgraph "Events"
        FC[FlakeCreated<br/>correlation: A<br/>causation: A]
        PB[PackageBuilt<br/>correlation: A<br/>causation: CF]
        CA[ConfigActivated<br/>correlation: A<br/>causation: BP]
    end
    
    subgraph "Side Effects"
        FS[Flake on disk]
        PKG[Package in /nix/store]
        SYS[System reconfigured]
    end
    
    U1 --> CF
    CF --> FC
    FC --> FS
    
    U2 --> BP
    BP --> PB
    PB --> PKG
    
    U3 --> AC
    AC --> CA
    CA --> SYS
    
    FC -.triggers.-> U2
    PB -.triggers.-> U3
    
    %% CIM Standard Styling
    style U1 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style U2 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style U3 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    
    style CF fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style BP fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AC fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    style FC fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PB fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CA fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    style FS fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PKG fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style SYS fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```

## Error Handling Flow

```mermaid
graph TB
    subgraph "Error Sources"
        CMD[Command Validation]
        AGG[Aggregate Rules]
        NIX[Nix Operations]
        FS[File System]
        PARSE[Parser]
    end
    
    subgraph "Error Types"
        VE[ValidationError]
        NF[NotFound]
        BF[BuildFailed]
        PE[ParseError]
        NCF[NixCommandFailed]
    end
    
    subgraph "Error Handling"
        LOG[Log Error]
        METRIC[Record Metric]
        NOTIFY[Notify User]
        RETRY[Retry Logic]
        FALLBACK[Fallback Strategy]
    end
    
    CMD --> VE
    AGG --> VE
    NIX --> BF
    NIX --> NCF
    FS --> NF
    PARSE --> PE
    
    VE --> LOG
    NF --> LOG
    BF --> LOG
    PE --> LOG
    NCF --> LOG
    
    LOG --> METRIC
    METRIC --> NOTIFY
    
    BF --> RETRY
    NCF --> RETRY
    RETRY --> FALLBACK
    
    %% CIM Standard Styling
    %% Error Sources - Primary (Red)
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style AGG fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style NIX fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style FS fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style PARSE fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    
    %% Error Types - Secondary (Teal)
    style VE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style NF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style BF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style NCF fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    
    %% Error Handling - Choice (Yellow)
    style LOG fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style METRIC fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style NOTIFY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style RETRY fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style FALLBACK fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
```

## Future NATS Integration

```mermaid
graph TB
    subgraph "Current State"
        CMD1[Commands]
        EVT1[Events]
        LOCAL[Local Processing]
    end
    
    subgraph "Future State with NATS"
        CMD2[Commands]
        EVT2[Events]
        NATS[NATS Server]
        
        subgraph "NATS Features"
            PUBSUB[Pub/Sub]
            REQREP[Request/Reply]
            JETSTREAM[JetStream]
            KV[Key-Value Store]
        end
        
        subgraph "Distributed Services"
            S1[Nix Service 1]
            S2[Nix Service 2]
            S3[Nix Service N]
        end
    end
    
    CMD1 --> LOCAL
    LOCAL --> EVT1
    
    CMD2 --> NATS
    NATS --> S1
    NATS --> S2
    NATS --> S3
    
    S1 --> EVT2
    S2 --> EVT2
    S3 --> EVT2
    
    EVT2 --> NATS
    
    NATS --> PUBSUB
    NATS --> REQREP
    NATS --> JETSTREAM
    NATS --> KV
    
    %% CIM Standard Styling
    style CMD1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style CMD2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style EVT1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style EVT2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style LOCAL fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style NATS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style S1 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style S2 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style S3 fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PUBSUB fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style REQREP fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style JETSTREAM fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style KV fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```