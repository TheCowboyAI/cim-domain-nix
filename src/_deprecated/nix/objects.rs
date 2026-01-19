// Copyright 2025 Cowboy AI, LLC.

//! Nix Objects - Higher-level Nix constructs
//!
//! This module defines the 7 Nix object types that represent complete,
//! meaningful Nix constructs. These are the **Objects** in our source
//! Category(Nix) that will be mapped to Infrastructure objects via the functor.
//!
//! ## Object Types
//!
//! 1. **NixAttrset** - Basic attribute set (the fundamental building block)
//! 2. **NixDerivation** - Build specification (.drv files)
//! 3. **NixPackage** - Installable software package
//! 4. **NixModule** - NixOS module with options/config/imports
//! 5. **NixOverlay** - Package set modification function
//! 6. **NixFlake** - Top-level composition with inputs/outputs/lock
//! 7. **NixApplication** - Executable program specification
//!
//! ## Category Theory Mapping
//!
//! ```text
//! NixFlake        → InfrastructureAggregate (top-level)
//! NixPackage      → SoftwareConfiguration
//! NixModule       → ComputeResource + configuration
//! NixDerivation   → Build specification metadata
//! NixOverlay      → Policy rules
//! NixApplication  → Deployed service
//! ```

use super::value_objects::NixAttrset;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// NixObject - Union type for all Nix object types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NixObject {
    /// Basic attribute set
    Attrset(NixAttrsetObject),
    /// Derivation (build specification)
    Derivation(NixDerivation),
    /// Package (installable software)
    Package(NixPackage),
    /// NixOS module
    Module(NixModule),
    /// Package overlay
    Overlay(NixOverlay),
    /// Flake (top-level composition)
    Flake(NixFlake),
    /// Application
    Application(NixApplication),
}

impl NixObject {
    /// Get the object type name
    pub fn type_name(&self) -> &'static str {
        match self {
            NixObject::Attrset(_) => "attrset",
            NixObject::Derivation(_) => "derivation",
            NixObject::Package(_) => "package",
            NixObject::Module(_) => "module",
            NixObject::Overlay(_) => "overlay",
            NixObject::Flake(_) => "flake",
            NixObject::Application(_) => "application",
        }
    }
}

// ============================================================================
// 1. NixAttrsetObject - Basic attribute set with metadata
// ============================================================================

/// Nix Attribute Set Object
///
/// A named attribute set with metadata. This is the fundamental building
/// block of all Nix expressions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixAttrsetObject {
    /// Unique identifier for this attrset
    pub id: Uuid,
    /// Optional name/identifier
    pub name: Option<String>,
    /// The attribute set data
    pub attrset: NixAttrset,
    /// Source file path if loaded from file
    pub source_path: Option<PathBuf>,
}

impl NixAttrsetObject {
    /// Create a new attribute set object
    pub fn new(attrset: NixAttrset) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: None,
            attrset,
            source_path: None,
        }
    }

    /// Create with name
    pub fn with_name(name: String, attrset: NixAttrset) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: Some(name),
            attrset,
            source_path: None,
        }
    }

    /// Set source path
    pub fn with_source_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }
}

// ============================================================================
// 2. NixDerivation - Build specification
// ============================================================================

/// Nix Derivation
///
/// A derivation is a build specification that describes how to build a
/// package. Derivations are stored in .drv files in the Nix store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixDerivation {
    /// Unique identifier
    pub id: Uuid,
    /// Derivation name
    pub name: String,
    /// Output paths (usually "out", but can have multiple outputs)
    pub outputs: HashMap<String, String>,
    /// Input derivations
    pub input_drvs: Vec<String>,
    /// Input source paths
    pub input_srcs: Vec<String>,
    /// System (e.g., "x86_64-linux")
    pub system: String,
    /// Builder (usually "/nix/store/.../bash")
    pub builder: String,
    /// Builder arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Store path (e.g., "/nix/store/abc-name.drv")
    pub drv_path: Option<String>,
}

impl NixDerivation {
    /// Create a new derivation
    pub fn new(name: String, system: String, builder: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            outputs: HashMap::from([("out".to_string(), String::new())]),
            input_drvs: Vec::new(),
            input_srcs: Vec::new(),
            system,
            builder,
            args: Vec::new(),
            env: HashMap::new(),
            drv_path: None,
        }
    }

    /// Add an output
    pub fn add_output(&mut self, name: String, path: String) {
        self.outputs.insert(name, path);
    }

    /// Add an input derivation
    pub fn add_input_drv(&mut self, drv_path: String) {
        self.input_drvs.push(drv_path);
    }

    /// Add an environment variable
    pub fn add_env(&mut self, key: String, value: String) {
        self.env.insert(key, value);
    }
}

// ============================================================================
// 3. NixPackage - Installable software
// ============================================================================

/// Nix Package
///
/// A package represents installable software. It's typically the result
/// of evaluating a derivation with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixPackage {
    /// Unique identifier
    pub id: Uuid,
    /// Package name (e.g., "hello")
    pub name: String,
    /// Package version (e.g., "2.10")
    pub version: Option<String>,
    /// Package description
    pub description: Option<String>,
    /// Output paths
    pub outputs: HashMap<String, String>,
    /// Package metadata
    pub meta: HashMap<String, String>,
    /// Underlying derivation path
    pub drv_path: Option<String>,
    /// System (e.g., "x86_64-linux")
    pub system: String,
}

impl NixPackage {
    /// Create a new package
    pub fn new(name: String, system: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            version: None,
            description: None,
            outputs: HashMap::from([("out".to_string(), String::new())]),
            meta: HashMap::new(),
            drv_path: None,
            system,
        }
    }

    /// Set version
    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Add metadata
    pub fn add_meta(&mut self, key: String, value: String) {
        self.meta.insert(key, value);
    }
}

// ============================================================================
// 4. NixModule - NixOS module
// ============================================================================

/// Nix Module
///
/// A NixOS module defines options, configuration, and imports. Modules
/// are the building blocks of NixOS system configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixModule {
    /// Unique identifier
    pub id: Uuid,
    /// Module name/path
    pub name: String,
    /// Imported modules
    pub imports: Vec<String>,
    /// Defined options
    pub options: HashMap<String, NixModuleOption>,
    /// Configuration values
    pub config: NixAttrset,
    /// Source file path
    pub source_path: Option<PathBuf>,
}

impl NixModule {
    /// Create a new module
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            imports: Vec::new(),
            options: HashMap::new(),
            config: NixAttrset::new(),
            source_path: None,
        }
    }

    /// Add an import
    pub fn add_import(&mut self, import: String) {
        self.imports.push(import);
    }

    /// Add an option
    pub fn add_option(&mut self, name: String, option: NixModuleOption) {
        self.options.insert(name, option);
    }

    /// Set source path
    pub fn with_source_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }
}

/// Module Option Definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixModuleOption {
    /// Option type (e.g., "bool", "str", "int", "attrs")
    pub option_type: String,
    /// Default value
    pub default: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Example value
    pub example: Option<String>,
}

impl NixModuleOption {
    /// Create a new option
    pub fn new(option_type: String) -> Self {
        Self {
            option_type,
            default: None,
            description: None,
            example: None,
        }
    }
}

// ============================================================================
// 5. NixOverlay - Package set modification
// ============================================================================

/// Nix Overlay
///
/// An overlay is a function that modifies the package set. Overlays are
/// used to customize packages or add new ones.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixOverlay {
    /// Unique identifier
    pub id: Uuid,
    /// Overlay name
    pub name: String,
    /// Modified packages
    pub modifications: Vec<String>,
    /// Source file path
    pub source_path: Option<PathBuf>,
}

impl NixOverlay {
    /// Create a new overlay
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            modifications: Vec::new(),
            source_path: None,
        }
    }

    /// Add a modification
    pub fn add_modification(&mut self, package_name: String) {
        self.modifications.push(package_name);
    }

    /// Set source path
    pub fn with_source_path(mut self, path: PathBuf) -> Self {
        self.source_path = Some(path);
        self
    }
}

// ============================================================================
// 6. NixFlake - Top-level composition
// ============================================================================

/// Nix Flake
///
/// A flake is a top-level Nix project with explicit inputs, outputs,
/// and a lock file. Flakes are the modern way to package Nix projects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixFlake {
    /// Unique identifier
    pub id: Uuid,
    /// Flake description
    pub description: String,
    /// Flake inputs (dependencies)
    pub inputs: HashMap<String, FlakeInput>,
    /// Flake outputs
    pub outputs: HashMap<String, String>,
    /// nixConfig settings
    pub nix_config: HashMap<String, String>,
    /// Flake directory path
    pub flake_path: PathBuf,
}

impl NixFlake {
    /// Create a new flake
    pub fn new(description: String, flake_path: PathBuf) -> Self {
        Self {
            id: Uuid::now_v7(),
            description,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            nix_config: HashMap::new(),
            flake_path,
        }
    }

    /// Add an input
    pub fn add_input(&mut self, name: String, input: FlakeInput) {
        self.inputs.insert(name, input);
    }

    /// Add an output
    pub fn add_output(&mut self, name: String, value: String) {
        self.outputs.insert(name, value);
    }

    /// Add nix config
    pub fn add_nix_config(&mut self, key: String, value: String) {
        self.nix_config.insert(key, value);
    }
}

/// Flake Input
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlakeInput {
    /// Input URL (e.g., "github:NixOS/nixpkgs")
    pub url: String,
    /// Follows another input
    pub follows: Option<String>,
    /// Whether this input is a flake
    pub flake: bool,
}

impl FlakeInput {
    /// Create a new flake input
    pub fn new(url: String) -> Self {
        Self {
            url,
            follows: None,
            flake: true,
        }
    }

    /// Create a non-flake input
    pub fn non_flake(url: String) -> Self {
        Self {
            url,
            follows: None,
            flake: false,
        }
    }

    /// Set follows
    pub fn with_follows(mut self, follows: String) -> Self {
        self.follows = Some(follows);
        self
    }
}

// ============================================================================
// 7. NixApplication - Executable program
// ============================================================================

/// Nix Application
///
/// An application represents an executable program that can be run.
/// Applications are specified in flake outputs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixApplication {
    /// Unique identifier
    pub id: Uuid,
    /// Application name
    pub name: String,
    /// Program path (the executable to run)
    pub program: String,
    /// Application type (e.g., "app", "defaultApp")
    pub app_type: String,
    /// System (e.g., "x86_64-linux")
    pub system: String,
}

impl NixApplication {
    /// Create a new application
    pub fn new(name: String, program: String, system: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name,
            program,
            app_type: "app".to_string(),
            system,
        }
    }

    /// Create default application
    pub fn default_app(program: String, system: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "default".to_string(),
            program,
            app_type: "defaultApp".to_string(),
            system,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attrset_object_creation() {
        let attrs = NixAttrset::new();
        let obj = NixAttrsetObject::new(attrs);
        assert!(obj.name.is_none());
        assert!(obj.source_path.is_none());
    }

    #[test]
    fn test_attrset_object_with_name() {
        let attrs = NixAttrset::new();
        let obj = NixAttrsetObject::with_name("test".to_string(), attrs);
        assert_eq!(obj.name, Some("test".to_string()));
    }

    #[test]
    fn test_derivation_creation() {
        let drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        assert_eq!(drv.name, "hello");
        assert_eq!(drv.system, "x86_64-linux");
        assert!(drv.outputs.contains_key("out"));
    }

    #[test]
    fn test_derivation_add_output() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.add_output("dev".to_string(), "/nix/store/dev".to_string());
        assert_eq!(drv.outputs.len(), 2);
        assert!(drv.outputs.contains_key("dev"));
    }

    #[test]
    fn test_package_creation() {
        let pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        assert_eq!(pkg.name, "hello");
        assert_eq!(pkg.system, "x86_64-linux");
        assert!(pkg.version.is_none());
    }

    #[test]
    fn test_package_with_version() {
        let pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string())
            .with_version("2.10".to_string());
        assert_eq!(pkg.version, Some("2.10".to_string()));
    }

    #[test]
    fn test_package_with_description() {
        let pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string())
            .with_description("Hello world program".to_string());
        assert_eq!(
            pkg.description,
            Some("Hello world program".to_string())
        );
    }

    #[test]
    fn test_module_creation() {
        let module = NixModule::new("services.nginx".to_string());
        assert_eq!(module.name, "services.nginx");
        assert!(module.imports.is_empty());
        assert!(module.options.is_empty());
    }

    #[test]
    fn test_module_add_import() {
        let mut module = NixModule::new("test".to_string());
        module.add_import("./other-module.nix".to_string());
        assert_eq!(module.imports.len(), 1);
    }

    #[test]
    fn test_module_add_option() {
        let mut module = NixModule::new("test".to_string());
        let option = NixModuleOption::new("bool".to_string());
        module.add_option("enable".to_string(), option);
        assert_eq!(module.options.len(), 1);
    }

    #[test]
    fn test_overlay_creation() {
        let overlay = NixOverlay::new("custom".to_string());
        assert_eq!(overlay.name, "custom");
        assert!(overlay.modifications.is_empty());
    }

    #[test]
    fn test_overlay_add_modification() {
        let mut overlay = NixOverlay::new("custom".to_string());
        overlay.add_modification("hello".to_string());
        assert_eq!(overlay.modifications.len(), 1);
    }

    #[test]
    fn test_flake_creation() {
        let flake = NixFlake::new(
            "Test flake".to_string(),
            PathBuf::from("/path/to/flake"),
        );
        assert_eq!(flake.description, "Test flake");
        assert!(flake.inputs.is_empty());
        assert!(flake.outputs.is_empty());
    }

    #[test]
    fn test_flake_add_input() {
        let mut flake = NixFlake::new(
            "Test flake".to_string(),
            PathBuf::from("/path/to/flake"),
        );
        let input = FlakeInput::new("github:NixOS/nixpkgs".to_string());
        flake.add_input("nixpkgs".to_string(), input);
        assert_eq!(flake.inputs.len(), 1);
    }

    #[test]
    fn test_flake_input_non_flake() {
        let input = FlakeInput::non_flake("path:/some/path".to_string());
        assert!(!input.flake);
    }

    #[test]
    fn test_flake_input_with_follows() {
        let input = FlakeInput::new("github:foo/bar".to_string())
            .with_follows("nixpkgs".to_string());
        assert_eq!(input.follows, Some("nixpkgs".to_string()));
    }

    #[test]
    fn test_application_creation() {
        let app = NixApplication::new(
            "hello".to_string(),
            "/nix/store/.../bin/hello".to_string(),
            "x86_64-linux".to_string(),
        );
        assert_eq!(app.name, "hello");
        assert_eq!(app.app_type, "app");
    }

    #[test]
    fn test_application_default() {
        let app = NixApplication::default_app(
            "/nix/store/.../bin/hello".to_string(),
            "x86_64-linux".to_string(),
        );
        assert_eq!(app.name, "default");
        assert_eq!(app.app_type, "defaultApp");
    }

    #[test]
    fn test_nix_object_type_name() {
        let pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        let obj = NixObject::Package(pkg);
        assert_eq!(obj.type_name(), "package");
    }

    // ============================================================================
    // Additional Tests for 90% Coverage
    // ============================================================================

    #[test]
    fn test_nix_object_attrset_type_name() {
        let obj = NixObject::Attrset(NixAttrsetObject::new(NixAttrset::new()));
        assert_eq!(obj.type_name(), "attrset");
    }

    #[test]
    fn test_nix_object_derivation_type_name() {
        let drv = NixDerivation::new(
            "test".to_string(),
            "x86_64-linux".to_string(),
            "/bin/bash".to_string(),
        );
        let obj = NixObject::Derivation(drv);
        assert_eq!(obj.type_name(), "derivation");
    }

    #[test]
    fn test_nix_object_module_type_name() {
        let module = NixModule::new("test".to_string());
        let obj = NixObject::Module(module);
        assert_eq!(obj.type_name(), "module");
    }

    #[test]
    fn test_nix_object_overlay_type_name() {
        let overlay = NixOverlay::new("test".to_string());
        let obj = NixObject::Overlay(overlay);
        assert_eq!(obj.type_name(), "overlay");
    }

    #[test]
    fn test_nix_object_flake_type_name() {
        let flake = NixFlake::new("test".to_string(), PathBuf::from("/test"));
        let obj = NixObject::Flake(flake);
        assert_eq!(obj.type_name(), "flake");
    }

    #[test]
    fn test_nix_object_application_type_name() {
        let app = NixApplication::new("test".to_string(), "/bin/test".to_string(), "x86_64-linux".to_string());
        let obj = NixObject::Application(app);
        assert_eq!(obj.type_name(), "application");
    }

    #[test]
    fn test_attrset_object_with_source_path() {
        let attrs = NixAttrset::new();
        let obj = NixAttrsetObject::new(attrs)
            .with_source_path(PathBuf::from("/test/path.nix"));
        assert_eq!(obj.source_path, Some(PathBuf::from("/test/path.nix")));
    }

    #[test]
    fn test_derivation_add_input_drv() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.add_input_drv("/nix/store/abc.drv".to_string());
        drv.add_input_drv("/nix/store/def.drv".to_string());
        assert_eq!(drv.input_drvs.len(), 2);
    }

    #[test]
    fn test_derivation_add_env() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.add_env("PATH".to_string(), "/bin".to_string());
        drv.add_env("HOME".to_string(), "/root".to_string());
        assert_eq!(drv.env.len(), 2);
        assert_eq!(drv.env.get("PATH"), Some(&"/bin".to_string()));
    }

    #[test]
    fn test_derivation_drv_path() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.drv_path = Some("/nix/store/abc-hello.drv".to_string());
        assert_eq!(drv.drv_path, Some("/nix/store/abc-hello.drv".to_string()));
    }

    #[test]
    fn test_package_add_meta() {
        let mut pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        pkg.add_meta("license".to_string(), "MIT".to_string());
        pkg.add_meta("homepage".to_string(), "https://example.com".to_string());
        assert_eq!(pkg.meta.len(), 2);
        assert_eq!(pkg.meta.get("license"), Some(&"MIT".to_string()));
    }

    #[test]
    fn test_module_with_source_path() {
        let module = NixModule::new("test".to_string())
            .with_source_path(PathBuf::from("/etc/nixos/test.nix"));
        assert_eq!(module.source_path, Some(PathBuf::from("/etc/nixos/test.nix")));
    }

    #[test]
    fn test_nix_module_option_new() {
        let option = NixModuleOption::new("str".to_string());
        assert_eq!(option.option_type, "str");
        assert!(option.default.is_none());
        assert!(option.description.is_none());
        assert!(option.example.is_none());
    }

    #[test]
    fn test_nix_module_option_with_all_fields() {
        let mut option = NixModuleOption::new("bool".to_string());
        option.default = Some("false".to_string());
        option.description = Some("Enable the service".to_string());
        option.example = Some("true".to_string());

        assert_eq!(option.default, Some("false".to_string()));
        assert_eq!(option.description, Some("Enable the service".to_string()));
        assert_eq!(option.example, Some("true".to_string()));
    }

    #[test]
    fn test_overlay_with_source_path() {
        let overlay = NixOverlay::new("custom".to_string())
            .with_source_path(PathBuf::from("/overlays/custom.nix"));
        assert_eq!(overlay.source_path, Some(PathBuf::from("/overlays/custom.nix")));
    }

    #[test]
    fn test_flake_add_output() {
        let mut flake = NixFlake::new(
            "Test flake".to_string(),
            PathBuf::from("/path/to/flake"),
        );
        flake.add_output("packages.x86_64-linux.default".to_string(), "hello".to_string());
        assert_eq!(flake.outputs.len(), 1);
        assert_eq!(
            flake.outputs.get("packages.x86_64-linux.default"),
            Some(&"hello".to_string())
        );
    }

    #[test]
    fn test_flake_add_nix_config() {
        let mut flake = NixFlake::new(
            "Test flake".to_string(),
            PathBuf::from("/path/to/flake"),
        );
        flake.add_nix_config("extra-substituters".to_string(), "https://cache.example.com".to_string());
        assert_eq!(flake.nix_config.len(), 1);
        assert_eq!(
            flake.nix_config.get("extra-substituters"),
            Some(&"https://cache.example.com".to_string())
        );
    }

    #[test]
    fn test_nix_object_serialization() {
        let pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string())
            .with_version("2.10".to_string());
        let obj = NixObject::Package(pkg);

        let serialized = serde_json::to_string(&obj).unwrap();
        assert!(serialized.contains("hello"));
        assert!(serialized.contains("2.10"));

        let deserialized: NixObject = serde_json::from_str(&serialized).unwrap();
        if let NixObject::Package(p) = deserialized {
            assert_eq!(p.name, "hello");
            assert_eq!(p.version, Some("2.10".to_string()));
        } else {
            panic!("Expected Package variant");
        }
    }

    #[test]
    fn test_flake_input_serialization() {
        let input = FlakeInput::new("github:NixOS/nixpkgs".to_string())
            .with_follows("nixpkgs".to_string());

        let serialized = serde_json::to_string(&input).unwrap();
        let deserialized: FlakeInput = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.url, "github:NixOS/nixpkgs");
        assert_eq!(deserialized.follows, Some("nixpkgs".to_string()));
        assert!(deserialized.flake);
    }

    #[test]
    fn test_derivation_input_srcs() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.input_srcs.push("/nix/store/src1".to_string());
        drv.input_srcs.push("/nix/store/src2".to_string());
        assert_eq!(drv.input_srcs.len(), 2);
    }

    #[test]
    fn test_derivation_args() {
        let mut drv = NixDerivation::new(
            "hello".to_string(),
            "x86_64-linux".to_string(),
            "/nix/store/.../bash".to_string(),
        );
        drv.args.push("-c".to_string());
        drv.args.push("build".to_string());
        assert_eq!(drv.args.len(), 2);
    }

    #[test]
    fn test_package_drv_path() {
        let mut pkg = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        pkg.drv_path = Some("/nix/store/abc-hello.drv".to_string());
        assert_eq!(pkg.drv_path, Some("/nix/store/abc-hello.drv".to_string()));
    }

    #[test]
    fn test_nix_object_equality() {
        let pkg1 = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        let pkg2 = NixPackage::new("hello".to_string(), "x86_64-linux".to_string());
        // IDs are UUIDs so they won't be equal, but structure is
        assert_eq!(pkg1.name, pkg2.name);
        assert_eq!(pkg1.system, pkg2.system);
    }
}
