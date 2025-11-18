//! Configuration management for Autonomous Orchestrator

use error_stack::{Report, ResultExt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// Failed to read configuration
    #[error("Failed to read configuration: {0}")]
    ReadError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Redis configuration
    pub redis: RedisConfig,

    /// Event monitor configuration
    pub event_monitor: EventMonitorConfig,

    /// Decision engine configuration
    pub decision_engine: DecisionEngineConfig,

    /// Anomaly detection configuration
    pub anomaly_detection: AnomalyDetectionConfig,

    /// Self-healing configuration
    pub self_healing: SelfHealingConfig,

    /// Analytics configuration
    pub analytics: AnalyticsConfig,

    /// Resource management configuration
    pub resource_manager: ResourceManagerConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    pub host: String,

    /// Server port
    pub port: u16,

    /// Number of worker threads
    pub workers: usize,

    /// Request timeout in seconds
    pub request_timeout: u64,

    /// Enable TLS
    pub enable_tls: bool,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Connection pool size
    pub pool_size: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Enable query logging
    pub log_queries: bool,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,

    /// Connection pool size
    pub pool_size: usize,

    /// Default TTL in seconds
    pub default_ttl: u64,

    /// Event stream name
    pub event_stream: String,

    /// Consumer group name
    pub consumer_group: String,
}

/// Event monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMonitorConfig {
    /// Enable event monitoring
    pub enabled: bool,

    /// Polling interval in milliseconds
    pub poll_interval_ms: u64,

    /// Batch size for event processing
    pub batch_size: usize,

    /// Event retention in days
    pub retention_days: u32,

    /// Enable real-time alerts
    pub enable_alerts: bool,
}

/// Decision engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEngineConfig {
    /// Enable ML-based routing
    pub enable_ml_routing: bool,

    /// Model update interval in hours
    pub model_update_interval_hours: u64,

    /// Minimum training samples
    pub min_training_samples: usize,

    /// Confidence threshold for decisions
    pub confidence_threshold: f64,

    /// Enable A/B testing
    pub enable_ab_testing: bool,
}

/// Anomaly detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetectionConfig {
    /// Enable anomaly detection
    pub enabled: bool,

    /// Detection sensitivity (0.0 - 1.0)
    pub sensitivity: f64,

    /// Window size for analysis (in minutes)
    pub window_size_minutes: u32,

    /// Alert threshold (number of anomalies before alerting)
    pub alert_threshold: u32,

    /// Enable fraud detection
    pub enable_fraud_detection: bool,
}

/// Self-healing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfHealingConfig {
    /// Enable self-healing
    pub enabled: bool,

    /// Maximum retry attempts
    pub max_retry_attempts: u32,

    /// Initial retry delay in seconds
    pub initial_retry_delay_seconds: u64,

    /// Retry backoff multiplier
    pub retry_backoff_multiplier: f64,

    /// Auto-switch connectors on failure
    pub auto_switch_connectors: bool,

    /// Failure threshold for connector switching
    pub failure_threshold: u32,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// Enable analytics
    pub enabled: bool,

    /// Analytics aggregation interval in minutes
    pub aggregation_interval_minutes: u32,

    /// Historical data retention in days
    pub retention_days: u32,

    /// Enable predictive analytics
    pub enable_predictions: bool,

    /// Forecast horizon in days
    pub forecast_horizon_days: u32,
}

/// Resource manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManagerConfig {
    /// Enable auto-scaling
    pub enable_auto_scaling: bool,

    /// CPU threshold for scaling up (percentage)
    pub cpu_scale_up_threshold: f64,

    /// CPU threshold for scaling down (percentage)
    pub cpu_scale_down_threshold: f64,

    /// Memory threshold for scaling up (percentage)
    pub memory_scale_up_threshold: f64,

    /// Memory threshold for scaling down (percentage)
    pub memory_scale_down_threshold: f64,

    /// Minimum instances
    pub min_instances: u32,

    /// Maximum instances
    pub max_instances: u32,

    /// Scale cooldown period in seconds
    pub scale_cooldown_seconds: u64,
}

impl Settings {
    /// Load configuration from environment or default file
    pub fn new() -> Result<Self, Report<ConfigError>> {
        // For now, return default configuration
        // In production, this would load from environment variables or config files
        Ok(Self::default())
    }

    /// Load configuration from specific file
    pub fn from_file(path: PathBuf) -> Result<Self, Report<ConfigError>> {
        let content = std::fs::read_to_string(&path)
            .change_context(ConfigError::ReadError(format!("Failed to read {:?}", path)))?;

        let config: Settings = toml::from_str(&content)
            .change_context(ConfigError::InvalidConfig("Failed to parse TOML".to_string()))?;

        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Report<ConfigError>> {
        // Validate server config
        if self.server.port == 0 {
            return Err(Report::new(ConfigError::InvalidConfig("Port cannot be 0".to_string())));
        }

        // Validate decision engine config
        if self.decision_engine.confidence_threshold < 0.0 || self.decision_engine.confidence_threshold > 1.0 {
            return Err(Report::new(ConfigError::InvalidConfig(
                "Confidence threshold must be between 0.0 and 1.0".to_string()
            )));
        }

        // Validate anomaly detection config
        if self.anomaly_detection.sensitivity < 0.0 || self.anomaly_detection.sensitivity > 1.0 {
            return Err(Report::new(ConfigError::InvalidConfig(
                "Anomaly detection sensitivity must be between 0.0 and 1.0".to_string()
            )));
        }

        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8090,
                workers: num_cpus::get(),
                request_timeout: 30,
                enable_tls: false,
            },
            database: DatabaseConfig {
                url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://localhost/hyperswitch_db".to_string()),
                pool_size: 10,
                connection_timeout: 30,
                log_queries: false,
            },
            redis: RedisConfig {
                url: std::env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                pool_size: 20,
                default_ttl: 3600,
                event_stream: "apos:events".to_string(),
                consumer_group: "apos_consumers".to_string(),
            },
            event_monitor: EventMonitorConfig {
                enabled: true,
                poll_interval_ms: 100,
                batch_size: 50,
                retention_days: 30,
                enable_alerts: true,
            },
            decision_engine: DecisionEngineConfig {
                enable_ml_routing: true,
                model_update_interval_hours: 24,
                min_training_samples: 1000,
                confidence_threshold: 0.75,
                enable_ab_testing: true,
            },
            anomaly_detection: AnomalyDetectionConfig {
                enabled: true,
                sensitivity: 0.85,
                window_size_minutes: 60,
                alert_threshold: 5,
                enable_fraud_detection: true,
            },
            self_healing: SelfHealingConfig {
                enabled: true,
                max_retry_attempts: 3,
                initial_retry_delay_seconds: 2,
                retry_backoff_multiplier: 2.0,
                auto_switch_connectors: true,
                failure_threshold: 5,
            },
            analytics: AnalyticsConfig {
                enabled: true,
                aggregation_interval_minutes: 15,
                retention_days: 90,
                enable_predictions: true,
                forecast_horizon_days: 7,
            },
            resource_manager: ResourceManagerConfig {
                enable_auto_scaling: true,
                cpu_scale_up_threshold: 75.0,
                cpu_scale_down_threshold: 30.0,
                memory_scale_up_threshold: 80.0,
                memory_scale_down_threshold: 40.0,
                min_instances: 1,
                max_instances: 10,
                scale_cooldown_seconds: 300,
            },
        }
    }
}
