//! NATS-specific error types

use thiserror::Error;

/// NATS domain errors
#[derive(Error, Debug)]
pub enum NatsError {
    /// Connection error
    #[error("NATS connection error: {0}")]
    ConnectionError(String),

    /// Authentication error
    #[error("NATS authentication failed: {0}")]
    AuthenticationError(String),

    /// Publishing error
    #[error("Failed to publish message: {0}")]
    PublishError(String),

    /// Subscription error
    #[error("Subscription error: {0}")]
    SubscriptionError(String),

    /// Serialization error
    #[error("Message serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Message deserialization error: {0}")]
    DeserializationError(String),

    /// Invalid subject
    #[error("Invalid NATS subject: {0}")]
    InvalidSubject(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    TimeoutError(String),

    /// Client error
    #[error("NATS client error: {0}")]
    ClientError(#[from] async_nats::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Other error
    #[error("NATS error: {0}")]
    Other(String),
}

/// Result type alias for NATS operations
pub type Result<T> = std::result::Result<T, NatsError>;
