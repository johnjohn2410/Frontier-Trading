use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, warn, error};

use crate::models::*;
use crate::database::Database;

pub struct AlertManager {
    db: Arc<Database>,
    active_alerts: Arc<RwLock<HashMap<Uuid, StockAlert>>>,
    price_cache: Arc<RwLock<HashMap<String, f64>>>,
}

impl AlertManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            price_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_price_alert(&self, user_id: Uuid, request: AlertRequest) -> Result<StockAlert> {
        let alert = StockAlert {
            id: Uuid::new_v4(),
            user_id,
            symbol: request.symbol.to_uppercase(),
            alert_type: request.alert_type,
            price_target: request.price_target,
            percentage_change: request.percentage_change,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Store in database
        self.db.create_stock_alert(&alert).await?;
        
        // Add to active alerts
        self.active_alerts.write().await.insert(alert.id, alert.clone());
        
        info!("Created price alert for user {} on {}", user_id, request.symbol);
        Ok(alert)
    }

    pub async fn create_news_alert(&self, user_id: Uuid, request: NewsAlertRequest) -> Result<NewsAlert> {
        let alert = NewsAlert {
            id: Uuid::new_v4(),
            user_id,
            symbol: request.symbol.to_uppercase(),
            keywords: request.keywords,
            sources: request.sources,
            is_active: true,
            created_at: Utc::now(),
        };

        self.db.create_news_alert(&alert).await?;
        
        info!("Created news alert for user {} on {}", user_id, request.symbol);
        Ok(alert)
    }

    pub async fn update_price(&self, symbol: String, price: f64) -> Result<()> {
        // Update price cache
        self.price_cache.write().await.insert(symbol.clone(), price);
        
        // Check all active alerts for this symbol
        let alerts = self.active_alerts.read().await;
        let symbol_alerts: Vec<_> = alerts
            .values()
            .filter(|alert| alert.symbol == symbol && alert.is_active)
            .cloned()
            .collect();
        
        drop(alerts); // Release read lock

        for alert in symbol_alerts {
            if let Err(e) = self.check_alert(&alert, price).await {
                error!("Error checking alert {}: {}", alert.id, e);
            }
        }

        Ok(())
    }

    async fn check_alert(&self, alert: &StockAlert, current_price: f64) -> Result<()> {
        let should_trigger = match alert.alert_type {
            AlertType::PriceAbove => {
                if let Some(target) = alert.price_target {
                    current_price >= target
                } else {
                    false
                }
            }
            AlertType::PriceBelow => {
                if let Some(target) = alert.price_target {
                    current_price <= target
                } else {
                    false
                }
            }
            AlertType::PercentageGain => {
                if let Some(percentage) = alert.percentage_change {
                    // Calculate percentage change from a baseline price
                    // For now, we'll use the alert creation price as baseline
                    // In a real implementation, you'd track the price when alert was created
                    false // TODO: Implement percentage calculation
                } else {
                    false
                }
            }
            AlertType::PercentageLoss => {
                if let Some(percentage) = alert.percentage_change {
                    // Similar to PercentageGain
                    false // TODO: Implement percentage calculation
                } else {
                    false
                }
            }
            AlertType::VolumeSpike => {
                // Volume alerts are handled separately by volume monitor
                false
            }
            AlertType::NewsMention => {
                // News alerts are handled by news monitor
                false
            }
            AlertType::EarningsAnnouncement => {
                // Earnings alerts are handled by earnings monitor
                false
            }
            AlertType::TechnicalBreakout => {
                // Technical alerts are handled by technical analysis
                false
            }
        };

        if should_trigger {
            self.trigger_alert(alert, current_price).await?;
        }

        Ok(())
    }

    async fn trigger_alert(&self, alert: &StockAlert, current_price: f64) -> Result<()> {
        let trigger = PriceAlertTrigger {
            alert_id: alert.id,
            symbol: alert.symbol.clone(),
            current_price,
            target_price: alert.price_target.unwrap_or(0.0),
            alert_type: alert.alert_type.clone(),
            triggered_at: Utc::now(),
        };

        // Store trigger in database
        self.db.create_price_alert_trigger(&trigger).await?;

        // Create notification
        let notification = self.create_price_notification(alert, &trigger).await?;
        self.db.create_notification(&notification).await?;

        // Send notification through channels
        self.send_notification(&notification).await?;

        // Deactivate alert if it's a one-time alert
        if matches!(alert.alert_type, AlertType::PriceAbove | AlertType::PriceBelow) {
            self.deactivate_alert(alert.id).await?;
        }

        info!("Triggered price alert {} for {} at ${:.2f}", 
              alert.id, alert.symbol, current_price);

        Ok(())
    }

    async fn create_price_notification(&self, alert: &StockAlert, trigger: &PriceAlertTrigger) -> Result<Notification> {
        let (title, message) = match alert.alert_type {
            AlertType::PriceAbove => (
                format!("{} Price Alert: Above Target", alert.symbol),
                format!("{} has reached ${:.2f}, above your target of ${:.2f}", 
                       alert.symbol, trigger.current_price, trigger.target_price)
            ),
            AlertType::PriceBelow => (
                format!("{} Price Alert: Below Target", alert.symbol),
                format!("{} has dropped to ${:.2f}, below your target of ${:.2f}", 
                       alert.symbol, trigger.current_price, trigger.target_price)
            ),
            _ => (
                format!("{} Alert Triggered", alert.symbol),
                format!("Your alert for {} has been triggered at ${:.2f}", 
                       alert.symbol, trigger.current_price)
            ),
        };

        Ok(Notification {
            id: Uuid::new_v4(),
            user_id: alert.user_id,
            title,
            message,
            notification_type: NotificationType::PriceAlert,
            priority: NotificationPriority::Medium,
            channels: vec![NotificationChannel::InApp, NotificationChannel::Push],
            metadata: serde_json::json!({
                "alert_id": alert.id,
                "symbol": alert.symbol,
                "current_price": trigger.current_price,
                "target_price": trigger.target_price,
                "alert_type": alert.alert_type
            }),
            is_read: false,
            created_at: Utc::now(),
        })
    }

    async fn send_notification(&self, notification: &Notification) -> Result<()> {
        // TODO: Implement actual notification sending
        // This would integrate with email, push, SMS services
        
        info!("Sending notification {} to user {}", notification.id, notification.user_id);
        Ok(())
    }

    pub async fn deactivate_alert(&self, alert_id: Uuid) -> Result<()> {
        // Update database
        self.db.deactivate_stock_alert(alert_id).await?;
        
        // Remove from active alerts
        self.active_alerts.write().await.remove(&alert_id);
        
        info!("Deactivated alert {}", alert_id);
        Ok(())
    }

    pub async fn get_user_alerts(&self, user_id: Uuid) -> Result<Vec<StockAlert>> {
        self.db.get_user_stock_alerts(user_id).await
    }

    pub async fn load_active_alerts(&self) -> Result<()> {
        let alerts = self.db.get_active_stock_alerts().await?;
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.clear();
        
        for alert in alerts {
            active_alerts.insert(alert.id, alert);
        }
        
        info!("Loaded {} active alerts", active_alerts.len());
        Ok(())
    }
}
