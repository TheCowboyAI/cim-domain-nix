//! Comprehensive tests for the Nix domain subject mapping

#[cfg(test)]
mod tests {
    use crate::nats::subject::*;

    #[test]
    fn test_all_command_subjects() {
        let subjects = SubjectMapper::all_command_subjects();

        // Verify we have all expected command subjects
        assert_eq!(subjects.len(), 16);

        // Check that all subjects follow the correct pattern
        for subject in &subjects {
            let s = subject.to_string();
            assert!(s.starts_with("nix.cmd."));
            assert_eq!(s.split('.').count(), 4);
        }

        // Verify specific subjects exist
        let subject_strings: Vec<String> = subjects.iter().map(|s| s.to_string()).collect();
        assert!(subject_strings.contains(&"nix.cmd.flake.create".to_string()));
        assert!(subject_strings.contains(&"nix.cmd.package.build".to_string()));
        assert!(subject_strings.contains(&"nix.cmd.config.activate".to_string()));
    }

    #[test]
    fn test_all_event_subjects() {
        let subjects = SubjectMapper::all_event_subjects();

        // Verify we have all expected event subjects
        assert_eq!(subjects.len(), 16);

        // Check that all subjects follow the correct pattern
        for subject in &subjects {
            let s = subject.to_string();
            assert!(s.starts_with("nix.event."));
            assert_eq!(s.split('.').count(), 4);
        }

        // Verify specific subjects exist
        let subject_strings: Vec<String> = subjects.iter().map(|s| s.to_string()).collect();
        assert!(subject_strings.contains(&"nix.event.flake.created".to_string()));
        assert!(subject_strings.contains(&"nix.event.package.built".to_string()));
        assert!(subject_strings.contains(&"nix.event.config.activated".to_string()));
    }

    #[test]
    fn test_command_to_event_mapping() {
        // Verify that each command has a corresponding event
        let command_actions = vec![
            (CommandAction::CreateFlake, EventAction::FlakeCreated),
            (CommandAction::UpdateFlake, EventAction::FlakeUpdated),
            (CommandAction::AddFlakeInput, EventAction::FlakeInputAdded),
            (CommandAction::BuildPackage, EventAction::PackageBuilt),
            (CommandAction::CreateModule, EventAction::ModuleCreated),
            (CommandAction::CreateOverlay, EventAction::OverlayCreated),
            (
                CommandAction::CreateConfiguration,
                EventAction::ConfigurationCreated,
            ),
            (
                CommandAction::ActivateConfiguration,
                EventAction::ConfigurationActivated,
            ),
        ];

        for (cmd, evt) in command_actions {
            assert_eq!(cmd.aggregate(), evt.aggregate());
        }
    }

    #[test]
    fn test_aggregate_consistency() {
        // Flake actions
        assert_eq!(CommandAction::CreateFlake.aggregate(), Aggregate::Flake);
        assert_eq!(CommandAction::UpdateFlake.aggregate(), Aggregate::Flake);
        assert_eq!(EventAction::FlakeCreated.aggregate(), Aggregate::Flake);
        assert_eq!(QueryAction::GetFlake.aggregate(), Aggregate::Flake);

        // Package actions
        assert_eq!(CommandAction::BuildPackage.aggregate(), Aggregate::Package);
        assert_eq!(EventAction::PackageBuilt.aggregate(), Aggregate::Package);
        assert_eq!(QueryAction::GetPackage.aggregate(), Aggregate::Package);

        // Configuration actions
        assert_eq!(
            CommandAction::CreateConfiguration.aggregate(),
            Aggregate::Configuration
        );
        assert_eq!(
            EventAction::ConfigurationCreated.aggregate(),
            Aggregate::Configuration
        );
        assert_eq!(
            QueryAction::GetConfiguration.aggregate(),
            Aggregate::Configuration
        );
    }

    #[test]
    fn test_wildcard_subscriptions() {
        // Test that wildcard patterns would work for subscriptions
        let all_flake_pattern = "nix.*.flake.*";
        let all_commands_pattern = "nix.cmd.*.*";
        let all_events_pattern = "nix.event.*.*";

        // These patterns would allow subscribing to:
        // - All flake-related messages
        // - All commands across all aggregates
        // - All events across all aggregates

        // Verify our subjects would match these patterns
        let flake_cmd = NixSubject::command(CommandAction::CreateFlake).to_string();
        assert!(flake_cmd.starts_with("nix.") && flake_cmd.contains(".flake."));

        let cmd = NixSubject::command(CommandAction::BuildPackage).to_string();
        assert!(cmd.starts_with("nix.cmd."));

        let evt = NixSubject::event(EventAction::PackageBuilt).to_string();
        assert!(evt.starts_with("nix.event."));
    }

    #[test]
    fn test_subject_round_trip() {
        // Test that we can serialize and deserialize subjects
        let original = NixSubject::command(CommandAction::CreateFlake);
        let serialized = original.to_string();
        let parsed = NixSubject::parse(&serialized).unwrap();

        assert_eq!(parsed.message_type, original.message_type);
        assert_eq!(parsed.aggregate, original.aggregate);
        assert_eq!(parsed.action, original.action);
    }

    #[test]
    fn test_invalid_subject_parsing() {
        // Test various invalid subject formats
        assert!(NixSubject::parse("").is_none());
        assert!(NixSubject::parse("nix").is_none());
        assert!(NixSubject::parse("nix.cmd").is_none());
        assert!(NixSubject::parse("nix.cmd.flake").is_none());
        assert!(NixSubject::parse("wrong.cmd.flake.create").is_none());
        assert!(NixSubject::parse("nix.wrong.flake.create").is_none());
        assert!(NixSubject::parse("nix.cmd.wrong.create").is_none());
        assert!(NixSubject::parse("nix.cmd.flake.create.extra").is_none());
    }

    #[test]
    fn test_special_subjects() {
        // Test special subjects that might be used for system operations
        let health_subject = "health.nix";
        let metrics_subject = "metrics.nix";
        let discovery_subject = "discovery.nix";

        // These would not parse as NixSubject since they don't follow the pattern
        assert!(NixSubject::parse(health_subject).is_none());
        assert!(NixSubject::parse(metrics_subject).is_none());
        assert!(NixSubject::parse(discovery_subject).is_none());
    }
}
