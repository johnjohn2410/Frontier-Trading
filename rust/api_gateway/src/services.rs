use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use futures_util::stream::Stream;
use anyhow::Result;
use tracing::{info, error, warn};

use shared::{
    Event, EventPayload, OrderRequestedEvent, MarketTickEvent,
    CopilotSuggestionEvent, RiskCheckResult, AccountState
};

use crate::config::GatewayConfig;

// ============================================================================
// MARKET DATA SERVICE
// ============================================================================

pub struct MarketDataService {
    client: Client,
    base_url: String,
}

impl MarketDataService {
    pub async fn new(config: &GatewayConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: config.market_data_url.clone(),
        })
    }

    pub async fn get_quote(&self, symbol: &str) -> Result<MarketTickEvent> {
        let response = self.client
            .get(&format!("{}/api/quotes/{}", self.base_url, symbol))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse quote: {}", e))
    }

    pub async fn get_quotes(&self, symbols: &[String]) -> Result<Vec<MarketTickEvent>> {
        let response = self.client
            .post(&format!("{}/api/quotes/batch", self.base_url))
            .json(symbols)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse quotes: {}", e))
    }

    pub async fn get_history(&self, symbol: &str, period: &str) -> Result<Vec<MarketTickEvent>> {
        let response = self.client
            .get(&format!("{}/api/history/{}/{}", self.base_url, symbol, period))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse history: {}", e))
    }

    pub async fn subscribe_quotes(&self, symbols: &[String]) -> impl Stream<Item = Event> {
        let (tx, rx) = mpsc::channel(100);
        
        // TODO: Implement WebSocket connection to market data service
        // For now, return empty stream
        tokio::spawn(async move {
            // WebSocket connection logic would go here
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

// ============================================================================
// TRADING ENGINE SERVICE
// ============================================================================

pub struct TradingEngineService {
    client: Client,
    base_url: String,
}

impl TradingEngineService {
    pub async fn new(config: &GatewayConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: config.trading_engine_url.clone(),
        })
    }

    pub async fn submit_order(&self, order: OrderRequestedEvent) -> Result<Event> {
        let response = self.client
            .post(&format!("{}/api/orders", self.base_url))
            .json(&order)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse order response: {}", e))
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Event> {
        let response = self.client
            .get(&format!("{}/api/orders/{}", self.base_url, order_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse order: {}", e))
    }

    pub async fn cancel_order(&self, order_id: &str) -> Result<Event> {
        let response = self.client
            .delete(&format!("{}/api/orders/{}", self.base_url, order_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse cancel response: {}", e))
    }

    pub async fn get_positions(&self, user_id: &str) -> Result<Vec<Event>> {
        let response = self.client
            .get(&format!("{}/api/positions/{}", self.base_url, user_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse positions: {}", e))
    }

    pub async fn mark_to_market(&self, prices: std::collections::HashMap<String, f64>) -> Result<Event> {
        let response = self.client
            .post(&format!("{}/api/mark-to-market", self.base_url))
            .json(&prices)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse mark-to-market response: {}", e))
    }

    pub async fn subscribe_orders(&self) -> impl Stream<Item = Event> {
        let (tx, rx) = mpsc::channel(100);
        
        // TODO: Implement WebSocket connection to trading engine
        tokio::spawn(async move {
            // WebSocket connection logic would go here
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

// ============================================================================
// NOTIFICATION SERVICE
// ============================================================================

pub struct NotificationService {
    client: Client,
    base_url: String,
}

impl NotificationService {
    pub async fn new(config: &GatewayConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: config.notification_url.clone(),
        })
    }

    pub async fn get_alerts(&self, user_id: &str) -> Result<Vec<Event>> {
        let response = self.client
            .get(&format!("{}/api/alerts/{}", self.base_url, user_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse alerts: {}", e))
    }

    pub async fn create_alert(&self, alert: serde_json::Value) -> Result<Event> {
        let response = self.client
            .post(&format!("{}/api/alerts", self.base_url))
            .json(&alert)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse alert response: {}", e))
    }

    pub async fn delete_alert(&self, alert_id: &str) -> Result<Event> {
        let response = self.client
            .delete(&format!("{}/api/alerts/{}", self.base_url, alert_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse delete response: {}", e))
    }

    pub async fn get_insights(&self, user_id: &str, symbol: &str) -> Result<Vec<Event>> {
        let response = self.client
            .get(&format!("{}/api/alerts/insights/{}/{}", self.base_url, user_id, symbol))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse insights: {}", e))
    }

    pub async fn subscribe_alerts(&self) -> impl Stream<Item = Event> {
        let (tx, rx) = mpsc::channel(100);
        
        // TODO: Implement WebSocket connection to notification service
        tokio::spawn(async move {
            // WebSocket connection logic would go here
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

// ============================================================================
// COPILOT SERVICE
// ============================================================================

pub struct CopilotService {
    client: Client,
    base_url: String,
}

impl CopilotService {
    pub async fn new(config: &GatewayConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: config.copilot_url.clone(),
        })
    }

    pub async fn analyze_stock(&self, user_id: &str, symbol: &str, 
                              market_data: serde_json::Value) -> Result<CopilotSuggestionEvent> {
        let payload = serde_json::json!({
            "user_id": user_id,
            "symbol": symbol,
            "market_data": market_data
        });

        let response = self.client
            .post(&format!("{}/api/analyze", self.base_url))
            .json(&payload)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse analysis: {}", e))
    }

    pub async fn get_suggestions(&self, user_id: &str) -> Result<Vec<CopilotSuggestionEvent>> {
        let response = self.client
            .get(&format!("{}/api/suggestions/{}", self.base_url, user_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse suggestions: {}", e))
    }

    pub async fn execute_suggestion(&self, suggestion_id: &str, 
                                  user_id: &str) -> Result<Event> {
        let payload = serde_json::json!({
            "user_id": user_id,
            "action": "execute"
        });

        let response = self.client
            .post(&format!("{}/api/suggestions/{}/execute", self.base_url, suggestion_id))
            .json(&payload)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse execution response: {}", e))
    }

    pub async fn process_suggestion(&self, suggestion: CopilotSuggestionEvent) -> Result<Event> {
        let response = self.client
            .post(&format!("{}/api/suggestions/process", self.base_url))
            .json(&suggestion)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse process response: {}", e))
    }

    pub async fn subscribe_suggestions(&self, user_id: &str) -> impl Stream<Item = CopilotSuggestionEvent> {
        let (tx, rx) = mpsc::channel(100);
        
        // TODO: Implement WebSocket connection to copilot service
        tokio::spawn(async move {
            // WebSocket connection logic would go here
        });
        
        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

// ============================================================================
// DATABASE SERVICE
// ============================================================================

pub struct DatabaseService {
    client: Client,
    base_url: String,
}

impl DatabaseService {
    pub async fn new(config: &GatewayConfig) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            base_url: config.database_url.clone(),
        })
    }

    pub async fn get_account(&self, user_id: &str) -> Result<AccountState> {
        let response = self.client
            .get(&format!("{}/api/accounts/{}", self.base_url, user_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse account: {}", e))
    }

    pub async fn update_account(&self, account: AccountState) -> Result<AccountState> {
        let response = self.client
            .put(&format!("{}/api/accounts/{}", self.base_url, account.user_id))
            .json(&account)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse account update: {}", e))
    }

    pub async fn get_risk_limits(&self, user_id: &str) -> Result<shared::RiskLimits> {
        let response = self.client
            .get(&format!("{}/api/risk-limits/{}", self.base_url, user_id))
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse risk limits: {}", e))
    }

    pub async fn update_risk_limits(&self, limits: shared::RiskLimits) -> Result<shared::RiskLimits> {
        let response = self.client
            .put(&format!("{}/api/risk-limits/{}", self.base_url, limits.user_id))
            .json(&limits)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse risk limits update: {}", e))
    }

    pub async fn get_trade_history(&self, user_id: &str, limit: Option<u32>) -> Result<Vec<Event>> {
        let mut url = format!("{}/api/trades/{}", self.base_url, user_id);
        if let Some(limit) = limit {
            url.push_str(&format!("?limit={}", limit));
        }

        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        response.json().await.map_err(|e| anyhow::anyhow!("Failed to parse trade history: {}", e))
    }
}

// ============================================================================
// SERVICE HEALTH CHECKS
// ============================================================================

pub async fn check_service_health(service_url: &str) -> Result<bool> {
    let client = Client::new();
    let response = client
        .get(&format!("{}/health", service_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match response {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e) => {
            warn!("Health check failed for {}: {}", service_url, e);
            Ok(false)
        }
    }
}

pub async fn check_all_services(config: &GatewayConfig) -> std::collections::HashMap<String, bool> {
    let mut health_status = std::collections::HashMap::new();
    
    let services = vec![
        ("market_data", &config.market_data_url),
        ("trading_engine", &config.trading_engine_url),
        ("notifications", &config.notification_url),
        ("copilot", &config.copilot_url),
        ("database", &config.database_url),
    ];
    
    for (name, url) in services {
        let health = check_service_health(url).await.unwrap_or(false);
        health_status.insert(name.to_string(), health);
    }
    
    health_status
}
