//! Demo of NATS subject mapping for the Nix domain

use cim_domain_nix::nats::{NixSubject, SubjectMapper};

fn main() {
    println!("=== Nix Domain NATS Subject Mapping Demo ===\n");

    // Command subjects
    println!("Command Subjects:");
    for subject in SubjectMapper::all_command_subjects() {
        println!("  {}", subject);
    }

    println!("\nEvent Subjects:");
    for subject in SubjectMapper::all_event_subjects() {
        println!("  {}", subject);
    }

    // Demonstrate subject mapping
    println!("\nSubject Mapping Examples:");

    if let Some(subject) = SubjectMapper::command_subject("CreateFlake") {
        println!("  CreateFlake command -> {}", subject);
    }

    if let Some(subject) = SubjectMapper::event_subject("FlakeCreated") {
        println!("  FlakeCreated event -> {}", subject);
    }

    // Demonstrate parsing
    println!("\nSubject Parsing Examples:");

    let subject_str = "nix.cmd.flake.create";
    if let Some(parsed) = NixSubject::parse(subject_str) {
        println!("  Parsed '{}' successfully", subject_str);
    }

    let invalid_str = "invalid.subject.format";
    if NixSubject::parse(invalid_str).is_none() {
        println!("  Correctly rejected invalid subject '{}'", invalid_str);
    }

    println!("\n=== Demo Complete ===");
}
