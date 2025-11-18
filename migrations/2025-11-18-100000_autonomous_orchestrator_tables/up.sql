-- Autonomous Orchestrator Tables
--
-- This migration creates tables for the Autonomous Payment Orchestration System (APOS)
-- which provides ML-powered decision making, anomaly detection, self-healing, and analytics.

-- Autonomous decisions table
CREATE TABLE IF NOT EXISTS autonomous_decisions (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    decision_type VARCHAR(64) NOT NULL,
    input_data JSONB NOT NULL,
    output_decision JSONB NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    executed BOOLEAN NOT NULL DEFAULT FALSE,
    execution_result JSONB,
    model_version VARCHAR(32) NOT NULL,
    INDEX idx_autonomous_decisions_created_at (created_at),
    INDEX idx_autonomous_decisions_decision_type (decision_type),
    INDEX idx_autonomous_decisions_executed (executed)
);

-- System event log table
CREATE TABLE IF NOT EXISTS system_event_log (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    event_type VARCHAR(64) NOT NULL,
    source VARCHAR(128) NOT NULL,
    severity VARCHAR(16) NOT NULL,
    event_data JSONB NOT NULL,
    affected_entities TEXT[],
    tags TEXT[],
    INDEX idx_system_event_log_timestamp (timestamp),
    INDEX idx_system_event_log_event_type (event_type),
    INDEX idx_system_event_log_severity (severity),
    INDEX idx_system_event_log_source (source)
);

-- Anomaly records table
CREATE TABLE IF NOT EXISTS anomaly_records (
    id UUID PRIMARY KEY,
    detected_at TIMESTAMP NOT NULL DEFAULT NOW(),
    anomaly_type VARCHAR(64) NOT NULL,
    score DOUBLE PRECISION NOT NULL,
    entity_id VARCHAR(128) NOT NULL,
    entity_type VARCHAR(64) NOT NULL,
    description TEXT NOT NULL,
    details JSONB NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'detected',
    resolution_actions TEXT[],
    resolved_at TIMESTAMP,
    INDEX idx_anomaly_records_detected_at (detected_at),
    INDEX idx_anomaly_records_anomaly_type (anomaly_type),
    INDEX idx_anomaly_records_entity_id (entity_id),
    INDEX idx_anomaly_records_status (status),
    INDEX idx_anomaly_records_score (score)
);

-- Healing actions table
CREATE TABLE IF NOT EXISTS healing_actions (
    id UUID PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    action_type VARCHAR(64) NOT NULL,
    target_entity VARCHAR(128) NOT NULL,
    trigger_event UUID,
    parameters JSONB NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    result_message TEXT,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    recovery_time_ms BIGINT,
    INDEX idx_healing_actions_created_at (created_at),
    INDEX idx_healing_actions_action_type (action_type),
    INDEX idx_healing_actions_target_entity (target_entity),
    INDEX idx_healing_actions_status (status),
    FOREIGN KEY (trigger_event) REFERENCES anomaly_records(id) ON DELETE SET NULL
);

-- Model training records table
CREATE TABLE IF NOT EXISTS model_training_records (
    id UUID PRIMARY KEY,
    model_type VARCHAR(64) NOT NULL,
    version VARCHAR(32) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP,
    samples_count BIGINT NOT NULL,
    validation_accuracy DOUBLE PRECISION,
    parameters JSONB NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    INDEX idx_model_training_records_model_type (model_type),
    INDEX idx_model_training_records_version (version),
    INDEX idx_model_training_records_started_at (started_at),
    INDEX idx_model_training_records_status (status)
);

-- Metrics snapshots table
CREATE TABLE IF NOT EXISTS metrics_snapshots (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    metric_type VARCHAR(64) NOT NULL,
    values JSONB NOT NULL,
    aggregation_period_seconds BIGINT NOT NULL,
    tags TEXT[],
    INDEX idx_metrics_snapshots_timestamp (timestamp),
    INDEX idx_metrics_snapshots_metric_type (metric_type)
);

-- Routing decisions table (for tracking ML routing performance)
CREATE TABLE IF NOT EXISTS routing_decisions (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    payment_id VARCHAR(64) NOT NULL,
    merchant_id VARCHAR(64),
    recommended_connector VARCHAR(64) NOT NULL,
    confidence DOUBLE PRECISION NOT NULL,
    alternatives JSONB,
    rationale TEXT,
    was_correct BOOLEAN,
    actual_connector VARCHAR(64),
    actual_result VARCHAR(32),
    INDEX idx_routing_decisions_timestamp (timestamp),
    INDEX idx_routing_decisions_payment_id (payment_id),
    INDEX idx_routing_decisions_recommended_connector (recommended_connector),
    INDEX idx_routing_decisions_was_correct (was_correct)
);

-- Connector performance metrics table
CREATE TABLE IF NOT EXISTS connector_performance_metrics (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    connector VARCHAR(64) NOT NULL,
    success_count BIGINT NOT NULL DEFAULT 0,
    failure_count BIGINT NOT NULL DEFAULT 0,
    total_latency_ms BIGINT NOT NULL DEFAULT 0,
    avg_latency_ms DOUBLE PRECISION NOT NULL DEFAULT 0,
    success_rate DOUBLE PRECISION NOT NULL DEFAULT 0,
    total_amount BIGINT NOT NULL DEFAULT 0,
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    INDEX idx_connector_performance_timestamp (timestamp),
    INDEX idx_connector_performance_connector (connector),
    UNIQUE (connector, period_start, period_end)
);

-- Scaling events table (for resource management tracking)
CREATE TABLE IF NOT EXISTS scaling_events (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW(),
    direction VARCHAR(16) NOT NULL,
    from_instances INTEGER NOT NULL,
    to_instances INTEGER NOT NULL,
    reason TEXT NOT NULL,
    trigger_metrics JSONB,
    execution_status VARCHAR(32) NOT NULL DEFAULT 'pending',
    execution_result TEXT,
    INDEX idx_scaling_events_timestamp (timestamp),
    INDEX idx_scaling_events_direction (direction),
    INDEX idx_scaling_events_execution_status (execution_status)
);

-- Create hypertables for time-series data (if using TimescaleDB)
-- SELECT create_hypertable('system_event_log', 'timestamp', if_not_exists => TRUE);
-- SELECT create_hypertable('metrics_snapshots', 'timestamp', if_not_exists => TRUE);
-- SELECT create_hypertable('connector_performance_metrics', 'timestamp', if_not_exists => TRUE);

-- Add comments for documentation
COMMENT ON TABLE autonomous_decisions IS 'Stores ML-powered autonomous decisions made by the system';
COMMENT ON TABLE system_event_log IS 'Comprehensive system event log for monitoring and debugging';
COMMENT ON TABLE anomaly_records IS 'Records of detected anomalies with details and resolution status';
COMMENT ON TABLE healing_actions IS 'Self-healing actions taken by the system to recover from failures';
COMMENT ON TABLE model_training_records IS 'ML model training history and performance metrics';
COMMENT ON TABLE metrics_snapshots IS 'Time-series metrics for analytics and monitoring';
COMMENT ON TABLE routing_decisions IS 'Payment routing decisions for tracking and learning';
COMMENT ON TABLE connector_performance_metrics IS 'Historical connector performance data for optimization';
COMMENT ON TABLE scaling_events IS 'Resource scaling events for capacity management tracking';
