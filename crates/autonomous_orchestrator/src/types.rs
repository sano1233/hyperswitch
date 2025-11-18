//! Type definitions for Autonomous Orchestrator

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;
use uuid::Uuid;

/// Event type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Payment created
    PaymentCreated,
    /// Payment succeeded
    PaymentSucceeded,
    /// Payment failed
    PaymentFailed,
    /// Payment requires action
    PaymentRequiresAction,
    /// Refund created
    RefundCreated,
    /// Refund succeeded
    RefundSucceeded,
    /// Refund failed
    RefundFailed,
    /// Connector failure
    ConnectorFailure,
    /// Fraud detected
    FraudDetected,
    /// Anomaly detected
    AnomalyDetected,
    /// System health check
    HealthCheck,
    /// Resource scaling event
    ResourceScaling,
    /// Custom event
    Custom(String),
}

/// Payment event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentEvent {
    /// Event ID
    pub event_id: String,

    /// Event type
    pub event_type: EventType,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Payment ID
    pub payment_id: String,

    /// Merchant ID
    pub merchant_id: String,

    /// Connector name
    pub connector: Option<String>,

    /// Payment method
    pub payment_method: Option<String>,

    /// Amount in minor units
    pub amount: Option<i64>,

    /// Currency code
    pub currency: Option<String>,

    /// Status
    pub status: String,

    /// Error code
    pub error_code: Option<String>,

    /// Error message
    pub error_message: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Detection ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Is anomalous
    pub is_anomaly: bool,

    /// Anomaly score (0.0 - 1.0)
    pub score: f64,

    /// Anomaly type
    pub anomaly_type: AnomalyType,

    /// Affected entity
    pub entity_id: String,

    /// Details
    pub details: String,

    /// Recommended actions
    pub recommended_actions: Vec<String>,
}

/// Anomaly type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyType {
    /// Payment volume spike
    VolumeSpike,
    /// Payment volume drop
    VolumeDrop,
    /// High failure rate
    HighFailureRate,
    /// Unusual payment pattern
    UnusualPattern,
    /// Potential fraud
    PotentialFraud,
    /// Performance degradation
    PerformanceDegradation,
    /// Other anomaly
    Other,
}

/// Routing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Decision ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Payment ID
    pub payment_id: String,

    /// Recommended connector
    pub recommended_connector: String,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,

    /// Alternative connectors
    pub alternatives: Vec<ConnectorScore>,

    /// Decision rationale
    pub rationale: String,

    /// Was prediction correct
    pub was_correct: Option<bool>,
}

/// Connector score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorScore {
    /// Connector name
    pub connector: String,

    /// Score (0.0 - 1.0)
    pub score: f64,

    /// Expected success rate
    pub expected_success_rate: f64,

    /// Expected latency in ms
    pub expected_latency_ms: u64,

    /// Cost estimate
    pub cost_estimate: Option<f64>,
}

/// Self-healing action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingAction {
    /// Action ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Action type
    pub action_type: HealingActionType,

    /// Target entity
    pub target: String,

    /// Action status
    pub status: ActionStatus,

    /// Result message
    pub result_message: Option<String>,

    /// Recovery time in ms
    pub recovery_time_ms: Option<u64>,
}

/// Healing action type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealingActionType {
    /// Retry payment
    RetryPayment,
    /// Switch connector
    SwitchConnector,
    /// Update routing
    UpdateRouting,
    /// Clear cache
    ClearCache,
    /// Restart service
    RestartService,
    /// Scale resources
    ScaleResources,
    /// Custom action
    Custom(String),
}

/// Action status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ActionStatus {
    /// Pending execution
    Pending,
    /// In progress
    InProgress,
    /// Successfully completed
    Success,
    /// Failed
    Failed,
    /// Rolled back
    RolledBack,
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// CPU usage percentage
    pub cpu_usage: f64,

    /// Memory usage percentage
    pub memory_usage: f64,

    /// Active connections
    pub active_connections: u64,

    /// Request rate per second
    pub request_rate: f64,

    /// Average response time in ms
    pub avg_response_time_ms: f64,

    /// Error rate percentage
    pub error_rate: f64,

    /// Queue depth
    pub queue_depth: usize,

    /// Database connection pool usage
    pub db_pool_usage: f64,

    /// Redis connection pool usage
    pub redis_pool_usage: f64,
}

/// Analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    /// Period start
    #[serde(with = "time::serde::rfc3339")]
    pub period_start: OffsetDateTime,

    /// Period end
    #[serde(with = "time::serde::rfc3339")]
    pub period_end: OffsetDateTime,

    /// Total payments
    pub total_payments: u64,

    /// Successful payments
    pub successful_payments: u64,

    /// Failed payments
    pub failed_payments: u64,

    /// Success rate
    pub success_rate: f64,

    /// Total amount processed
    pub total_amount: i64,

    /// Average payment amount
    pub avg_amount: f64,

    /// Top connectors
    pub top_connectors: Vec<ConnectorStats>,

    /// Top payment methods
    pub top_payment_methods: Vec<PaymentMethodStats>,

    /// Anomalies detected
    pub anomalies_detected: u32,

    /// Healing actions taken
    pub healing_actions_taken: u32,
}

/// Connector statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorStats {
    /// Connector name
    pub connector: String,

    /// Total transactions
    pub total_transactions: u64,

    /// Success rate
    pub success_rate: f64,

    /// Average latency in ms
    pub avg_latency_ms: f64,

    /// Total amount
    pub total_amount: i64,
}

/// Payment method statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethodStats {
    /// Payment method
    pub payment_method: String,

    /// Total transactions
    pub total_transactions: u64,

    /// Success rate
    pub success_rate: f64,

    /// Total amount
    pub total_amount: i64,
}

/// Prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Prediction ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Metric being predicted
    pub metric: String,

    /// Predicted values (future time series)
    pub predictions: Vec<TimeSeriesPoint>,

    /// Confidence interval
    pub confidence_interval: (f64, f64),

    /// Model accuracy
    pub model_accuracy: Option<f64>,
}

/// Time series point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Value
    pub value: f64,
}

/// Resource scaling recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    /// Recommendation ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Scaling direction
    pub direction: ScalingDirection,

    /// Target instance count
    pub target_instances: u32,

    /// Current instance count
    pub current_instances: u32,

    /// Reason
    pub reason: String,

    /// Expected impact
    pub expected_impact: String,

    /// Auto-apply
    pub auto_apply: bool,
}

/// Scaling direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScalingDirection {
    /// Scale up
    Up,
    /// Scale down
    Down,
    /// No change needed
    NoChange,
}
