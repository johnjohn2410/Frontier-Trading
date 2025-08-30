use std::sync::Arc;
use tokio::sync::RwLock;
use axum::{
    routing::{get, post, patch, delete},
    Router,
    extract::{Path, State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, error};

use crate::{
    AlertManager,
    NewsMonitor,
    PriceMonitor,
    models::*,
    database::Database,
};

#[derive(Clone)]
pub struct NotificationService {
    db: Arc<Database>,
    alert_manager: Arc<AlertManager>,
    news_monitor: Arc<NewsMonitor>,
    price_monitor: Arc<PriceMonitor>,
}

impl NotificationService {
    pub fn new(
        db: Arc<Database>,
        alert_manager: Arc<AlertManager>,
        news_monitor: Arc<NewsMonitor>,
        price_monitor: Arc<PriceMonitor>,
    ) -> Self {
        Self {
            db,
            alert_manager,
            news_monitor,
            price_monitor,
        }
    }

    pub fn router(self) -> Router {
        Router::new()
            // Alert routes
            .route("/api/alerts", get(self.clone().get_alerts))
            .route("/api/alerts/price", post(self.clone().create_price_alert))
            .route("/api/alerts/news", post(self.clone().create_news_alert))
            .route("/api/alerts/:id", patch(self.clone().update_alert))
            .route("/api/alerts/:id", delete(self.clone().delete_alert))
            .route("/api/alerts/user/:user_id", get(self.clone().get_user_alerts))
            
            // Notification routes
            .route("/api/notifications", get(self.clone().get_notifications))
            .route("/api/notifications/:id/read", patch(self.clone().mark_notification_read))
            .route("/api/notifications/read-all", patch(self.clone().mark_all_notifications_read))
            .route("/api/notifications/:id", delete(self.clone().delete_notification))
            
            // AI Copilot routes
            .route("/api/copilot/alert-insights", get(self.clone().get_alert_insights))
            .route("/api/copilot/setup-alerts", post(self.clone().setup_intelligent_alerts))
            
            // News routes
            .route("/api/news/:symbol", get(self.clone().get_news_for_symbol))
            
            // Health check
            .route("/health", get(|| async { "OK" }))
            .with_state(self)
    }

    async fn get_alerts(
        State(state): State<Self>,
    ) -> impl IntoResponse {
        // This would typically require authentication
        // For now, return a placeholder response
        Json(serde_json::json!({
            "priceAlerts": [],
            "newsAlerts": []
        }))
    }

    async fn create_price_alert(
        State(state): State<Self>,
        Json(payload): Json<CreatePriceAlertRequest>,
    ) -> impl IntoResponse {
        // TODO: Get user_id from authentication
        let user_id = Uuid::new_v4(); // Placeholder
        
        let alert_request = AlertRequest {
            symbol: payload.symbol,
            alert_type: payload.alert_type,
            price_target: payload.price_target,
            percentage_change: payload.percentage_change,
            notification_channels: payload.notification_channels,
        };

        match state.alert_manager.create_price_alert(user_id, alert_request).await {
            Ok(alert) => {
                info!("Created price alert: {:?}", alert);
                (StatusCode::CREATED, Json(NotificationResponse {
                    success: true,
                    message: "Price alert created successfully".to_string(),
                    notification_id: Some(alert.id),
                }))
            }
            Err(e) => {
                error!("Failed to create price alert: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                    success: false,
                    message: format!("Failed to create alert: {}", e),
                    notification_id: None,
                }))
            }
        }
    }

    async fn create_news_alert(
        State(state): State<Self>,
        Json(payload): Json<CreateNewsAlertRequest>,
    ) -> impl IntoResponse {
        // TODO: Get user_id from authentication
        let user_id = Uuid::new_v4(); // Placeholder
        
        let news_request = NewsAlertRequest {
            symbol: payload.symbol,
            keywords: payload.keywords,
            sources: payload.sources,
            notification_channels: payload.notification_channels,
        };

        match state.alert_manager.create_news_alert(user_id, news_request).await {
            Ok(alert) => {
                info!("Created news alert: {:?}", alert);
                (StatusCode::CREATED, Json(NotificationResponse {
                    success: true,
                    message: "News alert created successfully".to_string(),
                    notification_id: Some(alert.id),
                }))
            }
            Err(e) => {
                error!("Failed to create news alert: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                    success: false,
                    message: format!("Failed to create news alert: {}", e),
                    notification_id: None,
                }))
            }
        }
    }

    async fn update_alert(
        State(state): State<Self>,
        Path(alert_id): Path<Uuid>,
        Json(payload): Json<UpdateAlertRequest>,
    ) -> impl IntoResponse {
        match payload.is_active {
            Some(false) => {
                match state.alert_manager.deactivate_alert(alert_id).await {
                    Ok(_) => (StatusCode::OK, Json(NotificationResponse {
                        success: true,
                        message: "Alert deactivated successfully".to_string(),
                        notification_id: Some(alert_id),
                    })),
                    Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                        success: false,
                        message: format!("Failed to deactivate alert: {}", e),
                        notification_id: None,
                    }))
                }
            }
            _ => (StatusCode::BAD_REQUEST, Json(NotificationResponse {
                success: false,
                message: "Invalid update request".to_string(),
                notification_id: None,
            }))
        }
    }

    async fn delete_alert(
        State(state): State<Self>,
        Path(alert_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match state.alert_manager.deactivate_alert(alert_id).await {
            Ok(_) => (StatusCode::OK, Json(NotificationResponse {
                success: true,
                message: "Alert deleted successfully".to_string(),
                notification_id: Some(alert_id),
            })),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                success: false,
                message: format!("Failed to delete alert: {}", e),
                notification_id: None,
            }))
        }
    }

    async fn get_user_alerts(
        State(state): State<Self>,
        Path(user_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match state.alert_manager.get_user_alerts(user_id).await {
            Ok(alerts) => (StatusCode::OK, Json(alerts)),
            Err(e) => {
                error!("Failed to get user alerts: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<StockAlert>::new()))
            }
        }
    }

    async fn get_notifications(
        State(state): State<Self>,
    ) -> impl IntoResponse {
        // TODO: Get user_id from authentication
        let user_id = Uuid::new_v4(); // Placeholder
        
        match state.db.get_user_notifications(user_id, Some(50)).await {
            Ok(notifications) => (StatusCode::OK, Json(serde_json::json!({
                "notifications": notifications
            }))),
            Err(e) => {
                error!("Failed to get notifications: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "notifications": []
                })))
            }
        }
    }

    async fn mark_notification_read(
        State(state): State<Self>,
        Path(notification_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match state.db.mark_notification_read(notification_id).await {
            Ok(_) => (StatusCode::OK, Json(NotificationResponse {
                success: true,
                message: "Notification marked as read".to_string(),
                notification_id: Some(notification_id),
            })),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                success: false,
                message: format!("Failed to mark notification as read: {}", e),
                notification_id: None,
            }))
        }
    }

    async fn mark_all_notifications_read(
        State(state): State<Self>,
    ) -> impl IntoResponse {
        // TODO: Get user_id from authentication
        let user_id = Uuid::new_v4(); // Placeholder
        
        match state.db.mark_all_notifications_read(user_id).await {
            Ok(_) => (StatusCode::OK, Json(NotificationResponse {
                success: true,
                message: "All notifications marked as read".to_string(),
                notification_id: None,
            })),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                success: false,
                message: format!("Failed to mark notifications as read: {}", e),
                notification_id: None,
            }))
        }
    }

    async fn delete_notification(
        State(state): State<Self>,
        Path(notification_id): Path<Uuid>,
    ) -> impl IntoResponse {
        match state.db.delete_notification(notification_id).await {
            Ok(_) => (StatusCode::OK, Json(NotificationResponse {
                success: true,
                message: "Notification deleted successfully".to_string(),
                notification_id: Some(notification_id),
            })),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(NotificationResponse {
                success: false,
                message: format!("Failed to delete notification: {}", e),
                notification_id: None,
            }))
        }
    }

    async fn get_alert_insights(
        State(state): State<Self>,
    ) -> impl IntoResponse {
        // TODO: Get user_id and watchlist from authentication
        let user_id = Uuid::new_v4(); // Placeholder
        
        // Mock insights for now
        let insights = vec![
            serde_json::json!({
                "symbol": "AAPL",
                "alertType": "price_above",
                "confidence": 0.85,
                "reasoning": "AAPL is approaching resistance at $180.50 with strong volume",
                "suggestedAction": "Set price alert at $180.50 for potential breakout",
                "riskLevel": "medium"
            }),
            serde_json::json!({
                "symbol": "TSLA",
                "alertType": "volume_spike",
                "confidence": 0.92,
                "reasoning": "TSLA showing unusual volume patterns",
                "suggestedAction": "Monitor for potential price movement",
                "riskLevel": "high"
            })
        ];

        (StatusCode::OK, Json(serde_json::json!({
            "insights": insights
        })))
    }

    async fn setup_intelligent_alerts(
        State(state): State<Self>,
        Json(payload): Json<SetupAlertsRequest>,
    ) -> impl IntoResponse {
        // TODO: Implement AI-powered alert setup
        (StatusCode::OK, Json(NotificationResponse {
            success: true,
            message: "Intelligent alerts setup initiated".to_string(),
            notification_id: None,
        }))
    }

    async fn get_news_for_symbol(
        State(state): State<Self>,
        Path(symbol): Path<String>,
    ) -> impl IntoResponse {
        match state.news_monitor.get_recent_news(&symbol, 10).await {
            Ok(articles) => (StatusCode::OK, Json(articles)),
            Err(e) => {
                error!("Failed to get news for {}: {}", symbol, e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(Vec::<NewsArticle>::new()))
            }
        }
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting notification service monitoring");
        
        // Start news monitoring
        self.news_monitor.start_monitoring().await?;
        
        // Load active alerts
        self.alert_manager.load_active_alerts().await?;
        
        info!("Notification service monitoring started successfully");
        Ok(())
    }
}

// Request/Response types
#[derive(Deserialize)]
struct CreatePriceAlertRequest {
    symbol: String,
    alert_type: AlertType,
    price_target: Option<f64>,
    percentage_change: Option<f64>,
    notification_channels: Vec<NotificationChannel>,
}

#[derive(Deserialize)]
struct CreateNewsAlertRequest {
    symbol: String,
    keywords: Vec<String>,
    sources: Vec<String>,
    notification_channels: Vec<NotificationChannel>,
}

#[derive(Deserialize)]
struct UpdateAlertRequest {
    is_active: Option<bool>,
}

#[derive(Deserialize)]
struct SetupAlertsRequest {
    symbols: Vec<String>,
    user_interests: Vec<String>,
}
