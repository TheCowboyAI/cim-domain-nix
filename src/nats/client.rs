//! NATS client wrapper for the Nix domain

use async_nats::{Client, ConnectOptions};
use std::time::Duration;
use tracing::{debug, info, warn};

use super::{
    config::NatsConfig,
    error::{NatsError, Result},
};

/// NATS client wrapper with domain-specific configuration
pub struct NatsClient {
    /// The underlying NATS client
    client: Client,

    /// Configuration
    config: NatsConfig,
}

impl NatsClient {
    /// Connect to NATS with the given configuration
    pub async fn connect(config: NatsConfig) -> Result<Self> {
        info!("Connecting to NATS at {}", config.url);

        let mut options = ConnectOptions::new()
            .name(&config.service.name)
            .retry_on_initial_connect()
            // Note: max_reconnects is not available in async-nats 0.33
            // Note: reconnect_delay is also not directly available, would need callback
            .connection_timeout(Duration::from_millis(config.retry.connect_timeout_ms));

        // Add authentication if configured
        if let Some(auth) = &config.auth {
            if let (Some(user), Some(pass)) = (&auth.username, &auth.password) {
                debug!("Using username/password authentication");
                options = options.user_and_password(user.clone(), pass.clone());
            } else if let Some(token) = &auth.token {
                debug!("Using token authentication");
                options = options.token(token.clone());
            } else if let Some(nkey) = &auth.nkey {
                debug!("Using NKey authentication");
                // Note: async-nats requires parsing the nkey
                // This is a simplified example
                options = options.nkey(nkey.clone());
            }
        }

        // Add TLS if configured
        if let Some(tls) = &config.tls {
            debug!("Configuring TLS");
            let mut tls_config = async_nats::ConnectOptions::new();

            if let Some(ca_cert) = &tls.ca_cert {
                tls_config = tls_config.add_root_certificates(ca_cert.into());
            }

            if let (Some(cert), Some(key)) = (&tls.client_cert, &tls.client_key) {
                tls_config = tls_config.add_client_certificate(cert.into(), key.into());
            }

            if tls.insecure_skip_verify {
                warn!("TLS verification disabled - this is insecure!");
                // Note: tls_required is a field, not a method - need to use a different approach
                // For now, we'll skip this as async-nats doesn't have this option
            }

            options = tls_config;
        }

        // Connect with retry
        let client = async_nats::connect_with_options(&config.url, options)
            .await
            .map_err(|e| NatsError::ConnectionError(e.to_string()))?;

        info!("Successfully connected to NATS");

        Ok(Self { client, config })
    }

    /// Get the underlying NATS client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the configuration
    pub fn config(&self) -> &NatsConfig {
        &self.config
    }

    /// Check if the client is connected
    pub fn is_connected(&self) -> bool {
        self.client.connection_state() == async_nats::connection::State::Connected
    }

    /// Flush all pending operations
    pub async fn flush(&self) -> Result<()> {
        self.client
            .flush()
            .await
            .map_err(|e| NatsError::Other(e.to_string()))
    }

    /// Drain the connection (graceful shutdown)
    pub async fn drain(&self) -> Result<()> {
        info!("Draining NATS connection");
        // Note: async-nats doesn't have a direct drain method
        // We'll flush and let the client drop naturally
        self.flush().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires NATS server
    async fn test_connect_default() {
        let config = NatsConfig::default();
        let client = NatsClient::connect(config).await.unwrap();
        assert!(client.is_connected());
    }
}
