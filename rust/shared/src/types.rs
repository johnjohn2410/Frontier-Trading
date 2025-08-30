use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AssetType {
    Stock,
    Etf,
    Crypto,
    Forex,
    Futures,
    Options,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Partial,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TimeInForce {
    Day,
    Gtc, // Good Till Cancelled
    Ioc, // Immediate or Cancel
    Fok, // Fill or Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub symbol: String,
    pub exchange: String,
    pub asset_type: AssetType,
    pub name: String,
    pub currency: String,
    pub tick_size: Decimal,
    pub lot_size: Decimal,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Price {
    pub value: Decimal,
    pub precision: u32,
}

impl Price {
    pub fn new(value: Decimal, precision: u32) -> Self {
        Self { value, precision }
    }
    
    pub fn from_f64(value: f64, precision: u32) -> Self {
        Self {
            value: Decimal::from_f64(value).unwrap_or_default(),
            precision,
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("{:.prec$}", self.value, prec = self.precision as usize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    pub value: Decimal,
    pub precision: u32,
}

impl Quantity {
    pub fn new(value: Decimal, precision: u32) -> Self {
        Self { value, precision }
    }
    
    pub fn from_f64(value: f64, precision: u32) -> Self {
        Self {
            value: Decimal::from_f64(value).unwrap_or_default(),
            precision,
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("{:.prec$}", self.value, prec = self.precision as usize)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub asset: Asset,
    pub bid: Price,
    pub ask: Price,
    pub last: Price,
    pub bid_size: Quantity,
    pub ask_size: Quantity,
    pub volume: Quantity,
    pub timestamp: DateTime<Utc>,
}

impl MarketTick {
    pub fn spread(&self) -> Price {
        Price::new(self.ask.value - self.bid.value, self.bid.precision)
    }
    
    pub fn mid(&self) -> Price {
        Price::new((self.bid.value + self.ask.value) / Decimal::from(2), self.bid.precision)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub client_order_id: String,
    pub user_id: Uuid,
    pub asset: Asset,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: Quantity,
    pub limit_price: Option<Price>,
    pub stop_price: Option<Price>,
    pub time_in_force: TimeInForce,
    pub status: OrderStatus,
    pub filled_quantity: Quantity,
    pub average_fill_price: Option<Price>,
    pub commission: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub asset: Asset,
    pub side: OrderSide,
    pub quantity: Quantity,
    pub price: Price,
    pub commission: Decimal,
    pub exchange: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub user_id: Uuid,
    pub asset: Asset,
    pub quantity: Quantity,
    pub average_price: Price,
    pub current_price: Price,
    pub unrealized_pnl: Decimal,
    pub realized_pnl: Decimal,
    pub market_value: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Position {
    pub fn calculate_unrealized_pnl(&mut self) {
        let current_value = self.quantity.value * self.current_price.value;
        let cost_basis = self.quantity.value * self.average_price.value;
        self.unrealized_pnl = current_value - cost_basis;
        self.market_value = current_value;
    }
    
    pub fn total_pnl(&self) -> Decimal {
        self.unrealized_pnl + self.realized_pnl
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub currency: String,
    pub cash: Decimal,
    pub buying_power: Decimal,
    pub equity: Decimal,
    pub margin_used: Decimal,
    pub margin_available: Decimal,
    pub is_paper_trading: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub id: Uuid,
    pub user_id: Uuid,
    pub max_position_size: Decimal,
    pub max_daily_loss: Decimal,
    pub max_drawdown: Decimal,
    pub max_leverage: Decimal,
    pub allow_short_selling: bool,
    pub allow_options: bool,
    pub allow_futures: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub total_pnl: Decimal,
    pub daily_pnl: Decimal,
    pub max_drawdown: Decimal,
    pub current_drawdown: Decimal,
    pub portfolio_value: Decimal,
    pub margin_used: Decimal,
    pub margin_available: Decimal,
    pub leverage: Decimal,
    pub beta: Decimal,
    pub sharpe_ratio: Decimal,
    pub volatility: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskViolation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub violation_type: ViolationType,
    pub message: String,
    pub current_value: Decimal,
    pub limit_value: Decimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViolationType {
    PositionSize,
    DailyLoss,
    Drawdown,
    Leverage,
    Concentration,
    Margin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    pub price: Price,
    pub quantity: Quantity,
    pub order_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub symbol: String,
    pub bids: Vec<OrderBookEntry>,
    pub asks: Vec<OrderBookEntry>,
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    pub fn best_bid(&self) -> Option<&OrderBookEntry> {
        self.bids.first()
    }
    
    pub fn best_ask(&self) -> Option<&OrderBookEntry> {
        self.asks.first()
    }
    
    pub fn spread(&self) -> Option<Price> {
        if let (Some(best_bid), Some(best_ask)) = (self.best_bid(), self.best_ask()) {
            Some(Price::new(
                best_ask.price.value - best_bid.price.value,
                best_bid.price.precision,
            ))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key: String,
    pub secret: String,
    pub permissions: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerAdapter {
    pub id: Uuid,
    pub name: String,
    pub adapter_type: AdapterType,
    pub config: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdapterType {
    PaperTrading,
    Alpaca,
    InteractiveBrokers,
    Binance,
    Coinbase,
    Custom,
}

// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketMessage {
    MarketTick(MarketTick),
    OrderUpdate(Order),
    TradeExecution(Trade),
    PositionUpdate(Position),
    AccountUpdate(Account),
    RiskViolation(RiskViolation),
    OrderBookUpdate(OrderBook),
    Error { message: String },
    Ping,
    Pong,
}

// API request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub order_type: OrderType,
    pub side: OrderSide,
    pub quantity: f64,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
    pub time_in_force: TimeInForce,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderResponse {
    pub order: Order,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrdersRequest {
    pub symbol: Option<String>,
    pub status: Option<OrderStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetOrdersResponse {
    pub orders: Vec<Order>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPositionsResponse {
    pub positions: Vec<Position>,
    pub total_value: Decimal,
    pub total_pnl: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAccountResponse {
    pub account: Account,
    pub risk_metrics: RiskMetrics,
}

// Pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}
