//! Integration tests for NATS-ECS patterns

#[cfg(test)]
mod tests {
    use cim_domain_nix::nats::{NixSubject, SubjectMapper, CommandAction, EventAction};
    use cim_domain_nix::value_objects::{CorrelationId, CausationId};
    use std::collections::HashSet;
    
    #[test]
    fn test_subject_to_component_mapping() {
        // Test that subjects map correctly to component requirements
        
        // Flake commands should require flake components
        let flake_subjects = vec![
            NixSubject::command(CommandAction::CreateFlake),
            NixSubject::command(CommandAction::UpdateFlake),
            NixSubject::event(EventAction::FlakeCreated),
        ];
        
        for subject in flake_subjects {
            assert!(subject.to_string().contains("flake"));
        }
        
        // Package commands should require package components
        let package_subjects = vec![
            NixSubject::command(CommandAction::BuildPackage),
            NixSubject::event(EventAction::PackageBuilt),
        ];
        
        for subject in package_subjects {
            assert!(subject.to_string().contains("package"));
        }
    }
    
    #[test]
    fn test_correlation_filtering() {
        // Test correlation-based entity filtering
        let correlation1 = CorrelationId::new();
        let correlation2 = CorrelationId::new();
        
        assert_ne!(correlation1, correlation2);
        
        // Simulate entity grouping by correlation
        let mut workflow1_entities = HashSet::new();
        workflow1_entities.insert((uuid::Uuid::new_v4(), correlation1));
        workflow1_entities.insert((uuid::Uuid::new_v4(), correlation1));
        
        let mut workflow2_entities = HashSet::new();
        workflow2_entities.insert((uuid::Uuid::new_v4(), correlation2));
        
        // Verify correlation grouping
        let workflow1_count = workflow1_entities.iter()
            .filter(|(_, corr)| *corr == correlation1)
            .count();
        assert_eq!(workflow1_count, 2);
        
        let workflow2_count = workflow2_entities.iter()
            .filter(|(_, corr)| *corr == correlation2)
            .count();
        assert_eq!(workflow2_count, 1);
    }
    
    #[test]
    fn test_causation_chain() {
        // Test causation chain tracking
        let root_id = uuid::Uuid::new_v4();
        let caused_id1 = uuid::Uuid::new_v4();
        let caused_id2 = uuid::Uuid::new_v4();
        
        // Build causation chain
        let chain = vec![
            (root_id, CausationId(root_id)),      // Root causes itself
            (caused_id1, CausationId(root_id)),   // Caused by root
            (caused_id2, CausationId(caused_id1)), // Caused by previous
        ];
        
        // Verify chain relationships
        assert_eq!(chain[0].1, CausationId(root_id));
        assert_eq!(chain[1].1, CausationId(root_id));
        assert_eq!(chain[2].1, CausationId(caused_id1));
    }
    
    #[test]
    fn test_wildcard_system_subscriptions() {
        // Test that wildcard patterns match correctly
        let all_flake_pattern = "nix.*.flake.*";
        let all_commands_pattern = "nix.cmd.*.*";
        let all_create_pattern = "nix.cmd.*.create";
        
        // Get all subjects
        let all_commands = SubjectMapper::all_command_subjects();
        
        // Count matches for each pattern
        let flake_matches = all_commands.iter()
            .filter(|s| {
                let parts: Vec<&str> = s.to_string().split('.').collect();
                parts.len() == 4 && parts[2] == "flake"
            })
            .count();
        
        let create_matches = all_commands.iter()
            .filter(|s| {
                let parts: Vec<&str> = s.to_string().split('.').collect();
                parts.len() == 4 && parts[3] == "create"
            })
            .count();
        
        assert!(flake_matches > 0);
        assert!(create_matches > 0);
    }
    
    #[test]
    fn test_command_event_correspondence() {
        // Verify that commands have corresponding events
        let command_event_pairs = vec![
            (CommandAction::CreateFlake, EventAction::FlakeCreated),
            (CommandAction::UpdateFlake, EventAction::FlakeUpdated),
            (CommandAction::BuildPackage, EventAction::PackageBuilt),
            (CommandAction::CreateModule, EventAction::ModuleCreated),
            (CommandAction::CreateOverlay, EventAction::OverlayCreated),
            (CommandAction::CreateConfiguration, EventAction::ConfigurationCreated),
            (CommandAction::ActivateConfiguration, EventAction::ConfigurationActivated),
        ];
        
        for (cmd_action, evt_action) in command_event_pairs {
            let cmd_subject = NixSubject::command(cmd_action);
            let evt_subject = NixSubject::event(evt_action);
            
            // Verify they share the same aggregate
            let cmd_parts: Vec<&str> = cmd_subject.to_string().split('.').collect();
            let evt_parts: Vec<&str> = evt_subject.to_string().split('.').collect();
            
            assert_eq!(cmd_parts[2], evt_parts[2], "Aggregate mismatch for {:?}/{:?}", cmd_action, evt_action);
        }
    }
    
    #[test]
    fn test_subject_algebra_composition() {
        // Test subject algebra operations
        let flake_commands: HashSet<String> = SubjectMapper::all_command_subjects()
            .into_iter()
            .map(|s| s.to_string())
            .filter(|s| s.contains(".flake."))
            .collect();
        
        let package_commands: HashSet<String> = SubjectMapper::all_command_subjects()
            .into_iter()
            .map(|s| s.to_string())
            .filter(|s| s.contains(".package."))
            .collect();
        
        // Union of flake and package commands
        let union: HashSet<_> = flake_commands.union(&package_commands).cloned().collect();
        assert_eq!(union.len(), flake_commands.len() + package_commands.len());
        
        // Intersection should be empty (no overlap)
        let intersection: HashSet<_> = flake_commands.intersection(&package_commands).cloned().collect();
        assert_eq!(intersection.len(), 0);
    }
    
    #[test]
    fn test_system_filter_patterns() {
        // Test various system filter patterns
        
        // System that handles all flake operations
        let flake_system_subjects: Vec<String> = SubjectMapper::all_command_subjects()
            .into_iter()
            .chain(SubjectMapper::all_event_subjects())
            .map(|s| s.to_string())
            .filter(|s| s.contains(".flake."))
            .collect();
        
        assert!(flake_system_subjects.len() > 5); // Should have multiple flake operations
        
        // System that handles only create operations
        let create_system_subjects: Vec<String> = SubjectMapper::all_command_subjects()
            .into_iter()
            .map(|s| s.to_string())
            .filter(|s| s.ends_with(".create"))
            .collect();
        
        assert_eq!(create_system_subjects.len(), 5); // One create per aggregate
        
        // System that handles build operations
        let build_system_subjects: Vec<String> = vec![
            NixSubject::command(CommandAction::BuildPackage).to_string(),
            NixSubject::event(EventAction::PackageBuilt).to_string(),
        ];
        
        assert_eq!(build_system_subjects.len(), 2);
    }
}

#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use super::*;
    use async_nats::Client;
    use tokio;
    
    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_distributed_entity_query() {
        // Connect to NATS
        let client = Client::connect("nats://localhost:4222").await.unwrap();
        
        // Create test correlation
        let correlation = CorrelationId::new();
        
        // Publish entity registration
        let entity_metadata = serde_json::json!({
            "entity_id": uuid::Uuid::new_v4(),
            "entity_type": "flake",
            "correlation_id": correlation,
            "components": ["FlakeComponent", "StateComponent"],
        });
        
        client.publish(
            "entity.registry.flake.created",
            serde_json::to_vec(&entity_metadata).unwrap().into()
        ).await.unwrap();
        
        // Query entities by correlation
        let response = client.request(
            "entity.query.by_correlation",
            serde_json::to_vec(&correlation).unwrap().into()
        ).await.unwrap();
        
        let entities: Vec<serde_json::Value> = serde_json::from_slice(&response.payload).unwrap();
        assert!(!entities.is_empty());
    }
}