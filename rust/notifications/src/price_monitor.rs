use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, warn, error};

use crate::models::*;
use crate::database::Database;

pub struct PriceMonitor {
    db: Arc<Database>,
    price_history: Arc<RwLock<HashMap<String, Vec<PricePoint>>>>,
    volume_history: Arc<RwLock<HashMap<String, Vec<VolumePoint>>>>,
    alert_manager: Arc<AlertManager>,
}

#[derive(Debug, Clone)]
struct PricePoint {
    timestamp: DateTime<Utc>,
    price: f64,
    volume: u64,
}

#[derive(Debug, Clone)]
struct VolumePoint {
    timestamp: DateTime<Utc>,
    volume: u64,
    average_volume: u64,
}

impl PriceMonitor {
    pub fn new(db: Arc<Database>, alert_manager: Arc<AlertManager>) -> Self {
        Self {
            db,
            price_history: Arc::new(RwLock::new(HashMap::new())),
            volume_history: Arc::new(RwLock::new(HashMap::new())),
            alert_manager,
        }
    }

    pub async fn update_price(&self, symbol: String, price: f64, volume: u64) -> Result<()> {
        let now = Utc::now();
        
        // Update price history
        {
            let mut history = self.price_history.write().await;
            let symbol_history = history.entry(symbol.clone()).or_insert_with(Vec::new);
            symbol_history.push(PricePoint {
                timestamp: now,
                price,
                volume,
            });
            
            // Keep only last 1000 price points
            if symbol_history.len() > 1000 {
                symbol_history.remove(0);
            }
        }

        // Update volume history
        {
            let mut volume_history = self.volume_history.write().await;
            let symbol_volume = volume_history.entry(symbol.clone()).or_insert_with(Vec::new);
            
            // Calculate average volume over last 20 periods
            let avg_volume = self.calculate_average_volume(&symbol_volume);
            
            symbol_volume.push(VolumePoint {
                timestamp: now,
                volume,
                average_volume: avg_volume,
            });
            
            // Keep only last 100 volume points
            if symbol_volume.len() > 100 {
                symbol_volume.remove(0);
            }
        }

        // Check for percentage change alerts
        self.check_percentage_alerts(&symbol, price).await?;
        
        // Check for volume spikes
        self.check_volume_alerts(&symbol, volume).await?;
        
        // Check for technical breakouts
        self.check_technical_alerts(&symbol, price).await?;

        // Forward to alert manager
        self.alert_manager.update_price(symbol, price).await?;

        Ok(())
    }

    async fn check_percentage_alerts(&self, symbol: &str, current_price: f64) -> Result<()> {
        let history = self.price_history.read().await;
        let symbol_history = match history.get(symbol) {
            Some(h) => h,
            None => return Ok(()),
        };

        if symbol_history.len() < 2 {
            return Ok(());
        }

        // Get previous price (5 minutes ago)
        let five_minutes_ago = Utc::now() - Duration::minutes(5);
        let previous_price = symbol_history
            .iter()
            .rev()
            .find(|p| p.timestamp < five_minutes_ago)
            .map(|p| p.price);

        if let Some(prev_price) = previous_price {
            let percentage_change = ((current_price - prev_price) / prev_price) * 100.0;
            
            // Check for significant percentage changes
            if percentage_change.abs() >= 5.0 {
                self.create_percentage_notification(symbol, current_price, prev_price, percentage_change).await?;
            }
        }

        Ok(())
    }

    async fn check_volume_alerts(&self, symbol: &str, current_volume: u64) -> Result<()> {
        let volume_history = self.volume_history.read().await;
        let symbol_volume = match volume_history.get(symbol) {
            Some(v) => v,
            None => return Ok(()),
        };

        if symbol_volume.len() < 2 {
            return Ok(());
        }

        // Get average volume
        let avg_volume = self.calculate_average_volume(symbol_volume);
        
        if avg_volume > 0 {
            let volume_ratio = current_volume as f64 / avg_volume as f64;
            
            // Alert if volume is 3x higher than average
            if volume_ratio >= 3.0 {
                self.create_volume_notification(symbol, current_volume, avg_volume, volume_ratio).await?;
            }
        }

        Ok(())
    }

    async fn check_technical_alerts(&self, symbol: &str, current_price: f64) -> Result<()> {
        let history = self.price_history.read().await;
        let symbol_history = match history.get(symbol) {
            Some(h) => h,
            None => return Ok(()),
        };

        if symbol_history.len() < 20 {
            return Ok(());
        }

        // Calculate simple moving averages
        let sma_20 = self.calculate_sma(symbol_history, 20);
        let sma_50 = self.calculate_sma(symbol_history, 50.min(symbol_history.len()));

        // Check for moving average crossovers
        if let (Some(sma20), Some(sma50)) = (sma_20, sma_50) {
            let prev_sma20 = self.calculate_sma(&symbol_history[..symbol_history.len()-1], 20);
            let prev_sma50 = self.calculate_sma(&symbol_history[..symbol_history.len()-1], 50.min(symbol_history.len()-1));
            
            if let (Some(prev_sma20), Some(prev_sma50)) = (prev_sma20, prev_sma50) {
                // Golden cross (SMA 20 crosses above SMA 50)
                if prev_sma20 <= prev_sma50 && sma20 > sma50 {
                    self.create_technical_notification(symbol, "Golden Cross", current_price, sma20, sma50).await?;
                }
                // Death cross (SMA 20 crosses below SMA 50)
                else if prev_sma20 >= prev_sma50 && sma20 < sma50 {
                    self.create_technical_notification(symbol, "Death Cross", current_price, sma20, sma50).await?;
                }
            }
        }

        // Check for support/resistance levels
        self.check_support_resistance(symbol, current_price, symbol_history).await?;

        Ok(())
    }

    fn calculate_average_volume(&self, volume_history: &[VolumePoint]) -> u64 {
        if volume_history.is_empty() {
            return 0;
        }
        
        let sum: u64 = volume_history.iter().map(|v| v.volume).sum();
        sum / volume_history.len() as u64
    }

    fn calculate_sma(&self, history: &[PricePoint], period: usize) -> Option<f64> {
        if history.len() < period {
            return None;
        }
        
        let recent_prices: Vec<f64> = history.iter().rev().take(period).map(|p| p.price).collect();
        let sum: f64 = recent_prices.iter().sum();
        Some(sum / period as f64)
    }

    async fn check_support_resistance(&self, symbol: &str, current_price: f64, history: &[PricePoint]) -> Result<()> {
        if history.len() < 50 {
            return Ok(());
        }

        // Find recent highs and lows
        let recent_prices: Vec<f64> = history.iter().rev().take(50).map(|p| p.price).collect();
        let high = recent_prices.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let low = recent_prices.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        // Check if price is near resistance (within 1%)
        let resistance_threshold = high * 0.99;
        if current_price >= resistance_threshold {
            self.create_technical_notification(symbol, "Resistance Level", current_price, high, 0.0).await?;
        }

        // Check if price is near support (within 1%)
        let support_threshold = low * 1.01;
        if current_price <= support_threshold {
            self.create_technical_notification(symbol, "Support Level", current_price, low, 0.0).await?;
        }

        Ok(())
    }

    async fn create_percentage_notification(&self, symbol: &str, current_price: f64, previous_price: f64, percentage_change: f64) -> Result<()> {
        let (title, message) = if percentage_change > 0.0 {
            (
                format!("{} Price Surge", symbol),
                format!("{} has surged {:.1}% from ${:.2f} to ${:.2f}", 
                       symbol, percentage_change, previous_price, current_price)
            )
        } else {
            (
                format!("{} Price Drop", symbol),
                format!("{} has dropped {:.1}% from ${:.2f} to ${:.2f}", 
                       symbol, percentage_change.abs(), previous_price, current_price)
            )
        };

        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(), // Will be set by alert manager
            title,
            message,
            notification_type: NotificationType::PriceAlert,
            priority: if percentage_change.abs() >= 10.0 { NotificationPriority::High } else { NotificationPriority::Medium },
            channels: vec![NotificationChannel::InApp, NotificationChannel::Push],
            metadata: serde_json::json!({
                "symbol": symbol,
                "current_price": current_price,
                "previous_price": previous_price,
                "percentage_change": percentage_change,
                "alert_type": "percentage_change"
            }),
            is_read: false,
            created_at: Utc::now(),
        };

        self.db.create_notification(&notification).await?;
        
        info!("Created percentage change notification for {}: {:.1}%", symbol, percentage_change);
        Ok(())
    }

    async fn create_volume_notification(&self, symbol: &str, current_volume: u64, avg_volume: u64, volume_ratio: f64) -> Result<()> {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(), // Will be set by alert manager
            title: format!("{} Volume Spike", symbol),
            message: format!("{} trading volume is {:.1}x above average ({} vs {} avg)", 
                           symbol, volume_ratio, current_volume, avg_volume),
            notification_type: NotificationType::VolumeAlert,
            priority: NotificationPriority::Medium,
            channels: vec![NotificationChannel::InApp, NotificationChannel::Push],
            metadata: serde_json::json!({
                "symbol": symbol,
                "current_volume": current_volume,
                "average_volume": avg_volume,
                "volume_ratio": volume_ratio,
                "alert_type": "volume_spike"
            }),
            is_read: false,
            created_at: Utc::now(),
        };

        self.db.create_notification(&notification).await?;
        
        info!("Created volume spike notification for {}: {:.1}x average", symbol, volume_ratio);
        Ok(())
    }

    async fn create_technical_notification(&self, symbol: &str, signal: &str, current_price: f64, value1: f64, value2: f64) -> Result<()> {
        let message = if value2 > 0.0 {
            format!("{} {} signal: Price ${:.2f}, SMA20 ${:.2f}, SMA50 ${:.2f}", 
                   symbol, signal, current_price, value1, value2)
        } else {
            format!("{} {} signal: Price ${:.2f}, Level ${:.2f}", 
                   symbol, signal, current_price, value1)
        };

        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(), // Will be set by alert manager
            title: format!("{} Technical Alert", symbol),
            message,
            notification_type: NotificationType::TechnicalAlert,
            priority: NotificationPriority::Medium,
            channels: vec![NotificationChannel::InApp, NotificationChannel::Push],
            metadata: serde_json::json!({
                "symbol": symbol,
                "signal": signal,
                "current_price": current_price,
                "value1": value1,
                "value2": value2,
                "alert_type": "technical_breakout"
            }),
            is_read: false,
            created_at: Utc::now(),
        };

        self.db.create_notification(&notification).await?;
        
        info!("Created technical notification for {}: {}", symbol, signal);
        Ok(())
    }

    pub async fn get_price_history(&self, symbol: &str, hours: u32) -> Result<Vec<PricePoint>> {
        let history = self.price_history.read().await;
        let symbol_history = match history.get(symbol) {
            Some(h) => h,
            None => return Ok(vec![]),
        };

        let cutoff_time = Utc::now() - Duration::hours(hours as i64);
        let recent_history: Vec<PricePoint> = symbol_history
            .iter()
            .filter(|p| p.timestamp >= cutoff_time)
            .cloned()
            .collect();

        Ok(recent_history)
    }

    pub async fn get_volume_history(&self, symbol: &str, hours: u32) -> Result<Vec<VolumePoint>> {
        let volume_history = self.volume_history.read().await;
        let symbol_volume = match volume_history.get(symbol) {
            Some(v) => v,
            None => return Ok(vec![]),
        };

        let cutoff_time = Utc::now() - Duration::hours(hours as i64);
        let recent_volume: Vec<VolumePoint> = symbol_volume
            .iter()
            .filter(|v| v.timestamp >= cutoff_time)
            .cloned()
            .collect();

        Ok(recent_volume)
    }
}
