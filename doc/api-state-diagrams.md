# Nix Domain API State Diagrams

## Flake Lifecycle

```mermaid
stateDiagram-v2
    [*] --> NonExistent
    NonExistent --> Created: CreateFlake
    Created --> Updated: UpdateFlake
    Created --> WithInputs: AddFlakeInput
    Updated --> WithInputs: AddFlakeInput
    WithInputs --> Updated: UpdateFlake
    WithInputs --> WithInputs: AddFlakeInput
    
    Created --> Built: BuildPackage
    Updated --> Built: BuildPackage
    WithInputs --> Built: BuildPackage
    
    Built --> Updated: UpdateFlake
    
    note right of Created
        FlakeCreated event
        - flake_id assigned
        - path established
        - template applied
    end note
    
    note right of Updated
        FlakeUpdated event
        - flake.lock updated
        - inputs resolved
    end note
    
    note right of WithInputs
        FlakeInputAdded event
        - new dependency added
        - flake.nix modified
    end note
    
    note right of Built
        PackageBuilt event
        - derivation created
        - store path allocated
    end note
```

## Configuration Lifecycle

```mermaid
stateDiagram-v2
    [*] --> NonExistent
    NonExistent --> Created: CreateConfiguration
    Created --> Activated: ActivateConfiguration(Switch)
    Created --> BootReady: ActivateConfiguration(Boot)
    Created --> Testing: ActivateConfiguration(Test)
    
    Activated --> Modified: UpdateConfiguration
    Modified --> Activated: ActivateConfiguration(Switch)
    
    Testing --> Activated: ActivateConfiguration(Switch)
    Testing --> Created: Rollback
    
    BootReady --> Activated: SystemReboot
    
    note right of Created
        ConfigurationCreated event
        - configuration_id assigned
        - modules loaded
        - overlays applied
    end note
    
    note right of Activated
        ConfigurationActivated event
        - generation number assigned
        - system reconfigured
        - services restarted
    end note
```

## Build Process State Machine

```mermaid
stateDiagram-v2
    [*] --> Queued: BuildPackage command
    Queued --> Evaluating: Start build
    Evaluating --> Fetching: Dependencies needed
    Evaluating --> Building: All deps available
    Fetching --> Building: Downloads complete
    Building --> Testing: Build success
    Building --> Failed: Build error
    Testing --> Completed: Tests pass
    Testing --> Failed: Tests fail
    Completed --> [*]: PackageBuilt event
    Failed --> [*]: BuildFailed error
    
    note right of Evaluating
        - Parse flake
        - Resolve attribute
        - Check cache
    end note
    
    note right of Fetching
        - Download sources
        - Fetch dependencies
        - Verify hashes
    end note
    
    note right of Building
        - Run build phases
        - Create derivation
        - Generate store path
    end note
```

## Command Processing Pipeline

```mermaid
graph LR
    subgraph "Input Stage"
        CMD[Command]
        MI[MessageIdentity]
        VAL{Validation}
    end
    
    subgraph "Processing Stage"
        AGG[Aggregate]
        BR{Business<br/>Rules}
        EVT[Events]
    end
    
    subgraph "Output Stage"
        STORE[Event Store]
        PROJ[Projections]
        NATS[NATS]
    end
    
    CMD --> MI
    MI --> VAL
    VAL -->|Valid| AGG
    VAL -->|Invalid| ERR1[ValidationError]
    
    AGG --> BR
    BR -->|Success| EVT
    BR -->|Failure| ERR2[DomainError]
    
    EVT --> STORE
    EVT --> PROJ
    EVT --> NATS
    
    %% CIM Standard Styling
    style CMD fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style MI fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style VAL fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style AGG fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style BR fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style EVT fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style STORE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PROJ fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style NATS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style ERR1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style ERR2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Parser State Flow

```mermaid
graph TD
    subgraph "Input"
        STR[Nix String]
        FILE[Nix File]
    end
    
    subgraph "Parsing"
        LEX[Lexer]
        PARSE[Parser]
        AST[AST]
        VALID{Validate}
    end
    
    subgraph "Manipulation"
        MANIP[Manipulator]
        ADD[Add Node]
        MOD[Modify Node]
        DEL[Delete Node]
    end
    
    subgraph "Output"
        FMT[Format]
        WRITE[Write File]
        RESULT[Result String]
    end
    
    STR --> LEX
    FILE --> LEX
    LEX --> PARSE
    PARSE --> AST
    AST --> VALID
    
    VALID -->|Valid| MANIP
    VALID -->|Invalid| PERR[ParseError]
    
    MANIP --> ADD
    MANIP --> MOD
    MANIP --> DEL
    
    ADD --> AST
    MOD --> AST
    DEL --> AST
    
    AST --> FMT
    FMT --> WRITE
    FMT --> RESULT
    
    %% CIM Standard Styling
    style STR fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style FILE fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style LEX fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PARSE fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AST fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style VALID fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style MANIP fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style ADD fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style MOD fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style DEL fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FMT fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style WRITE fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style RESULT fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PERR fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Analyzer Workflow

```mermaid
graph TB
    subgraph "Input Sources"
        F1[flake.nix]
        F2[*.nix files]
        F3[configuration.nix]
    end
    
    subgraph "Analysis Pipeline"
        subgraph "Parsing"
            P[Parser]
            AST[AST]
        end
        
        subgraph "Analyzers"
            SA[Security<br/>Analyzer]
            PA[Performance<br/>Analyzer]
            DA[DeadCode<br/>Analyzer]
            DEPA[Dependency<br/>Analyzer]
        end
        
        subgraph "Analysis Engine"
            RULES[Rule Engine]
            PATTERNS[Pattern Matcher]
            METRICS[Metrics Collector]
        end
    end
    
    subgraph "Results"
        SR[Security Report]
        PR[Performance Report]
        DR[Dead Code Report]
        DEPR[Dependency Graph]
        
        subgraph "Actions"
            FIX[Auto-fix]
            WARN[Warnings]
            ERR[Errors]
        end
    end
    
    F1 --> P
    F2 --> P
    F3 --> P
    
    P --> AST
    
    AST --> SA
    AST --> PA
    AST --> DA
    AST --> DEPA
    
    SA --> RULES
    PA --> PATTERNS
    DA --> METRICS
    DEPA --> PATTERNS
    
    RULES --> SR
    PATTERNS --> PR
    PATTERNS --> DEPR
    METRICS --> DR
    
    SR --> WARN
    SR --> ERR
    PR --> FIX
    PR --> WARN
    DR --> FIX
    
    %% CIM Standard Styling
    style F1 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style F2 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style F3 fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style P fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style AST fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style SA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style DA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style DEPA fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style RULES fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style PATTERNS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style METRICS fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style SR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style PR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style DEPR fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style FIX fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style WARN fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style ERR fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
```

## Correlation Chain Example

```mermaid
graph TD
    subgraph "User Story: Deploy New Service"
        U[User Action:<br/>Deploy Service X]
    end
    
    subgraph "Command Chain"
        C1[CreateFlake<br/>ID: A1<br/>Corr: R1<br/>Caus: R1]
        C2[AddFlakeInput<br/>ID: A2<br/>Corr: R1<br/>Caus: A1]
        C3[CreateModule<br/>ID: A3<br/>Corr: R1<br/>Caus: A2]
        C4[CreateConfiguration<br/>ID: A4<br/>Corr: R1<br/>Caus: A3]
        C5[BuildPackage<br/>ID: A5<br/>Corr: R1<br/>Caus: A4]
        C6[ActivateConfiguration<br/>ID: A6<br/>Corr: R1<br/>Caus: A5]
    end
    
    subgraph "Event Chain"
        E1[FlakeCreated<br/>Corr: R1<br/>Caus: R1]
        E2[FlakeInputAdded<br/>Corr: R1<br/>Caus: A1]
        E3[ModuleCreated<br/>Corr: R1<br/>Caus: A2]
        E4[ConfigurationCreated<br/>Corr: R1<br/>Caus: A3]
        E5[PackageBuilt<br/>Corr: R1<br/>Caus: A4]
        E6[ConfigurationActivated<br/>Corr: R1<br/>Caus: A5]
    end
    
    U --> C1
    C1 --> E1
    E1 --> C2
    C2 --> E2
    E2 --> C3
    C3 --> E3
    E3 --> C4
    C4 --> E4
    E4 --> C5
    C5 --> E5
    E5 --> C6
    C6 --> E6
    
    %% CIM Standard Styling
    style U fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style C1 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C2 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C3 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C4 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C5 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style C6 fill:#FF6B6B,stroke:#C92A2A,stroke-width:4px,color:#FFF
    style E1 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E2 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E3 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E4 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E5 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style E6 fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
```

## Service Dependencies

```mermaid
graph TB
    subgraph "External Dependencies"
        NIX[Nix CLI]
        FS[File System]
        GIT[Git]
        NET[Network]
    end
    
    subgraph "Core Services"
        FS_SVC[FlakeService]
        BS_SVC[BuildService]
        CS_SVC[ConfigurationService]
    end
    
    subgraph "Support Services"
        PARSER[Parser Service]
        ANALYZER[Analyzer Service]
        FORMATTER[Formatter Service]
    end
    
    subgraph "Infrastructure Services"
        CACHE[Cache Service]
        LOG[Logging Service]
        METRIC[Metrics Service]
    end
    
    FS_SVC --> NIX
    FS_SVC --> FS
    FS_SVC --> GIT
    FS_SVC --> PARSER
    FS_SVC --> FORMATTER
    
    BS_SVC --> NIX
    BS_SVC --> NET
    BS_SVC --> CACHE
    BS_SVC --> LOG
    
    CS_SVC --> NIX
    CS_SVC --> FS
    CS_SVC --> PARSER
    CS_SVC --> ANALYZER
    CS_SVC --> LOG
    
    PARSER --> FS
    ANALYZER --> PARSER
    FORMATTER --> NIX
    
    CACHE --> FS
    LOG --> FS
    METRIC --> LOG
    
    %% CIM Standard Styling
    style NIX fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style FS fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style GIT fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style NET fill:#2D3436,stroke:#000,stroke-width:3px,color:#FFF
    style FS_SVC fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style BS_SVC fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style CS_SVC fill:#FFE66D,stroke:#FCC419,stroke-width:3px,color:#000
    style PARSER fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style ANALYZER fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style FORMATTER fill:#4ECDC4,stroke:#2B8A89,stroke-width:3px,color:#FFF
    style CACHE fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style LOG fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
    style METRIC fill:#95E1D3,stroke:#63C7B8,stroke-width:2px,color:#000
```