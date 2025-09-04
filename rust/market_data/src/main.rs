use axum::{
    routing::{get, post},
    Json, extract::Path,
};
use redis::{Client, Commands, RedisResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};
use rand::Rng;

mod alpaca_adapter;
mod alpaca_client;
mod binance_provider;
mod dexscreener_provider;
mod provider_registry;

use alpaca_adapter::{AlpacaAdapter, AlpacaConfig};
use alpaca_client::{AlpacaClient, AlpacaConfig as ClientConfig};
use binance_provider::BinanceProvider;
use dexscreener_provider::DexScreenerProvider;
use provider_registry::{ProviderRegistry, MarketDataProvider, AssetType};

// ============================================================================
// MARKET TICK STRUCTURE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub schema_version: String,
    pub symbol: String,
    pub ts: i64,
    pub price: String,
    pub source: String,
    pub volume: Option<String>,
    pub bid: Option<String>,
    pub ask: Option<String>,
    pub high: Option<String>,
    pub low: Option<String>,
}

impl MarketTick {
    pub fn new(symbol: String, price: f64, source: &str) -> Self {
        let ts = Utc::now().timestamp_millis();
        Self {
            schema_version: "1".to_string(),
            symbol,
            ts,
            price: format!("{:.2}", price),
            source: source.to_string(),
            volume: None,
            bid: None,
            ask: None,
            high: None,
            low: None,
        }
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = Some(format!("{:.0}", volume));
        self
    }

    pub fn with_bid_ask(mut self, bid: f64, ask: f64) -> Self {
        self.bid = Some(format!("{:.2}", bid));
        self.ask = Some(format!("{:.2}", ask));
        self
    }
}

// ============================================================================
// MARKET DATA SERVICE
// ============================================================================

pub struct MarketDataService {
    redis_client: Arc<Client>,
    symbols: Vec<String>,
    base_prices: HashMap<String, f64>,
    price_history: Arc<RwLock<HashMap<String, Vec<f64>>>>,
    provider_registry: ProviderRegistry,
    alpaca_client: Option<AlpacaClient>,
    use_providers: bool,
}

impl MarketDataService {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let redis_client = Arc::new(Client::open(redis_url)?);
        
        let mut provider_registry = ProviderRegistry::new();
        let mut use_providers = false;
        let mut symbols = Vec::new();
        let mut alpaca_client = None;
        
        // Try to initialize Alpaca adapter for US equities
        if let Ok(config) = AlpacaConfig::from_env() {
            info!("Alpaca API configured - using real US equity data");
            let alpaca_adapter = Box::new(AlpacaAdapter::new(
                config.api_key.clone(),
                config.secret_key.clone(),
                config.is_paper,
            ));
            provider_registry.register_provider(alpaca_adapter);
            symbols.extend(config.symbols.clone());
            use_providers = true;
            
            // Initialize Alpaca client for real-time streaming
            let client_config = ClientConfig {
                api_key: config.api_key,
                secret_key: config.secret_key,
                rest_url: "https://data.alpaca.markets/v2".to_string(),
                ws_url: "wss://stream.data.alpaca.markets/v2/iex".to_string(),
                test_ws_url: "wss://stream.data.alpaca.markets/v2/test".to_string(),
                watchlist: config.symbols,
                max_symbols: 30,
                rate_limit_per_minute: 200,
                rate_limit_per_second: 3,
                data_feed: "iex".to_string(),
                paper_trading: config.is_paper,
            };
            
            alpaca_client = Some(AlpacaClient::new(client_config));
        }
        
        // Always add Binance provider for crypto majors
        info!("Adding Binance provider for crypto majors");
        let binance_provider = Box::new(BinanceProvider::new());
        provider_registry.register_provider(binance_provider);
        symbols.extend(vec![
            "BTCUSD.BIN".to_string(),
            "ETHUSD.BIN".to_string(),
            "ADAUSD.BIN".to_string(),
            "DOTUSD.BIN".to_string(),
        ]);
        use_providers = true;
        
        // Always add DEXScreener provider for meme coins
        info!("Adding DEXScreener provider for meme coins and DEX tokens");
        let dexscreener_provider = Box::new(DexScreenerProvider::new());
        provider_registry.register_provider(dexscreener_provider);
        symbols.extend(vec![
            "SOL:So11111111111111111111111111111111111111112".to_string(), // Wrapped SOL
            "ETH:0xA0b86a33E6441b8c4C8D3F3C1B8C4C8D3F3C1B8C".to_string(), // Example ETH token
        ]);
        use_providers = true;
        
        // Fallback to simulation if no providers configured
        if !use_providers {
            warn!("No providers configured, using simulated data");
            symbols = vec![
                "AAPL".to_string(),
                "SPY".to_string(), 
                "BTCUSD".to_string(),
                "GOOGL".to_string(),
                "TSLA".to_string(),
                "MSFT".to_string(),
            ];
        }
        
        let mut base_prices = HashMap::new();
        if !use_providers {
            // Only set base prices for simulation
            base_prices.insert("AAPL".to_string(), 192.34);
            base_prices.insert("SPY".to_string(), 485.67);
            base_prices.insert("BTCUSD".to_string(), 43250.0);
            base_prices.insert("GOOGL".to_string(), 142.89);
            base_prices.insert("TSLA".to_string(), 245.12);
            base_prices.insert("MSFT".to_string(), 378.45);
        }
        
        Ok(Self {
            redis_client,
            symbols,
            base_prices,
            price_history: Arc::new(RwLock::new(HashMap::new())),
            provider_registry,
            alpaca_client,
            use_providers,
        })
    }

    pub async fn start_simulation(&self) {
        if self.use_providers {
            info!("Starting multi-provider market data feed for symbols: {:?}", self.symbols);
            
            // Start Alpaca WebSocket stream if available
            if let Some(alpaca_client) = &self.alpaca_client {
                let redis_client = self.redis_client.clone();
                let alpaca_client = alpaca_client.clone();
                
                tokio::spawn(async move {
                    if let Err(e) = alpaca_client.start_websocket_stream(|symbol, price, timestamp| {
                        let redis_client = redis_client.clone();
                        tokio::spawn(async move {
                            let tick = MarketTick::new(symbol, price, "alpaca:iex");
                            if let Err(e) = Self::publish_tick_to_redis(&redis_client, &tick).await {
                                error!("Failed to publish Alpaca tick: {}", e);
                            }
                        });
                    }).await {
                        error!("Alpaca WebSocket stream failed: {}", e);
                    }
                });
            }
            
            self.start_provider_feed().await;
        } else {
            info!("Starting market data simulation for symbols: {:?}", self.symbols);
            
            for symbol in self.symbols.clone() {
                let service = self.clone();
                tokio::spawn(async move {
                    service.simulate_symbol(symbol).await;
                });
            }
        }
    }

    async fn start_provider_feed(&self) {
        for symbol in self.symbols.clone() {
            let service = self.clone();
            let symbol_clone = symbol.clone();
            tokio::spawn(async move {
                service.feed_provider_data(symbol_clone).await;
            });
        }
    }

    async fn feed_provider_data(&self, symbol: String) {
        loop {
            match self.provider_registry.get_quote(&symbol).await {
                Ok(Some(tick)) => {
                    // Publish to Redis Stream
                    if let Err(e) = self.publish_tick(&tick).await {
                        error!("Failed to publish provider tick for {}: {}", symbol, e);
                    }
                    
                    // Update price history
                    {
                        let mut history = self.price_history.write().await;
                        let symbol_history = history.entry(symbol.clone()).or_insert_with(Vec::new);
                        if let Ok(price) = tick.price.parse::<f64>() {
                            symbol_history.push(price);
                            
                            // Keep only last 1000 prices
                            if symbol_history.len() > 1000 {
                                symbol_history.remove(0);
                            }
                        }
                    }
                }
                Ok(None) => {
                    warn!("No quote data received for {}", symbol);
                }
                Err(e) => {
                    error!("Provider API error for {}: {}", symbol, e);
                }
            }
            
            // Wait before next request (respect rate limits)
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        }
    }

    async fn simulate_symbol(&self, symbol: String) {
        let mut rng = rand::thread_rng();
        let base_price = self.base_prices.get(&symbol).unwrap_or(&100.0);
        let mut current_price = *base_price;
        
        loop {
            // Generate realistic price movement
            let change_percent = rng.gen_range(-0.5..0.5); // ±0.5% change
            let change = current_price * (change_percent / 100.0);
            current_price += change;
            
            // Ensure price stays positive
            current_price = current_price.max(0.01);
            
            // Generate volume
            let volume = rng.gen_range(1000.0..100000.0);
            
            // Generate bid/ask spread
            let spread = current_price * 0.001; // 0.1% spread
            let bid = current_price - spread / 2.0;
            let ask = current_price + spread / 2.0;
            
            // Create tick
            let tick = MarketTick::new(symbol.clone(), current_price, "sim")
                .with_volume(volume)
                .with_bid_ask(bid, ask);
            
            // Publish to Redis Stream
            if let Err(e) = self.publish_tick(&tick).await {
                error!("Failed to publish tick for {}: {}", symbol, e);
            }
            
            // Update price history
            {
                let mut history = self.price_history.write().await;
                let symbol_history = history.entry(symbol.clone()).or_insert_with(Vec::new);
                symbol_history.push(current_price);
                
                // Keep only last 1000 prices
                if symbol_history.len() > 1000 {
                    symbol_history.remove(0);
                }
            }
            
            // Wait 1-3 seconds before next tick
            let delay = rng.gen_range(1000..3000);
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }

    async fn publish_tick(&self, tick: &MarketTick) -> RedisResult<String> {
        let mut conn = self.redis_client.get_connection()?;
        
        let stream_key = format!("ticks.{}", tick.symbol);
        let tick_json = serde_json::to_string(tick)
            .map_err(|e| redis::RedisError::from((
                redis::ErrorKind::InvalidArgument,
                "Failed to serialize tick",
                e.to_string(),
            )))?;
        
        let message_id = conn.xadd(&stream_key, "*", &[("tick", &tick_json)])?;
        
        info!("Published tick to {}: {} at {}", stream_key, message_id, tick.price);
        Ok(message_id)
    }

    pub async fn get_latest_price(&self, symbol: &str) -> Option<f64> {
        let history = self.price_history.read().await;
        history.get(symbol).and_then(|prices| prices.last().copied())
    }

    pub async fn get_price_history(&self, symbol: &str, count: usize) -> Vec<f64> {
        let history = self.price_history.read().await;
        history.get(symbol)
            .map(|prices| prices.iter().rev().take(count).copied().collect())
            .unwrap_or_default()
    }
}

impl Clone for MarketDataService {
    fn clone(&self) -> Self {
        Self {
            redis_client: self.redis_client.clone(),
            symbols: self.symbols.clone(),
            base_prices: self.base_prices.clone(),
            price_history: self.price_history.clone(),
        }
    }
}

// ============================================================================
// API HANDLERS
// ============================================================================

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "market-data",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_quote(
    Path(symbol): Path<String>,
    axum::extract::State(service): axum::extract::State<Arc<MarketDataService>>,
) -> Json<serde_json::Value> {
    if let Some(price) = service.get_latest_price(&symbol).await {
        Json(serde_json::json!({
            "symbol": symbol,
            "price": price,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    } else {
        Json(serde_json::json!({
            "error": "Symbol not found",
            "symbol": symbol
        }))
    }
}

async fn get_quotes(
    Json(symbols): Json<Vec<String>>,
    axum::extract::State(service): axum::extract::State<Arc<MarketDataService>>,
) -> Json<serde_json::Value> {
    let mut quotes = Vec::new();
    
    for symbol in symbols {
        if let Some(price) = service.get_latest_price(&symbol).await {
            quotes.push(serde_json::json!({
                "symbol": symbol,
                "price": price,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }
    }
    
    Json(serde_json::json!({ "quotes": quotes }))
}

async fn get_history(
    Path((symbol, period)): Path<(String, String)>,
    axum::extract::State(service): axum::extract::State<Arc<MarketDataService>>,
) -> Json<serde_json::Value> {
    let count = match period.as_str() {
        "1h" => 60,
        "1d" => 1440,
        "1w" => 10080,
        _ => 100,
    };
    
    let prices = service.get_price_history(&symbol, count).await;
    
    Json(serde_json::json!({
        "symbol": symbol,
        "period": period,
        "prices": prices,
        "count": prices.len()
    }))
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    dotenv::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://localhost:6379".to_string());
    
    let port = std::env::var("MARKET_DATA_PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse::<u16>()?;
    
    info!("Starting Market Data Service on port {}", port);
    
    // Initialize market data service
    let market_data = Arc::new(MarketDataService::new(&redis_url)?);
    
    // Start simulation
    market_data.start_simulation().await;
    
    // Build router
    let app = axum::Router::new()
        .route("/health", get(health_check))
        .route("/api/quotes/:symbol", get(get_quote))
        .route("/api/quotes/batch", post(get_quotes))
        .route("/api/history/:symbol/:period", get(get_history))
        .with_state(market_data);
    
    // Start server
    let addr = format!("0.0.0.0:{}", port).parse()?;
    info!("Market Data Service listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
