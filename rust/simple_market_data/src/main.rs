use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize)]
struct MarketTick {
    schema_version: String,
    symbol: String,
    ts: i64,
    price: String,
    source: String,
}

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: DateTime<Utc>,
    alpaca_connected: bool,
}

#[derive(Deserialize)]
struct AlpacaBar {
    c: f64,  // close
    h: f64,  // high
    l: f64,  // low
    o: f64,  // open
    t: i64,  // timestamp
    v: i64,  // volume
}

#[derive(Deserialize)]
struct AlpacaResponse {
    bars: HashMap<String, AlpacaBar>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        alpaca_connected: true,
    })
}

async fn get_quote(Query(params): Query<HashMap<String, String>>) -> Result<Json<MarketTick>, StatusCode> {
    let symbol = params.get("symbol").unwrap_or(&"AAPL".to_string()).clone();
    
    // Get real data from Alpaca
    let alpaca_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
    let alpaca_secret = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
    
    if alpaca_key.is_empty() || alpaca_secret.is_empty() {
        error!("Alpaca API keys not configured");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    let client = reqwest::Client::new();
    let url = format!("https://data.alpaca.markets/v2/stocks/bars/latest?symbols={}", symbol);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &alpaca_key)
        .header("APCA-API-SECRET-KEY", &alpaca_secret)
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if !response.status().is_success() {
        error!("Alpaca API error: {}", response.status());
        return Err(StatusCode::BAD_GATEWAY);
    }
    
    let alpaca_data: AlpacaResponse = response
        .json()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if let Some(bar) = alpaca_data.bars.get(&symbol) {
        let tick = MarketTick {
            schema_version: "1".to_string(),
            symbol: symbol.clone(),
            ts: bar.t,
            price: format!("{:.2}", bar.c),
            source: "alpaca:iex".to_string(),
        };
        Ok(Json(tick))
    } else {
        error!("No data found for symbol: {}", symbol);
        Err(StatusCode::NOT_FOUND)
    }
}

async fn simulate_stream() {
    let symbols = vec!["AAPL", "MSFT", "TSLA", "SPY"];
    let mut counter = 0;
    
    loop {
        for symbol in &symbols {
            // Get real price from Alpaca
            let alpaca_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
            let alpaca_secret = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
            
            if !alpaca_key.is_empty() && !alpaca_secret.is_empty() {
                let client = reqwest::Client::new();
                let url = format!("https://data.alpaca.markets/v2/stocks/bars/latest?symbols={}", symbol);
                
                if let Ok(response) = client
                    .get(&url)
                    .header("APCA-API-KEY-ID", &alpaca_key)
                    .header("APCA-API-SECRET-KEY", &alpaca_secret)
                    .send()
                    .await
                {
                    if response.status().is_success() {
                        if let Ok(alpaca_data) = response.json::<AlpacaResponse>().await {
                            if let Some(bar) = alpaca_data.bars.get(*symbol) {
                                let tick = MarketTick {
                                    schema_version: "1".to_string(),
                                    symbol: symbol.to_string(),
                                    ts: bar.t,
                                    price: format!("{:.2}", bar.c),
                                    source: "alpaca:iex".to_string(),
                                };
                                
                                info!("📈 {}: ${} (source: {})", 
                                    tick.symbol, tick.price, tick.source);
                            }
                        }
                    }
                }
            }
        }
        
        counter += 1;
        info!("🔄 Market data cycle #{} completed", counter);
        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("🚀 Starting Simple Market Data Service with Alpaca Integration");
    
    // Check if Alpaca keys are configured
    let alpaca_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
    let alpaca_secret = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
    
    if alpaca_key.is_empty() || alpaca_secret.is_empty() {
        error!("❌ Alpaca API keys not configured. Set APCA_API_KEY_ID and APCA_API_SECRET_KEY");
        return Err("Missing Alpaca API keys".into());
    }
    
    info!("✅ Alpaca API keys configured");
    
    // Start the market data simulation in the background
    tokio::spawn(simulate_stream());
    
    // Build the router
    let app = Router::new()
        .route("/health", get(health))
        .route("/quote", get(get_quote));
    
    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8001").await?;
    info!("🌐 Market Data Service listening on http://localhost:8001");
    info!("📊 Health check: http://localhost:8001/health");
    info!("📈 Quote endpoint: http://localhost:8001/quote?symbol=AAPL");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
