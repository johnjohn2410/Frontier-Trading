use axum::{
    routing::{get, post, put, delete},
    http::{StatusCode, Method},
    Json, extract::{State, Path, Query},
    response::sse::{Event as SseEvent, Sse},
    response::{Response, IntoResponse},
};
use axum_extra::routing::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error, warn};
use tracing_subscriber;
use futures_util::stream::Stream;
use std::pin::Pin;
use std::convert::Infallible;

use shared::{
    Event, EventPayload, OrderRequestedEvent, MarketTickEvent, 
    CopilotSuggestionEvent, RiskCheckResult, RiskMonitor, EventBusManager
};

mod services;
mod handlers;
mod middleware;
mod config;

use services::{
    MarketDataService, TradingEngineService, NotificationService, 
    CopilotService, DatabaseService
};
use handlers::{
    place_order, get_orders, cancel_order,
    get_quote, get_quotes, get_history,
    get_account, get_risk_summary, update_risk_limits,
    get_alerts, create_alert, delete_alert, update_alert, get_insights,
    analyze_stock, get_suggestions, execute_suggestion
};
use config::GatewayConfig;

// ============================================================================
// API GATEWAY STATE
// ============================================================================

#[derive(Clone)]
pub struct GatewayState {
    pub config: GatewayConfig,
    pub market_data: Arc<MarketDataService>,
    pub trading_engine: Arc<TradingEngineService>,
    pub notifications: Arc<NotificationService>,
    pub copilot: Arc<CopilotService>,
    pub database: Arc<DatabaseService>,
    pub risk_monitor: Arc<RwLock<RiskMonitor>>,
    pub event_bus: Arc<EventBusManager>,
}

// ============================================================================
// MAIN APPLICATION
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = GatewayConfig::load()?;
    info!("API Gateway starting on port {}", config.port);
    
    // Initialize services
    let market_data = Arc::new(MarketDataService::new(&config).await?);
    let trading_engine = Arc::new(TradingEngineService::new(&config).await?);
    let notifications = Arc::new(NotificationService::new(&config).await?);
    let copilot = Arc::new(CopilotService::new(&config).await?);
    let database = Arc::new(DatabaseService::new(&config).await?);
    let risk_monitor = Arc::new(RwLock::new(RiskMonitor::new()));
    let event_bus = Arc::new(EventBusManager::new(&config.redis_url).await?);
    
    // Create shared state
    let state = GatewayState {
        config: config.clone(),
        market_data,
        trading_engine,
        notifications,
        copilot,
        database,
        risk_monitor,
        event_bus,
    };
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);
    
    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Market data routes
        .route("/api/market/quotes/:symbol", get(get_quote))
        .route("/api/market/quotes", get(get_quotes))
        .route("/api/market/history/:symbol/:period", get(get_history))
        .route("/api/market/stream", get(market_data_stream))
        
        // Trading routes
        .route("/api/trading/orders", post(place_order))
        .route("/api/trading/orders/:order_id", delete(cancel_order))
        .route("/api/trading/orders", get(get_orders))
        .route("/orders", post(place_order))  // Simple endpoint for frontend
        
        // Account routes
        .route("/api/account", get(get_account))
        .route("/api/account/risk", get(get_risk_summary))
        .route("/api/account/limits", put(update_risk_limits))
        
        // Alert routes
        .route("/api/alerts", get(get_alerts))
        .route("/api/alerts", post(create_alert))
        .route("/api/alerts/:alert_id", delete(delete_alert))
        .route("/api/alerts/:alert_id", put(update_alert))
        .route("/api/alerts/insights/:symbol", get(get_insights))
        
        // Copilot routes
        .route("/api/copilot/analyze/:symbol", post(analyze_stock))
        .route("/api/copilot/suggestions", get(get_suggestions))
        .route("/api/copilot/suggestions/:suggestion_id", post(execute_suggestion))
        .route("/api/copilot/stream", get(copilot_suggestions_stream))
        
        // WebSocket routes
        .route("/ws", get(websocket_handler))
        .route("/ws/stream", get(websocket_stream_handler))
        
        // Apply middleware
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);
    
    // Start server
    let addr = format!("0.0.0.0:{}", config.port).parse()?;
    info!("API Gateway listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "service": "frontier-trading-api-gateway"
    }))
}

// ============================================================================
// STREAMING ENDPOINTS
// ============================================================================

async fn market_data_stream(
    State(state): State<GatewayState>,
    Query(params): Query<serde_json::Value>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let symbols = params.get("symbols")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
        .unwrap_or_default();
    
    let stream = state.market_data.subscribe_quotes(&symbols).await;
    
    Sse::new(stream.map(|event| {
        Ok(SseEvent::default().json_data(event).unwrap())
    }))
}

async fn copilot_suggestions_stream(
    State(state): State<GatewayState>,
    Path(user_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = state.copilot.subscribe_suggestions(&user_id).await;
    
    Sse::new(stream.map(|suggestion| {
        Ok(Event::default().json_data(suggestion).unwrap())
    }))
}

// ============================================================================
// WEBSOCKET HANDLER
// ============================================================================

async fn websocket_handler(
    State(state): State<GatewayState>,
    upgrade: axum::extract::WebSocketUpgrade,
) -> Response {
    upgrade.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(socket: axum::extract::WebSocket, state: GatewayState) {
    use axum::extract::ws::{Message, WebSocket};
    use futures_util::{sink::SinkExt, stream::StreamExt};
    
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to relevant streams
    let mut market_stream = state.market_data.subscribe_quotes(&[]).await;
    let mut order_stream = state.trading_engine.subscribe_orders().await;
    let mut alert_stream = state.notifications.subscribe_alerts().await;
    
    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(event) = serde_json::from_str::<Event>(&text) {
                    // Route event to appropriate service
                    match event.payload {
                        EventPayload::OrderRequested(order) => {
                            // Send to trading engine
                            if let Err(e) = state.trading_engine.submit_order(order).await {
                                error!("Failed to submit order: {}", e);
                            }
                        }
                        EventPayload::CopilotSuggestion(suggestion) => {
                            // Send to copilot service
                            if let Err(e) = state.copilot.process_suggestion(suggestion).await {
                                error!("Failed to process suggestion: {}", e);
                            }
                        }
                        _ => {
                            warn!("Unhandled event type: {}", event.event_type);
                        }
                    }
                }
            }
        }
    });
    
    // Forward streams to WebSocket
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                market_event = market_stream.next() => {
                    if let Some(event) = market_event {
                        if let Err(e) = sender.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                            error!("Failed to send market event: {}", e);
                            break;
                        }
                    }
                }
                order_event = order_stream.next() => {
                    if let Some(event) = order_event {
                        if let Err(e) = sender.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                            error!("Failed to send order event: {}", e);
                            break;
                        }
                    }
                }
                alert_event = alert_stream.next() => {
                    if let Some(event) = alert_event {
                        if let Err(e) = sender.send(Message::Text(serde_json::to_string(&event).unwrap())).await {
                            error!("Failed to send alert event: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = recv_task => info!("WebSocket receive task completed"),
        _ = send_task => info!("WebSocket send task completed"),
    }
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub timestamp: String,
}

impl From<anyhow::Error> for ErrorResponse {
    fn from(err: anyhow::Error) -> Self {
        ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: err.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

pub async fn handle_error(err: anyhow::Error) -> (StatusCode, Json<ErrorResponse>) {
    error!("Request failed: {}", err);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::from(err))
    )
}

// ============================================================================
// WEB SOCKET STREAM HANDLER
// ============================================================================

async fn websocket_stream_handler(
    Query(params): Query<serde_json::Value>,
    State(state): State<GatewayState>,
) -> Response {
    let stream_name = params.get("s")
        .and_then(|v| v.as_str())
        .unwrap_or("suggestions.stream");
    
    info!("WebSocket stream request for: {}", stream_name);
    
    // For now, return a simple response indicating the stream
    // In a real implementation, this would set up a WebSocket connection
    // and stream events from the specified Redis stream
    
    let response = serde_json::json!({
        "stream": stream_name,
        "status": "connected",
        "message": "WebSocket stream handler - implement actual streaming"
    });
    
    Json(response).into_response()
}
