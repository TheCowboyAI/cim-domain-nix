//! NATS configuration for the Nix domain

use serde::{Deserialize, Serialize};

/// NATS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL
    pub url: String,

    /// Subject prefix for this domain
    pub subject_prefix: String,

    /// Authentication configuration
    pub auth: Option<NatsAuth>,

    /// TLS configuration
    pub tls: Option<NatsTls>,

    /// Connection retry configuration
    pub retry: RetryConfig,

    /// Service identification
    pub service: ServiceConfig,
}

/// NATS authentication options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsAuth {
    /// Username/password authentication
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,

    /// Token authentication
    pub token: Option<String>,

    /// NKey authentication
    pub nkey: Option<String>,

    /// JWT authentication
    pub jwt: Option<String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsTls {
    /// Path to CA certificate
    pub ca_cert: Option<String>,

    /// Path to client certificate
    pub client_cert: Option<String>,

    /// Path to client key
    pub client_key: Option<String>,

    /// Skip server certificate verification (dangerous!)
    pub insecure_skip_verify: bool,
}

/// Connection retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of reconnect attempts
    pub max_reconnects: usize,

    /// Initial reconnect delay in milliseconds
    pub reconnect_delay_ms: u64,

    /// Maximum reconnect delay in milliseconds
    pub max_reconnect_delay_ms: u64,

    /// Reconnect time jitter in milliseconds
    pub reconnect_jitter_ms: u64,

    /// Connect timeout in milliseconds
    pub connect_timeout_ms: u64,
}

/// Service identification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service instance ID
    pub instance_id: String,

    /// Service description
    pub description: String,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            subject_prefix: "nix".to_string(),
            auth: None,
            tls: None,
            retry: RetryConfig::default(),
            service: ServiceConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_reconnects: 10,
            reconnect_delay_ms: 100,
            max_reconnect_delay_ms: 10_000,
            reconnect_jitter_ms: 100,
            connect_timeout_ms: 5_000,
        }
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "cim-domain-nix".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instance_id: uuid::Uuid::new_v4().to_string(),
            description: "Nix domain service for CIM".to_string(),
        }
    }
}

impl NatsConfig {
    /// Create a new NATS configuration with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set authentication
    pub fn with_auth(mut self, auth: NatsAuth) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set TLS configuration
    pub fn with_tls(mut self, tls: NatsTls) -> Self {
        self.tls = Some(tls);
        self
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(url) = std::env::var("NATS_URL") {
            config.url = url;
        }

        if let Ok(user) = std::env::var("NATS_USER") {
            let password = std::env::var("NATS_PASSWORD").ok();
            config.auth = Some(NatsAuth {
                username: Some(user),
                password,
                token: None,
                nkey: None,
                jwt: None,
            });
        } else if let Ok(token) = std::env::var("NATS_TOKEN") {
            config.auth = Some(NatsAuth {
                username: None,
                password: None,
                token: Some(token),
                nkey: None,
                jwt: None,
            });
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.subject_prefix, "nix");
        assert!(config.auth.is_none());
        assert!(config.tls.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = NatsConfig::new("nats://remote:4222").with_auth(NatsAuth {
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            token: None,
            nkey: None,
            jwt: None,
        });

        assert_eq!(config.url, "nats://remote:4222");
        assert!(config.auth.is_some());
    }
}
