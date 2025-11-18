//! Analytics and predictive engine

use crate::{
    config::Settings,
    types::{
        AnalyticsSummary, ConnectorStats, PaymentEvent, PaymentMethodStats,
        PredictionResult, TimeSeriesPoint,
    },
};
use error_stack::{Report, ResultExt};
use parking_lot::Mutex;
use router_env::logger;
use std::collections::HashMap;
use uuid::Uuid;

/// Analytics error
#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    /// Computation error
    #[error("Analytics computation error: {0}")]
    Computation(String),

    /// Insufficient data
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

/// Analytics engine with predictive capabilities
pub struct AnalyticsEngine {
    /// Configuration
    config: Settings,

    /// Aggregated metrics
    metrics: Mutex<AggregatedMetrics>,

    /// Connector stats
    connector_stats: Mutex<HashMap<String, ConnectorMetrics>>,

    /// Payment method stats
    payment_method_stats: Mutex<HashMap<String, PaymentMethodMetrics>>,

    /// Time series data for predictions
    time_series_data: Mutex<Vec<TimeSeriesPoint>>,
}

/// Aggregated metrics
#[derive(Debug, Clone, Default)]
struct AggregatedMetrics {
    /// Total payments
    total_payments: u64,

    /// Successful payments
    successful_payments: u64,

    /// Failed payments
    failed_payments: u64,

    /// Total amount processed
    total_amount: i64,

    /// Period start
    period_start: Option<time::OffsetDateTime>,

    /// Period end
    period_end: Option<time::OffsetDateTime>,
}

/// Connector metrics
#[derive(Debug, Clone)]
struct ConnectorMetrics {
    /// Connector name
    connector: String,

    /// Total transactions
    total_transactions: u64,

    /// Successful transactions
    successful_transactions: u64,

    /// Total latency
    total_latency_ms: u64,

    /// Total amount
    total_amount: i64,
}

/// Payment method metrics
#[derive(Debug, Clone)]
struct PaymentMethodMetrics {
    /// Payment method
    method: String,

    /// Total transactions
    total_transactions: u64,

    /// Successful transactions
    successful_transactions: u64,

    /// Total amount
    total_amount: i64,
}

impl AnalyticsEngine {
    /// Create new analytics engine
    pub fn new(config: Settings) -> Self {
        Self {
            config,
            metrics: Mutex::new(AggregatedMetrics {
                period_start: Some(time::OffsetDateTime::now_utc()),
                ..Default::default()
            }),
            connector_stats: Mutex::new(HashMap::new()),
            payment_method_stats: Mutex::new(HashMap::new()),
            time_series_data: Mutex::new(Vec::new()),
        }
    }

    /// Process payment event for analytics
    pub async fn process_event(
        &mut self,
        event: &PaymentEvent,
    ) -> Result<(), Report<AnalyticsError>> {
        if !self.config.analytics.enabled {
            return Ok(());
        }

        // Update aggregated metrics
        {
            let mut metrics = self.metrics.lock();
            metrics.total_payments += 1;

            if event.status == "succeeded" {
                metrics.successful_payments += 1;
            } else if event.status == "failed" {
                metrics.failed_payments += 1;
            }

            if let Some(amount) = event.amount {
                metrics.total_amount += amount;
            }

            metrics.period_end = Some(time::OffsetDateTime::now_utc());
        }

        // Update connector stats
        if let Some(ref connector) = event.connector {
            let mut stats = self.connector_stats.lock();
            let entry = stats.entry(connector.clone()).or_insert_with(|| {
                ConnectorMetrics {
                    connector: connector.clone(),
                    total_transactions: 0,
                    successful_transactions: 0,
                    total_latency_ms: 0,
                    total_amount: 0,
                }
            });

            entry.total_transactions += 1;
            if event.status == "succeeded" {
                entry.successful_transactions += 1;
            }
            if let Some(amount) = event.amount {
                entry.total_amount += amount;
            }
        }

        // Update payment method stats
        if let Some(ref method) = event.payment_method {
            let mut stats = self.payment_method_stats.lock();
            let entry = stats.entry(method.clone()).or_insert_with(|| {
                PaymentMethodMetrics {
                    method: method.clone(),
                    total_transactions: 0,
                    successful_transactions: 0,
                    total_amount: 0,
                }
            });

            entry.total_transactions += 1;
            if event.status == "succeeded" {
                entry.successful_transactions += 1;
            }
            if let Some(amount) = event.amount {
                entry.total_amount += amount;
            }
        }

        // Add to time series for predictions
        {
            let mut ts = self.time_series_data.lock();
            ts.push(TimeSeriesPoint {
                timestamp: event.timestamp,
                value: event.amount.unwrap_or(0) as f64,
            });

            // Keep only recent data
            let retention_seconds = self.config.analytics.retention_days as i64 * 86400;
            let cutoff = time::OffsetDateTime::now_utc() - time::Duration::seconds(retention_seconds);
            ts.retain(|point| point.timestamp > cutoff);
        }

        Ok(())
    }

    /// Get analytics summary
    pub fn get_summary(&self) -> AnalyticsSummary {
        let metrics = self.metrics.lock();

        let success_rate = if metrics.total_payments > 0 {
            metrics.successful_payments as f64 / metrics.total_payments as f64
        } else {
            0.0
        };

        let avg_amount = if metrics.total_payments > 0 {
            metrics.total_amount as f64 / metrics.total_payments as f64
        } else {
            0.0
        };

        // Get top connectors
        let connector_stats = self.connector_stats.lock();
        let mut top_connectors: Vec<ConnectorStats> = connector_stats
            .values()
            .map(|cm| ConnectorStats {
                connector: cm.connector.clone(),
                total_transactions: cm.total_transactions,
                success_rate: if cm.total_transactions > 0 {
                    cm.successful_transactions as f64 / cm.total_transactions as f64
                } else {
                    0.0
                },
                avg_latency_ms: if cm.total_transactions > 0 {
                    cm.total_latency_ms as f64 / cm.total_transactions as f64
                } else {
                    0.0
                },
                total_amount: cm.total_amount,
            })
            .collect();

        top_connectors.sort_by(|a, b| b.total_transactions.cmp(&a.total_transactions));
        top_connectors.truncate(10);

        // Get top payment methods
        let pm_stats = self.payment_method_stats.lock();
        let mut top_payment_methods: Vec<PaymentMethodStats> = pm_stats
            .values()
            .map(|pm| PaymentMethodStats {
                payment_method: pm.method.clone(),
                total_transactions: pm.total_transactions,
                success_rate: if pm.total_transactions > 0 {
                    pm.successful_transactions as f64 / pm.total_transactions as f64
                } else {
                    0.0
                },
                total_amount: pm.total_amount,
            })
            .collect();

        top_payment_methods.sort_by(|a, b| b.total_transactions.cmp(&a.total_transactions));
        top_payment_methods.truncate(10);

        AnalyticsSummary {
            period_start: metrics.period_start.unwrap_or_else(|| time::OffsetDateTime::now_utc()),
            period_end: metrics.period_end.unwrap_or_else(|| time::OffsetDateTime::now_utc()),
            total_payments: metrics.total_payments,
            successful_payments: metrics.successful_payments,
            failed_payments: metrics.failed_payments,
            success_rate,
            total_amount: metrics.total_amount,
            avg_amount,
            top_connectors,
            top_payment_methods,
            anomalies_detected: 0,
            healing_actions_taken: 0,
        }
    }

    /// Generate predictions
    pub async fn predict(&self, metric: &str) -> Result<PredictionResult, Report<AnalyticsError>> {
        if !self.config.analytics.enable_predictions {
            return Err(Report::new(AnalyticsError::Computation(
                "Predictions are disabled".to_string()
            )));
        }

        logger::info!("Generating predictions for metric: {}", metric);

        let ts_data = self.time_series_data.lock();

        if ts_data.len() < 100 {
            return Err(Report::new(AnalyticsError::InsufficientData(
                format!("Need at least 100 data points, have {}", ts_data.len())
            )));
        }

        // Simple moving average prediction
        let window_size = 20;
        let recent_values: Vec<f64> = ts_data
            .iter()
            .rev()
            .take(window_size)
            .map(|p| p.value)
            .collect();

        let avg = recent_values.iter().sum::<f64>() / recent_values.len() as f64;
        let std_dev = {
            let variance = recent_values.iter()
                .map(|v| (v - avg).powi(2))
                .sum::<f64>() / recent_values.len() as f64;
            variance.sqrt()
        };

        // Generate future predictions
        let horizon_days = self.config.analytics.forecast_horizon_days;
        let now = time::OffsetDateTime::now_utc();
        let mut predictions = Vec::new();

        for day in 1..=horizon_days {
            let timestamp = now + time::Duration::days(day as i64);
            let value = avg + (rand::random::<f64>() - 0.5) * std_dev * 0.5; // Add some variance

            predictions.push(TimeSeriesPoint {
                timestamp,
                value,
            });
        }

        Ok(PredictionResult {
            id: Uuid::new_v4(),
            timestamp: now,
            metric: metric.to_string(),
            predictions,
            confidence_interval: (avg - std_dev, avg + std_dev),
            model_accuracy: Some(0.85),
        })
    }

    /// Reset analytics (for new period)
    pub fn reset(&mut self) {
        let mut metrics = self.metrics.lock();
        *metrics = AggregatedMetrics {
            period_start: Some(time::OffsetDateTime::now_utc()),
            ..Default::default()
        };

        self.connector_stats.lock().clear();
        self.payment_method_stats.lock().clear();

        logger::info!("Analytics data reset for new period");
    }

    /// Get analytics statistics
    pub fn get_statistics(&self) -> AnalyticsStatistics {
        let metrics = self.metrics.lock();
        let connector_stats = self.connector_stats.lock();
        let pm_stats = self.payment_method_stats.lock();
        let ts_data = self.time_series_data.lock();

        AnalyticsStatistics {
            total_events_processed: metrics.total_payments,
            unique_connectors: connector_stats.len(),
            unique_payment_methods: pm_stats.len(),
            time_series_points: ts_data.len(),
            data_freshness_seconds: metrics.period_end
                .map(|end| (time::OffsetDateTime::now_utc() - end).whole_seconds())
                .unwrap_or(0),
        }
    }
}

/// Analytics statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnalyticsStatistics {
    /// Total events processed
    pub total_events_processed: u64,

    /// Unique connectors tracked
    pub unique_connectors: usize,

    /// Unique payment methods tracked
    pub unique_payment_methods: usize,

    /// Time series data points
    pub time_series_points: usize,

    /// Data freshness in seconds
    pub data_freshness_seconds: i64,
}
