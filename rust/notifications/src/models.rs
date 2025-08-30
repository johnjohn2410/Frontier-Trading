use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use shared::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub alert_type: AlertType,
    pub price_target: Option<f64>,
    pub percentage_change: Option<f64>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    PriceAbove,
    PriceBelow,
    PercentageGain,
    PercentageLoss,
    VolumeSpike,
    NewsMention,
    EarningsAnnouncement,
    TechnicalBreakout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub keywords: Vec<String>,
    pub sources: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub priority: NotificationPriority,
    pub channels: Vec<NotificationChannel>,
    pub metadata: serde_json::Value,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    PriceAlert,
    NewsAlert,
    TechnicalAlert,
    EarningsAlert,
    VolumeAlert,
    SystemAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    InApp,
    Email,
    Push,
    SMS,
    WebSocket,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotificationPreferences {
    pub user_id: Uuid,
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub sms_enabled: bool,
    pub in_app_enabled: bool,
    pub quiet_hours_start: Option<String>, // HH:MM format
    pub quiet_hours_end: Option<String>,   // HH:MM format
    pub timezone: String,
    pub daily_digest: bool,
    pub weekly_summary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub url: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub symbols: Vec<String>,
    pub sentiment: Option<f64>, // -1.0 to 1.0
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlertTrigger {
    pub alert_id: Uuid,
    pub symbol: String,
    pub current_price: f64,
    pub target_price: f64,
    pub alert_type: AlertType,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRequest {
    pub symbol: String,
    pub alert_type: AlertType,
    pub price_target: Option<f64>,
    pub percentage_change: Option<f64>,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsAlertRequest {
    pub symbol: String,
    pub keywords: Vec<String>,
    pub sources: Vec<String>,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub success: bool,
    pub message: String,
    pub notification_id: Option<Uuid>,
}

// WebSocket message types for real-time notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "price_alert")]
    PriceAlert {
        symbol: String,
        current_price: f64,
        target_price: f64,
        alert_type: AlertType,
    },
    #[serde(rename = "news_alert")]
    NewsAlert {
        symbol: String,
        title: String,
        summary: String,
        url: String,
        source: String,
    },
    #[serde(rename = "technical_alert")]
    TechnicalAlert {
        symbol: String,
        indicator: String,
        value: f64,
        signal: String,
    },
    #[serde(rename = "volume_alert")]
    VolumeAlert {
        symbol: String,
        current_volume: u64,
        average_volume: u64,
        percentage_change: f64,
    },
}
