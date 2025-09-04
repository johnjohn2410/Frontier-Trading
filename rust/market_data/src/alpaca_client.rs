use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::{sleep, interval};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use tracing::{info, error, warn, debug};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaConfig {
    pub api_key: String,
    pub secret_key: String,
    pub rest_url: String,
    pub ws_url: String,
    pub test_ws_url: String,
    pub watchlist: Vec<String>,
    pub max_symbols: usize,
    pub rate_limit_per_minute: u32,
    pub rate_limit_per_second: u32,
    pub data_feed: String,
    pub paper_trading: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaBar {
    pub symbol: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
    pub trade_count: Option<i32>,
    pub vwap: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaQuote {
    pub symbol: String,
    pub timestamp: i64,
    pub bid: f64,
    pub ask: f64,
    pub bid_size: i32,
    pub ask_size: i32,
    pub tape: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaTrade {
    pub symbol: String,
    pub timestamp: i64,
    pub price: f64,
    pub size: i32,
    pub conditions: Option<Vec<String>>,
    pub tape: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaRestResponse<T> {
    pub bars: Option<HashMap<String, Vec<T>>>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpacaWsMessage {
    #[serde(rename = "T")]
    pub message_type: String,
    #[serde(rename = "S")]
    pub symbol: Option<String>,
    #[serde(rename = "t")]
    pub timestamp: Option<i64>,
    #[serde(rename = "o")]
    pub open: Option<f64>,
    #[serde(rename = "h")]
    pub high: Option<f64>,
    #[serde(rename = "l")]
    pub low: Option<f64>,
    #[serde(rename = "c")]
    pub close: Option<f64>,
    #[serde(rename = "v")]
    pub volume: Option<i64>,
    #[serde(rename = "p")]
    pub price: Option<f64>,
    #[serde(rename = "s")]
    pub size: Option<i32>,
    #[serde(rename = "bp")]
    pub bid_price: Option<f64>,
    #[serde(rename = "bs")]
    pub bid_size: Option<i32>,
    #[serde(rename = "ap")]
    pub ask_price: Option<f64>,
    #[serde(rename = "as")]
    pub ask_size: Option<i32>,
}

pub struct AlpacaClient {
    config: AlpacaConfig,
    client: reqwest::Client,
    rate_limiter: tokio::sync::Semaphore,
}

impl AlpacaClient {
    pub fn new(config: AlpacaConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("FrontierTrading/1.0")
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let rate_limiter = tokio::sync::Semaphore::new(config.rate_limit_per_second as usize);

        Self {
            config,
            client,
            rate_limiter,
        }
    }

    pub async fn get_latest_bars(&self, symbols: &[String]) -> Result<HashMap<String, AlpacaBar>> {
        debug!("Fetching latest bars for symbols: {:?}", symbols);
        
        // Rate limiting
        let _permit = self.rate_limiter.acquire().await?;
        
        let symbols_str = symbols.join(",");
        let url = format!("{}/stocks/bars/latest?symbols={}", self.config.rest_url, symbols_str);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Alpaca API error {}: {}", status, body);
            return Err(anyhow!("Alpaca API error: {} - {}", status, body));
        }

        let response_data: AlpacaRestResponse<AlpacaBar> = response.json().await?;
        
        let mut result = HashMap::new();
        if let Some(bars) = response_data.bars {
            for (symbol, bar_list) in bars {
                if let Some(bar) = bar_list.first() {
                    result.insert(symbol, bar.clone());
                }
            }
        }

        info!("Fetched {} latest bars", result.len());
        Ok(result)
    }

    pub async fn get_historical_bars(
        &self,
        symbol: &str,
        start: &str,
        end: &str,
        timeframe: &str,
    ) -> Result<Vec<AlpacaBar>> {
        debug!("Fetching historical bars for {} from {} to {}", symbol, start, end);
        
        let _permit = self.rate_limiter.acquire().await?;
        
        let url = format!(
            "{}/stocks/{}/bars?start={}&end={}&timeframe={}",
            self.config.rest_url, symbol, start, end, timeframe
        );
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.config.api_key)
            .header("APCA-API-SECRET-KEY", &self.config.secret_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Alpaca API error {}: {}", status, body);
            return Err(anyhow!("Alpaca API error: {} - {}", status, body));
        }

        let response_data: AlpacaRestResponse<AlpacaBar> = response.json().await?;
        
        let bars = response_data.bars
            .and_then(|bars_map| bars_map.get(symbol).cloned())
            .unwrap_or_default();

        info!("Fetched {} historical bars for {}", bars.len(), symbol);
        Ok(bars)
    }

    pub async fn start_websocket_stream<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(String, f64, i64) + Send + 'static,
    {
        info!("Starting Alpaca WebSocket stream");
        
        let ws_url = if self.config.paper_trading {
            &self.config.test_ws_url
        } else {
            &self.config.ws_url
        };

        let mut reconnect_attempts = 0;
        let max_attempts = 10;

        loop {
            match self.connect_websocket(ws_url, &mut callback).await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    error!("WebSocket connection failed: {}", e);
                    reconnect_attempts += 1;
                    
                    if reconnect_attempts >= max_attempts {
                        return Err(anyhow!("Max reconnection attempts reached"));
                    }
                    
                    let delay = Duration::from_millis(5000 * reconnect_attempts as u64);
                    warn!("Reconnecting in {:?} (attempt {}/{})", delay, reconnect_attempts, max_attempts);
                    sleep(delay).await;
                }
            }
        }

        Ok(())
    }

    async fn connect_websocket<F>(&self, ws_url: &str, callback: &mut F) -> Result<()>
    where
        F: FnMut(String, f64, i64) + Send + 'static,
    {
        debug!("Connecting to WebSocket: {}", ws_url);
        
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Send authentication
        let auth_msg = serde_json::json!({
            "action": "auth",
            "key": self.config.api_key,
            "secret": self.config.secret_key
        });
        
        write.send(Message::Text(auth_msg.to_string())).await?;
        info!("Sent authentication message");

        // Wait for authentication response
        if let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => {
                    debug!("Auth response: {}", text);
                    if !text.contains("authenticated") {
                        return Err(anyhow!("Authentication failed: {}", text));
                    }
                }
                _ => return Err(anyhow!("Unexpected auth response format")),
            }
        }

        // Subscribe to symbols
        let subscribe_msg = serde_json::json!({
            "action": "subscribe",
            "trades": self.config.watchlist,
            "quotes": self.config.watchlist,
            "bars": self.config.watchlist
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        info!("Subscribed to symbols: {:?}", self.config.watchlist);

        // Start heartbeat
        let mut heartbeat_interval = interval(Duration::from_millis(30000));
        let mut heartbeat_task = tokio::spawn(async move {
            loop {
                heartbeat_interval.tick().await;
                // Send heartbeat if needed
            }
        });

        // Process messages
        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = self.process_websocket_message(&text, callback).await {
                                error!("Error processing message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed by server");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            return Err(anyhow!("WebSocket error: {}", e));
                        }
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
                _ = heartbeat_interval.tick() => {
                    // Send heartbeat
                    let heartbeat = serde_json::json!({"action": "heartbeat"});
                    if let Err(e) = write.send(Message::Text(heartbeat.to_string())).await {
                        error!("Failed to send heartbeat: {}", e);
                        break;
                    }
                }
            }
        }

        heartbeat_task.abort();
        Ok(())
    }

    async fn process_websocket_message<F>(&self, text: &str, callback: &mut F) -> Result<()>
    where
        F: FnMut(String, f64, i64) + Send + 'static,
    {
        let msg: AlpacaWsMessage = serde_json::from_str(text)?;
        
        match msg.message_type.as_str() {
            "b" => {
                // Bar message
                if let (Some(symbol), Some(close), Some(timestamp)) = (msg.symbol, msg.close, msg.timestamp) {
                    debug!("Bar: {} = ${:.2} at {}", symbol, close, timestamp);
                    callback(symbol, close, timestamp);
                }
            }
            "q" => {
                // Quote message
                if let (Some(symbol), Some(bid_price), Some(timestamp)) = (msg.symbol, msg.bid_price, msg.timestamp) {
                    debug!("Quote: {} bid ${:.2} at {}", symbol, bid_price, timestamp);
                    callback(symbol, bid_price, timestamp);
                }
            }
            "t" => {
                // Trade message
                if let (Some(symbol), Some(price), Some(timestamp)) = (msg.symbol, msg.price, msg.timestamp) {
                    debug!("Trade: {} = ${:.2} at {}", symbol, price, timestamp);
                    callback(symbol, price, timestamp);
                }
            }
            "error" => {
                error!("WebSocket error message: {}", text);
                return Err(anyhow!("WebSocket error: {}", text));
            }
            _ => {
                debug!("Unknown message type: {}", msg.message_type);
            }
        }

        Ok(())
    }

    pub fn validate_config(&self) -> Result<()> {
        if self.config.api_key.is_empty() {
            return Err(anyhow!("Alpaca API key is required"));
        }
        
        if self.config.secret_key.is_empty() {
            return Err(anyhow!("Alpaca secret key is required"));
        }
        
        if self.config.watchlist.len() > self.config.max_symbols {
            return Err(anyhow!(
                "Watchlist size {} exceeds maximum {} symbols",
                self.config.watchlist.len(),
                self.config.max_symbols
            ));
        }
        
        Ok(())
    }
}

impl Default for AlpacaConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("APCA_API_KEY_ID").unwrap_or_default(),
            secret_key: std::env::var("APCA_API_SECRET_KEY").unwrap_or_default(),
            rest_url: std::env::var("ALPACA_REST_URL")
                .unwrap_or_else(|_| "https://data.alpaca.markets/v2".to_string()),
            ws_url: std::env::var("ALPACA_WS_URL")
                .unwrap_or_else(|_| "wss://stream.data.alpaca.markets/v2/iex".to_string()),
            test_ws_url: std::env::var("ALPACA_TEST_WS_URL")
                .unwrap_or_else(|_| "wss://stream.data.alpaca.markets/v2/test".to_string()),
            watchlist: std::env::var("ALPACA_WATCHLIST")
                .unwrap_or_else(|_| "AAPL,MSFT,AMZN,TSLA,SPY".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            max_symbols: std::env::var("ALPACA_MAX_SYMBOLS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            rate_limit_per_minute: std::env::var("ALPACA_RATE_LIMIT_PER_MINUTE")
                .unwrap_or_else(|_| "200".to_string())
                .parse()
                .unwrap_or(200),
            rate_limit_per_second: std::env::var("ALPACA_RATE_LIMIT_PER_SECOND")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            data_feed: std::env::var("ALPACA_DATA_FEED")
                .unwrap_or_else(|_| "iex".to_string()),
            paper_trading: std::env::var("ALPACA_PAPER_TRADING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}
