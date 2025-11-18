-- Rollback Autonomous Orchestrator Tables
--
-- This migration removes all tables created for the Autonomous Payment Orchestration System (APOS)

-- Drop tables in reverse order to handle foreign key constraints
DROP TABLE IF EXISTS scaling_events;
DROP TABLE IF EXISTS connector_performance_metrics;
DROP TABLE IF EXISTS routing_decisions;
DROP TABLE IF EXISTS metrics_snapshots;
DROP TABLE IF EXISTS model_training_records;
DROP TABLE IF EXISTS healing_actions;
DROP TABLE IF EXISTS anomaly_records;
DROP TABLE IF EXISTS system_event_log;
DROP TABLE IF EXISTS autonomous_decisions;
