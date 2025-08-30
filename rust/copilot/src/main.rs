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
use uuid::Uuid;

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

// ============================================================================
// COPILOT SUGGESTION STRUCTURE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotSuggestion {
    pub schema_version: String,
    pub correlation_id: String,
    pub timestamp: String,
    pub event_type: String,
    pub payload: SuggestionPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionPayload {
    pub symbol: String,
    pub suggestion: String,
    pub features: SuggestionFeatures,
    pub confidence: f64,
    pub what_if: WhatIfAnalysis,
    pub guardrails: GuardrailCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeatures {
    pub cmin: String,
    pub cmax: String,
    pub vol_z: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfAnalysis {
    pub qty: String,
    pub limit: String,
    pub delta_bp: String,
    pub est_fee: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailCheck {
    pub max_pos_ok: bool,
    pub daily_loss_ok: bool,
    pub mkt_hours_ok: bool,
}

// ============================================================================
// EMA CALCULATOR
// ============================================================================

pub struct EmaCalculator {
    short_period: usize,
    long_period: usize,
    short_ema: f64,
    long_ema: f64,
    price_history: Vec<f64>,
    volume_history: Vec<f64>,
}

impl EmaCalculator {
    pub fn new(short_period: usize, long_period: usize) -> Self {
        Self {
            short_period,
            long_period,
            short_ema: 0.0,
            long_ema: 0.0,
            price_history: Vec::new(),
            volume_history: Vec::new(),
        }
    }

    pub fn update(&mut self, price: f64, volume: f64) -> Option<EmaSignal> {
        self.price_history.push(price);
        self.volume_history.push(volume);
        
        // Keep only recent history
        if self.price_history.len() > self.long_period * 2 {
            self.price_history.remove(0);
            self.volume_history.remove(0);
        }
        
        // Calculate EMAs
        if self.price_history.len() >= self.long_period {
            self.short_ema = self.calculate_ema(&self.price_history, self.short_period);
            self.long_ema = self.calculate_ema(&self.price_history, self.long_period);
            
            // Check for crossover
            if self.price_history.len() >= 2 {
                let prev_short = self.calculate_ema(&self.price_history[..self.price_history.len()-1], self.short_period);
                let prev_long = self.calculate_ema(&self.price_history[..self.price_history.len()-1], self.long_period);
                
                let current_cross = self.short_ema > self.long_ema;
                let previous_cross = prev_short > prev_long;
                
                if current_cross != previous_cross {
                    // Calculate volume Z-score
                    let vol_z = self.calculate_volume_z_score(volume);
                    
                    return Some(EmaSignal {
                        symbol: "".to_string(), // Will be set by caller
                        price,
                        short_ema: self.short_ema,
                        long_ema: self.long_ema,
                        volume_z_score: vol_z,
                        crossover_type: if current_cross { "bullish" } else { "bearish" },
                    });
                }
            }
        }
        
        None
    }

    fn calculate_ema(&self, prices: &[f64], period: usize) -> f64 {
        if prices.len() < period {
            return prices.last().copied().unwrap_or(0.0);
        }
        
        let multiplier = 2.0 / (period as f64 + 1.0);
        let mut ema = prices[0];
        
        for &price in prices.iter().skip(1) {
            ema = (price * multiplier) + (ema * (1.0 - multiplier));
        }
        
        ema
    }

    fn calculate_volume_z_score(&self, current_volume: f64) -> f64 {
        if self.volume_history.len() < 20 {
            return 0.0;
        }
        
        let mean = self.volume_history.iter().sum::<f64>() / self.volume_history.len() as f64;
        let variance = self.volume_history.iter()
            .map(|&v| (v - mean).powi(2))
            .sum::<f64>() / self.volume_history.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev == 0.0 {
            return 0.0;
        }
        
        (current_volume - mean) / std_dev
    }
}

#[derive(Debug, Clone)]
pub struct EmaSignal {
    pub symbol: String,
    pub price: f64,
    pub short_ema: f64,
    pub long_ema: f64,
    pub volume_z_score: f64,
    pub crossover_type: &'static str,
}

// ============================================================================
// COPILOT SERVICE
// ============================================================================

pub struct CopilotService {
    redis_client: Arc<Client>,
    ema_calculators: Arc<RwLock<HashMap<String, EmaCalculator>>>,
    suggestions: Arc<RwLock<HashMap<String, Vec<CopilotSuggestion>>>>,
}

impl CopilotService {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        
        Ok(Self {
            redis_client: Arc::new(client),
            ema_calculators: Arc::new(RwLock::new(HashMap::new())),
            suggestions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start_monitoring(&self) {
        info!("Starting Copilot monitoring for market ticks");
        
        let service = self.clone();
        tokio::spawn(async move {
            service.monitor_ticks().await;
        });
    }

    async fn monitor_ticks(&self) {
        let symbols = vec!["AAPL", "SPY", "BTCUSD", "GOOGL", "TSLA", "MSFT"];
        
        for symbol in symbols {
            let service = self.clone();
            tokio::spawn(async move {
                service.monitor_symbol(symbol).await;
            });
        }
    }

    async fn monitor_symbol(&self, symbol: &str) {
        let mut conn = match self.redis_client.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            }
        };
        
        // Create consumer group if it doesn't exist
        let stream_key = format!("ticks.{}", symbol);
        let consumer_group = "md.cg.copilot";
        
        if let Err(_) = conn.xgroup_create(&stream_key, consumer_group, "$", true) {
            info!("Consumer group {} already exists for stream {}", consumer_group, stream_key);
        }
        
        let mut last_id = "0".to_string();
        
        loop {
            match conn.xreadgroup(
                consumer_group,
                &format!("copilot-{}", symbol),
                &[(&stream_key, ">")],
                Some(100),
                Some(1000),
            ) {
                Ok(messages) => {
                    for (stream_name, stream_messages) in messages {
                        for (message_id, fields) in stream_messages {
                            if let Some(tick_field) = fields.iter().find(|(k, _)| k == "tick") {
                                if let Ok(tick) = serde_json::from_str::<MarketTick>(&tick_field.1) {
                                    self.process_tick(tick).await;
                                }
                            }
                            
                            // Acknowledge the message
                            if let Err(e) = conn.xack(&stream_name, consumer_group, &message_id) {
                                error!("Failed to acknowledge message {}: {}", message_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from stream {}: {}", stream_key, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn process_tick(&self, tick: MarketTick) {
        let price = tick.price.parse::<f64>().unwrap_or(0.0);
        let volume = tick.volume.as_ref()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1000.0);
        
        // Get or create EMA calculator for this symbol
        let mut calculators = self.ema_calculators.write().await;
        let calculator = calculators.entry(tick.symbol.clone()).or_insert_with(|| {
            EmaCalculator::new(12, 26) // 12 and 26 period EMAs
        });
        
        // Update EMA and check for signals
        if let Some(signal) = calculator.update(price, volume) {
            let mut signal = signal;
            signal.symbol = tick.symbol.clone();
            
            // Generate suggestion if volume spike is significant
            if signal.volume_z_score.abs() > 1.5 {
                self.generate_suggestion(signal).await;
            }
        }
    }

    async fn generate_suggestion(&self, signal: EmaSignal) {
        let correlation_id = Uuid::new_v4().to_string();
        
        let suggestion = if signal.crossover_type == "bullish" {
            "EMA band crossed up; consider a limit buy."
        } else {
            "EMA band crossed down; consider a limit sell."
        };
        
        let action = if signal.crossover_type == "bullish" { "buy" } else { "sell" };
        let limit_price = if signal.crossover_type == "bullish" {
            signal.price * 1.001 // Slightly above current price
        } else {
            signal.price * 0.999 // Slightly below current price
        };
        
        let quantity = 5.0; // Fixed quantity for now
        let delta_bp = -(quantity * limit_price);
        
        let copilot_suggestion = CopilotSuggestion {
            schema_version: "1".to_string(),
            correlation_id: correlation_id.clone(),
            timestamp: Utc::now().to_rfc3339(),
            event_type: "copilot.suggestion".to_string(),
            payload: SuggestionPayload {
                symbol: signal.symbol.clone(),
                suggestion: suggestion.to_string(),
                features: SuggestionFeatures {
                    cmin: format!("{:.2}", signal.short_ema),
                    cmax: format!("{:.2}", signal.long_ema),
                    vol_z: format!("{:.1}", signal.volume_z_score),
                },
                confidence: 0.63, // Fixed confidence for now
                what_if: WhatIfAnalysis {
                    qty: format!("{:.0}", quantity),
                    limit: format!("{:.2}", limit_price),
                    delta_bp: format!("{:.2}", delta_bp),
                    est_fee: "0".to_string(),
                },
                guardrails: GuardrailCheck {
                    max_pos_ok: true,
                    daily_loss_ok: true,
                    mkt_hours_ok: true,
                },
            },
        };
        
        // Publish to suggestions stream
        if let Err(e) = self.publish_suggestion(&copilot_suggestion).await {
            error!("Failed to publish suggestion: {}", e);
        }
        
        // Store suggestion
        {
            let mut suggestions = self.suggestions.write().await;
            let symbol_suggestions = suggestions.entry(signal.symbol.clone()).or_insert_with(Vec::new);
            symbol_suggestions.push(copilot_suggestion);
            
            // Keep only recent suggestions
            if symbol_suggestions.len() > 100 {
                symbol_suggestions.remove(0);
            }
        }
        
        info!("Generated suggestion for {}: {}", signal.symbol, suggestion);
    }

    async fn publish_suggestion(&self, suggestion: &CopilotSuggestion) -> RedisResult<String> {
        let mut conn = self.redis_client.get_connection()?;
        
        let suggestion_json = serde_json::to_string(suggestion)
            .map_err(|e| redis::RedisError::from((
                redis::ErrorKind::InvalidArgument,
                "Failed to serialize suggestion",
                e.to_string(),
            )))?;
        
        let message_id = conn.xadd("suggestions.stream", "*", &[("suggestion", &suggestion_json)])?;
        
        info!("Published suggestion to suggestions.stream: {}", message_id);
        Ok(message_id)
    }

    pub async fn get_suggestions(&self, symbol: &str) -> Vec<CopilotSuggestion> {
        let suggestions = self.suggestions.read().await;
        suggestions.get(symbol)
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}

impl Clone for CopilotService {
    fn clone(&self) -> Self {
        Self {
            redis_client: self.redis_client.clone(),
            ema_calculators: self.ema_calculators.clone(),
            suggestions: self.suggestions.clone(),
        }
    }
}

// ============================================================================
// API HANDLERS
// ============================================================================

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "copilot",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_suggestions(
    Path(symbol): Path<String>,
    axum::extract::State(service): axum::extract::State<Arc<CopilotService>>,
) -> Json<serde_json::Value> {
    let suggestions = service.get_suggestions(&symbol).await;
    
    Json(serde_json::json!({
        "symbol": symbol,
        "suggestions": suggestions,
        "count": suggestions.len()
    }))
}

async fn analyze_stock(
    Json(payload): Json<serde_json::Value>,
    axum::extract::State(service): axum::extract::State<Arc<CopilotService>>,
) -> Json<serde_json::Value> {
    // For now, return a mock analysis
    // In a real implementation, this would trigger AI analysis
    Json(serde_json::json!({
        "symbol": payload.get("symbol").and_then(|s| s.as_str()).unwrap_or("UNKNOWN"),
        "analysis": "Mock analysis - implement AI integration",
        "confidence": 0.5,
        "timestamp": chrono::Utc::now().to_rfc3339()
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
    
    let port = std::env::var("COPILOT_PORT")
        .unwrap_or_else(|_| "8004".to_string())
        .parse::<u16>()?;
    
    info!("Starting Copilot Service on port {}", port);
    
    // Initialize copilot service
    let copilot = Arc::new(CopilotService::new(&redis_url)?);
    
    // Start monitoring
    copilot.start_monitoring().await;
    
    // Build router
    let app = axum::Router::new()
        .route("/health", get(health_check))
        .route("/api/suggestions/:symbol", get(get_suggestions))
        .route("/api/analyze", post(analyze_stock))
        .with_state(copilot);
    
    // Start server
    let addr = format!("0.0.0.0:{}", port).parse()?;
    info!("Copilot Service listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}
