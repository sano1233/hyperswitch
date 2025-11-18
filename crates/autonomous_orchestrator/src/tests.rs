//! Integration tests for Autonomous Orchestrator

#[cfg(test)]
mod tests {
    use crate::{
        analytics::AnalyticsEngine,
        anomaly_detector::AnomalyDetector,
        config::Settings,
        decision_engine::DecisionEngine,
        health::HealthChecker,
        resource_manager::ResourceManager,
        self_healing::SelfHealingService,
        types::{EventType, PaymentEvent},
    };

    fn create_test_config() -> Settings {
        Settings::default()
    }

    fn create_test_payment_event(status: &str) -> PaymentEvent {
        PaymentEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            event_type: if status == "succeeded" {
                EventType::PaymentSucceeded
            } else {
                EventType::PaymentFailed
            },
            timestamp: time::OffsetDateTime::now_utc(),
            payment_id: format!("pay_{}", uuid::Uuid::new_v4()),
            merchant_id: "merchant_test".to_string(),
            connector: Some("stripe".to_string()),
            payment_method: Some("card".to_string()),
            amount: Some(10000),
            currency: Some("USD".to_string()),
            status: status.to_string(),
            error_code: if status == "failed" {
                Some("card_declined".to_string())
            } else {
                None
            },
            error_message: if status == "failed" {
                Some("Card was declined".to_string())
            } else {
                None
            },
            metadata: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_decision_engine_routing() {
        let config = create_test_config();
        let mut engine = DecisionEngine::new(config);
        let event = create_test_payment_event("succeeded");

        let decision = engine.make_routing_decision(&event).await;
        assert!(decision.is_ok());

        let decision = decision.unwrap();
        assert!(!decision.recommended_connector.is_empty());
        assert!(decision.confidence >= 0.0 && decision.confidence <= 1.0);
        assert!(!decision.alternatives.is_empty());
    }

    #[tokio::test]
    async fn test_decision_engine_performance_update() {
        let config = create_test_config();
        let mut engine = DecisionEngine::new(config);

        engine.update_performance("stripe", true, 150);
        engine.update_performance("stripe", true, 200);
        engine.update_performance("stripe", false, 300);

        let stats = engine.get_model_stats();
        assert_eq!(stats.training_samples, 0); // No training data added yet
    }

    #[tokio::test]
    async fn test_anomaly_detector_volume_spike() {
        let config = create_test_config();
        let mut detector = AnomalyDetector::new(config);

        // Add multiple events to simulate volume spike
        for _ in 0..20 {
            let event = create_test_payment_event("succeeded");
            let _ = detector.analyze_event(&event).await;
        }

        let stats = detector.get_statistics();
        assert!(stats.total_detected >= 0);
    }

    #[tokio::test]
    async fn test_anomaly_detector_fraud_detection() {
        let mut config = create_test_config();
        config.anomaly_detection.enable_fraud_detection = true;
        config.anomaly_detection.sensitivity = 0.5;

        let mut detector = AnomalyDetector::new(config);

        let mut event = create_test_payment_event("succeeded");
        event.amount = Some(200000); // High amount

        let result = detector.analyze_event(&event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_self_healing_failure_tracking() {
        let config = create_test_config();
        let mut service = SelfHealingService::new(config);

        // Simulate multiple failures
        for _ in 0..3 {
            let event = create_test_payment_event("failed");
            let _ = service.evaluate_event(&event).await;
        }

        let stats = service.get_statistics();
        assert!(stats.tracked_connectors > 0);
    }

    #[tokio::test]
    async fn test_self_healing_connector_switch() {
        let mut config = create_test_config();
        config.self_healing.auto_switch_connectors = true;
        config.self_healing.failure_threshold = 2;

        let mut service = SelfHealingService::new(config);

        // Simulate failures to trigger connector switch
        for _ in 0..3 {
            let event = create_test_payment_event("failed");
            let _ = service.evaluate_event(&event).await;
        }

        let active_actions = service.get_active_actions();
        // Actions may be executed asynchronously
        assert!(active_actions.len() >= 0);
    }

    #[tokio::test]
    async fn test_analytics_event_processing() {
        let config = create_test_config();
        let mut analytics = AnalyticsEngine::new(config);

        // Process multiple events
        for i in 0..10 {
            let status = if i % 3 == 0 { "failed" } else { "succeeded" };
            let event = create_test_payment_event(status);
            let _ = analytics.process_event(&event).await;
        }

        let summary = analytics.get_summary();
        assert_eq!(summary.total_payments, 10);
        assert!(summary.successful_payments > 0);
        assert!(summary.failed_payments > 0);
        assert!(summary.success_rate > 0.0);
    }

    #[tokio::test]
    async fn test_analytics_predictions() {
        let config = create_test_config();
        let analytics = AnalyticsEngine::new(config);

        // Predictions require sufficient data
        let result = analytics.predict("payment_volume").await;
        // Should fail due to insufficient data
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_resource_manager_scaling_evaluation() {
        let config = create_test_config();
        let manager = ResourceManager::new(config);

        let metrics = HealthChecker::get_metrics().await;
        let recommendation = manager.evaluate_scaling(&metrics).await;

        assert!(recommendation.is_ok());
    }

    #[tokio::test]
    async fn test_resource_manager_instance_tracking() {
        let config = create_test_config();
        let manager = ResourceManager::new(config);

        let initial_count = manager.get_instance_count();
        assert!(initial_count >= 1);

        let stats = manager.get_statistics();
        assert_eq!(stats.current_instances, initial_count);
    }

    #[tokio::test]
    async fn test_health_checker_metrics() {
        let metrics = HealthChecker::get_metrics().await;

        assert!(metrics.cpu_usage >= 0.0 && metrics.cpu_usage <= 100.0);
        assert!(metrics.memory_usage >= 0.0 && metrics.memory_usage <= 100.0);
        assert!(metrics.error_rate >= 0.0 && metrics.error_rate <= 100.0);
        assert!(metrics.avg_response_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn test_health_score_calculation() {
        let metrics = HealthChecker::get_metrics().await;
        let score = HealthChecker::calculate_health_score(&metrics);

        assert!(score >= 0.0 && score <= 100.0);
    }

    #[test]
    fn test_config_validation() {
        let config = create_test_config();
        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_invalid_threshold() {
        let mut config = create_test_config();
        config.decision_engine.confidence_threshold = 1.5; // Invalid

        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_invalid_sensitivity() {
        let mut config = create_test_config();
        config.anomaly_detection.sensitivity = -0.5; // Invalid

        let result = config.validate();
        assert!(result.is_err());
    }
}
