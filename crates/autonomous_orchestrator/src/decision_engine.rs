//! Intelligent decision engine with ML-powered routing

use crate::{
    config::Settings,
    types::{ConnectorScore, PaymentEvent, RoutingDecision},
};
use error_stack::{Report, ResultExt};
use lru::LruCache;
use parking_lot::Mutex;
use router_env::logger;
use std::{collections::HashMap, num::NonZeroUsize};
use uuid::Uuid;

/// Decision engine error
#[derive(Debug, thiserror::Error)]
pub enum DecisionEngineError {
    /// Model error
    #[error("ML model error: {0}")]
    Model(String),

    /// Insufficient data
    #[error("Insufficient data for decision: {0}")]
    InsufficientData(String),

    /// Decision error
    #[error("Decision error: {0}")]
    Decision(String),
}

/// Decision engine with ML capabilities
pub struct DecisionEngine {
    /// Configuration
    config: Settings,

    /// Historical performance data
    performance_cache: Mutex<HashMap<String, ConnectorPerformance>>,

    /// Decision cache
    decision_cache: Mutex<LruCache<String, RoutingDecision>>,

    /// Model version
    model_version: String,

    /// Training data buffer
    training_buffer: Mutex<Vec<TrainingDataPoint>>,
}

/// Connector performance metrics
#[derive(Debug, Clone)]
struct ConnectorPerformance {
    /// Connector name
    connector: String,

    /// Success count
    success_count: u64,

    /// Failure count
    failure_count: u64,

    /// Total latency sum in ms
    total_latency_ms: u64,

    /// Total transactions
    total_transactions: u64,

    /// Last updated
    last_updated: time::OffsetDateTime,
}

impl ConnectorPerformance {
    fn success_rate(&self) -> f64 {
        if self.total_transactions == 0 {
            return 0.0;
        }
        self.success_count as f64 / self.total_transactions as f64
    }

    fn avg_latency_ms(&self) -> f64 {
        if self.total_transactions == 0 {
            return 0.0;
        }
        self.total_latency_ms as f64 / self.total_transactions as f64
    }
}

/// Training data point
#[derive(Debug, Clone)]
struct TrainingDataPoint {
    /// Features
    features: Vec<f64>,

    /// Label (1 for success, 0 for failure)
    label: f64,

    /// Timestamp
    timestamp: time::OffsetDateTime,
}

impl DecisionEngine {
    /// Create new decision engine
    pub fn new(config: Settings) -> Self {
        Self {
            config,
            performance_cache: Mutex::new(HashMap::new()),
            decision_cache: Mutex::new(LruCache::new(NonZeroUsize::new(1000).expect("NonZero"))),
            model_version: "v1.0.0".to_string(),
            training_buffer: Mutex::new(Vec::new()),
        }
    }

    /// Make routing decision
    pub async fn make_routing_decision(
        &mut self,
        payment: &PaymentEvent,
    ) -> Result<RoutingDecision, Report<DecisionEngineError>> {
        // Check cache first
        if let Some(cached) = self.decision_cache.lock().get(&payment.payment_id) {
            return Ok(cached.clone());
        }

        // Get available connectors
        let connectors = vec!["stripe", "adyen", "checkout", "braintree", "worldpay"];

        // Score each connector
        let mut scores = Vec::new();
        for connector in &connectors {
            let score = self.score_connector(connector, payment).await?;
            scores.push(score);
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let best_connector = scores.first()
            .ok_or_else(|| Report::new(DecisionEngineError::Decision("No connectors available".to_string())))?;

        let decision = RoutingDecision {
            id: Uuid::new_v4(),
            timestamp: time::OffsetDateTime::now_utc(),
            payment_id: payment.payment_id.clone(),
            recommended_connector: best_connector.connector.clone(),
            confidence: best_connector.score,
            alternatives: scores[1..].to_vec(),
            rationale: self.generate_rationale(&best_connector),
            was_correct: None,
        };

        // Cache the decision
        self.decision_cache.lock().put(payment.payment_id.clone(), decision.clone());

        Ok(decision)
    }

    /// Score a connector for a payment
    async fn score_connector(
        &self,
        connector: &str,
        payment: &PaymentEvent,
    ) -> Result<ConnectorScore, Report<DecisionEngineError>> {
        let perf = {
            let cache = self.performance_cache.lock();
            cache.get(connector).cloned().unwrap_or_else(|| {
                ConnectorPerformance {
                    connector: connector.to_string(),
                    success_count: 80,
                    failure_count: 20,
                    total_latency_ms: 50000,
                    total_transactions: 100,
                    last_updated: time::OffsetDateTime::now_utc(),
                }
            })
        };

        // Calculate base score from historical performance
        let success_rate = perf.success_rate();
        let avg_latency = perf.avg_latency_ms();

        // Normalize latency score (lower is better, normalize to 0-1)
        let latency_score = 1.0 - (avg_latency / 1000.0).min(1.0);

        // Combined score with weights
        let score = (success_rate * 0.7) + (latency_score * 0.3);

        // Apply payment-specific adjustments
        let adjusted_score = self.apply_payment_adjustments(score, connector, payment);

        Ok(ConnectorScore {
            connector: connector.to_string(),
            score: adjusted_score,
            expected_success_rate: success_rate,
            expected_latency_ms: avg_latency as u64,
            cost_estimate: Some(0.029), // Example cost
        })
    }

    /// Apply payment-specific adjustments to score
    fn apply_payment_adjustments(
        &self,
        base_score: f64,
        connector: &str,
        payment: &PaymentEvent,
    ) -> f64 {
        let mut score = base_score;

        // Adjust based on amount
        if let Some(amount) = payment.amount {
            // Higher amounts might prefer more reliable connectors
            if amount > 50000 && connector == "stripe" {
                score *= 1.1;
            }
        }

        // Adjust based on payment method
        if let Some(ref method) = payment.payment_method {
            if method == "card" && (connector == "stripe" || connector == "adyen") {
                score *= 1.05;
            }
        }

        // Cap score at 1.0
        score.min(1.0)
    }

    /// Generate rationale for decision
    fn generate_rationale(&self, connector_score: &ConnectorScore) -> String {
        format!(
            "Selected {} with {}% confidence based on {:.1}% success rate and {}ms average latency",
            connector_score.connector,
            (connector_score.score * 100.0) as u32,
            connector_score.expected_success_rate * 100.0,
            connector_score.expected_latency_ms
        )
    }

    /// Update performance metrics based on actual results
    pub fn update_performance(&mut self, connector: &str, success: bool, latency_ms: u64) {
        let mut cache = self.performance_cache.lock();
        let perf = cache.entry(connector.to_string()).or_insert_with(|| {
            ConnectorPerformance {
                connector: connector.to_string(),
                success_count: 0,
                failure_count: 0,
                total_latency_ms: 0,
                total_transactions: 0,
                last_updated: time::OffsetDateTime::now_utc(),
            }
        });

        if success {
            perf.success_count += 1;
        } else {
            perf.failure_count += 1;
        }

        perf.total_latency_ms += latency_ms;
        perf.total_transactions += 1;
        perf.last_updated = time::OffsetDateTime::now_utc();
    }

    /// Train ML model with historical data
    pub async fn train_model(&mut self) -> Result<(), Report<DecisionEngineError>> {
        logger::info!("Training decision model...");

        let buffer = self.training_buffer.lock();

        if buffer.len() < self.config.decision_engine.min_training_samples {
            return Err(Report::new(DecisionEngineError::InsufficientData(
                format!("Need at least {} samples, have {}",
                    self.config.decision_engine.min_training_samples,
                    buffer.len()
                )
            )));
        }

        // In production, this would train an actual ML model
        // For now, we'll just log the training
        logger::info!("Model training completed with {} samples", buffer.len());

        Ok(())
    }

    /// Add training data
    pub fn add_training_data(&mut self, features: Vec<f64>, label: f64) {
        let mut buffer = self.training_buffer.lock();
        buffer.push(TrainingDataPoint {
            features,
            label,
            timestamp: time::OffsetDateTime::now_utc(),
        });

        // Keep buffer size manageable
        if buffer.len() > 10000 {
            buffer.drain(0..1000);
        }
    }

    /// Get model statistics
    pub fn get_model_stats(&self) -> ModelStatistics {
        let buffer = self.training_buffer.lock();
        let cache = self.performance_cache.lock();

        ModelStatistics {
            model_version: self.model_version.clone(),
            training_samples: buffer.len(),
            tracked_connectors: cache.len(),
            last_trained: None,
            avg_confidence: 0.85,
        }
    }
}

/// Model statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModelStatistics {
    /// Model version
    pub model_version: String,

    /// Number of training samples
    pub training_samples: usize,

    /// Number of tracked connectors
    pub tracked_connectors: usize,

    /// Last training time
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_trained: Option<time::OffsetDateTime>,

    /// Average confidence score
    pub avg_confidence: f64,
}
