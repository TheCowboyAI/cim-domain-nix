# Flake Evaluation with Nix CLI

**Date**: 2025-11-13
**Status**: ✅ COMPLETE

## Overview

We now support **complete flake evaluation** using Nix's own evaluation engine. This extracts everything from flakes, including packages, devShells, checks, and apps that are defined inside the `outputs` function.

## Two Analysis Methods

### Method 1: Static Analysis (AST Parsing)
Fast, no dependencies, limited information:

```rust
use cim_domain_nix::nix::*;

// Parse flake.nix file
let content = std::fs::read_to_string("flake.nix")?;
let parser = NixParser::new();
let ast = parser.parse_str(&content)?;
let value = ast_to_value(&ast)?;

// Analyze statically
let analyzer = FlakeAnalyzer::new();
let analysis = analyzer.analyze(&value)?;

// Can extract:
// ✓ description
// ✓ inputs (with URLs, follows, etc.)
// ✗ packages (inside function)
// ✗ devShells (inside function)
```

### Method 2: Evaluated Analysis (Nix Evaluation)
Slower, requires Nix, complete information:

```rust
use cim_domain_nix::nix::*;

// Check if Nix is available
if !nix_available() {
    println!("Nix not installed");
    return;
}

// Evaluate flake with Nix
let evaluator = FlakeEvaluator::new();
let evaluated = evaluator.evaluate("/path/to/flake")?;

// Can extract:
// ✓ description
// ✓ inputs
// ✓ packages (all systems!)
// ✓ devShells (all systems!)
// ✓ checks (all systems!)
// ✓ apps (all systems!)
```

## Test Results: cim-domain-git

### Static Analysis Results
```
Inputs: 5 ✓
  • nixpkgs
  • rust-overlay
  • flake-utils
  • crane (with follows: "nixpkgs")
  • advisory-db (non-flake)

Packages: 0 (inside function - not extracted)
DevShells: 0 (inside function - not extracted)
```

### Evaluated Analysis Results
```
Systems: 4
  • x86_64-linux
  • aarch64-darwin
  • aarch64-linux
  • x86_64-darwin

Total Packages: 8 (2 per system)
  • default
  • cim-domain-git

Total DevShells: 4 (1 per system)
  • default

Total Checks: 20 (5 per system)
  • cim-domain-git
  • cim-domain-git-tests
  • cim-domain-git-clippy
  • cim-domain-git-fmt
  • cim-domain-git-audit

Total Apps: 4 (1 per system)
  • default
```

## API Documentation

### FlakeEvaluator

```rust
pub struct FlakeEvaluator {
    // Nix command to use (default: "nix")
}

impl FlakeEvaluator {
    /// Create a new flake evaluator
    pub fn new() -> Self;

    /// Create evaluator with custom nix command
    pub fn with_command(command: String) -> Self;

    /// Evaluate a flake at the given path
    pub fn evaluate<P: AsRef<Path>>(&self, flake_path: P)
        -> Result<EvaluatedFlake, EvaluationError>;

    /// Check if nix command is available
    pub fn is_available(&self) -> bool;
}
```

### EvaluatedFlake

```rust
pub struct EvaluatedFlake {
    /// Flake description
    pub description: Option<String>,

    /// Packages per system
    pub packages: HashMap<String, HashMap<String, PackageInfo>>,

    /// Development shells per system
    pub dev_shells: HashMap<String, HashMap<String, DevShellInfo>>,

    /// Checks per system
    pub checks: HashMap<String, HashMap<String, CheckInfo>>,

    /// Apps per system
    pub apps: HashMap<String, HashMap<String, AppInfo>>,
}
```

### Convenience Functions

```rust
/// Evaluate a flake at the given path
pub fn evaluate_flake<P: AsRef<Path>>(path: P)
    -> Result<EvaluatedFlake, EvaluationError>;

/// Check if Nix is available
pub fn nix_available() -> bool;
```

## Usage Examples

### Example 1: Check What's Available

```rust
use cim_domain_nix::nix::*;

if nix_available() {
    println!("Nix is available - can do full evaluation");
} else {
    println!("Nix not available - static analysis only");
}
```

### Example 2: Evaluate Current Directory

```rust
use cim_domain_nix::nix::*;

let evaluator = FlakeEvaluator::new();
match evaluator.evaluate(".") {
    Ok(flake) => {
        println!("Packages: {} systems", flake.packages.len());
        for (system, packages) in &flake.packages {
            println!("  {}: {} packages", system, packages.len());
        }
    }
    Err(e) => {
        println!("Evaluation failed: {}", e);
    }
}
```

### Example 3: Hybrid Approach

```rust
use cim_domain_nix::nix::*;

// Always do static analysis
let content = std::fs::read_to_string("flake.nix")?;
let parser = NixParser::new();
let ast = parser.parse_str(&content)?;
let value = ast_to_value(&ast)?;

let analyzer = FlakeAnalyzer::new();
let static_info = analyzer.analyze(&value)?;

println!("Inputs: {}", static_info.inputs.len());

// Optionally add evaluation if available
if nix_available() {
    let evaluator = FlakeEvaluator::new();
    if let Ok(evaluated) = evaluator.evaluate(".") {
        let total_packages: usize = evaluated.packages.values()
            .map(|m| m.len())
            .sum();
        println!("Packages: {}", total_packages);
    }
}
```

### Example 4: Extract Package Names

```rust
use cim_domain_nix::nix::*;

let evaluated = evaluate_flake(".")?;

for (system, packages) in &evaluated.packages {
    println!("System: {}", system);
    for (name, pkg) in packages {
        println!("  Package: {} (type: {})", name, pkg.pkg_type);
        if let Some(desc) = &pkg.description {
            println!("    Description: {}", desc);
        }
    }
}
```

## How It Works

### Under the Hood

When you call `evaluator.evaluate()`, it:

1. **Runs Nix CLI**
   ```bash
   nix flake show --json /path/to/flake
   ```

2. **Parses JSON Output**
   ```json
   {
     "packages": {
       "x86_64-linux": {
         "default": { "type": "derivation" },
         "cim-domain-git": { "type": "derivation" }
       }
     },
     "devShells": { ... },
     "checks": { ... },
     "apps": { ... }
   }
   ```

3. **Converts to Rust Structures**
   ```rust
   EvaluatedFlake {
       packages: HashMap<String, HashMap<String, PackageInfo>>,
       dev_shells: HashMap<String, HashMap<String, DevShellInfo>>,
       ...
   }
   ```

### Why This Approach?

**Advantages:**
- ✅ Uses Nix's official evaluator (accurate)
- ✅ Handles all Nix language features
- ✅ No need to reimplement Nix
- ✅ Gets lazy evaluation right
- ✅ Respects flake lock files
- ✅ Supports all systems/architectures

**Disadvantages:**
- ❌ Requires Nix installation
- ❌ Slower than static analysis (needs evaluation)
- ❌ Can't work offline without Nix

## Performance

| Operation | Time | Dependencies |
|-----------|------|--------------|
| Static Analysis | ~5ms | None |
| Nix Evaluation | ~2-5s | Nix installed |
| Hybrid (both) | ~2-5s | Nix optional |

## Error Handling

```rust
match evaluate_flake(".") {
    Ok(flake) => {
        // Success - use flake data
    }
    Err(EvaluationError::CommandFailed(msg)) => {
        // Nix command failed to execute
        println!("Command error: {}", msg);
    }
    Err(EvaluationError::NixError(msg)) => {
        // Nix evaluation error (invalid flake, etc.)
        println!("Nix error: {}", msg);
    }
    Err(EvaluationError::ParseError(msg)) => {
        // Failed to parse JSON output
        println!("Parse error: {}", msg);
    }
    Err(EvaluationError::NixNotAvailable) => {
        // Nix not installed
        println!("Install Nix from nixos.org");
    }
}
```

## Comparison: Static vs Evaluated

| Feature | Static Analysis | Evaluated Analysis |
|---------|----------------|-------------------|
| Speed | Fast (5ms) | Slower (2-5s) |
| Dependencies | None | Requires Nix |
| Description | ✅ Yes | ✅ Yes |
| Inputs | ✅ Yes | ✅ Yes |
| Packages | ❌ No (in function) | ✅ Yes |
| DevShells | ❌ No (in function) | ✅ Yes |
| Checks | ❌ No (in function) | ✅ Yes |
| Apps | ❌ No (in function) | ✅ Yes |
| Multi-system | ❌ N/A | ✅ Yes |
| Offline capable | ✅ Yes | ❌ No |

## Best Practices

### 1. Check Availability First

```rust
if nix_available() {
    // Full evaluation
} else {
    // Static analysis fallback
}
```

### 2. Handle Evaluation Errors

```rust
match evaluator.evaluate(path) {
    Ok(flake) => { /* use data */ }
    Err(e) => {
        eprintln!("Evaluation failed: {}", e);
        // Fall back to static analysis
    }
}
```

### 3. Cache Evaluation Results

Evaluation is slow - cache if calling multiple times:

```rust
lazy_static! {
    static ref FLAKE_CACHE: Mutex<HashMap<PathBuf, EvaluatedFlake>>
        = Mutex::new(HashMap::new());
}
```

### 4. Use for CI/CD

Perfect for build systems that need to know what packages exist:

```rust
let evaluated = evaluate_flake(".")?;
for (system, packages) in &evaluated.packages {
    for (name, _) in packages {
        println!("Build package {} for {}", name, system);
        // Trigger build
    }
}
```

## Conclusion

We now have **two complementary analysis methods**:

1. **Static Analysis**: Fast, no dependencies, extracts inputs
2. **Evaluated Analysis**: Slower, requires Nix, extracts everything

Together, they provide complete flake introspection capabilities:
- ✅ Parse any valid Nix flake
- ✅ Extract metadata without evaluation
- ✅ Evaluate to get complete package/shell/check/app info
- ✅ Handle errors gracefully
- ✅ Support offline operation (static only)
- ✅ Production-ready with comprehensive tests

**Status**: ✅ Complete and tested with real-world flakes
