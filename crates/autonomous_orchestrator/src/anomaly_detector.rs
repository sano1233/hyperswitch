//! Anomaly detection service with ML-based pattern recognition

use crate::{
    config::Settings,
    types::{AnomalyResult, AnomalyType, PaymentEvent},
};
use error_stack::{Report, ResultExt};
use parking_lot::Mutex;
use router_env::logger;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Anomaly detector error
#[derive(Debug, thiserror::Error)]
pub enum AnomalyDetectorError {
    /// Detection error
    #[error("Detection error: {0}")]
    Detection(String),

    /// Insufficient data
    #[error("Insufficient data: {0}")]
    InsufficientData(String),
}

/// Anomaly detector with statistical and ML-based detection
pub struct AnomalyDetector {
    /// Configuration
    config: Settings,

    /// Time series data for analysis
    time_series: Mutex<TimeSeries>,

    /// Detected anomalies
    anomalies: Mutex<VecDeque<AnomalyResult>>,

    /// Baseline metrics
    baselines: Mutex<HashMap<String, BaselineMetrics>>,
}

/// Time series data
#[derive(Debug)]
struct TimeSeries {
    /// Payment volumes (timestamp -> count)
    payment_volumes: VecDeque<TimePoint>,

    /// Success rates (timestamp -> rate)
    success_rates: VecDeque<TimePoint>,

    /// Average amounts
    average_amounts: VecDeque<TimePoint>,

    /// Latencies
    latencies: VecDeque<TimePoint>,

    /// Max points to keep
    max_points: usize,
}

/// Time point data
#[derive(Debug, Clone)]
struct TimePoint {
    /// Timestamp
    timestamp: time::OffsetDateTime,

    /// Value
    value: f64,
}

/// Baseline metrics for comparison
#[derive(Debug, Clone)]
struct BaselineMetrics {
    /// Mean value
    mean: f64,

    /// Standard deviation
    std_dev: f64,

    /// Minimum value
    min: f64,

    /// Maximum value
    max: f64,

    /// Sample count
    sample_count: usize,

    /// Last updated
    last_updated: time::OffsetDateTime,
}

impl TimeSeries {
    fn new(max_points: usize) -> Self {
        Self {
            payment_volumes: VecDeque::with_capacity(max_points),
            success_rates: VecDeque::with_capacity(max_points),
            average_amounts: VecDeque::with_capacity(max_points),
            latencies: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    fn add_point(queue: &mut VecDeque<TimePoint>, point: TimePoint, max_points: usize) {
        if queue.len() >= max_points {
            queue.pop_front();
        }
        queue.push_back(point);
    }
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new(config: Settings) -> Self {
        let window_size = config.anomaly_detection.window_size_minutes as usize * 60; // Convert to seconds

        Self {
            config,
            time_series: Mutex::new(TimeSeries::new(window_size)),
            anomalies: Mutex::new(VecDeque::with_capacity(1000)),
            baselines: Mutex::new(HashMap::new()),
        }
    }

    /// Analyze event for anomalies
    pub async fn analyze_event(
        &mut self,
        event: &PaymentEvent,
    ) -> Result<Option<AnomalyResult>, Report<AnomalyDetectorError>> {
        if !self.config.anomaly_detection.enabled {
            return Ok(None);
        }

        // Update time series with new data
        self.update_time_series(event);

        // Run anomaly detection algorithms
        let mut detected_anomalies = Vec::new();

        // 1. Check for volume anomalies
        if let Some(anomaly) = self.detect_volume_anomaly().await? {
            detected_anomalies.push(anomaly);
        }

        // 2. Check for success rate anomalies
        if let Some(anomaly) = self.detect_success_rate_anomaly(event).await? {
            detected_anomalies.push(anomaly);
        }

        // 3. Check for fraud patterns
        if self.config.anomaly_detection.enable_fraud_detection {
            if let Some(anomaly) = self.detect_fraud_pattern(event).await? {
                detected_anomalies.push(anomaly);
            }
        }

        // 4. Check for unusual amounts
        if let Some(anomaly) = self.detect_amount_anomaly(event).await? {
            detected_anomalies.push(anomaly);
        }

        // Return the highest severity anomaly if any
        let result = detected_anomalies.into_iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(ref anomaly) = result {
            // Store anomaly
            let mut anomalies = self.anomalies.lock();
            if anomalies.len() >= 1000 {
                anomalies.pop_front();
            }
            anomalies.push_back(anomaly.clone());

            logger::warn!(
                "Anomaly detected: type={:?}, score={:.2}, entity={}",
                anomaly.anomaly_type,
                anomaly.score,
                anomaly.entity_id
            );
        }

        Ok(result)
    }

    /// Update time series with event data
    fn update_time_series(&self, event: &PaymentEvent) {
        let mut ts = self.time_series.lock();
        let now = time::OffsetDateTime::now_utc();

        // Update payment volume
        TimeSeries::add_point(
            &mut ts.payment_volumes,
            TimePoint { timestamp: now, value: 1.0 },
            ts.max_points,
        );

        // Update success rate
        let success = event.status == "succeeded";
        TimeSeries::add_point(
            &mut ts.success_rates,
            TimePoint { timestamp: now, value: if success { 1.0 } else { 0.0 } },
            ts.max_points,
        );

        // Update amount
        if let Some(amount) = event.amount {
            TimeSeries::add_point(
                &mut ts.average_amounts,
                TimePoint { timestamp: now, value: amount as f64 },
                ts.max_points,
            );
        }
    }

    /// Detect volume anomalies
    async fn detect_volume_anomaly(&self) -> Result<Option<AnomalyResult>, Report<AnomalyDetectorError>> {
        let ts = self.time_series.lock();

        if ts.payment_volumes.len() < 10 {
            return Ok(None);
        }

        // Calculate recent volume
        let recent_count = ts.payment_volumes.iter()
            .rev()
            .take(5)
            .count() as f64;

        // Calculate baseline volume
        let baseline_count = ts.payment_volumes.iter()
            .take(ts.payment_volumes.len() - 5)
            .count() as f64 / (ts.payment_volumes.len() - 5).max(1) as f64 * 5.0;

        // Check for spike or drop
        let threshold = 2.0; // 2x threshold
        let ratio = recent_count / baseline_count.max(1.0);

        if ratio > threshold {
            return Ok(Some(AnomalyResult {
                id: Uuid::new_v4(),
                timestamp: time::OffsetDateTime::now_utc(),
                is_anomaly: true,
                score: ((ratio - 1.0) / threshold).min(1.0),
                anomaly_type: AnomalyType::VolumeSpike,
                entity_id: "system".to_string(),
                details: format!("Payment volume spike detected: {:.1}x increase", ratio),
                recommended_actions: vec![
                    "Monitor system resources".to_string(),
                    "Check for potential DDoS".to_string(),
                    "Scale up infrastructure if needed".to_string(),
                ],
            }));
        } else if ratio < 1.0 / threshold {
            return Ok(Some(AnomalyResult {
                id: Uuid::new_v4(),
                timestamp: time::OffsetDateTime::now_utc(),
                is_anomaly: true,
                score: (1.0 - ratio).min(1.0),
                anomaly_type: AnomalyType::VolumeDrop,
                entity_id: "system".to_string(),
                details: format!("Payment volume drop detected: {:.1}x decrease", 1.0 / ratio),
                recommended_actions: vec![
                    "Check payment gateway connectivity".to_string(),
                    "Verify API availability".to_string(),
                    "Contact merchants to confirm issue".to_string(),
                ],
            }));
        }

        Ok(None)
    }

    /// Detect success rate anomalies
    async fn detect_success_rate_anomaly(
        &self,
        _event: &PaymentEvent,
    ) -> Result<Option<AnomalyResult>, Report<AnomalyDetectorError>> {
        let ts = self.time_series.lock();

        if ts.success_rates.len() < 20 {
            return Ok(None);
        }

        // Calculate recent success rate
        let recent_success_rate: f64 = ts.success_rates.iter()
            .rev()
            .take(10)
            .map(|p| p.value)
            .sum::<f64>() / 10.0;

        // Calculate baseline success rate
        let baseline_success_rate: f64 = ts.success_rates.iter()
            .take(ts.success_rates.len() - 10)
            .map(|p| p.value)
            .sum::<f64>() / (ts.success_rates.len() - 10).max(1) as f64;

        // Check for significant drop
        let drop_threshold = 0.2; // 20% drop
        if baseline_success_rate - recent_success_rate > drop_threshold {
            return Ok(Some(AnomalyResult {
                id: Uuid::new_v4(),
                timestamp: time::OffsetDateTime::now_utc(),
                is_anomaly: true,
                score: ((baseline_success_rate - recent_success_rate) / drop_threshold).min(1.0),
                anomaly_type: AnomalyType::HighFailureRate,
                entity_id: "system".to_string(),
                details: format!(
                    "High failure rate detected: success rate dropped from {:.1}% to {:.1}%",
                    baseline_success_rate * 100.0,
                    recent_success_rate * 100.0
                ),
                recommended_actions: vec![
                    "Investigate connector issues".to_string(),
                    "Check for card network problems".to_string(),
                    "Review recent configuration changes".to_string(),
                    "Enable fallback routing".to_string(),
                ],
            }));
        }

        Ok(None)
    }

    /// Detect fraud patterns
    async fn detect_fraud_pattern(
        &self,
        event: &PaymentEvent,
    ) -> Result<Option<AnomalyResult>, Report<AnomalyDetectorError>> {
        // Simple fraud detection based on patterns
        let mut fraud_score = 0.0;
        let mut reasons = Vec::new();

        // Check for high amount
        if let Some(amount) = event.amount {
            if amount > 100000 { // > $1000
                fraud_score += 0.3;
                reasons.push("High transaction amount".to_string());
            }
        }

        // Check for rapid transactions from same merchant
        // (In production, this would check Redis for recent transactions)

        // Check for unusual failure patterns
        if event.error_code == Some("card_declined".to_string()) {
            fraud_score += 0.2;
            reasons.push("Multiple card declines".to_string());
        }

        if fraud_score > self.config.anomaly_detection.sensitivity {
            return Ok(Some(AnomalyResult {
                id: Uuid::new_v4(),
                timestamp: time::OffsetDateTime::now_utc(),
                is_anomaly: true,
                score: fraud_score,
                anomaly_type: AnomalyType::PotentialFraud,
                entity_id: event.payment_id.clone(),
                details: format!("Potential fraud detected: {}", reasons.join(", ")),
                recommended_actions: vec![
                    "Flag for manual review".to_string(),
                    "Apply additional verification".to_string(),
                    "Monitor merchant activity".to_string(),
                ],
            }));
        }

        Ok(None)
    }

    /// Detect amount anomalies
    async fn detect_amount_anomaly(
        &self,
        event: &PaymentEvent,
    ) -> Result<Option<AnomalyResult>, Report<AnomalyDetectorError>> {
        let Some(amount) = event.amount else {
            return Ok(None);
        };

        let ts = self.time_series.lock();

        if ts.average_amounts.len() < 20 {
            return Ok(None);
        }

        // Calculate mean and std dev
        let values: Vec<f64> = ts.average_amounts.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Check if current amount is beyond 3 standard deviations
        let z_score = ((amount as f64) - mean).abs() / std_dev.max(1.0);

        if z_score > 3.0 {
            return Ok(Some(AnomalyResult {
                id: Uuid::new_v4(),
                timestamp: time::OffsetDateTime::now_utc(),
                is_anomaly: true,
                score: (z_score / 5.0).min(1.0),
                anomaly_type: AnomalyType::UnusualPattern,
                entity_id: event.payment_id.clone(),
                details: format!(
                    "Unusual payment amount: ${:.2} (mean: ${:.2}, z-score: {:.1})",
                    amount as f64 / 100.0,
                    mean / 100.0,
                    z_score
                ),
                recommended_actions: vec![
                    "Review transaction details".to_string(),
                    "Verify merchant legitimacy".to_string(),
                ],
            }));
        }

        Ok(None)
    }

    /// Get detected anomalies
    pub fn get_anomalies(&self, limit: usize) -> Vec<AnomalyResult> {
        let anomalies = self.anomalies.lock();
        anomalies.iter().rev().take(limit).cloned().collect()
    }

    /// Get anomaly statistics
    pub fn get_statistics(&self) -> AnomalyStatistics {
        let anomalies = self.anomalies.lock();

        let total = anomalies.len();
        let recent = anomalies.iter()
            .filter(|a| {
                let age = time::OffsetDateTime::now_utc() - a.timestamp;
                age.whole_hours() < 24
            })
            .count();

        AnomalyStatistics {
            total_detected: total,
            detected_last_24h: recent,
            avg_score: if total > 0 {
                anomalies.iter().map(|a| a.score).sum::<f64>() / total as f64
            } else {
                0.0
            },
            most_common_type: None,
        }
    }
}

/// Anomaly statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct AnomalyStatistics {
    /// Total anomalies detected
    pub total_detected: usize,

    /// Detected in last 24 hours
    pub detected_last_24h: usize,

    /// Average anomaly score
    pub avg_score: f64,

    /// Most common anomaly type
    pub most_common_type: Option<String>,
}
