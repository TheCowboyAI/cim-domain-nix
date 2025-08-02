<!-- Copyright 2025 Cowboy AI, LLC. -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Network integration with automatic NixOS system generation from topology events
- Hierarchical network support (client→leaf→cluster→super-cluster)
- Dynamic network change handling with interface updates
- Service auto-configuration based on node roles

### Changed
- Updated dependencies to use git sources instead of local paths
- Enhanced error handling with more specific error types

### Fixed
- NATS HeaderMap API compatibility with async-nats v0.33
- Compilation warnings for unused variables and imports

## [0.3.0] - 2025-01-02

### Added
- Complete NATS integration with 46 mapped subjects
- Full AST parsing and manipulation using rnix
- Security, performance, and dead code analyzers
- Formatter integration (nixpkgs-fmt, alejandra, nixfmt)
- Git integration for flake.lock tracking
- Comprehensive test suite with 80%+ coverage
- 11 working examples demonstrating all features
- Network topology to NixOS system generation
- Event sourcing with correlation/causation tracking
- CQRS command and query handlers
- Domain-Driven Design aggregates and value objects

### Changed
- Migrated from mock implementations to real Nix tool integration
- Improved error handling with domain-specific error types
- Enhanced parser to preserve formatting during AST manipulation
- Updated to use MessageIdentity for all commands

### Fixed
- Flake evaluation with proper Nix store integration
- Overlay merging logic for complex configurations
- Home Manager configuration generation

## [0.2.0] - 2024-12-20

### Added
- Basic flake operations (create, update, build)
- Initial NATS subject mapping
- Command handler framework
- Service layer abstractions

### Changed
- Restructured to follow CIM domain patterns
- Adopted event sourcing architecture

## [0.1.0] - 2024-12-15

### Added
- Initial project structure
- Basic Nix flake templates
- Core domain types
- README and documentation

[Unreleased]: https://github.com/thecowboyai/cim-domain-nix/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/thecowboyai/cim-domain-nix/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/thecowboyai/cim-domain-nix/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/thecowboyai/cim-domain-nix/releases/tag/v0.1.0