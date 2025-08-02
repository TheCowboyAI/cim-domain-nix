//! Event publisher for the Nix domain

use async_nats::{Client, HeaderMap};
use bytes::Bytes;
use serde::Serialize;
use tracing::{debug, info};

use crate::events::NixDomainEvent;
// use crate::value_objects::{CorrelationId, CausationId};
use super::{
    error::{NatsError, Result},
    subject::{EventAction, NixSubject, SubjectMapper},
};

/// Event publisher for Nix domain events
pub struct EventPublisher {
    /// NATS client
    client: Client,

    /// Subject prefix (usually "nix")
    subject_prefix: String,
}

impl EventPublisher {
    /// Create a new event publisher
    pub fn new(client: Client, subject_prefix: String) -> Self {
        Self {
            client,
            subject_prefix,
        }
    }

    /// Publish a domain event
    pub async fn publish_event<E>(&self, event: &E) -> Result<()>
    where
        E: NixDomainEvent + Serialize,
    {
        // Get the event type name
        let event_type = event.event_type();

        // Map to NATS subject
        let subject = SubjectMapper::event_subject(&event_type).ok_or_else(|| {
            NatsError::InvalidSubject(format!("Unknown event type: {}", event_type))
        })?;

        let subject_str = subject.to_string();
        debug!("Publishing event {} to subject {}", event_type, subject_str);

        // Create headers with event metadata
        let mut headers = HeaderMap::new();
        let event_id = event.event_id().to_string();
        let correlation_id = event.correlation_id().to_string();
        let causation_id = event.causation_id().to_string();
        let aggregate_id = event.aggregate_id().to_string();
        let timestamp = event.occurred_at().to_rfc3339();

        headers.insert("X-Event-ID", event_id.as_str());
        headers.insert("X-Event-Type", event_type);
        headers.insert("X-Correlation-ID", correlation_id.as_str());
        headers.insert("X-Causation-ID", causation_id.as_str());
        headers.insert("X-Aggregate-ID", aggregate_id.as_str());
        headers.insert("X-Timestamp", timestamp.as_str());
        headers.insert("X-Domain", self.subject_prefix.as_str());

        // Serialize the event
        let payload =
            serde_json::to_vec(&event).map_err(|e| NatsError::SerializationError(e.to_string()))?;

        // Publish with headers
        self.client
            .publish_with_headers(subject_str, headers, Bytes::from(payload))
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        info!(
            "Published event {} with ID {}",
            event_type,
            event.event_id()
        );

        Ok(())
    }

    /// Publish multiple events
    pub async fn publish_events<E>(&self, events: &[E]) -> Result<()>
    where
        E: NixDomainEvent + Serialize,
    {
        for event in events {
            self.publish_event(event).await?;
        }
        Ok(())
    }

    /// Publish a raw event (for special cases)
    pub async fn publish_raw(
        &self,
        action: EventAction,
        headers: HeaderMap,
        payload: Bytes,
    ) -> Result<()> {
        let subject = NixSubject::event(action);
        let subject_str = subject.to_string();

        self.client
            .publish_with_headers(subject_str, headers, payload)
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        Ok(())
    }

    /// Flush all pending publishes
    pub async fn flush(&self) -> Result<()> {
        self.client
            .flush()
            .await
            .map_err(|e| NatsError::Other(e.to_string()))
    }
}

/// Extension trait for publishing events directly from aggregates
#[async_trait::async_trait]
pub trait EventPublishing {
    /// Publish all pending events
    async fn publish_events(&self, publisher: &EventPublisher) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{FlakeCreated, NixEventFactory};
    use crate::value_objects::MessageIdentity;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_publish_event() {
        // Create a test event
        let event = NixEventFactory::create_flake_created_root(
            Uuid::new_v4(),
            PathBuf::from("/tmp/test"),
            "Test flake".to_string(),
            None,
        );

        // Connect to NATS
        let client = async_nats::connect("nats://localhost:4222").await.unwrap();
        let publisher = EventPublisher::new(client, "nix".to_string());

        // Publish the event
        publisher.publish_event(&event).await.unwrap();

        // Flush to ensure it's sent
        publisher.flush().await.unwrap();
    }
}
