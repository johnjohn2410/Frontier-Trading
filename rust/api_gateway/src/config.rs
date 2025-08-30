use serde::{Deserialize, Serialize};
use std::env;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub port: u16,
    pub market_data_url: String,
    pub trading_engine_url: String,
    pub notification_url: String,
    pub copilot_url: String,
    pub database_url: String,
    pub redis_url: String,
    pub cors_origins: Vec<String>,
    pub log_level: String,
}

impl GatewayConfig {
    pub fn load() -> Result<Self> {
        dotenv::dotenv().ok();
        
        Ok(Self {
            port: env::var("GATEWAY_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            
            market_data_url: env::var("MARKET_DATA_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            
            trading_engine_url: env::var("TRADING_ENGINE_URL")
                .unwrap_or_else(|_| "http://localhost:8003".to_string()),
            
            notification_url: env::var("NOTIFICATION_URL")
                .unwrap_or_else(|_| "http://localhost:8002".to_string()),
            
            copilot_url: env::var("COPILOT_URL")
                .unwrap_or_else(|_| "http://localhost:8004".to_string()),
            
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "http://localhost:8005".to_string()),
            
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        })
    }

    pub fn validate(&self) -> Result<()> {
        // Validate URLs
        if !self.market_data_url.starts_with("http") {
            anyhow::bail!("Invalid market data URL: {}", self.market_data_url);
        }
        
        if !self.trading_engine_url.starts_with("http") {
            anyhow::bail!("Invalid trading engine URL: {}", self.trading_engine_url);
        }
        
        if !self.notification_url.starts_with("http") {
            anyhow::bail!("Invalid notification URL: {}", self.notification_url);
        }
        
        if !self.copilot_url.starts_with("http") {
            anyhow::bail!("Invalid copilot URL: {}", self.copilot_url);
        }
        
        if !self.database_url.starts_with("http") {
            anyhow::bail!("Invalid database URL: {}", self.database_url);
        }
        
        if !self.redis_url.starts_with("redis") {
            anyhow::bail!("Invalid Redis URL: {}", self.redis_url);
        }
        
        // Validate port
        if self.port == 0 {
            anyhow::bail!("Invalid port: {}", self.port);
        }
        
        Ok(())
    }
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            market_data_url: "http://localhost:8001".to_string(),
            trading_engine_url: "http://localhost:8003".to_string(),
            notification_url: "http://localhost:8002".to_string(),
            copilot_url: "http://localhost:8004".to_string(),
            database_url: "http://localhost:8005".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            cors_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:3001".to_string(),
            ],
            log_level: "info".to_string(),
        }
    }
}
