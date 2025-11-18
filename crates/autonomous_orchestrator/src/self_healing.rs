//! Self-healing service for automatic recovery

use crate::{
    config::Settings,
    types::{ActionStatus, HealingAction, HealingActionType, PaymentEvent},
};
use error_stack::{Report, ResultExt};
use parking_lot::Mutex;
use router_env::logger;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Self-healing error
#[derive(Debug, thiserror::Error)]
pub enum SelfHealingError {
    /// Action execution error
    #[error("Action execution error: {0}")]
    Execution(String),

    /// Invalid action
    #[error("Invalid action: {0}")]
    InvalidAction(String),
}

/// Self-healing service
pub struct SelfHealingService {
    /// Configuration
    config: Settings,

    /// Active healing actions
    active_actions: Mutex<HashMap<Uuid, HealingAction>>,

    /// Completed actions history
    action_history: Mutex<VecDeque<HealingAction>>,

    /// Connector failure tracking
    connector_failures: Mutex<HashMap<String, FailureTracker>>,
}

/// Failure tracker for connectors
#[derive(Debug, Clone)]
struct FailureTracker {
    /// Connector name
    connector: String,

    /// Consecutive failures
    consecutive_failures: u32,

    /// Total failures in window
    total_failures: u32,

    /// Last failure time
    last_failure: time::OffsetDateTime,

    /// Is currently failed
    is_failed: bool,
}

impl SelfHealingService {
    /// Create new self-healing service
    pub fn new(config: Settings) -> Self {
        Self {
            config,
            active_actions: Mutex::new(HashMap::new()),
            action_history: Mutex::new(VecDeque::with_capacity(1000)),
            connector_failures: Mutex::new(HashMap::new()),
        }
    }

    /// Evaluate event for healing needs
    pub async fn evaluate_event(
        &mut self,
        event: &PaymentEvent,
    ) -> Result<Option<HealingAction>, Report<SelfHealingError>> {
        if !self.config.self_healing.enabled {
            return Ok(None);
        }

        // Check if event indicates failure
        if event.status == "failed" {
            // Track failure
            if let Some(ref connector) = event.connector {
                self.track_failure(connector);

                // Check if we should take action
                if self.should_heal_connector(connector) {
                    return self.heal_connector_failure(connector, event).await;
                }
            }

            // Check if we should retry the payment
            if self.should_retry_payment(event) {
                return self.retry_payment(event).await;
            }
        } else if event.status == "succeeded" {
            // Reset failure tracking on success
            if let Some(ref connector) = event.connector {
                self.reset_failure_tracking(connector);
            }
        }

        Ok(None)
    }

    /// Track connector failure
    fn track_failure(&self, connector: &str) {
        let mut failures = self.connector_failures.lock();
        let tracker = failures.entry(connector.to_string()).or_insert_with(|| {
            FailureTracker {
                connector: connector.to_string(),
                consecutive_failures: 0,
                total_failures: 0,
                last_failure: time::OffsetDateTime::now_utc(),
                is_failed: false,
            }
        });

        tracker.consecutive_failures += 1;
        tracker.total_failures += 1;
        tracker.last_failure = time::OffsetDateTime::now_utc();

        if tracker.consecutive_failures >= self.config.self_healing.failure_threshold {
            tracker.is_failed = true;
            logger::warn!(
                "Connector {} marked as failed after {} consecutive failures",
                connector,
                tracker.consecutive_failures
            );
        }
    }

    /// Reset failure tracking on success
    fn reset_failure_tracking(&self, connector: &str) {
        let mut failures = self.connector_failures.lock();
        if let Some(tracker) = failures.get_mut(connector) {
            tracker.consecutive_failures = 0;
            tracker.is_failed = false;
            logger::info!("Connector {} recovered", connector);
        }
    }

    /// Check if connector needs healing
    fn should_heal_connector(&self, connector: &str) -> bool {
        let failures = self.connector_failures.lock();
        failures.get(connector)
            .map(|t| t.is_failed && self.config.self_healing.auto_switch_connectors)
            .unwrap_or(false)
    }

    /// Check if payment should be retried
    fn should_retry_payment(&self, event: &PaymentEvent) -> bool {
        // Don't retry certain error codes
        if let Some(ref error_code) = event.error_code {
            match error_code.as_str() {
                "invalid_card" | "expired_card" | "insufficient_funds" => return false,
                _ => {}
            }
        }

        true
    }

    /// Heal connector failure by switching
    async fn heal_connector_failure(
        &mut self,
        connector: &str,
        event: &PaymentEvent,
    ) -> Result<Option<HealingAction>, Report<SelfHealingError>> {
        logger::info!("Initiating healing action for failed connector: {}", connector);

        let action = HealingAction {
            id: Uuid::new_v4(),
            timestamp: time::OffsetDateTime::now_utc(),
            action_type: HealingActionType::SwitchConnector,
            target: event.payment_id.clone(),
            status: ActionStatus::Pending,
            result_message: None,
            recovery_time_ms: None,
        };

        // Store action
        {
            let mut active = self.active_actions.lock();
            active.insert(action.id, action.clone());
        }

        // Execute healing action
        tokio::spawn({
            let action_id = action.id;
            let connector = connector.to_string();
            let payment_id = event.payment_id.clone();
            async move {
                // In production, this would:
                // 1. Select alternative connector
                // 2. Retry payment with new connector
                // 3. Update routing preferences
                // 4. Notify monitoring systems

                logger::info!(
                    "Switching connector for payment {} from {} to alternative",
                    payment_id,
                    connector
                );

                // Simulate healing action
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                logger::info!("Connector switch completed for payment {}", payment_id);
            }
        });

        Ok(Some(action))
    }

    /// Retry failed payment
    async fn retry_payment(
        &mut self,
        event: &PaymentEvent,
    ) -> Result<Option<HealingAction>, Report<SelfHealingError>> {
        logger::info!("Initiating payment retry: {}", event.payment_id);

        let action = HealingAction {
            id: Uuid::new_v4(),
            timestamp: time::OffsetDateTime::now_utc(),
            action_type: HealingActionType::RetryPayment,
            target: event.payment_id.clone(),
            status: ActionStatus::Pending,
            result_message: None,
            recovery_time_ms: None,
        };

        // Store action
        {
            let mut active = self.active_actions.lock();
            active.insert(action.id, action.clone());
        }

        // Execute retry with exponential backoff
        tokio::spawn({
            let action_id = action.id;
            let payment_id = event.payment_id.clone();
            let initial_delay = self.config.self_healing.initial_retry_delay_seconds;
            let max_attempts = self.config.self_healing.max_retry_attempts;
            let backoff = self.config.self_healing.retry_backoff_multiplier;

            async move {
                let mut delay = initial_delay;

                for attempt in 1..=max_attempts {
                    logger::info!(
                        "Retry attempt {}/{} for payment {} (delay: {}s)",
                        attempt,
                        max_attempts,
                        payment_id,
                        delay
                    );

                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;

                    // In production, this would actually retry the payment
                    // For now, simulate with random success
                    let success = rand::random::<f64>() > 0.5;

                    if success {
                        logger::info!("Payment {} retry succeeded on attempt {}", payment_id, attempt);
                        break;
                    }

                    delay = (delay as f64 * backoff) as u64;
                }
            }
        });

        Ok(Some(action))
    }

    /// Complete healing action
    pub fn complete_action(&mut self, action_id: Uuid, status: ActionStatus, result: String) {
        let mut active = self.active_actions.lock();

        if let Some(mut action) = active.remove(&action_id) {
            action.status = status;
            action.result_message = Some(result);

            // Move to history
            let mut history = self.action_history.lock();
            if history.len() >= 1000 {
                history.pop_front();
            }
            history.push_back(action);
        }
    }

    /// Get active actions
    pub fn get_active_actions(&self) -> Vec<HealingAction> {
        self.active_actions.lock().values().cloned().collect()
    }

    /// Get action history
    pub fn get_action_history(&self, limit: usize) -> Vec<HealingAction> {
        let history = self.action_history.lock();
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get healing statistics
    pub fn get_statistics(&self) -> HealingStatistics {
        let active = self.active_actions.lock();
        let history = self.action_history.lock();
        let failures = self.connector_failures.lock();

        let successful = history.iter()
            .filter(|a| a.status == ActionStatus::Success)
            .count();

        let failed = history.iter()
            .filter(|a| a.status == ActionStatus::Failed)
            .count();

        let avg_recovery_time = if successful > 0 {
            history.iter()
                .filter(|a| a.status == ActionStatus::Success && a.recovery_time_ms.is_some())
                .map(|a| a.recovery_time_ms.unwrap_or(0) as f64)
                .sum::<f64>() / successful as f64
        } else {
            0.0
        };

        HealingStatistics {
            active_actions: active.len(),
            total_actions: history.len(),
            successful_actions: successful,
            failed_actions: failed,
            avg_recovery_time_ms: avg_recovery_time,
            tracked_connectors: failures.len(),
            failed_connectors: failures.values().filter(|t| t.is_failed).count(),
        }
    }
}

/// Healing statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealingStatistics {
    /// Active healing actions
    pub active_actions: usize,

    /// Total actions taken
    pub total_actions: usize,

    /// Successful actions
    pub successful_actions: usize,

    /// Failed actions
    pub failed_actions: usize,

    /// Average recovery time in ms
    pub avg_recovery_time_ms: f64,

    /// Number of tracked connectors
    pub tracked_connectors: usize,

    /// Number of currently failed connectors
    pub failed_connectors: usize,
}
