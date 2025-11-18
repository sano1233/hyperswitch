//! HTTP API routes

use crate::{
    health::{HealthCheckResponse, HealthChecker, SystemInfo},
    state::AppState,
};
use actix_web::{get, post, web, HttpResponse, Responder};
use router_env::logger;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configure all routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health_check)
            .service(get_system_status)
            .service(get_analytics_summary)
            .service(get_anomalies)
            .service(get_healing_actions)
            .service(get_routing_stats)
            .service(get_resource_stats)
            .service(trigger_model_training)
            .service(generate_prediction)
            .service(evaluate_scaling),
    );
}

/// Health check endpoint
#[get("/health")]
async fn health_check(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let metrics = HealthChecker::get_metrics().await;
    let score = HealthChecker::calculate_health_score(&metrics);
    let status = HealthChecker::get_health_status(score);

    let response = HealthCheckResponse {
        status,
        score,
        metrics,
        system_info: SystemInfo::new(),
    };

    HttpResponse::Ok().json(response)
}

/// Get comprehensive system status
#[get("/status")]
async fn get_system_status(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let state = state.read().await;

    let decision_stats = state.decision_engine.read().get_model_stats();
    let anomaly_stats = state.anomaly_detector.read().get_statistics();
    let healing_stats = state.self_healing.read().get_statistics();
    let analytics_stats = state.analytics.read().get_statistics();
    let resource_stats = state.resource_manager.read().get_statistics();

    let response = SystemStatusResponse {
        decision_engine: decision_stats,
        anomaly_detection: anomaly_stats,
        self_healing: healing_stats,
        analytics: analytics_stats,
        resource_management: resource_stats,
        health_score: state.get_health_score(),
    };

    HttpResponse::Ok().json(response)
}

/// Get analytics summary
#[get("/analytics/summary")]
async fn get_analytics_summary(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let state = state.read().await;
    let summary = state.analytics.read().get_summary();

    HttpResponse::Ok().json(summary)
}

/// Get detected anomalies
#[get("/anomalies")]
async fn get_anomalies(
    state: web::Data<Arc<RwLock<AppState>>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let state = state.read().await;
    let limit = query.limit.unwrap_or(50).min(100);
    let anomalies = state.anomaly_detector.read().get_anomalies(limit);

    HttpResponse::Ok().json(anomalies)
}

/// Get healing actions
#[get("/healing/actions")]
async fn get_healing_actions(
    state: web::Data<Arc<RwLock<AppState>>>,
    query: web::Query<PaginationQuery>,
) -> impl Responder {
    let state = state.read().await;
    let limit = query.limit.unwrap_or(50).min(100);
    let actions = state.self_healing.read().get_action_history(limit);

    HttpResponse::Ok().json(actions)
}

/// Get routing statistics
#[get("/routing/stats")]
async fn get_routing_stats(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let state = state.read().await;
    let stats = state.decision_engine.read().get_model_stats();

    HttpResponse::Ok().json(stats)
}

/// Get resource management statistics
#[get("/resources/stats")]
async fn get_resource_stats(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let state = state.read().await;
    let stats = state.resource_manager.read().get_statistics();
    let scaling_history = state.resource_manager.read().get_scaling_history(20);

    let response = ResourceStatsResponse {
        statistics: stats,
        scaling_history,
    };

    HttpResponse::Ok().json(response)
}

/// Trigger model training
#[post("/ml/train")]
async fn trigger_model_training(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    logger::info!("Manual model training triggered");

    let state = state.read().await;
    let mut engine = state.decision_engine.write();

    match engine.train_model().await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "message": "Model training completed successfully"
        })),
        Err(e) => {
            logger::error!("Model training failed: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Training failed: {}", e)
            }))
        }
    }
}

/// Generate prediction
#[post("/analytics/predict")]
async fn generate_prediction(
    state: web::Data<Arc<RwLock<AppState>>>,
    body: web::Json<PredictionRequest>,
) -> impl Responder {
    let state = state.read().await;
    let analytics = state.analytics.read();

    match analytics.predict(&body.metric).await {
        Ok(prediction) => HttpResponse::Ok().json(prediction),
        Err(e) => {
            logger::error!("Prediction generation failed: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Prediction failed: {}", e)
            }))
        }
    }
}

/// Evaluate scaling needs
#[post("/resources/evaluate-scaling")]
async fn evaluate_scaling(state: web::Data<Arc<RwLock<AppState>>>) -> impl Responder {
    let state = state.read().await;

    // Get current metrics
    let metrics = HealthChecker::get_metrics().await;

    // Evaluate scaling
    let resource_manager = state.resource_manager.read();
    match resource_manager.evaluate_scaling(&metrics).await {
        Ok(Some(recommendation)) => HttpResponse::Ok().json(recommendation),
        Ok(None) => HttpResponse::Ok().json(serde_json::json!({
            "status": "no_action_needed",
            "message": "System resources are optimal"
        })),
        Err(e) => {
            logger::error!("Scaling evaluation failed: {:?}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": format!("Evaluation failed: {}", e)
            }))
        }
    }
}

// ===== Request/Response Types =====

/// Pagination query parameters
#[derive(Debug, Deserialize)]
struct PaginationQuery {
    /// Number of items to return
    limit: Option<usize>,
    /// Offset for pagination
    #[allow(dead_code)]
    offset: Option<usize>,
}

/// System status response
#[derive(Debug, Serialize)]
struct SystemStatusResponse {
    /// Decision engine stats
    decision_engine: crate::decision_engine::ModelStatistics,
    /// Anomaly detection stats
    anomaly_detection: crate::anomaly_detector::AnomalyStatistics,
    /// Self-healing stats
    self_healing: crate::self_healing::HealingStatistics,
    /// Analytics stats
    analytics: crate::analytics::AnalyticsStatistics,
    /// Resource management stats
    resource_management: crate::resource_manager::ResourceStatistics,
    /// Overall health score
    health_score: f64,
}

/// Resource stats response
#[derive(Debug, Serialize)]
struct ResourceStatsResponse {
    /// Statistics
    statistics: crate::resource_manager::ResourceStatistics,
    /// Scaling history
    scaling_history: Vec<crate::resource_manager::ScalingEventInfo>,
}

/// Prediction request
#[derive(Debug, Deserialize)]
struct PredictionRequest {
    /// Metric to predict
    metric: String,
}
