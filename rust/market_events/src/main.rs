use anyhow::Result;
use tracing::{info, error};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

mod config;
mod ingestors;
mod entity_linker;
mod severity_scorer;
mod alert_poster;
mod event_processor;
mod types;

use config::MarketEventsConfig;
use event_processor::EventProcessor;
use types::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Market Events Alert System");

    // Load configuration
    let config = MarketEventsConfig::load()?;
    info!("Configuration loaded: {:?}", config);

    // Initialize event processor
    let processor = EventProcessor::new(config.clone()).await?;
    let processor = Arc::new(processor);

    // Start ingestion tasks
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = processor_clone.start_sec_edgar_monitoring().await {
            error!("SEC EDGAR monitoring failed: {}", e);
        }
    });

    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = processor_clone.start_news_feed_monitoring().await {
            error!("News feed monitoring failed: {}", e);
        }
    });

    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = processor_clone.start_halt_monitoring().await {
            error!("Halt monitoring failed: {}", e);
        }
    });

    // Start event processing loop
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = processor_clone.process_events_loop().await {
            error!("Event processing loop failed: {}", e);
        }
    });

    // Start alert posting loop
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = processor_clone.post_alerts_loop().await {
            error!("Alert posting loop failed: {}", e);
        }
    });

    // Health check endpoint
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        if let Err(e) = start_health_server(processor_clone).await {
            error!("Health server failed: {}", e);
        }
    });

    info!("Market Events Alert System started successfully");

    // Keep the main thread alive
    loop {
        sleep(Duration::from_secs(60)).await;
        info!("Market Events Alert System running...");
    }
}

async fn start_health_server(processor: Arc<EventProcessor>) -> Result<()> {
    use axum::{
        routing::get,
        Router,
        Json,
    };
    use serde_json::json;

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        .with_state(processor);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8005").await?;
    info!("Health server listening on port 8005");
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "market_events",
        "timestamp": chrono::Utc::now()
    }))
}

async fn metrics(axum::extract::State(processor): axum::extract::State<Arc<EventProcessor>>) -> Json<serde_json::Value> {
    let metrics = processor.get_metrics().await;
    Json(serde_json::to_value(metrics).unwrap_or(json!({})))
}
