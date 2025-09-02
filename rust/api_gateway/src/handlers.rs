use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;
use tracing::{info, error, warn};
use chrono::{DateTime, Utc};

use crate::GatewayState;
use shared::types::{Money, Event, EventPayload, OrderRequestedEvent, OrderFilledEvent, PositionUpdatedEvent};

// ============================================================================
// ORDER REQUEST STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub qty: String,
    pub limit_price: Option<String>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub filled_qty: Option<String>,
    pub filled_price: Option<String>,
    pub correlation_id: String,
}

// ============================================================================
// ORDER HANDLERS
// ============================================================================

pub async fn place_order(
    State(state): State<GatewayState>,
    Json(req): Json<OrderRequest>,
) -> Json<OrderResponse> {
    let order_id = Uuid::new_v4().to_string();
    let correlation_id = req.correlation_id.unwrap_or_else(|| order_id.clone());
    
    info!("Placing order: {} {} {} {} @ {:?}", 
          req.side, req.qty, req.symbol, req.order_type, req.limit_price);
    
    // 1) Publish order.requested event
    let order_requested = Event::new(
        "order.requested".to_string(),
        EventPayload::OrderRequested(OrderRequestedEvent {
            order_id: order_id.clone(),
            user_id: "test-user".to_string(), // TODO: Get from auth
            symbol: req.symbol.clone(),
            side: req.side.clone(),
            order_type: req.order_type.clone(),
            quantity: req.qty.clone(),
            price: req.limit_price.clone(),
            time_in_force: "day".to_string(),
            source: "manual".to_string(),
        }),
        correlation_id.clone(),
    );
    
    if let Err(e) = state.event_bus.publish(&order_requested).await {
        error!("Failed to publish order.requested: {}", e);
    }
    
    // 2) Call C++ engine JSON-RPC
    let rpc_request = json!({
        "jsonrpc": "2.0",
        "id": order_id,
        "method": match (req.order_type.as_str(), req.limit_price.as_ref()) {
            ("limit", Some(_)) => "place_limit_order",
            _ => "place_market_order"
        },
        "params": {
            "symbol": req.symbol,
            "side": req.side,
            "quantity": req.qty,
            "price": req.limit_price.unwrap_or_else(|| "0".to_string())
        }
    });
    
    let engine_response = match state.trading_engine.place_order(&rpc_request).await {
        Ok(response) => response,
        Err(e) => {
            error!("Engine RPC failed: {}", e);
            return Json(OrderResponse {
                order_id,
                status: "rejected".to_string(),
                filled_qty: None,
                filled_price: None,
                correlation_id,
            });
        }
    };
    
    // 3) Parse engine response and publish events
    if let Ok(response_json) = serde_json::from_str::<Value>(&engine_response) {
        if let Some(result) = response_json.get("result") {
            // Engine successfully processed the order
            let status = result.get("status").and_then(|s| s.as_str()).unwrap_or("filled");
            
            if status == "filled" {
                // Publish order.filled event
                let order_filled = Event::new(
                    "order.filled".to_string(),
                    EventPayload::OrderFilled(OrderFilledEvent {
                        order_id: order_id.clone(),
                        fill_id: Uuid::new_v4().to_string(),
                        quantity: result.get("filled_qty").and_then(|q| q.as_str()).unwrap_or(&req.qty).to_string(),
                        price: result.get("filled_price").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
                        commission: "0".to_string(),
                        timestamp: chrono::Utc::now(),
                    }),
                    correlation_id.clone(),
                );
                
                if let Err(e) = state.event_bus.publish(&order_filled).await {
                    error!("Failed to publish order.filled: {}", e);
                }
                
                // Publish position.updated event
                let position_updated = Event::new(
                    "position.updated".to_string(),
                    EventPayload::PositionUpdated(PositionUpdatedEvent {
                        user_id: "test-user".to_string(),
                        symbol: req.symbol,
                        quantity: result.get("filled_qty").and_then(|q| q.as_str()).unwrap_or(&req.qty).to_string(),
                        average_price: result.get("filled_price").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
                        market_price: result.get("filled_price").and_then(|p| p.as_str()).unwrap_or("0").to_string(),
                        realized_pnl: "0".to_string(),
                        unrealized_pnl: "0".to_string(),
                        market_value: "0".to_string(),
                    }),
                    correlation_id.clone(),
                );
                
                if let Err(e) = state.event_bus.publish(&position_updated).await {
                    error!("Failed to publish position.updated: {}", e);
                }
            }
        }
    } else {
        // Handle JSON-RPC error responses
        if let Ok(error_json) = serde_json::from_str::<Value>(&engine_response) {
            if let Some(error) = error_json.get("error") {
                let error_message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                
                // Map engine errors to appropriate HTTP status codes
                if error_message.contains("INSUFFICIENT_BUYING_POWER") {
                    return Json(OrderResponse {
                        order_id,
                        status: "rejected".to_string(),
                        filled_qty: None,
                        filled_price: None,
                        correlation_id,
                    });
                } else if error_message.contains("RISK_LIMIT") {
                    return Json(OrderResponse {
                        order_id,
                        status: "rejected".to_string(),
                        filled_qty: None,
                        filled_price: None,
                        correlation_id,
                    });
                } else if error_message.contains("MARKET_CLOSED") {
                    return Json(OrderResponse {
                        order_id,
                        status: "rejected".to_string(),
                        filled_qty: None,
                        filled_price: None,
                        correlation_id,
                    });
                } else if error_message.contains("INVALID_SYMBOL") {
                    return Json(OrderResponse {
                        order_id,
                        status: "rejected".to_string(),
                        filled_qty: None,
                        filled_price: None,
                        correlation_id,
                    });
                }
            }
        }
    }
    
    Json(OrderResponse {
        order_id,
        status: "filled".to_string(),
        filled_qty: Some(req.qty),
        filled_price: req.limit_price,
        correlation_id,
    })
}

pub async fn get_orders(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement order history
    Json(json!({
        "orders": [],
        "count": 0
    }))
}

pub async fn cancel_order(
    State(_state): State<GatewayState>,
    Path(order_id): Path<String>,
) -> Json<Value> {
    // TODO: Implement order cancellation
    Json(json!({
        "order_id": order_id,
        "status": "canceled"
    }))
}

// ============================================================================
// MARKET DATA HANDLERS
// ============================================================================

pub async fn get_quote(
    State(_state): State<GatewayState>,
    Path(symbol): Path<String>,
) -> Json<Value> {
    // TODO: Implement market data quote
    Json(json!({
        "symbol": symbol,
        "price": "192.34",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn get_quotes(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement batch quotes
    Json(json!({
        "quotes": []
    }))
}

pub async fn get_history(
    State(_state): State<GatewayState>,
    Path((symbol, period)): Path<(String, String)>,
) -> Json<Value> {
    // TODO: Implement price history
    Json(json!({
        "symbol": symbol,
        "period": period,
        "prices": []
    }))
}

// ============================================================================
// ACCOUNT HANDLERS
// ============================================================================

pub async fn get_account(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement account info
    Json(json!({
        "user_id": "test-user",
        "cash": "100000",
        "equity": "100000",
        "buying_power": "100000"
    }))
}

pub async fn get_risk_summary(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement risk summary
    Json(json!({
        "max_position_size": "10000",
        "max_daily_loss": "1000",
        "current_risk": "low"
    }))
}

pub async fn update_risk_limits(
    State(_state): State<GatewayState>,
    Json(_limits): Json<Value>,
) -> Json<Value> {
    // TODO: Implement risk limits update
    Json(json!({
        "status": "updated"
    }))
}

// ============================================================================
// ALERT HANDLERS
// ============================================================================

pub async fn get_alerts(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement alerts
    Json(json!({
        "alerts": []
    }))
}

pub async fn create_alert(
    State(_state): State<GatewayState>,
    Json(_alert): Json<Value>,
) -> Json<Value> {
    // TODO: Implement alert creation
    Json(json!({
        "alert_id": Uuid::new_v4().to_string(),
        "status": "created"
    }))
}

pub async fn delete_alert(
    State(_state): State<GatewayState>,
    Path(alert_id): Path<String>,
) -> Json<Value> {
    // TODO: Implement alert deletion
    Json(json!({
        "alert_id": alert_id,
        "status": "deleted"
    }))
}

pub async fn update_alert(
    State(_state): State<GatewayState>,
    Path(alert_id): Path<String>,
    Json(_alert): Json<Value>,
) -> Json<Value> {
    // TODO: Implement alert update
    Json(json!({
        "alert_id": alert_id,
        "status": "updated"
    }))
}

pub async fn get_insights(
    State(_state): State<GatewayState>,
    Path(symbol): Path<String>,
) -> Json<Value> {
    // TODO: Implement insights
    Json(json!({
        "symbol": symbol,
        "insights": []
    }))
}

// ============================================================================
// COPILOT HANDLERS
// ============================================================================

pub async fn analyze_stock(
    State(_state): State<GatewayState>,
    Path(symbol): Path<String>,
) -> Json<Value> {
    // TODO: Implement stock analysis
    Json(json!({
        "symbol": symbol,
        "analysis": "Mock analysis",
        "confidence": 0.5
    }))
}

pub async fn get_suggestions(
    State(_state): State<GatewayState>,
) -> Json<Value> {
    // TODO: Implement suggestions
    Json(json!({
        "suggestions": []
    }))
}

pub async fn execute_suggestion(
    State(_state): State<GatewayState>,
    Path(suggestion_id): Path<String>,
) -> Json<Value> {
    // TODO: Implement suggestion execution
    Json(json!({
        "suggestion_id": suggestion_id,
        "status": "executed"
    }))
}
