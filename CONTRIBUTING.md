<!-- Copyright 2025 Cowboy AI, LLC. -->

# Contributing to CIM Domain Nix

Thank you for your interest in contributing to cim-domain-nix! This document provides guidelines and instructions for contributing.

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. All participants are expected to:

- Be respectful and considerate
- Welcome newcomers and help them get started
- Focus on what is best for the community
- Show empathy towards other community members

## How to Contribute

### Reporting Issues

Before creating an issue, please:

1. Check existing issues to avoid duplicates
2. Use the issue search feature
3. Include relevant information:
   - Rust version (`rustc --version`)
   - Nix version (`nix --version`)
   - Operating system
   - Minimal reproduction steps
   - Expected vs actual behavior
   - Error messages and stack traces

### Pull Request Process

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/cim-domain-nix.git
   cd cim-domain-nix
   ```

2. **Create a Branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/issue-description
   ```

3. **Make Changes**
   - Follow coding standards (see below)
   - Add tests for new functionality
   - Update documentation as needed
   - Ensure all tests pass

4. **Commit Changes**
   ```bash
   git add .
   git commit -m "feat: add new feature
   
   - Detailed description
   - Additional context
   
   Closes #123"
   ```

5. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a pull request on GitHub.

## Coding Standards

### Rust Code Style

1. **Format**: Use `cargo fmt` before committing
   ```bash
   cargo fmt --all
   ```

2. **Linting**: Ensure no clippy warnings
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Documentation**: Document all public APIs
   ```rust
   /// Creates a new Nix flake at the specified path.
   ///
   /// # Arguments
   /// * `name` - The flake name
   /// * `path` - Directory path for the flake
   ///
   /// # Example
   /// ```
   /// use cim_domain_nix::services::FlakeService;
   /// 
   /// let service = FlakeService::new();
   /// service.create_flake("my-project", "/tmp/project")?;
   /// ```
   pub fn create_flake(&self, name: &str, path: &Path) -> Result<()> {
       // ...
   }
   ```

4. **Error Handling**: Use the domain error type
   ```rust
   use crate::{Result, NixDomainError};
   
   pub fn risky_operation() -> Result<String> {
       something_that_might_fail()
           .map_err(|e| NixDomainError::Other(e.to_string()))?;
       Ok("success".to_string())
   }
   ```

5. **Async/Await**: Prefer async for I/O operations
   ```rust
   pub async fn fetch_data() -> Result<Data> {
       // ...
   }
   ```

### Domain-Driven Design Principles

1. **Aggregates**: Ensure consistency boundaries
2. **Value Objects**: Keep immutable
3. **Events**: Include correlation/causation IDs
4. **Commands**: Include MessageIdentity

### Event Sourcing Standards

All events must follow CIM standards:

```rust
pub struct FlakeCreated {
    pub flake_id: FlakeId,
    pub name: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    // From DomainEvent trait
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
}
```

## Testing Requirements

### Unit Tests

- Test individual components in isolation
- Use mocks for external dependencies
- Aim for >80% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flake_creation() {
        let flake = Flake::new("test", "/tmp/test");
        assert_eq!(flake.name(), "test");
    }
}
```

### Integration Tests

- Test complete workflows
- Use real Nix when available
- Test NATS integration with test containers

```rust
#[tokio::test]
async fn test_create_and_build_flake() {
    let service = FlakeService::new();
    let flake_id = service.create_flake("test", "/tmp/test").await?;
    let result = service.build_flake(flake_id).await?;
    assert!(result.success);
}
```

### Documentation Tests

- Ensure all examples in documentation compile
- Test example code in doc comments

## Documentation Requirements

1. **API Documentation**: All public items must be documented
2. **Examples**: Provide usage examples for complex features
3. **Architecture**: Update architecture docs for significant changes
4. **README**: Keep README.md up to date

## Development Setup

### Using Nix (Recommended)

```bash
# Enter development shell
nix develop

# Run tests
cargo test

# Run lints
cargo clippy
```

### Manual Setup

1. Install Rust: https://rustup.rs/
2. Install Nix: https://nixos.org/download.html
3. Enable flakes:
   ```bash
   echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
   ```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. After merge, tag release:
   ```bash
   git tag -s v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```

## Getting Help

- Open an issue for bugs or feature requests
- Join discussions in issues and PRs
- Check existing documentation
- Review examples in the `examples/` directory

## Recognition

Contributors will be recognized in:
- Release notes
- Contributors list
- Special thanks in documentation

Thank you for contributing to CIM Domain Nix!