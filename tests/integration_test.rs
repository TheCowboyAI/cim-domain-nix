//! Integration tests for the Nix domain

use cim_domain_nix::{
    aggregate::ModuleAggregate,
    commands::CreateModule,
    events::{ConfigurationCreated, FlakeCreated, PackageBuilt},
    projections::{ConfigurationProjection, NixProjection, PackageBuildProjection},
    value_objects::*,
};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

// TODO: Update these tests to use the actual FlakeAggregate API

/*
#[tokio::test]
async fn test_flake_aggregate_creation() {
    let temp_dir = TempDir::new().unwrap();
    let flake_path = temp_dir.path().to_path_buf();

    // Create a new flake
    let cmd = CreateFlake {
        path: flake_path.clone(),
        description: "Test flake".to_string(),
        nixpkgs_follows: None,
    };

    let mut aggregate = FlakeAggregate::new();
    let events = aggregate.handle_create_flake(cmd).unwrap();

    assert_eq!(events.len(), 1);
    match &events[0] {
        NixDomainEvent::FlakeCreated { path, description, .. } => {
            assert_eq!(path, &flake_path);
            assert_eq!(description, "Test flake");
        }
        _ => panic!("Expected FlakeCreated event"),
    }
}
*/

#[test]
fn test_flake_projection() {
    let mut projection = NixProjection::default();

    // Create a flake created event
    let event = FlakeCreated {
        flake_id: Uuid::new_v4(),
        path: PathBuf::from("/tmp/test"),
        description: "Test flake".to_string(),
        template: None,
        timestamp: chrono::Utc::now(),
    };

    // Apply the event
    projection.flake_projection.handle_flake_created(&event);

    // Verify the projection was updated
    assert_eq!(projection.flake_projection.flakes.len(), 1);
    assert!(projection
        .flake_projection
        .flakes_by_path
        .contains_key(&PathBuf::from("/tmp/test")));
}

#[test]
fn test_module_aggregate() {
    let module = NixModule {
        id: Uuid::new_v4(),
        name: "test-module".to_string(),
        options: HashMap::new(),
        config: serde_json::json!({}),
        imports: vec![],
    };

    let cmd = CreateModule {
        command_id: Uuid::new_v4(),
        name: "test-module".to_string(),
        module: module.clone(),
    };

    let result = ModuleAggregate::handle_create_module(cmd);
    assert!(result.is_ok());

    let (aggregate, events) = result.unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(aggregate.module.name, "test-module");
}

#[test]
fn test_attribute_path() {
    let path = AttributePath::from_str("packages.x86_64-linux.hello");
    assert_eq!(path.segments.len(), 3);
    assert_eq!(path.to_string(), "packages.x86_64-linux.hello");
}

#[test]
fn test_store_path_parsing() {
    let path_str = "/nix/store/abc123-hello-1.0";
    let store_path = StorePath::parse(path_str).unwrap();

    assert_eq!(store_path.hash, "abc123");
    assert_eq!(store_path.name, "hello-1.0");
    assert_eq!(store_path.to_string(), path_str);
}

#[test]
fn test_package_build_projection() {
    let mut projection = PackageBuildProjection::default();

    // Create a package built event
    let event = PackageBuilt {
        package_id: Uuid::new_v4(),
        flake_ref: "nixpkgs".to_string(),
        attribute: AttributePath::from_str("hello"),
        output_path: PathBuf::from("/nix/store/abc123-hello"),
        build_time: std::time::Duration::from_secs(10),
        timestamp: chrono::Utc::now(),
    };

    // Apply the event
    projection.handle_package_built(&event);

    // Verify the projection was updated
    assert_eq!(projection.total_builds, 1);
    assert_eq!(projection.successful_builds, 1);
    assert_eq!(projection.builds.len(), 1);
}

#[test]
fn test_configuration_projection() {
    let mut projection = ConfigurationProjection::default();

    // Create a configuration
    let config = NixOSConfiguration {
        id: Uuid::new_v4(),
        name: "test-config".to_string(),
        system: "x86_64-linux".to_string(),
        path: PathBuf::from("/etc/nixos/test-config"),
        hostname: "test-host".to_string(),
        modules: vec![],
        overlays: vec![],
        packages: vec![],
        specialisations: HashMap::new(),
    };

    let event = ConfigurationCreated {
        event_id: Uuid::new_v4(),
        occurred_at: chrono::Utc::now(),
        configuration: config,
    };

    // Apply the event
    projection.handle_configuration_created(&event);

    // Verify the projection was updated
    assert_eq!(projection.configurations.len(), 1);
    assert!(projection.configurations.contains_key("test-config"));
}

#[test]
fn test_attribute_path_parsing() {
    let path = AttributePath::from_str("nixpkgs.hello");
    assert_eq!(path.segments, vec!["nixpkgs", "hello"]);

    let complex_path = AttributePath::from_str("nixpkgs.python3Packages.numpy");
    assert_eq!(
        complex_path.segments,
        vec!["nixpkgs", "python3Packages", "numpy"]
    );
}

#[test]
fn test_flake_ref_construction() {
    let flake_ref = FlakeRef::new("github:NixOS/nixpkgs")
        .with_revision("nixos-23.11")
        .with_subflake("lib");

    assert_eq!(
        flake_ref.to_string(),
        "github:NixOS/nixpkgs/nixos-23.11#lib"
    );
}

/*
#[tokio::test]
async fn test_create_and_query_flake() {
    let temp_dir = TempDir::new().unwrap();
    let flake_path = temp_dir.path().to_path_buf();

    // Create a flake
    let cmd = CreateFlake {
        path: flake_path.clone(),
        description: "Test flake".to_string(),
        template: Some("rust".to_string()),
    };

    let result = FlakeAggregate::handle_create_flake(cmd);
    assert!(result.is_ok());

    // Query flake info
    let query = QueryFlakeInfo {
        flake_ref: FlakeRef::new("path:.")
            .with_directory(flake_path.to_str().unwrap()),
    };

    let service = NixServiceFactory::create_evaluation_service();
    let info = service.query_flake_info(query).await;

    // Since we're in a test environment without actual Nix, this might fail
    // but we're testing the integration pattern
    assert!(info.is_ok() || matches!(info.unwrap_err(), NixDomainError::CommandError(_)));
}
*/
