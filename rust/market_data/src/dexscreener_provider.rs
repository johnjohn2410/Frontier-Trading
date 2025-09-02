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
// DEXSCREENER API TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DexScreenerResponse {
    pub pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerPair {
    pub chainId: String,
    pub dexId: String,
    pub url: String,
    pub pairAddress: String,
    pub baseToken: DexScreenerToken,
    pub quoteToken: DexScreenerToken,
    pub priceNative: String,
    pub priceUsd: String,
    pub txns: DexScreenerTxns,
    pub volume: DexScreenerVolume,
    pub priceChange: DexScreenerPriceChange,
    pub liquidity: Option<DexScreenerLiquidity>,
    pub fdv: f64,
    pub pairCreatedAt: i64,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerToken {
    pub address: String,
    pub name: String,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerTxns {
    pub h24: DexScreenerTxnsData,
    pub h6: DexScreenerTxnsData,
    pub h1: DexScreenerTxnsData,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerTxnsData {
    pub buys: i64,
    pub sells: i64,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerVolume {
    pub h24: f64,
    pub h6: f64,
    pub h1: f64,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerPriceChange {
    pub h24: f64,
    pub h6: f64,
    pub h1: f64,
}

#[derive(Debug, Deserialize)]
pub struct DexScreenerLiquidity {
    pub usd: f64,
    pub base: f64,
    pub quote: f64,
}

// ============================================================================
// DEXSCREENER PROVIDER
// ============================================================================

pub struct DexScreenerProvider {
    client: Client,
    base_url: String,
    last_quotes: HashMap<String, DexScreenerPair>,
    rate_limit_reset: Instant,
    request_count: u32,
}

impl DexScreenerProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: "https://api.dexscreener.com".to_string(),
            last_quotes: HashMap::new(),
            rate_limit_reset: Instant::now(),
            request_count: 0,
        }
    }

    async fn check_rate_limit(&mut self) {
        // DEXScreener allows 300 requests per minute
        const MAX_REQUESTS_PER_MINUTE: u32 = 300;
        
        if self.request_count >= MAX_REQUESTS_PER_MINUTE {
            let elapsed = self.rate_limit_reset.elapsed();
            if elapsed < Duration::from_secs(60) {
                let sleep_duration = Duration::from_secs(60) - elapsed;
                warn!("DEXScreener rate limit reached, sleeping for {:?}", sleep_duration);
                tokio::time::sleep(sleep_duration).await;
            }
            self.request_count = 0;
            self.rate_limit_reset = Instant::now();
        }
    }

    fn resolve_symbol(&self, symbol: &str) -> (String, String) {
        // Handle chain:contract format
        if let Some((chain, contract)) = symbol.split_once(':') {
            (chain.to_string(), contract.to_string())
        } else {
            // Default to Ethereum
            ("ethereum".to_string(), symbol.to_string())
        }
    }

    async fn search_by_contract(&self, chain: &str, contract: &str) -> Result<Option<DexScreenerPair>> {
        let url = format!("{}/latest/dex/tokens/{}", self.base_url, contract);
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("DEXScreener API error for {}: {}", contract, response.status());
            return Ok(None);
        }

        let dex_response: DexScreenerResponse = response.json().await?;
        
        // Find the best pair (highest liquidity)
        let best_pair = dex_response.pairs.into_iter()
            .filter(|pair| pair.chainId.to_lowercase() == chain.to_lowercase())
            .max_by(|a, b| {
                let liquidity_a = a.liquidity.as_ref().map(|l| l.usd).unwrap_or(0.0);
                let liquidity_b = b.liquidity.as_ref().map(|l| l.usd).unwrap_or(0.0);
                liquidity_a.partial_cmp(&liquidity_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        Ok(best_pair)
    }

    async fn search_by_symbol(&self, symbol: &str) -> Result<Option<DexScreenerPair>> {
        let url = format!("{}/latest/dex/search?q={}", self.base_url, symbol);
        
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            error!("DEXScreener search error for {}: {}", symbol, response.status());
            return Ok(None);
        }

        let dex_response: DexScreenerResponse = response.json().await?;
        
        // Find the best pair (highest liquidity)
        let best_pair = dex_response.pairs.into_iter()
            .max_by(|a, b| {
                let liquidity_a = a.liquidity.as_ref().map(|l| l.usd).unwrap_or(0.0);
                let liquidity_b = b.liquidity.as_ref().map(|l| l.usd).unwrap_or(0.0);
                liquidity_a.partial_cmp(&liquidity_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        Ok(best_pair)
    }
}

#[async_trait]
impl MarketDataProvider for DexScreenerProvider {
    async fn get_quote(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        self.check_rate_limit().await;

        let (chain, contract) = self.resolve_symbol(symbol);
        
        // Try to find by contract first, then by symbol
        let pair = if contract.len() >= 20 {
            // Looks like a contract address
            self.search_by_contract(&chain, &contract).await?
        } else {
            // Try symbol search
            self.search_by_symbol(&contract).await?
        };

        if let Some(pair) = pair {
            self.last_quotes.insert(symbol.to_string(), pair.clone());

            // Convert to our MarketTick format
            let tick = MarketTick {
                schema_version: "1".to_string(),
                symbol: symbol.to_string(),
                ts: pair.pairCreatedAt,
                price: pair.priceUsd,
                source: "dexscreener".to_string(),
                volume: Some(format!("{:.2}", pair.volume.h24)),
                bid: None,
                ask: None,
                high: None,
                low: None,
            };

            self.request_count += 1;
            Ok(Some(tick))
        } else {
            Ok(None)
        }
    }

    async fn get_trades(&mut self, _symbol: &str, _limit: u32) -> Result<Vec<MarketTick>> {
        // DEXScreener doesn't provide individual trades
        Ok(vec![])
    }

    async fn get_bars(&mut self, _symbol: &str, _timeframe: &str, _limit: u32) -> Result<Vec<MarketTick>> {
        // DEXScreener doesn't provide historical bars
        Ok(vec![])
    }

    fn get_provider_name(&self) -> &'static str {
        "dexscreener"
    }

    fn get_asset_type(&self) -> AssetType {
        AssetType::CryptoDEX
    }

    fn get_rate_limit(&self) -> RateLimit {
        RateLimit::dexscreener()
    }

    fn is_realtime(&self) -> bool {
        false // DEXScreener is polled, not real-time
    }
}
