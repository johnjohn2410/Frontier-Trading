use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Money {
    pub amount: Decimal,
    pub currency: String, // String => not Copy; that's fine, derive Clone only
}

#[derive(thiserror::Error, Debug)]
pub enum MoneyError {
    #[error("invalid decimal: {0}")]
    InvalidDecimal(String),
}

impl Money {
    pub fn parse<S: AsRef<str>>(s: S, ccy: impl Into<String>) -> Result<Self, MoneyError> {
        let d = Decimal::from_str(s.as_ref())
            .map_err(|e| MoneyError::InvalidDecimal(e.to_string()))?; // don't match variants
        Ok(Self { amount: d, currency: ccy.into() })
    }
    
    pub fn new(amount: Decimal, currency: impl Into<String>) -> Self {
        Self {
            amount,
            currency: currency.into(),
        }
    }
    
    pub fn zero(currency: impl Into<String>) -> Self {
        Self::new(Decimal::ZERO, currency)
    }
}

// Minimal event id/types used across services
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTick {
    pub schema_version: String, // "1"
    pub symbol: String,
    pub ts: i64,           // unix ms
    pub price: String,     // decimal string
    pub source: String,    // e.g. "alpaca:iex"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub side: String,      // "buy" or "sell"
    pub qty: String,       // decimal string
    pub order_type: String, // "market" or "limit"
    pub limit_price: Option<String>, // decimal string
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: String,
    pub status: String,
    pub filled_qty: String,
    pub filled_price: String,
    pub correlation_id: String,
}
