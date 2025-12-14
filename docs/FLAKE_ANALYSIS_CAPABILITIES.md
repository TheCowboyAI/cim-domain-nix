# Flake Analysis Capabilities

## Overview

`cim-domain-nix` provides **static analysis** of Nix flakes, extracting infrastructure information without full Nix evaluation. This document explains what we can and cannot extract from flakes.

## What We Successfully Extract

### ✅ Static Attributes

These are directly accessible in the flake attribute set:

#### 1. Description
```nix
{
  description = "CIM Domain Git - Git repository introspection...";
  # ✅ Successfully extracted
}
```

#### 2. Inputs
```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # ✅ Successfully extracted

    rust-overlay.url = "github:oxalica/rust-overlay";
    # ✅ Successfully extracted

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
      # ✅ Successfully extracted including 'follows'
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
      # ✅ Successfully extracted including 'flake = false'
    };
  };
}
```

**Extraction results:**
- Input names: nixpkgs, rust-overlay, flake-utils, crane, advisory-db
- URLs: Full GitHub references
- Relationships: `follows` directives
- Attributes: `flake = false` for non-flake inputs

## What Requires Evaluation

### ❌ Function Bodies

Flake outputs are typically functions, which we don't evaluate:

```nix
{
  outputs = { self, nixpkgs, ... }:
    # ❌ This is a FUNCTION - requires evaluation
    flake-utils.lib.eachDefaultSystem (system:
      # ❌ Nested function call - requires evaluation
      let
        # ❌ Let bindings - requires evaluation
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = /* ... */;
        # ❌ Inside function body - requires evaluation

        devShells.default = /* ... */;
        # ❌ Inside function body - requires evaluation

        checks = { /* ... */ };
        # ❌ Inside function body - requires evaluation
      }
    );
}
```

**What we cannot extract without evaluation:**
- Packages definitions
- DevShells definitions
- Checks definitions
- Apps definitions
- Any computed or derived values

## Why This Limitation Exists

### Static Analysis vs. Evaluation

**Static Analysis** (what we do):
- Parse Nix syntax → AST
- Convert AST → semantic values (NixValue)
- Extract top-level attributes
- No code execution

**Evaluation** (what we'd need):
- Execute Nix expressions
- Call functions
- Evaluate `let` bindings
- Import external modules
- Handle conditionals, list comprehensions, etc.

### Complexity of Nix Evaluation

To evaluate `outputs`, we'd need to:

1. **Implement a Nix evaluator**
   - Full language semantics
   - Function application
   - Lazy evaluation
   - Built-in functions

2. **Handle external dependencies**
   - `flake-utils.lib.eachDefaultSystem`
   - `import nixpkgs`
   - All transitive dependencies

3. **Provide evaluation context**
   - System architecture (x86_64-linux, etc.)
   - Current platform
   - Environment variables

4. **Manage side effects**
   - File system access
   - Network fetching
   - Derivation building

This is essentially reimplementing `nix eval`.

## What We Do Instead

### Focus on Infrastructure Discovery

We extract what we can and map it to Infrastructure domain:

```
Flake Inputs → Infrastructure Network
─────────────────────────────────────
nixpkgs         →  External dependency
rust-overlay    →  External dependency
flake-utils     →  External dependency
crane           →  External dependency
advisory-db     →  External dependency

Mapped to: Network "external-dependencies"
```

### Round-Trip Verification

We verify complete bidirectional conversion for the data we extract:

```
Original Flake (4631 bytes)
  ↓ extract inputs
Infrastructure (1 network, 5 dependency refs)
  ↓ project
NixTopology (1 network)
  ↓ serialize
Round-Trip Nix (81 bytes)
  ↓ parse
Verified NixTopology (1 network)
  ↓ map
Verified Infrastructure (1 network)
  ✅ MATCHES ORIGINAL
```

## Tested Flakes

### Simple Flake: cim-domain-nix
- **Size**: 3,073 bytes
- **Inputs**: 3 (nixpkgs, rust-overlay, flake-utils)
- **Extracted**: 3 inputs → 1 network
- **Round-trip**: ✅ Successful

### Complex Flake: cim-domain-git
- **Size**: 4,631 bytes
- **Inputs**: 5 (including follows and non-flake)
- **Extracted**: 5 inputs → 1 network
- **Complexity**: Handles `follows`, `flake = false`
- **Round-trip**: ✅ Successful

## Alternative Approaches

### Option 1: Use Nix CLI

If you need full flake information:

```bash
# Show flake structure (evaluates outputs)
nix flake show

# Show flake metadata
nix flake metadata

# Evaluate specific outputs
nix eval .#packages.x86_64-linux.default
```

### Option 2: Hybrid Approach

Combine our static analysis with Nix CLI:

```rust
// 1. Extract static info (our library)
let inputs = analyze_flake(&flake)?;

// 2. Get evaluated info (nix CLI)
let output = Command::new("nix")
    .args(["flake", "show", "--json"])
    .output()?;

// 3. Combine both for complete picture
let complete_analysis = merge(inputs, json::parse(output));
```

### Option 3: Future Enhancement

We could add limited evaluation:

```rust
// Detect common patterns
if outputs.is_function() {
    if let Some(result) = try_evaluate_flake_utils_pattern(outputs) {
        // Extract packages from well-known pattern
    }
}
```

This would handle common patterns without full evaluation.

## Current Capabilities Summary

| Feature | Static Analysis | Evaluation Required |
|---------|----------------|---------------------|
| Description | ✅ Yes | |
| Inputs | ✅ Yes | |
| Input URLs | ✅ Yes | |
| Input `follows` | ✅ Yes | |
| Input `flake = false` | ✅ Yes | |
| Packages | | ❌ Yes |
| DevShells | | ❌ Yes |
| Checks | | ❌ Yes |
| Apps | | ❌ Yes |
| Computed values | | ❌ Yes |

## Conclusion

Our static analysis successfully:
- ✅ Parses real-world flakes (tested with 3KB and 4.6KB flakes)
- ✅ Extracts dependency information (inputs)
- ✅ Handles complex input relationships
- ✅ Converts to Infrastructure domain
- ✅ Performs bidirectional round-trip
- ✅ Verifies functor properties

**Limitation**: We don't evaluate function bodies, so packages/devShells inside `outputs` aren't extracted.

**Workaround**: Use `nix flake show` for evaluated output, or combine our static analysis with Nix CLI calls.

**Status**: ✅ Production-ready for dependency analysis and round-trip conversion of extracted data.
