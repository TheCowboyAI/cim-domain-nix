//! NATS integration for the Nix domain
//!
//! This module provides NATS messaging capabilities for the Nix domain,
//! enabling distributed command processing and event streaming.

pub mod client;
pub mod config;
pub mod error;
pub mod health;
pub mod publisher;
pub mod subject;
pub mod subscriber;

#[cfg(test)]
mod subject_tests;

// Re-export commonly used types
pub use client::NatsClient;
pub use config::{NatsConfig, NatsAuth, NatsTls};
pub use error::{NatsError, Result};
pub use health::{HealthService, ServiceDiscovery, ServiceInfo};
pub use publisher::{EventPublisher, EventPublishing};
pub use subject::{NixSubject, SubjectMapper, Aggregate, CommandAction, EventAction, QueryAction};
pub use subscriber::{CommandSubscriber, EventSubscriber, CommandHandler, EventHandler};