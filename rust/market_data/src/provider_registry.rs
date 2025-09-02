use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};
use anyhow::Result;
use chrono::{DateTime, Utc};

use crate::MarketTick;

// ============================================================================
// PROVIDER TRAIT
// ============================================================================

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_quote(&mut self, symbol: &str) -> Result<Option<MarketTick>>;
    async fn get_trades(&mut self, symbol: &str, limit: u32) -> Result<Vec<MarketTick>>;
    async fn get_bars(&mut self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<MarketTick>>;
    fn get_provider_name(&self) -> &'static str;
    fn get_asset_type(&self) -> AssetType;
    fn get_rate_limit(&self) -> RateLimit;
    fn is_realtime(&self) -> bool;
}

// ============================================================================
// ASSET TYPES
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetType {
    USEquity,
    CryptoCEX,
    CryptoDEX,
    CryptoMetadata,
}

impl AssetType {
    pub fn from_symbol(symbol: &str) -> Self {
        if symbol.contains('.') {
            // Check for venue-specific symbols
            if symbol.ends_with(".BIN") || symbol.ends_with(".CBSE") {
                return AssetType::CryptoCEX;
            }
        }
        
        if symbol.contains(':') {
            // Chain:contract format for DEX
            return AssetType::CryptoDEX;
        }
        
        if symbol.len() <= 5 && symbol.chars().all(|c| c.is_ascii_uppercase()) {
            // Likely US equity
            return AssetType::USEquity;
        }
        
        // Default to crypto
        AssetType::CryptoCEX
    }
}

// ============================================================================
// RATE LIMITS
// ============================================================================

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_second: u32,
    pub burst_limit: u32,
}

impl RateLimit {
    pub fn alpaca() -> Self {
        Self {
            requests_per_minute: 200,
            requests_per_second: 10,
            burst_limit: 50,
        }
    }
    
    pub fn binance() -> Self {
        Self {
            requests_per_minute: 1200,
            requests_per_second: 20,
            burst_limit: 100,
        }
    }
    
    pub fn dexscreener() -> Self {
        Self {
            requests_per_minute: 300,
            requests_per_second: 5,
            burst_limit: 10,
        }
    }
    
    pub fn birdeye() -> Self {
        Self {
            requests_per_minute: 1800,
            requests_per_second: 1,
            burst_limit: 30,
        }
    }
    
    pub fn coingecko() -> Self {
        Self {
            requests_per_minute: 30,
            requests_per_second: 1,
            burst_limit: 5,
        }
    }
}

// ============================================================================
// PROVIDER REGISTRY
// ============================================================================

pub struct ProviderRegistry {
    providers: HashMap<AssetType, Vec<Box<dyn MarketDataProvider>>>,
    symbol_cache: Arc<RwLock<HashMap<String, CachedTick>>>,
    rate_limit_tracker: Arc<RwLock<HashMap<String, RateLimitTracker>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            symbol_cache: Arc::new(RwLock::new(HashMap::new())),
            rate_limit_tracker: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register_provider(&mut self, provider: Box<dyn MarketDataProvider>) {
        let asset_type = provider.get_asset_type();
        let providers = self.providers.entry(asset_type).or_insert_with(Vec::new);
        providers.push(provider);
        
        info!("Registered {} provider for {:?}", provider.get_provider_name(), asset_type);
    }
    
    pub async fn get_quote(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        let asset_type = AssetType::from_symbol(symbol);
        
        // Check cache first
        if let Some(cached) = self.get_cached_tick(symbol).await {
            if !cached.is_expired() {
                return Ok(Some(cached.tick));
            }
        }
        
        // Find appropriate provider
        if let Some(providers) = self.providers.get_mut(&asset_type) {
            for provider in providers.iter_mut() {
                // Check rate limits
                if !self.check_rate_limit(provider.get_provider_name()).await {
                    continue;
                }
                
                match provider.get_quote(symbol).await {
                    Ok(Some(tick)) => {
                        // Cache the tick
                        self.cache_tick(symbol, tick.clone()).await;
                        return Ok(Some(tick));
                    }
                    Ok(None) => {
                        warn!("No data from {} for {}", provider.get_provider_name(), symbol);
                        continue;
                    }
                    Err(e) => {
                        error!("Error from {} for {}: {}", provider.get_provider_name(), symbol, e);
                        continue;
                    }
                }
            }
        }
        
        // Try fallback providers
        self.try_fallback_providers(symbol).await
    }
    
    async fn try_fallback_providers(&mut self, symbol: &str) -> Result<Option<MarketTick>> {
        // Try metadata providers as fallback
        if let Some(providers) = self.providers.get_mut(&AssetType::CryptoMetadata) {
            for provider in providers.iter_mut() {
                if !self.check_rate_limit(provider.get_provider_name()).await {
                    continue;
                }
                
                if let Ok(Some(tick)) = provider.get_quote(symbol).await {
                    self.cache_tick(symbol, tick.clone()).await;
                    return Ok(Some(tick));
                }
            }
        }
        
        Ok(None)
    }
    
    async fn get_cached_tick(&self, symbol: &str) -> Option<CachedTick> {
        let cache = self.symbol_cache.read().await;
        cache.get(symbol).cloned()
    }
    
    async fn cache_tick(&self, symbol: &str, tick: MarketTick) {
        let mut cache = self.symbol_cache.write().await;
        let cached = CachedTick {
            tick,
            cached_at: Utc::now(),
            ttl_seconds: 10, // 10 second TTL for most data
        };
        cache.insert(symbol.to_string(), cached);
    }
    
    async fn check_rate_limit(&self, provider_name: &str) -> bool {
        let mut tracker = self.rate_limit_tracker.write().await;
        let tracker_entry = tracker.entry(provider_name.to_string()).or_insert_with(|| {
            RateLimitTracker::new(RateLimit::alpaca()) // Default to Alpaca limits
        });
        
        tracker_entry.check_limit()
    }
    
    pub fn get_provider_stats(&self) -> HashMap<String, ProviderStats> {
        let mut stats = HashMap::new();
        
        for (asset_type, providers) in &self.providers {
            for provider in providers {
                let name = provider.get_provider_name().to_string();
                stats.insert(name, ProviderStats {
                    asset_type: asset_type.clone(),
                    is_realtime: provider.is_realtime(),
                    rate_limit: provider.get_rate_limit(),
                });
            }
        }
        
        stats
    }
}

// ============================================================================
// CACHING & RATE LIMITING
// ============================================================================

#[derive(Debug, Clone)]
struct CachedTick {
    tick: MarketTick,
    cached_at: DateTime<Utc>,
    ttl_seconds: u64,
}

impl CachedTick {
    fn is_expired(&self) -> bool {
        let now = Utc::now();
        let age = now.signed_duration_since(self.cached_at);
        age.num_seconds() as u64 > self.ttl_seconds
    }
}

struct RateLimitTracker {
    rate_limit: RateLimit,
    request_count: u32,
    window_start: std::time::Instant,
    last_request: std::time::Instant,
}

impl RateLimitTracker {
    fn new(rate_limit: RateLimit) -> Self {
        Self {
            rate_limit,
            request_count: 0,
            window_start: std::time::Instant::now(),
            last_request: std::time::Instant::now(),
        }
    }
    
    fn check_limit(&mut self) -> bool {
        let now = std::time::Instant::now();
        
        // Reset window if needed
        if now.duration_since(self.window_start).as_secs() >= 60 {
            self.request_count = 0;
            self.window_start = now;
        }
        
        // Check per-minute limit
        if self.request_count >= self.rate_limit.requests_per_minute {
            return false;
        }
        
        // Check per-second limit
        if now.duration_since(self.last_request).as_millis() < 1000 {
            // Less than 1 second since last request
            if self.request_count % 60 >= self.rate_limit.requests_per_second as u32 {
                return false;
            }
        }
        
        self.request_count += 1;
        self.last_request = now;
        true
    }
}

// ============================================================================
// PROVIDER STATS
// ============================================================================

#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub asset_type: AssetType,
    pub is_realtime: bool,
    pub rate_limit: RateLimit,
}

// ============================================================================
// SYMBOL RESOLUTION
// ============================================================================

pub struct SymbolResolver;

impl SymbolResolver {
    pub fn resolve_provider(symbol: &str) -> (AssetType, String, Option<String>) {
        let asset_type = AssetType::from_symbol(symbol);
        
        match asset_type {
            AssetType::USEquity => {
                (asset_type, symbol.to_string(), None)
            }
            AssetType::CryptoCEX => {
                if let Some((base, venue)) = symbol.split_once('.') {
                    (asset_type, base.to_string(), Some(venue.to_string()))
                } else {
                    (asset_type, symbol.to_string(), None)
                }
            }
            AssetType::CryptoDEX => {
                if let Some((chain, contract)) = symbol.split_once(':') {
                    (asset_type, contract.to_string(), Some(chain.to_string()))
                } else {
                    (asset_type, symbol.to_string(), None)
                }
            }
            AssetType::CryptoMetadata => {
                (asset_type, symbol.to_string(), None)
            }
        }
    }
    
    pub fn normalize_symbol(symbol: &str) -> String {
        symbol.to_uppercase().replace(['.', ':'], "_")
    }
}
