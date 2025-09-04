use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use frontier_types::{MarketTick, OrderRequest, OrderResponse};
use frontier_bus::{connect, xreadgroup_block};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, error};
use uuid::Uuid;
use chrono::Utc;

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    timestamp: chrono::DateTime<Utc>,
    services: HashMap<String, String>,
}

async fn health() -> Json<HealthResponse> {
    let mut services = HashMap::new();
    services.insert("market_data".to_string(), "connected".to_string());
    services.insert("redis".to_string(), "connected".to_string());
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        services,
    })
}

async fn get_quote(Query(params): Query<HashMap<String, String>>) -> Result<Json<MarketTick>, StatusCode> {
    let symbol = params.get("symbol").unwrap_or(&"AAPL".to_string()).clone();
    
    // For now, return a mock tick - in production this would read from Redis
    let tick = MarketTick {
        schema_version: "1".to_string(),
        symbol: symbol.clone(),
        ts: Utc::now().timestamp_millis(),
        price: "238.50".to_string(), // Mock price
        source: "alpaca:iex".to_string(),
    };
    
    Ok(Json(tick))
}

async fn place_order(Json(order): Json<OrderRequest>) -> Result<Json<OrderResponse>, StatusCode> {
    let order_id = Uuid::new_v4().to_string();
    let correlation_id = Uuid::new_v4().to_string();
    
    // Mock order processing - in production this would call the C++ engine
    let response = OrderResponse {
        order_id: order_id.clone(),
        status: "filled".to_string(),
        filled_qty: order.qty.clone(),
        filled_price: "238.50".to_string(), // Mock price
        correlation_id,
    };
    
    info!("📋 Order placed: {} {} {} {} @ {}", 
        order.side, order.qty, order.symbol, order.order_type, response.filled_price);
    
    Ok(Json(response))
}

async fn get_positions() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Mock positions - in production this would read from database
    let positions = vec![
        serde_json::json!({
            "symbol": "AAPL",
            "qty": "10",
            "avg_price": "235.00",
            "unrealized_pnl": "35.00",
            "realized_pnl": "0.00"
        }),
        serde_json::json!({
            "symbol": "MSFT",
            "qty": "5",
            "avg_price": "400.00",
            "unrealized_pnl": "-25.00",
            "realized_pnl": "0.00"
        })
    ];
    
    Ok(Json(positions))
}

async fn get_suggestions() -> Result<Json<Vec<serde_json::Value>>, StatusCode> {
    // Mock suggestions - in production this would read from Redis streams
    let suggestions = vec![
        serde_json::json!({
            "symbol": "AAPL",
            "action": "buy",
            "confidence": 0.75,
            "description": "EMA band crossed up; consider a limit buy",
            "features": {
                "cmin": "191.90",
                "cmax": "191.70",
                "vol_z": 2.2
            }
        })
    ];
    
    Ok(Json(suggestions))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    
    info!("🚀 Starting API Gateway");
    
    // Build the router with CORS
    let app = Router::new()
        .route("/health", get(health))
        .route("/quote", get(get_quote))
        .route("/orders", post(place_order))
        .route("/positions", get(get_positions))
        .route("/suggestions", get(get_suggestions))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        );
    
    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await?;
    info!("🌐 API Gateway listening on http://localhost:8000");
    info!("📊 Health check: http://localhost:8000/health");
    info!("📈 Quote endpoint: http://localhost:8000/quote?symbol=AAPL");
    info!("📋 Orders endpoint: http://localhost:8000/orders");
    info!("💼 Positions endpoint: http://localhost:8000/positions");
    info!("🤖 Suggestions endpoint: http://localhost:8000/suggestions");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
