//! Autonomous Payment Orchestration System (APOS)
//!
//! A self-managing, AI-powered system that autonomously optimizes payment processing,
//! detects anomalies, self-heals failed transactions, and scales resources automatically.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod analytics;
mod anomaly_detector;
mod config;
mod decision_engine;
mod event_monitor;
mod health;
mod models;
mod resource_manager;
mod routes;
mod self_healing;
mod state;
mod types;
mod utils;

#[cfg(test)]
mod tests;

use actix_web::{web, App, HttpServer};
use common_utils::signals::get_allowed_signals;
use error_stack::{Report, ResultExt};
use router_env::logger;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    config::Settings,
    event_monitor::EventMonitor,
    state::AppState,
};

/// Main application errors
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    /// Server startup error
    #[error("Failed to start server: {0}")]
    ServerStartup(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Redis error
    #[error("Redis error: {0}")]
    Redis(String),
}

#[tokio::main]
async fn main() -> Result<(), Report<ApplicationError>> {
    // Initialize logger
    router_env::setup(&Default::default(), "autonomous_orchestrator");

    logger::info!("Starting Autonomous Payment Orchestration System (APOS)");

    // Load configuration
    let config = Settings::new()
        .change_context(ApplicationError::Configuration("Failed to load configuration".to_string()))?;

    logger::info!("Configuration loaded successfully");

    // Initialize application state
    let app_state = Arc::new(RwLock::new(
        AppState::new(config.clone())
            .await
            .change_context(ApplicationError::ServerStartup("Failed to initialize app state".to_string()))?
    ));

    logger::info!("Application state initialized");

    // Start event monitor in background
    let event_monitor = EventMonitor::new(app_state.clone());
    tokio::spawn(async move {
        if let Err(e) = event_monitor.start().await {
            logger::error!("Event monitor failed: {:?}", e);
        }
    });

    logger::info!("Event monitor started");

    // Start HTTP server
    let server_address = format!("{}:{}", config.server.host, config.server.port);
    logger::info!("Starting HTTP server on {}", server_address);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .configure(routes::configure_routes)
    })
    .bind(&server_address)
    .change_context(ApplicationError::ServerStartup(format!(
        "Failed to bind to {}",
        server_address
    )))?
    .run();

    logger::info!("APOS is running on {}", server_address);
    logger::info!("System is now autonomous and operational");

    // Handle graceful shutdown
    let server_handle = server.handle();
    tokio::spawn(async move {
        let signals = get_allowed_signals()
            .change_context(ApplicationError::ServerStartup("Failed to setup signal handlers".to_string()))
            .expect("Failed to get allowed signals");

        signals.await;
        logger::info!("Received shutdown signal, gracefully shutting down...");
        server_handle.stop(true).await;
    });

    server.await
        .change_context(ApplicationError::ServerStartup("Server error".to_string()))?;

    logger::info!("APOS shutdown complete");

    Ok(())
}
