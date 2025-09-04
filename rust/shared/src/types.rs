use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// MONEY MATH - Fixed-point decimal for all monetary values
// ============================================================================

/// Represents a monetary amount with fixed-point precision
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String,
}

impl Money {
    pub fn new(amount: Decimal, currency: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
        }
    }

    pub fn zero(currency: &str) -> Self {
        Self::new(Decimal::ZERO, currency)
    }

    pub fn from_cents(cents: i64, currency: &str) -> Self {
        Self::new(Decimal::new(cents, 2), currency)
    }

    pub fn to_cents(&self) -> i64 {
        (self.amount * Decimal::new(100, 0)).to_i64().unwrap_or(0)
    }
}

impl std::ops::Add for Money {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        assert_eq!(self.currency, other.currency, "Currency mismatch");
        Self::new(self.amount + other.amount, &self.currency)
    }
}

impl std::ops::Sub for Money {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        assert_eq!(self.currency, other.currency, "Currency mismatch");
        Self::new(self.amount - other.amount, &self.currency)
    }
}

impl std::ops::Mul<Decimal> for Money {
    type Output = Self;
    fn mul(self, multiplier: Decimal) -> Self {
        Self::new(self.amount * multiplier, &self.currency)
    }
}

// ============================================================================
// UNIFIED EVENT MODEL - Versioned events across all services
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub schema_version: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub payload: EventPayload,
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EventPayload {
    // Order Events
    OrderRequested(OrderRequestedEvent),
    OrderAccepted(OrderAcceptedEvent),
    OrderRejected(OrderRejectedEvent),
    OrderFilled(OrderFilledEvent),
    OrderCancelled(OrderCancelledEvent),
    
    // Position Events
    PositionUpdated(PositionUpdatedEvent),
    PositionClosed(PositionClosedEvent),
    
    // Account Events
    AccountUpdated(AccountUpdatedEvent),
    CashUpdated(CashUpdatedEvent),
    
    // Market Data Events
    MarketTick(MarketTickEvent),
    PriceAlert(PriceAlertEvent),
    
    // Alert Events
    AlertRaised(AlertRaisedEvent),
    AlertTriggered(AlertTriggeredEvent),
    
    // Copilot Events
    CopilotSuggestion(CopilotSuggestionEvent),
    CopilotAnalysis(CopilotAnalysisEvent),
    
    // News Events
    NewsArticle(NewsArticleEvent),
    SentimentUpdate(SentimentUpdateEvent),
}

// ============================================================================
// ORDER EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequestedEvent {
    pub order_id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: Decimal,
    pub price: Option<Money>,
    pub time_in_force: TimeInForce,
    pub source: String, // "manual", "copilot", "algo"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderAcceptedEvent {
    pub order_id: Uuid,
    pub broker_order_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRejectedEvent {
    pub order_id: Uuid,
    pub reason: String,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFilledEvent {
    pub order_id: Uuid,
    pub fill_id: Uuid,
    pub quantity: Decimal,
    pub price: Money,
    pub commission: Money,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCancelledEvent {
    pub order_id: Uuid,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// POSITION EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdatedEvent {
    pub user_id: String,
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Money,
    pub market_price: Money,
    pub realized_pnl: Money,
    pub unrealized_pnl: Money,
    pub market_value: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionClosedEvent {
    pub user_id: String,
    pub symbol: String,
    pub final_pnl: Money,
    pub close_reason: String,
}

// ============================================================================
// ACCOUNT EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdatedEvent {
    pub user_id: String,
    pub cash: Money,
    pub equity: Money,
    pub buying_power: Money,
    pub day_trade_count: i32,
    pub pattern_day_trader: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashUpdatedEvent {
    pub user_id: String,
    pub previous_cash: Money,
    pub new_cash: Money,
    pub change_reason: String,
}

// ============================================================================
// MARKET DATA EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTickEvent {
    pub symbol: String,
    pub price: Money,
    pub volume: u64,
    pub timestamp: DateTime<Utc>,
    pub bid: Option<Money>,
    pub ask: Option<Money>,
    pub high: Option<Money>,
    pub low: Option<Money>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlertEvent {
    pub alert_id: Uuid,
    pub symbol: String,
    pub trigger_price: Money,
    pub current_price: Money,
    pub alert_type: String,
}

// ============================================================================
// ALERT EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRaisedEvent {
    pub alert_id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub alert_type: String,
    pub message: String,
    pub priority: AlertPriority,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTriggeredEvent {
    pub alert_id: Uuid,
    pub trigger_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// COPILOT EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotSuggestionEvent {
    pub suggestion_id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub suggestion: String,
    pub action_type: CopilotActionType,
    pub confidence: f64, // 0.0 to 1.0
    pub risk_impact: RiskImpact,
    pub features: CopilotFeatures,
    pub what_if: WhatIfAnalysis,
    pub guardrails: GuardrailCheck,
    pub compliance: ComplianceInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotAnalysisEvent {
    pub analysis_id: Uuid,
    pub user_id: String,
    pub symbol: String,
    pub analysis_type: String,
    pub insights: Vec<String>,
    pub sentiment: f64, // -1.0 to 1.0
    pub technical_indicators: serde_json::Value,
    pub news_summary: Option<String>,
}

// ============================================================================
// NEWS EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticleEvent {
    pub article_id: Uuid,
    pub title: String,
    pub summary: String,
    pub url: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub symbols: Vec<String>,
    pub sentiment: f64, // -1.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentUpdateEvent {
    pub symbol: String,
    pub sentiment: f64,
    pub confidence: f64,
    pub sources: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeInForce {
    Day,
    GTC, // Good Till Cancelled
    IOC, // Immediate or Cancel
    FOK, // Fill or Kill
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CopilotActionType {
    Buy,
    Sell,
    Hold,
    Watch,
    Alert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskImpact {
    pub estimated_drawdown: Money,
    pub bp_usage: Decimal, // Percentage of buying power
    pub max_loss: Money,
    pub risk_reward_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotFeatures {
    pub volume_z_score: Option<f64>,
    pub rsi: Option<f64>,
    pub news_sentiment: Option<f64>,
    pub technical_signals: Vec<String>,
    pub fundamental_metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatIfAnalysis {
    pub quantity: Decimal,
    pub price: Money,
    pub estimated_cost: Money,
    pub estimated_fees: Money,
    pub potential_pnl: Money,
    pub risk_metrics: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailCheck {
    pub max_position_ok: bool,
    pub daily_loss_ok: bool,
    pub market_hours_ok: bool,
    pub pattern_day_trader_ok: bool,
    pub risk_limits_ok: bool,
    pub violations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceInfo {
    pub disclaimer: String,
    pub requires_confirmation: bool,
    pub not_financial_advice: bool,
    pub risk_disclosure: String,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

impl Event {
    pub fn new(
        event_type: &str,
        payload: EventPayload,
        source: &str,
        user_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            schema_version: "v1".to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
            payload,
            metadata: EventMetadata {
                correlation_id: None,
                user_id,
                session_id: None,
                trace_id: None,
            },
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.metadata.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.metadata.session_id = Some(session_id);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.metadata.trace_id = Some(trace_id);
        self
    }
}

// ============================================================================
// SERIALIZATION HELPERS
// ============================================================================

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.amount, self.currency)
    }
}

impl std::str::FromStr for Money {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid money format".parse::<Decimal>().unwrap_err());
        }
        let amount = parts[0].parse::<Decimal>()?;
        let currency = parts[1].to_string();
        Ok(Money::new(amount, &currency))
    }
}
