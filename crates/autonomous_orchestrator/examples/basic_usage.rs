//! Basic usage example for Autonomous Orchestrator
//!
//! This example demonstrates how to use the autonomous orchestrator APIs.
//!
//! Run with: cargo run --example basic_usage

use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Autonomous Orchestrator - Basic Usage Example\n");

    let base_url = "http://localhost:8090/api/v1";
    let client = reqwest::Client::new();

    // 1. Health Check
    println!("1. Checking system health...");
    let health_response = client
        .get(format!("{}/health", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("   Status: {}", health_response["status"]);
    println!("   Health Score: {}\n", health_response["score"]);

    // 2. Get System Status
    println!("2. Getting system status...");
    let status_response = client
        .get(format!("{}/status", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("   Decision Engine Samples: {}",
        status_response["decision_engine"]["training_samples"]);
    println!("   Anomalies Detected: {}\n",
        status_response["anomaly_detection"]["total_detected"]);

    // 3. Get Analytics Summary
    println!("3. Getting analytics summary...");
    let analytics_response = client
        .get(format!("{}/analytics/summary", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("   Total Payments: {}", analytics_response["total_payments"]);
    println!("   Success Rate: {}%\n",
        analytics_response["success_rate"].as_f64().unwrap_or(0.0) * 100.0);

    // 4. Get Detected Anomalies
    println!("4. Getting detected anomalies...");
    let anomalies_response = client
        .get(format!("{}/anomalies?limit=5", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(anomalies) = anomalies_response.as_array() {
        println!("   Found {} recent anomalies", anomalies.len());
        for (i, anomaly) in anomalies.iter().take(3).enumerate() {
            println!("   {}. Type: {}, Score: {}",
                i + 1,
                anomaly["anomaly_type"],
                anomaly["score"]
            );
        }
    }
    println!();

    // 5. Get Healing Actions
    println!("5. Getting healing actions...");
    let healing_response = client
        .get(format!("{}/healing/actions?limit=5", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(actions) = healing_response.as_array() {
        println!("   Found {} healing actions", actions.len());
        for (i, action) in actions.iter().take(3).enumerate() {
            println!("   {}. Type: {}, Status: {}",
                i + 1,
                action["action_type"],
                action["status"]
            );
        }
    }
    println!();

    // 6. Get Routing Statistics
    println!("6. Getting routing statistics...");
    let routing_response = client
        .get(format!("{}/routing/stats", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    println!("   Model Version: {}", routing_response["model_version"]);
    println!("   Tracked Connectors: {}\n", routing_response["tracked_connectors"]);

    // 7. Evaluate Scaling
    println!("7. Evaluating resource scaling...");
    let scaling_response = client
        .post(format!("{}/resources/evaluate-scaling", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(direction) = scaling_response.get("direction") {
        println!("   Scaling Direction: {}", direction);
        println!("   Current Instances: {}",
            scaling_response["current_instances"]);
    } else {
        println!("   {}", scaling_response["message"]);
    }
    println!();

    // 8. Generate Prediction (may fail if insufficient data)
    println!("8. Generating prediction...");
    let prediction_result = client
        .post(format!("{}/analytics/predict", base_url))
        .json(&json!({ "metric": "payment_volume" }))
        .send()
        .await;

    match prediction_result {
        Ok(response) => {
            if response.status().is_success() {
                let pred = response.json::<serde_json::Value>().await?;
                println!("   Metric: {}", pred["metric"]);
                if let Some(predictions) = pred["predictions"].as_array() {
                    println!("   Forecast Points: {}", predictions.len());
                }
            } else {
                println!("   Prediction not available (insufficient data)");
            }
        }
        Err(e) => {
            println!("   Error: {}", e);
        }
    }
    println!();

    println!("✅ Example completed successfully!");

    Ok(())
}
