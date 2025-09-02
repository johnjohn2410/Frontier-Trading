use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{info, error, warn};
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::MarketTick;
use super::provider_registry::{MarketDataProvider, AssetType, RateLimit};

// ============================================================================
// ALPACA API TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct AlpacaQuote {
    pub symbol: String,
    pub askprice: f64,
    pub asksize: f64,
    pub bidprice: f64,
    pub bidsize: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AlpacaTrade {
    pub symbol: String,
    pub price: f64,
    pub size: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AlpacaBar {
    pub symbol: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// ALPACA ADAPTER
// ============================================================================

pub struct AlpacaAdapter {
    client: Client,
    api_key: String,
    secret_key: String,
    base_url: String,
    is_paper: bool,
    last_quotes: HashMap<String, AlpacaQuote>,
    last_trades: HashMap<String, AlpacaTrade>,
    rate_limit_reset: Instant,
    request_count: u32,
}

impl AlpacaAdapter {
    pub fn new(api_key: String, secret_key: String, is_paper: bool) -> Self {
        let base_url = if is_paper {
            "https://paper-api.alpaca.markets".to_string()
        } else {
            "https://api.alpaca.markets".to_string()
        };

        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key,
            secret_key,
            base_url,
            is_paper,
            last_quotes: HashMap::new(),
            last_trades: HashMap::new(),
            rate_limit_reset: Instant::now(),
            request_count: 0,
        }
    }

    pub async fn get_quote_internal(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        // Rate limiting: Alpaca allows 200 requests per minute
        self.check_rate_limit().await;

        let url = format!("{}/v2/stocks/{}/quote", self.base_url, symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Alpaca API error for {}: {}", symbol, response.status());
            return Ok(None);
        }

        let quote: AlpacaQuote = response.json().await?;
        self.last_quotes.insert(symbol.to_string(), quote.clone());

        // Convert to our MarketTick format
        let tick = MarketTick {
            schema_version: "1".to_string(),
            symbol: quote.symbol,
            ts: quote.timestamp.timestamp_millis(),
            price: format!("{:.2}", quote.askprice), // Use ask price as current price
            source: "alpaca".to_string(),
            volume: None,
            bid: Some(format!("{:.2}", quote.bidprice)),
            ask: Some(format!("{:.2}", quote.askprice)),
            high: None,
            low: None,
        };

        self.request_count += 1;
        Ok(Some(tick))
    }

    pub async fn get_trade_internal(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        // Rate limiting
        self.check_rate_limit().await;

        let url = format!("{}/v2/stocks/{}/trades/latest", self.base_url, symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Alpaca API error for {}: {}", symbol, response.status());
            return Ok(None);
        }

        let trade: AlpacaTrade = response.json().await?;
        self.last_trades.insert(symbol.to_string(), trade.clone());

        let tick = MarketTick {
            schema_version: "1".to_string(),
            symbol: trade.symbol,
            ts: trade.timestamp.timestamp_millis(),
            price: format!("{:.2}", trade.price),
            source: "alpaca".to_string(),
            volume: Some(format!("{:.0}", trade.size)),
            bid: None,
            ask: None,
            high: None,
            low: None,
        };

        self.request_count += 1;
        Ok(Some(tick))
    }

    pub async fn get_bars(&mut self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<MarketTick>> {
        // Rate limiting
        self.check_rate_limit().await;

        let url = format!(
            "{}/v2/stocks/{}/bars?timeframe={}&limit={}",
            self.base_url, symbol, timeframe, limit
        );
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Alpaca API error for {}: {}", symbol, response.status());
            return Ok(vec![]);
        }

        #[derive(Deserialize)]
        struct BarsResponse {
            bars: Vec<AlpacaBar>,
        }

        let bars_response: BarsResponse = response.json().await?;
        
        let ticks: Vec<MarketTick> = bars_response.bars.into_iter().map(|bar| MarketTick {
            schema_version: "1".to_string(),
            symbol: bar.symbol,
            ts: bar.timestamp.timestamp_millis(),
            price: format!("{:.2}", bar.close),
            source: "alpaca".to_string(),
            volume: Some(format!("{:.0}", bar.volume)),
            bid: None,
            ask: None,
            high: Some(format!("{:.2}", bar.high)),
            low: Some(format!("{:.2}", bar.low)),
        }).collect();

        self.request_count += 1;
        Ok(ticks)
    }

    pub async fn get_account(&self) -> Result<Option<serde_json::Value>> {
        let url = format!("{}/v2/account", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Alpaca account API error: {}", response.status());
            return Ok(None);
        }

        let account: serde_json::Value = response.json().await?;
        Ok(Some(account))
    }

    pub async fn get_positions(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("{}/v2/positions", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("Alpaca positions API error: {}", response.status());
            return Ok(vec![]);
        }

        let positions: Vec<serde_json::Value> = response.json().await?;
        Ok(positions)
    }

    async fn check_rate_limit(&mut self) {
        // Alpaca allows 200 requests per minute
        const MAX_REQUESTS_PER_MINUTE: u32 = 200;
        
        if self.request_count >= MAX_REQUESTS_PER_MINUTE {
            let elapsed = self.rate_limit_reset.elapsed();
            if elapsed < Duration::from_secs(60) {
                let sleep_duration = Duration::from_secs(60) - elapsed;
                warn!("Rate limit reached, sleeping for {:?}", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
            self.request_count = 0;
            self.rate_limit_reset = Instant::now();
        }
    }

    pub fn is_paper_trading(&self) -> bool {
        self.is_paper
    }

    pub fn get_last_quote(&self, symbol: &str) -> Option<&AlpacaQuote> {
        self.last_quotes.get(symbol)
    }

    pub fn get_last_trade(&self, symbol: &str) -> Option<&AlpacaTrade> {
        self.last_trades.get(symbol)
    }
}

#[async_trait]
impl MarketDataProvider for AlpacaAdapter {
    async fn get_quote(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        // Use existing get_quote method
        self.get_quote_internal(symbol).await
    }

    async fn get_trades(&mut self, symbol: &str, limit: u32) -> Result<Vec<MarketTick>> {
        // Use existing get_trade method and convert
        if let Ok(Some(tick)) = self.get_trade_internal(symbol).await {
            Ok(vec![tick])
        } else {
            Ok(vec![])
        }
    }

    async fn get_bars(&mut self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<MarketTick>> {
        // Use existing get_bars method
        self.get_bars_internal(symbol, timeframe, limit).await
    }

    fn get_provider_name(&self) -> &'static str {
        "alpaca"
    }

    fn get_asset_type(&self) -> AssetType {
        AssetType::USEquity
    }

    fn get_rate_limit(&self) -> RateLimit {
        RateLimit::alpaca()
    }

    fn is_realtime(&self) -> bool {
        true
    }
}

// ============================================================================
// CONFIGURATION
// ============================================================================

#[derive(Debug, Clone)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub is_paper: bool,
    pub symbols: Vec<String>,
    pub update_interval_ms: u64,
}

impl AlpacaConfig {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ALPACA_API_KEY")
            .map_err(|_| anyhow::anyhow!("ALPACA_API_KEY not set"))?;
        
        let secret_key = std::env::var("ALPACA_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("ALPACA_SECRET_KEY not set"))?;
        
        let is_paper = std::env::var("ALPACA_PAPER_TRADING")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let symbols = std::env::var("ALPACA_SYMBOLS")
            .unwrap_or_else(|_| "AAPL,GOOGL,MSFT,TSLA,SPY".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        
        let update_interval_ms = std::env::var("ALPACA_UPDATE_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .unwrap_or(1000);

        Ok(Self {
            api_key,
            secret_key,
            is_paper,
            symbols,
            update_interval_ms,
        })
    }
}
