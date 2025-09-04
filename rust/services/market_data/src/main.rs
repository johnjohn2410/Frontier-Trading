use frontier_types::MarketTick;
use frontier_bus::{connect, xgroup_create_mkstream, publish_tick};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, error};
use chrono::Utc;

#[derive(Deserialize)]
struct AlpacaBar {
    c: f64,  // close
    h: f64,  // high
    l: f64,  // low
    o: f64,  // open
    t: String,  // timestamp (ISO string)
    v: i64,  // volume
    n: Option<i64>,  // number of trades
    vw: Option<f64>, // volume weighted average price
}

#[derive(Deserialize)]
struct AlpacaResponse {
    bars: HashMap<String, AlpacaBar>,
}

async fn fetch_alpaca_price(symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
    let alpaca_key = std::env::var("APCA_API_KEY_ID")?;
    let alpaca_secret = std::env::var("APCA_API_SECRET_KEY")?;
    
    let client = reqwest::Client::new();
    let url = format!("https://data.alpaca.markets/v2/stocks/bars/latest?symbols={}", symbol);
    
    let response = client
        .get(&url)
        .header("APCA-API-KEY-ID", &alpaca_key)
        .header("APCA-API-SECRET-KEY", &alpaca_secret)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("Alpaca API error: {}", response.status()).into());
    }
    
    let alpaca_data: AlpacaResponse = response.json().await?;
    
    if let Some(bar) = alpaca_data.bars.get(symbol) {
        Ok(bar.c)
    } else {
        Err(format!("No data found for symbol: {}", symbol).into())
    }
}

async fn market_data_loop() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Connect to Redis
    let mut conn = connect("redis://127.0.0.1/").await?;
    
    // Create consumer group for market data
    xgroup_create_mkstream(&mut conn, "ticks.AAPL", "md.cg.copilot").await?;
    xgroup_create_mkstream(&mut conn, "ticks.MSFT", "md.cg.copilot").await?;
    xgroup_create_mkstream(&mut conn, "ticks.TSLA", "md.cg.copilot").await?;
    xgroup_create_mkstream(&mut conn, "ticks.SPY", "md.cg.copilot").await?;
    
    let symbols = vec!["AAPL", "MSFT", "TSLA", "SPY"];
    let mut counter = 0;
    
    info!("🚀 Starting market data feed with Alpaca integration");
    
    loop {
        for symbol in &symbols {
            match fetch_alpaca_price(symbol).await {
                Ok(price) => {
                    let tick = MarketTick {
                        schema_version: "1".to_string(),
                        symbol: symbol.to_string(),
                        ts: Utc::now().timestamp_millis(),
                        price: format!("{:.2}", price),
                        source: "alpaca:iex".to_string(),
                    };
                    
                    let stream = format!("ticks.{}", symbol);
                    match publish_tick(&mut conn, &stream, &tick).await {
                        Ok(_) => {
                            info!("📈 {}: ${} (source: {})", 
                                tick.symbol, tick.price, tick.source);
                        }
                        Err(e) => {
                            error!("Failed to publish tick for {}: {}", symbol, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to fetch price for {}: {}", symbol, e);
                }
            }
        }
        
        counter += 1;
        info!("🔄 Market data cycle #{} completed", counter);
        sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("🚀 Starting Market Data Service with Alpaca Integration");
    
    // Check if Alpaca keys are configured
    let alpaca_key = std::env::var("APCA_API_KEY_ID").unwrap_or_default();
    let alpaca_secret = std::env::var("APCA_API_SECRET_KEY").unwrap_or_default();
    
    if alpaca_key.is_empty() || alpaca_secret.is_empty() {
        error!("❌ Alpaca API keys not configured. Set APCA_API_KEY_ID and APCA_API_SECRET_KEY");
        return Err("Missing Alpaca API keys".into());
    }
    
    info!("✅ Alpaca API keys configured");
    
    // Start the market data loop
    market_data_loop().await?;
    
    Ok(())
}
