//! Database models for Autonomous Orchestrator

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Autonomous decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousDecision {
    /// Unique identifier
    pub id: Uuid,

    /// Created timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Decision type
    pub decision_type: String,

    /// Input data (JSON)
    pub input_data: serde_json::Value,

    /// Output decision (JSON)
    pub output_decision: serde_json::Value,

    /// Confidence score
    pub confidence: f64,

    /// Was executed
    pub executed: bool,

    /// Execution result (JSON)
    pub execution_result: Option<serde_json::Value>,

    /// Model version
    pub model_version: String,
}

/// System event log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEventLog {
    /// Event ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Event type
    pub event_type: String,

    /// Event source
    pub source: String,

    /// Severity level
    pub severity: EventSeverity,

    /// Event data (JSON)
    pub event_data: serde_json::Value,

    /// Affected entities
    pub affected_entities: Vec<String>,

    /// Tags
    pub tags: Vec<String>,
}

/// Event severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum EventSeverity {
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical level
    Critical,
}

/// Anomaly record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyRecord {
    /// Anomaly ID
    pub id: Uuid,

    /// Detected at
    #[serde(with = "time::serde::rfc3339")]
    pub detected_at: OffsetDateTime,

    /// Anomaly type
    pub anomaly_type: String,

    /// Anomaly score
    pub score: f64,

    /// Entity ID
    pub entity_id: String,

    /// Entity type
    pub entity_type: String,

    /// Description
    pub description: String,

    /// Details (JSON)
    pub details: serde_json::Value,

    /// Status
    pub status: AnomalyStatus,

    /// Resolution actions
    pub resolution_actions: Vec<String>,

    /// Resolved at
    #[serde(with = "time::serde::rfc3339::option")]
    pub resolved_at: Option<OffsetDateTime>,
}

/// Anomaly status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyStatus {
    /// Detected
    Detected,
    /// Under investigation
    Investigating,
    /// Resolved
    Resolved,
    /// False positive
    FalsePositive,
}

/// Healing action record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealingActionRecord {
    /// Action ID
    pub id: Uuid,

    /// Created at
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Action type
    pub action_type: String,

    /// Target entity
    pub target_entity: String,

    /// Trigger event
    pub trigger_event: Option<Uuid>,

    /// Action parameters (JSON)
    pub parameters: serde_json::Value,

    /// Status
    pub status: String,

    /// Result message
    pub result_message: Option<String>,

    /// Started at
    #[serde(with = "time::serde::rfc3339::option")]
    pub started_at: Option<OffsetDateTime>,

    /// Completed at
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,

    /// Recovery time in milliseconds
    pub recovery_time_ms: Option<i64>,
}

/// Model training record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelTrainingRecord {
    /// Training ID
    pub id: Uuid,

    /// Model type
    pub model_type: String,

    /// Model version
    pub version: String,

    /// Training started at
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,

    /// Training completed at
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,

    /// Training samples count
    pub samples_count: i64,

    /// Validation accuracy
    pub validation_accuracy: Option<f64>,

    /// Model parameters (JSON)
    pub parameters: serde_json::Value,

    /// Training status
    pub status: TrainingStatus,

    /// Error message
    pub error_message: Option<String>,
}

/// Training status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TrainingStatus {
    /// Pending
    Pending,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Failed
    Failed,
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    /// Snapshot ID
    pub id: Uuid,

    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,

    /// Metric type
    pub metric_type: String,

    /// Metric values (JSON)
    pub values: serde_json::Value,

    /// Aggregation period in seconds
    pub aggregation_period_seconds: i64,

    /// Tags
    pub tags: Vec<String>,
}
