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
// BINANCE API TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct BinanceTicker {
    pub symbol: String,
    pub priceChange: String,
    pub priceChangePercent: String,
    pub weightedAvgPrice: String,
    pub prevClosePrice: String,
    pub lastPrice: String,
    pub lastQty: String,
    pub bidPrice: String,
    pub bidQty: String,
    pub askPrice: String,
    pub askQty: String,
    pub openPrice: String,
    pub highPrice: String,
    pub lowPrice: String,
    pub volume: String,
    pub quoteVolume: String,
    pub openTime: i64,
    pub closeTime: i64,
    pub count: i64,
}

#[derive(Debug, Deserialize)]
pub struct BinanceTrade {
    pub id: i64,
    pub price: String,
    pub qty: String,
    pub quoteQty: String,
    pub time: i64,
    pub isBuyerMaker: bool,
    pub isBestMatch: bool,
}

#[derive(Debug, Deserialize)]
pub struct BinanceKline {
    pub openTime: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub closeTime: i64,
    pub quoteAssetVolume: String,
    pub numberOfTrades: i64,
    pub takerBuyBaseAssetVolume: String,
    pub takerBuyQuoteAssetVolume: String,
}

// ============================================================================
// BINANCE PROVIDER
// ============================================================================

pub struct BinanceProvider {
    client: Client,
    base_url: String,
    last_quotes: HashMap<String, BinanceTicker>,
    last_trades: HashMap<String, BinanceTrade>,
    rate_limit_reset: Instant,
    request_count: u32,
}

impl BinanceProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.binance.com".to_string(),
            last_quotes: HashMap::new(),
            last_trades: HashMap::new(),
            rate_limit_reset: Instant::now(),
            request_count: 0,
        }
    }

    async fn check_rate_limit(&mut self) {
        // Binance allows 1200 requests per minute
        const MAX_REQUESTS_PER_MINUTE: u32 = 1200;
        
        if self.request_count >= MAX_REQUESTS_PER_MINUTE {
            let elapsed = self.rate_limit_reset.elapsed();
            if elapsed < Duration::from_secs(60) {
                let sleep_duration = Duration::from_secs(60) - elapsed;
                warn!("Binance rate limit reached, sleeping for {:?}", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
            self.request_count = 0;
            self.rate_limit_reset = Instant::now();
        }
    }

    fn normalize_symbol(&self, symbol: &str) -> String {
        // Convert BTCUSD.BIN to BTCUSDT for Binance
        if symbol.ends_with(".BIN") {
            let base = symbol.trim_end_matches(".BIN");
            if base == "BTCUSD" {
                "BTCUSDT".to_string()
            } else if base == "ETHUSD" {
                "ETHUSDT".to_string()
            } else {
                format!("{}USDT", base)
            }
        } else {
            symbol.to_string()
        }
    }
}

#[async_trait]
impl MarketDataProvider for BinanceProvider {
    async fn get_quote(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        self.check_rate_limit().await;

        let binance_symbol = self.normalize_symbol(symbol);
        let url = format!("{}/api/v3/ticker/24hr?symbol={}", self.base_url, binance_symbol);
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("Binance API error for {}: {}", symbol, response.status());
            return Ok(None);
        }

        let ticker: BinanceTicker = response.json().await?;
        self.last_quotes.insert(symbol.to_string(), ticker.clone());

        // Convert to our MarketTick format
        let tick = MarketTick {
            schema_version: "1".to_string(),
            symbol: symbol.to_string(),
            ts: ticker.closeTime,
            price: ticker.lastPrice,
            source: "binance".to_string(),
            volume: Some(ticker.volume),
            bid: Some(ticker.bidPrice),
            ask: Some(ticker.askPrice),
            high: Some(ticker.highPrice),
            low: Some(ticker.lowPrice),
        };

        self.request_count += 1;
        Ok(Some(tick))
    }

    async fn get_trades(&mut self, symbol: &str, limit: u32) -> Result<Vec<MarketTick>> {
        self.check_rate_limit().await;

        let binance_symbol = self.normalize_symbol(symbol);
        let url = format!("{}/api/v3/trades?symbol={}&limit={}", self.base_url, binance_symbol, limit);
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("Binance trades API error for {}: {}", symbol, response.status());
            return Ok(vec![]);
        }

        let trades: Vec<BinanceTrade> = response.json().await?;
        
        let ticks: Vec<MarketTick> = trades.into_iter().map(|trade| MarketTick {
            schema_version: "1".to_string(),
            symbol: symbol.to_string(),
            ts: trade.time,
            price: trade.price,
            source: "binance".to_string(),
            volume: Some(trade.qty),
            bid: None,
            ask: None,
            high: None,
            low: None,
        }).collect();

        self.request_count += 1;
        Ok(ticks)
    }

    async fn get_bars(&mut self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<MarketTick>> {
        self.check_rate_limit().await;

        let binance_symbol = self.normalize_symbol(symbol);
        let interval = match timeframe {
            "1m" => "1m",
            "5m" => "5m",
            "15m" => "15m",
            "1h" => "1h",
            "4h" => "4h",
            "1d" => "1d",
            _ => "1h",
        };
        
        let url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            self.base_url, binance_symbol, interval, limit
        );
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("Binance klines API error for {}: {}", symbol, response.status());
            return Ok(vec![]);
        }

        let klines: Vec<Vec<serde_json::Value>> = response.json().await?;
        
        let ticks: Vec<MarketTick> = klines.into_iter().map(|kline| {
            let close_time = kline[6].as_i64().unwrap_or(0);
            let close_price = kline[4].as_str().unwrap_or("0").to_string();
            let volume = kline[5].as_str().unwrap_or("0").to_string();
            let high = kline[2].as_str().unwrap_or("0").to_string();
            let low = kline[3].as_str().unwrap_or("0").to_string();

            MarketTick {
                schema_version: "1".to_string(),
                symbol: symbol.to_string(),
                ts: close_time,
                price: close_price,
                source: "binance".to_string(),
                volume: Some(volume),
                bid: None,
                ask: None,
                high: Some(high),
                low: Some(low),
            }
        }).collect();

        self.request_count += 1;
        Ok(ticks)
    }

    fn get_provider_name(&self) -> &'static str {
        "binance"
    }

    fn get_asset_type(&self) -> AssetType {
        AssetType::CryptoCEX
    }

    fn get_rate_limit(&self) -> RateLimit {
        RateLimit::binance()
    }

    fn is_realtime(&self) -> bool {
        true
    }
}
