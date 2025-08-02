//! Message identity value objects for correlation and causation tracking
//!
//! Implements CIM event sourcing patterns for message correlation and causation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    /// Create a new message ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Correlation ID for grouping related messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub Uuid);

impl CorrelationId {
    /// Create a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Causation ID indicating what caused this message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(pub Uuid);

impl CausationId {
    /// Create a new causation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CausationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CausationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message identity containing correlation and causation information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique identifier for this message
    pub message_id: MessageId,
    /// Groups related messages together
    pub correlation_id: CorrelationId,
    /// Indicates what caused this message
    pub causation_id: CausationId,
}

impl MessageIdentity {
    /// Create a new root message identity (self-correlated)
    /// For root messages: message_id = correlation_id = causation_id
    pub fn new_root() -> Self {
        let id = Uuid::new_v4();
        Self {
            message_id: MessageId(id),
            correlation_id: CorrelationId(id),
            causation_id: CausationId(id),
        }
    }

    /// Create a message identity caused by another message
    /// Inherits correlation_id from parent, causation_id = parent.message_id
    pub fn new_caused_by(parent: &MessageIdentity) -> Self {
        Self {
            message_id: MessageId::new(),
            correlation_id: parent.correlation_id,
            causation_id: CausationId(parent.message_id.0),
        }
    }

    /// Create a message identity with explicit IDs
    pub fn new(
        message_id: MessageId,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self {
            message_id,
            correlation_id,
            causation_id,
        }
    }
}

/// Factory for creating messages with proper correlation/causation
pub struct MessageFactory;

impl MessageFactory {
    /// Create a root message (self-correlated)
    pub fn create_root_identity() -> MessageIdentity {
        MessageIdentity::new_root()
    }

    /// Create a message caused by another
    pub fn create_caused_identity(parent: &MessageIdentity) -> MessageIdentity {
        MessageIdentity::new_caused_by(parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_message_identity() {
        let identity = MessageIdentity::new_root();

        // For root messages, all IDs should be the same
        assert_eq!(identity.message_id.0, identity.correlation_id.0);
        assert_eq!(identity.message_id.0, identity.causation_id.0);
    }

    #[test]
    fn test_caused_message_identity() {
        let root = MessageIdentity::new_root();
        let caused = MessageIdentity::new_caused_by(&root);

        // Caused message should have same correlation but different message ID
        assert_ne!(caused.message_id.0, root.message_id.0);
        assert_eq!(caused.correlation_id.0, root.correlation_id.0);
        assert_eq!(caused.causation_id.0, root.message_id.0);
    }

    #[test]
    fn test_message_factory() {
        let root = MessageFactory::create_root_identity();
        let caused = MessageFactory::create_caused_identity(&root);

        assert_eq!(root.message_id.0, root.correlation_id.0);
        assert_eq!(caused.correlation_id.0, root.correlation_id.0);
        assert_eq!(caused.causation_id.0, root.message_id.0);
    }
}
