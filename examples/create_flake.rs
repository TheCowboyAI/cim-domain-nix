//! Example of creating a Nix flake using the domain module

use cim_domain_nix::{
    commands::CreateFlake,
    handlers::NixCommandHandler,
    value_objects::MessageIdentity,
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary directory for our flake
    let temp_dir = TempDir::new()?;
    let flake_path = temp_dir.path().to_path_buf();

    println!("Creating flake in: {}", flake_path.display());

    // Create the handler
    let handler = NixCommandHandler::new();

    // Create the command
    let cmd = CreateFlake {
        identity: MessageIdentity::new_root(),
        path: flake_path.clone(),
        description: "My awesome Rust project".to_string(),
        template: Some("rust".to_string()),
    };

    // Handle the command
    let events = handler.handle_command(Box::new(cmd)).await?;

    println!("Generated {} events", events.len());

    // Print event details
    for event in &events {
        println!("Event: {:?}", event);
    }

    // Keep the directory around so we can inspect it
    let path = temp_dir.path().to_path_buf();
    std::mem::forget(temp_dir); // Keep the directory
    println!("\nFlake created at: {}", path.display());
    println!("You can inspect the flake.nix file there.");

    Ok(())
} 