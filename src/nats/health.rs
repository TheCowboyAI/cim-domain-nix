//! Health check service for NATS

use async_nats::{Client, Message};
use chrono::Utc;
use futures::StreamExt;
use serde_json::json;
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

use super::error::{NatsError, Result};

/// Health check responder for the Nix domain service
pub struct HealthService {
    /// NATS client
    client: Client,

    /// Service name
    service_name: String,

    /// Service version
    service_version: String,

    /// Health check handler
    handle: Option<JoinHandle<()>>,
}

impl HealthService {
    /// Create a new health service
    pub fn new(client: Client, service_name: String, service_version: String) -> Self {
        Self {
            client,
            service_name,
            service_version,
            handle: None,
        }
    }

    /// Start the health check responder
    pub async fn start(&mut self) -> Result<()> {
        let subject = format!("health.{}", self.service_name);
        info!("Starting health check responder on subject: {}", subject);

        let mut subscriber = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

        let service_name = self.service_name.clone();
        let service_version = self.service_version.clone();
        let client = self.client.clone();

        let handle = tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let Err(e) =
                    Self::handle_health_check_static(&client, msg, &service_name, &service_version)
                        .await
                {
                    error!("Error handling health check: {}", e);
                }
            }
        });

        self.handle = Some(handle);

        Ok(())
    }

    /// Handle a health check request (static version for use in spawned tasks)
    async fn handle_health_check_static(
        client: &Client,
        msg: Message,
        service_name: &str,
        service_version: &str,
    ) -> Result<()> {
        debug!("Received health check request");

        // Build health response
        let health = json!({
            "status": "healthy",
            "service": service_name,
            "version": service_version,
            "timestamp": Utc::now().to_rfc3339(),
            "details": {
                "uptime": Self::get_uptime(),
                "memory": Self::get_memory_usage(),
                "connections": {
                    "nats": "connected"
                }
            }
        });

        // Send response
        if let Some(reply) = msg.reply {
            client
                .publish(reply, serde_json::to_vec(&health)?.into())
                .await
                .map_err(|e| NatsError::PublishError(e.to_string()))?;

            debug!("Health check response sent");
        }

        Ok(())
    }

    /// Get service uptime in seconds
    fn get_uptime() -> u64 {
        // This is a simplified implementation
        // In production, you'd track the actual start time
        0
    }

    /// Get memory usage information
    fn get_memory_usage() -> serde_json::Value {
        // This is a simplified implementation
        // In production, you'd use actual memory stats
        json!({
            "used_mb": 0,
            "total_mb": 0,
            "percentage": 0.0
        })
    }

    /// Stop the health service
    pub async fn stop(mut self) {
        info!("Stopping health check service");

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// Service discovery announcer
pub struct ServiceDiscovery {
    /// NATS client
    client: Client,

    /// Service information
    service_info: ServiceInfo,

    /// Announcement handle
    handle: Option<JoinHandle<()>>,
}

/// Service information for discovery
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub version: String,
    pub instance_id: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub endpoints: Vec<String>,
}

impl ServiceDiscovery {
    /// Create a new service discovery announcer
    pub fn new(client: Client, service_info: ServiceInfo) -> Self {
        Self {
            client,
            service_info,
            handle: None,
        }
    }

    /// Start announcing service availability
    pub async fn start(&mut self) -> Result<()> {
        let subject = format!("discovery.{}", self.service_info.name);
        info!("Starting service discovery on subject: {}", subject);

        let mut subscriber = self
            .client
            .subscribe(subject)
            .await
            .map_err(|e| NatsError::SubscriptionError(e.to_string()))?;

        let service_info = self.service_info.clone();
        let client = self.client.clone();

        let handle = tokio::spawn(async move {
            while let Some(msg) = subscriber.next().await {
                if let Err(e) =
                    Self::handle_discovery_request_static(&client, msg, &service_info).await
                {
                    error!("Error handling discovery request: {}", e);
                }
            }
        });

        self.handle = Some(handle);

        // Also announce on startup
        self.announce().await?;

        Ok(())
    }

    /// Announce service availability
    pub async fn announce(&self) -> Result<()> {
        let subject = "discovery.announce";

        let announcement = json!({
            "service": self.service_info,
            "timestamp": Utc::now().to_rfc3339(),
        });

        self.client
            .publish(subject, serde_json::to_vec(&announcement)?.into())
            .await
            .map_err(|e| NatsError::PublishError(e.to_string()))?;

        info!("Service announcement sent");

        Ok(())
    }

    /// Handle a discovery request (static version for use in spawned tasks)
    async fn handle_discovery_request_static(
        client: &Client,
        msg: Message,
        service_info: &ServiceInfo,
    ) -> Result<()> {
        debug!("Received discovery request");

        let response = json!({
            "service": service_info,
            "timestamp": Utc::now().to_rfc3339(),
        });

        if let Some(reply) = msg.reply {
            client
                .publish(reply, serde_json::to_vec(&response)?.into())
                .await
                .map_err(|e| NatsError::PublishError(e.to_string()))?;

            debug!("Discovery response sent");
        }

        Ok(())
    }

    /// Stop the service discovery
    pub async fn stop(mut self) {
        info!("Stopping service discovery");

        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}
