//! Command and event subscribers for the Nix domain

use async_nats::{Client, Message, Subscriber};
use futures::StreamExt;
// use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use super::{
    error::{NatsError, Result},
    publisher::EventPublisher,
    subject::{NixSubject, SubjectMapper},
};
use crate::commands::NixCommand;
use crate::events::NixDomainEvent;
use crate::value_objects::{CausationId, CorrelationId};

/// Handler for incoming commands
#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// Handle a command and return resulting events
    async fn handle_command(
        &self,
        command: Box<dyn NixCommand>,
    ) -> Result<Vec<Box<dyn std::any::Any + Send>>>;
}

/// Handler for incoming events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    /// Handle an event
    async fn handle_event(&self, event: Box<dyn NixDomainEvent>) -> Result<()>;
}

/// Command subscriber for the Nix domain
pub struct CommandSubscriber {
    /// NATS client
    client: Client,

    /// Command handler
    handler: Arc<dyn CommandHandler>,

    /// Event publisher for publishing results
    publisher: Arc<EventPublisher>,

    /// Active subscription handles
    handles: Vec<JoinHandle<()>>,
}

impl CommandSubscriber {
    /// Create a new command subscriber
    pub fn new(
        client: Client,
        handler: Arc<dyn CommandHandler>,
        publisher: Arc<EventPublisher>,
    ) -> Self {
        Self {
            client,
            handler,
            publisher,
            handles: Vec::new(),
        }
    }

    /// Start subscribing to all command subjects
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting command subscriber");

        // Subscribe to all command subjects
        for subject in SubjectMapper::all_command_subjects() {
            let subject_str = subject.to_string();
            info!("Subscribing to command subject: {}", subject_str);

            let subscriber = self
                .client
                .subscribe(subject_str.clone())
                .await
                .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

            let handler = self.handler.clone();
            let publisher = self.publisher.clone();
            let client = self.client.clone();

            // Spawn handler task
            let handle = tokio::spawn(async move {
                Self::handle_messages(subscriber, handler, publisher, client, subject_str).await;
            });

            self.handles.push(handle);
        }

        info!(
            "Command subscriber started with {} subscriptions",
            self.handles.len()
        );
        Ok(())
    }

    /// Handle messages from a subscription
    async fn handle_messages(
        mut subscriber: Subscriber,
        handler: Arc<dyn CommandHandler>,
        publisher: Arc<EventPublisher>,
        client: Client,
        subject: String,
    ) {
        info!("Starting message handler for subject: {}", subject);

        while let Some(msg) = subscriber.next().await {
            if let Err(e) = Self::handle_single_message(
                msg,
                handler.clone(),
                publisher.clone(),
                &client,
                &subject,
            )
            .await
            {
                error!("Error handling message on {}: {}", subject, e);
            }
        }

        warn!("Message handler for {} terminated", subject);
    }

    /// Handle a single command message
    async fn handle_single_message(
        msg: Message,
        handler: Arc<dyn CommandHandler>,
        publisher: Arc<EventPublisher>,
        client: &Client,
        subject: &str,
    ) -> Result<()> {
        debug!("Received command on subject: {}", subject);

        // Extract correlation/causation from headers
        let correlation_id = msg
            .headers
            .as_ref()
            .and_then(|h| h.get("X-Correlation-ID"))
            .and_then(|v| Some(v.as_str()))
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(CorrelationId)
            .unwrap_or_else(|| {
                warn!("Missing correlation ID in command headers");
                CorrelationId::new()
            });

        let causation_id = msg
            .headers
            .as_ref()
            .and_then(|h| h.get("X-Causation-ID"))
            .and_then(|v| Some(v.as_str()))
            .and_then(|s| uuid::Uuid::parse_str(s).ok())
            .map(CausationId)
            .unwrap_or_else(|| {
                warn!("Missing causation ID in command headers");
                CausationId::new()
            });

        // Parse the subject to determine command type
        let parsed_subject = NixSubject::parse(subject)
            .ok_or_else(|| NatsError::InvalidSubject(subject.to_string()))?;

        // Deserialize command based on subject
        // This is simplified - in production you'd use a command registry
        let command: Box<dyn NixCommand> = match &parsed_subject.action[..] {
            "create" if parsed_subject.aggregate.to_string() == "flake" => {
                let cmd: crate::commands::CreateFlake = serde_json::from_slice(&msg.payload)
                    .map_err(|e| NatsError::DeserializationError(e.to_string()))?;
                Box::new(cmd)
            }
            "build" if parsed_subject.aggregate.to_string() == "package" => {
                let cmd: crate::commands::BuildPackage = serde_json::from_slice(&msg.payload)
                    .map_err(|e| NatsError::DeserializationError(e.to_string()))?;
                Box::new(cmd)
            }
            // Add other command types...
            _ => {
                return Err(NatsError::InvalidSubject(format!(
                    "Unknown command type for subject: {}",
                    subject
                )));
            }
        };

        // Handle the command
        match handler.handle_command(command).await {
            Ok(events) => {
                info!(
                    "Command handled successfully, publishing {} events",
                    events.len()
                );

                // Publish resulting events
                for event in events {
                    // Since handlers return Box<dyn NixDomainEvent> but we cast them as Box<dyn Any>,
                    // we can pass them directly
                    Self::publish_event_boxed(&publisher, event).await?;
                }

                // Send reply if requested
                if let Some(reply) = msg.reply {
                    let response = serde_json::json!({
                        "status": "success",
                        "message": "Command processed successfully"
                    });

                    client
                        .publish(reply, serde_json::to_vec(&response)?.into())
                        .await
                        .map_err(|e| NatsError::PublishError(e.to_string()))?;
                }
            }
            Err(e) => {
                error!("Command handling failed: {}", e);

                // Send error reply if requested
                if let Some(reply) = msg.reply {
                    let response = serde_json::json!({
                        "status": "error",
                        "message": e.to_string()
                    });

                    client
                        .publish(reply, serde_json::to_vec(&response)?.into())
                        .await
                        .map_err(|e| NatsError::PublishError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }

    /// Publish a boxed event by downcasting to concrete types
    async fn publish_event_boxed(
        publisher: &EventPublisher,
        event: Box<dyn std::any::Any + Send>,
    ) -> Result<()> {
        use crate::events::*;

        // Try to downcast to each concrete event type
        if let Some(e) = event.downcast_ref::<FlakeCreated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<FlakeUpdated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<FlakeInputAdded>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<PackageBuilt>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<ModuleCreated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<OverlayCreated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<ConfigurationCreated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<ConfigurationActivated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<ExpressionEvaluated>() {
            publisher.publish_event(e).await?;
        } else if let Some(e) = event.downcast_ref::<GarbageCollected>() {
            publisher.publish_event(e).await?;
        } else {
            warn!("Unknown event type, cannot publish");
        }

        Ok(())
    }

    /// Stop the subscriber
    pub async fn stop(self) {
        info!("Stopping command subscriber");

        // Cancel all subscription tasks
        for handle in self.handles {
            handle.abort();
        }
    }
}

/// Event subscriber for the Nix domain
pub struct EventSubscriber {
    /// NATS client
    client: Client,

    /// Event handler
    handler: Arc<dyn EventHandler>,

    /// Active subscription handles
    handles: Vec<JoinHandle<()>>,
}

impl EventSubscriber {
    /// Create a new event subscriber
    pub fn new(client: Client, handler: Arc<dyn EventHandler>) -> Self {
        Self {
            client,
            handler,
            handles: Vec::new(),
        }
    }

    /// Subscribe to specific event subjects
    pub async fn subscribe(&mut self, subjects: Vec<NixSubject>) -> Result<()> {
        for subject in subjects {
            let subject_str = subject.to_string();
            info!("Subscribing to event subject: {}", subject_str);

            let subscriber = self
                .client
                .subscribe(subject_str.clone())
                .await
                .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

            let handler = self.handler.clone();

            // Spawn handler task
            let handle = tokio::spawn(async move {
                Self::handle_events(subscriber, handler, subject_str).await;
            });

            self.handles.push(handle);
        }

        Ok(())
    }

    /// Handle events from a subscription
    async fn handle_events(
        mut subscriber: Subscriber,
        handler: Arc<dyn EventHandler>,
        subject: String,
    ) {
        info!("Starting event handler for subject: {}", subject);

        while let Some(msg) = subscriber.next().await {
            if let Err(e) = Self::handle_single_event(msg, handler.clone(), &subject).await {
                error!("Error handling event on {}: {}", subject, e);
            }
        }

        warn!("Event handler for {} terminated", subject);
    }

    /// Handle a single event message
    async fn handle_single_event(
        msg: Message,
        _handler: Arc<dyn EventHandler>,
        subject: &str,
    ) -> Result<()> {
        debug!("Received event on subject: {}", subject);

        // Parse subject and deserialize event
        // This is simplified - in production you'd use an event registry
        let parsed_subject = NixSubject::parse(subject)
            .ok_or_else(|| NatsError::InvalidSubject(subject.to_string()))?;

        // Deserialize based on subject (simplified)
        // In production, you'd have a proper event type registry

        // For now, just log that we received it
        info!(
            "Received event on subject: {} with {} bytes",
            subject,
            msg.payload.len()
        );

        Ok(())
    }

    /// Stop the subscriber
    pub async fn stop(self) {
        info!("Stopping event subscriber");

        // Cancel all subscription tasks
        for handle in self.handles {
            handle.abort();
        }
    }
}
