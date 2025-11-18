//! Auto-scaling resource manager

use crate::{
    config::Settings,
    types::{HealthMetrics, ScalingDirection, ScalingRecommendation},
};
use error_stack::{Report, ResultExt};
use parking_lot::Mutex;
use router_env::logger;
use std::collections::VecDeque;
use uuid::Uuid;

/// Resource manager error
#[derive(Debug, thiserror::Error)]
pub enum ResourceManagerError {
    /// Scaling error
    #[error("Scaling error: {0}")]
    Scaling(String),

    /// Metrics error
    #[error("Metrics error: {0}")]
    Metrics(String),
}

/// Resource manager for auto-scaling
pub struct ResourceManager {
    /// Configuration
    config: Settings,

    /// Current instance count
    current_instances: Mutex<u32>,

    /// Metrics history
    metrics_history: Mutex<VecDeque<HealthMetrics>>,

    /// Last scaling action
    last_scaling: Mutex<Option<time::OffsetDateTime>>,

    /// Scaling history
    scaling_history: Mutex<VecDeque<ScalingEvent>>,
}

/// Scaling event record
#[derive(Debug, Clone)]
struct ScalingEvent {
    /// Timestamp
    timestamp: time::OffsetDateTime,

    /// Direction
    direction: ScalingDirection,

    /// From instance count
    from_instances: u32,

    /// To instance count
    to_instances: u32,

    /// Reason
    reason: String,
}

impl ResourceManager {
    /// Create new resource manager
    pub fn new(config: Settings) -> Self {
        Self {
            current_instances: Mutex::new(config.resource_manager.min_instances),
            metrics_history: Mutex::new(VecDeque::with_capacity(1000)),
            last_scaling: Mutex::new(None),
            scaling_history: Mutex::new(VecDeque::with_capacity(100)),
            config,
        }
    }

    /// Evaluate metrics and recommend scaling
    pub async fn evaluate_scaling(
        &self,
        metrics: &HealthMetrics,
    ) -> Result<Option<ScalingRecommendation>, Report<ResourceManagerError>> {
        if !self.config.resource_manager.enable_auto_scaling {
            return Ok(None);
        }

        // Add metrics to history
        {
            let mut history = self.metrics_history.lock();
            if history.len() >= 1000 {
                history.pop_front();
            }
            history.push_back(metrics.clone());
        }

        // Check if in cooldown period
        if self.is_in_cooldown() {
            return Ok(None);
        }

        // Analyze metrics for scaling decision
        let current = *self.current_instances.lock();
        let recommendation = self.analyze_metrics(metrics, current)?;

        if recommendation.direction != ScalingDirection::NoChange {
            logger::info!(
                "Scaling recommendation: {:?} from {} to {} instances - {}",
                recommendation.direction,
                current,
                recommendation.target_instances,
                recommendation.reason
            );
        }

        Ok(Some(recommendation))
    }

    /// Analyze metrics and determine scaling need
    fn analyze_metrics(
        &self,
        metrics: &HealthMetrics,
        current_instances: u32,
    ) -> Result<ScalingRecommendation, Report<ResourceManagerError>> {
        let mut reasons = Vec::new();
        let mut scale_up_score = 0;
        let mut scale_down_score = 0;

        // Check CPU usage
        if metrics.cpu_usage > self.config.resource_manager.cpu_scale_up_threshold {
            scale_up_score += 2;
            reasons.push(format!("High CPU usage: {:.1}%", metrics.cpu_usage));
        } else if metrics.cpu_usage < self.config.resource_manager.cpu_scale_down_threshold {
            scale_down_score += 1;
            reasons.push(format!("Low CPU usage: {:.1}%", metrics.cpu_usage));
        }

        // Check memory usage
        if metrics.memory_usage > self.config.resource_manager.memory_scale_up_threshold {
            scale_up_score += 2;
            reasons.push(format!("High memory usage: {:.1}%", metrics.memory_usage));
        } else if metrics.memory_usage < self.config.resource_manager.memory_scale_down_threshold {
            scale_down_score += 1;
            reasons.push(format!("Low memory usage: {:.1}%", metrics.memory_usage));
        }

        // Check request rate
        if metrics.request_rate > 1000.0 {
            scale_up_score += 1;
            reasons.push(format!("High request rate: {:.1} req/s", metrics.request_rate));
        } else if metrics.request_rate < 100.0 {
            scale_down_score += 1;
            reasons.push(format!("Low request rate: {:.1} req/s", metrics.request_rate));
        }

        // Check error rate
        if metrics.error_rate > 5.0 {
            scale_up_score += 1;
            reasons.push(format!("High error rate: {:.1}%", metrics.error_rate));
        }

        // Check queue depth
        if metrics.queue_depth > 100 {
            scale_up_score += 2;
            reasons.push(format!("High queue depth: {}", metrics.queue_depth));
        }

        // Determine scaling direction
        let (direction, target_instances, reason) = if scale_up_score >= 2 {
            let target = (current_instances + 1).min(self.config.resource_manager.max_instances);
            (
                if target > current_instances {
                    ScalingDirection::Up
                } else {
                    ScalingDirection::NoChange
                },
                target,
                reasons.join("; "),
            )
        } else if scale_down_score >= 2 && current_instances > self.config.resource_manager.min_instances {
            let target = (current_instances - 1).max(self.config.resource_manager.min_instances);
            (ScalingDirection::Down, target, reasons.join("; "))
        } else {
            (ScalingDirection::NoChange, current_instances, "No scaling needed".to_string())
        };

        let expected_impact = match direction {
            ScalingDirection::Up => format!(
                "Increased capacity, improved response times, reduced error rate"
            ),
            ScalingDirection::Down => format!(
                "Reduced costs, maintained service levels"
            ),
            ScalingDirection::NoChange => "No impact".to_string(),
        };

        Ok(ScalingRecommendation {
            id: Uuid::new_v4(),
            timestamp: time::OffsetDateTime::now_utc(),
            direction,
            target_instances,
            current_instances,
            reason,
            expected_impact,
            auto_apply: self.config.resource_manager.enable_auto_scaling,
        })
    }

    /// Execute scaling action
    pub async fn execute_scaling(
        &mut self,
        recommendation: ScalingRecommendation,
    ) -> Result<(), Report<ResourceManagerError>> {
        if recommendation.direction == ScalingDirection::NoChange {
            return Ok(());
        }

        logger::info!(
            "Executing scaling action: {:?} to {} instances",
            recommendation.direction,
            recommendation.target_instances
        );

        // Update instance count
        let old_count = {
            let mut current = self.current_instances.lock();
            let old = *current;
            *current = recommendation.target_instances;
            old
        };

        // Record scaling event
        {
            let mut history = self.scaling_history.lock();
            if history.len() >= 100 {
                history.pop_front();
            }
            history.push_back(ScalingEvent {
                timestamp: time::OffsetDateTime::now_utc(),
                direction: recommendation.direction,
                from_instances: old_count,
                to_instances: recommendation.target_instances,
                reason: recommendation.reason.clone(),
            });
        }

        // Update last scaling time
        {
            let mut last = self.last_scaling.lock();
            *last = Some(time::OffsetDateTime::now_utc());
        }

        // In production, this would:
        // 1. Call cloud provider API to scale instances
        // 2. Update load balancer configuration
        // 3. Wait for health checks
        // 4. Verify scaling completed successfully

        logger::info!(
            "Scaling completed: {} -> {} instances",
            old_count,
            recommendation.target_instances
        );

        Ok(())
    }

    /// Check if in cooldown period
    fn is_in_cooldown(&self) -> bool {
        let last = self.last_scaling.lock();
        if let Some(last_time) = *last {
            let elapsed = (time::OffsetDateTime::now_utc() - last_time).whole_seconds();
            elapsed < self.config.resource_manager.scale_cooldown_seconds as i64
        } else {
            false
        }
    }

    /// Get current instance count
    pub fn get_instance_count(&self) -> u32 {
        *self.current_instances.lock()
    }

    /// Get scaling history
    pub fn get_scaling_history(&self, limit: usize) -> Vec<ScalingEventInfo> {
        let history = self.scaling_history.lock();
        history.iter()
            .rev()
            .take(limit)
            .map(|e| ScalingEventInfo {
                timestamp: e.timestamp,
                direction: format!("{:?}", e.direction),
                from_instances: e.from_instances,
                to_instances: e.to_instances,
                reason: e.reason.clone(),
            })
            .collect()
    }

    /// Get resource statistics
    pub fn get_statistics(&self) -> ResourceStatistics {
        let history = self.scaling_history.lock();
        let metrics_history = self.metrics_history.lock();

        let scale_up_count = history.iter()
            .filter(|e| matches!(e.direction, ScalingDirection::Up))
            .count();

        let scale_down_count = history.iter()
            .filter(|e| matches!(e.direction, ScalingDirection::Down))
            .count();

        let avg_cpu = if !metrics_history.is_empty() {
            metrics_history.iter().map(|m| m.cpu_usage).sum::<f64>() / metrics_history.len() as f64
        } else {
            0.0
        };

        let avg_memory = if !metrics_history.is_empty() {
            metrics_history.iter().map(|m| m.memory_usage).sum::<f64>() / metrics_history.len() as f64
        } else {
            0.0
        };

        ResourceStatistics {
            current_instances: *self.current_instances.lock(),
            total_scaling_events: history.len(),
            scale_up_events: scale_up_count,
            scale_down_events: scale_down_count,
            avg_cpu_usage: avg_cpu,
            avg_memory_usage: avg_memory,
            is_in_cooldown: self.is_in_cooldown(),
        }
    }
}

/// Scaling event info for API
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScalingEventInfo {
    /// Timestamp
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: time::OffsetDateTime,

    /// Direction
    pub direction: String,

    /// From instance count
    pub from_instances: u32,

    /// To instance count
    pub to_instances: u32,

    /// Reason
    pub reason: String,
}

/// Resource statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceStatistics {
    /// Current instance count
    pub current_instances: u32,

    /// Total scaling events
    pub total_scaling_events: usize,

    /// Scale up events
    pub scale_up_events: usize,

    /// Scale down events
    pub scale_down_events: usize,

    /// Average CPU usage
    pub avg_cpu_usage: f64,

    /// Average memory usage
    pub avg_memory_usage: f64,

    /// Is in cooldown
    pub is_in_cooldown: bool,
}
