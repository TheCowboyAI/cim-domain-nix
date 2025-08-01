//! Tests for correlation/causation ID implementation

#[cfg(test)]
mod tests {
    use crate::value_objects::{MessageIdentity, MessageFactory};
    use crate::events::{FlakeCreated, NixEventFactory, NixDomainEvent};
    use crate::commands::{CreateFlake, NixCommand};
    use std::path::PathBuf;

    #[test]
    fn test_command_has_message_identity() {
        let identity = MessageIdentity::new_root();
        let cmd = CreateFlake {
            identity: identity.clone(),
            path: PathBuf::from("/tmp/test"),
            description: "Test flake".to_string(),
            template: None,
        };

        assert_eq!(cmd.command_id(), identity.message_id.0);
        assert_eq!(cmd.identity().correlation_id, identity.correlation_id);
        assert_eq!(cmd.identity().causation_id, identity.causation_id);
    }

    #[test]
    fn test_root_event_creation() {
        let event = NixEventFactory::create_flake_created_root(
            uuid::Uuid::new_v4(),
            PathBuf::from("/tmp/test"),
            "Test flake".to_string(),
            None,
        );

        // Root events should have self-correlation
        assert_eq!(
            event.identity.message_id.0,
            event.identity.correlation_id.0
        );
        assert_eq!(
            event.identity.message_id.0,
            event.identity.causation_id.0
        );
    }

    #[test]
    fn test_caused_event_creation() {
        let parent_identity = MessageIdentity::new_root();
        let event = NixEventFactory::create_flake_created_caused_by(
            uuid::Uuid::new_v4(),
            PathBuf::from("/tmp/test"),
            "Test flake".to_string(),
            None,
            &parent_identity,
        );

        // Caused events should inherit correlation but have parent as causation
        assert_ne!(event.identity.message_id.0, parent_identity.message_id.0);
        assert_eq!(event.identity.correlation_id.0, parent_identity.correlation_id.0);
        assert_eq!(event.identity.causation_id.0, parent_identity.message_id.0);
    }

    #[test]
    fn test_event_trait_provides_correlation_causation() {
        let parent_identity = MessageIdentity::new_root();
        let event = NixEventFactory::create_flake_created_caused_by(
            uuid::Uuid::new_v4(),
            PathBuf::from("/tmp/test"),
            "Test flake".to_string(),
            None,
            &parent_identity,
        );

        // Test through the trait
        assert_eq!(event.correlation_id(), parent_identity.correlation_id);
        assert_eq!(event.causation_id().0, parent_identity.message_id.0);
    }

    #[test]
    fn test_command_to_event_correlation_chain() {
        // Create a root command
        let cmd_identity = MessageIdentity::new_root();
        let cmd = CreateFlake {
            identity: cmd_identity.clone(),
            path: PathBuf::from("/tmp/test"),
            description: "Test flake".to_string(),
            template: None,
        };

        // Create event caused by the command
        let event = NixEventFactory::create_flake_created_caused_by(
            uuid::Uuid::new_v4(),
            cmd.path.clone(),
            cmd.description.clone(),
            cmd.template.clone(),
            &cmd.identity,
        );

        // Verify correlation chain
        assert_eq!(event.correlation_id(), cmd.identity().correlation_id);
        assert_eq!(event.causation_id().0, cmd.command_id());
    }
}