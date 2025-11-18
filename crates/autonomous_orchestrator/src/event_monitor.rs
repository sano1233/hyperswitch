//! Event monitoring system

use crate::{
    state::AppState,
    types::{EventType, PaymentEvent},
};
use error_stack::{Report, ResultExt};
use router_env::logger;
use std::sync::Arc;
use tokio::{sync::RwLock, time::{interval, Duration}};

/// Event monitor error
#[derive(Debug, thiserror::Error)]
pub enum EventMonitorError {
    /// Redis error
    #[error("Redis error: {0}")]
    Redis(String),

    /// Processing error
    #[error("Event processing error: {0}")]
    Processing(String),
}

/// Event monitor service
pub struct EventMonitor {
    /// Application state
    state: Arc<RwLock<AppState>>,
}

impl EventMonitor {
    /// Create new event monitor
    pub fn new(state: Arc<RwLock<AppState>>) -> Self {
        Self { state }
    }

    /// Start monitoring events
    pub async fn start(self) -> Result<(), Report<EventMonitorError>> {
        logger::info!("Event monitor starting...");

        let poll_interval = {
            let state = self.state.read().await;
            state.config.event_monitor.poll_interval_ms
        };

        let mut ticker = interval(Duration::from_millis(poll_interval));

        loop {
            ticker.tick().await;

            if let Err(e) = self.poll_events().await {
                logger::error!("Error polling events: {:?}", e);
                // Continue monitoring even if there's an error
            }
        }
    }

    /// Poll for new events
    async fn poll_events(&self) -> Result<(), Report<EventMonitorError>> {
        let state = self.state.read().await;

        // Check if event monitoring is enabled
        if !state.config.event_monitor.enabled {
            return Ok(());
        }

        // Simulate event polling (in production, this would read from Redis Streams)
        // For now, we'll just process synthetic events for demonstration

        // Process events through different systems
        self.process_events(&state).await?;

        Ok(())
    }

    /// Process events through autonomous systems
    async fn process_events(&self, state: &AppState) -> Result<(), Report<EventMonitorError>> {
        // In a real implementation, this would:
        // 1. Read events from Redis Streams
        // 2. Parse events
        // 3. Send to anomaly detector
        // 4. Send to decision engine
        // 5. Trigger self-healing if needed
        // 6. Update analytics

        // For now, generate sample event for testing
        let sample_event = self.generate_sample_event();

        // Send to anomaly detector
        {
            let mut detector = state.anomaly_detector.write();
            if let Err(e) = detector.analyze_event(&sample_event).await {
                logger::warn!("Anomaly detection failed: {:?}", e);
            }
        }

        // Update analytics
        {
            let mut analytics = state.analytics.write();
            if let Err(e) = analytics.process_event(&sample_event).await {
                logger::warn!("Analytics processing failed: {:?}", e);
            }
        }

        // Check if healing is needed
        {
            let mut healing = state.self_healing.write();
            if let Err(e) = healing.evaluate_event(&sample_event).await {
                logger::warn!("Self-healing evaluation failed: {:?}", e);
            }
        }

        Ok(())
    }

    /// Generate sample event for testing
    fn generate_sample_event(&self) -> PaymentEvent {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let event_types = vec![
            EventType::PaymentCreated,
            EventType::PaymentSucceeded,
            EventType::PaymentFailed,
        ];

        let event_type = event_types[rng.gen_range(0..event_types.len())].clone();
        let is_success = matches!(event_type, EventType::PaymentSucceeded);

        PaymentEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: time::OffsetDateTime::now_utc(),
            payment_id: format!("pay_{}", uuid::Uuid::new_v4()),
            merchant_id: format!("merchant_{}", rng.gen_range(1..100)),
            connector: Some(vec!["stripe", "adyen", "checkout", "braintree"][rng.gen_range(0..4)].to_string()),
            payment_method: Some(vec!["card", "wallet", "bank_transfer"][rng.gen_range(0..3)].to_string()),
            amount: Some(rng.gen_range(1000..100000)),
            currency: Some("USD".to_string()),
            status: if is_success { "succeeded" } else { "failed" }.to_string(),
            error_code: if is_success { None } else { Some("card_declined".to_string()) },
            error_message: if is_success { None } else { Some("Card was declined".to_string()) },
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Subscribe to specific event types
    pub async fn subscribe(&self, event_types: Vec<EventType>) -> Result<(), Report<EventMonitorError>> {
        logger::info!("Subscribing to event types: {:?}", event_types);

        // In production, this would set up Redis Stream consumer groups
        // for specific event patterns

        Ok(())
    }

    /// Get event statistics
    pub async fn get_statistics(&self) -> EventStatistics {
        EventStatistics {
            total_events_processed: 0,
            events_per_second: 0.0,
            last_event_timestamp: None,
            active_subscriptions: 0,
        }
    }
}

/// Event statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventStatistics {
    /// Total events processed
    pub total_events_processed: u64,

    /// Events per second
    pub events_per_second: f64,

    /// Last event timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_event_timestamp: Option<time::OffsetDateTime>,

    /// Active subscriptions
    pub active_subscriptions: usize,
}
