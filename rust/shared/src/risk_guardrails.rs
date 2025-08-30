use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone, Local};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::types::{Money, Event, EventPayload, OrderRequestedEvent, PositionUpdatedEvent};

// ============================================================================
// RISK GUARDRAILS SYSTEM
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimits {
    pub user_id: String,
    pub max_position_size: Money,           // Max $ per position
    pub max_portfolio_concentration: Decimal, // Max % of portfolio per position
    pub max_daily_loss: Money,              // Daily loss limit
    pub max_drawdown: Decimal,              // Max drawdown percentage
    pub max_leverage: Decimal,              // Max leverage ratio
    pub allow_short_selling: bool,
    pub allow_options: bool,
    pub allow_futures: bool,
    pub pattern_day_trader_limit: i32,      // Max day trades per 5 days
    pub market_hours_only: bool,            // Only trade during market hours
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskViolation {
    pub id: String,
    pub user_id: String,
    pub violation_type: ViolationType,
    pub message: String,
    pub current_value: Money,
    pub limit_value: Money,
    pub severity: ViolationSeverity,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    PositionSize,
    PortfolioConcentration,
    DailyLoss,
    Drawdown,
    Leverage,
    PatternDayTrader,
    MarketHours,
    InsufficientFunds,
    MarginCall,
    CircuitBreaker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Critical,
    KillSwitch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub user_id: String,
    pub cash: Money,
    pub equity: Money,
    pub buying_power: Money,
    pub margin_used: Money,
    pub day_trade_count: i32,
    pub pattern_day_trader: bool,
    pub last_day_trade_date: Option<DateTime<Utc>>,
    pub daily_pnl: Money,
    pub total_pnl: Money,
    pub max_equity: Money,  // For drawdown calculation
    pub positions: HashMap<String, PositionState>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionState {
    pub symbol: String,
    pub quantity: Decimal,
    pub average_price: Money,
    pub market_value: Money,
    pub unrealized_pnl: Money,
    pub realized_pnl: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCheckResult {
    pub allowed: bool,
    pub violations: Vec<RiskViolation>,
    pub warnings: Vec<String>,
    pub risk_score: f64,  // 0.0 to 1.0
    pub margin_requirement: Money,
}

// ============================================================================
// RISK MANAGER
// ============================================================================

pub struct RiskManager {
    limits: HashMap<String, RiskLimits>,
    account_states: HashMap<String, AccountState>,
    violations: HashMap<String, Vec<RiskViolation>>,
    market_hours: MarketHours,
}

impl RiskManager {
    pub fn new() -> Self {
        Self {
            limits: HashMap::new(),
            account_states: HashMap::new(),
            violations: HashMap::new(),
            market_hours: MarketHours::new(),
        }
    }

    pub fn set_risk_limits(&mut self, limits: RiskLimits) {
        self.limits.insert(limits.user_id.clone(), limits);
        info!("Set risk limits for user {}", limits.user_id);
    }

    pub fn get_risk_limits(&self, user_id: &str) -> Option<&RiskLimits> {
        self.limits.get(user_id)
    }

    pub fn update_account_state(&mut self, state: AccountState) {
        self.account_states.insert(state.user_id.clone(), state);
    }

    pub fn get_account_state(&self, user_id: &str) -> Option<&AccountState> {
        self.account_states.get(user_id)
    }

    pub async fn check_order_risk(&self, order: &OrderRequestedEvent, 
                                account_state: &AccountState) -> RiskCheckResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let mut risk_score = 0.0;

        // Get user's risk limits
        let limits = match self.limits.get(&order.user_id) {
            Some(limits) => limits,
            None => {
                return RiskCheckResult {
                    allowed: false,
                    violations: vec![RiskViolation {
                        id: uuid::Uuid::new_v4().to_string(),
                        user_id: order.user_id.clone(),
                        violation_type: ViolationType::PositionSize,
                        message: "No risk limits configured".to_string(),
                        current_value: Money::zero("USD"),
                        limit_value: Money::zero("USD"),
                        severity: ViolationSeverity::Critical,
                        timestamp: Utc::now(),
                        resolved: false,
                    }],
                    warnings: vec!["No risk limits configured".to_string()],
                    risk_score: 1.0,
                    margin_requirement: Money::zero("USD"),
                };
            }
        };

        // Check market hours
        if limits.market_hours_only && !self.market_hours.is_market_open() {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::MarketHours,
                message: "Trading outside market hours".to_string(),
                current_value: Money::zero("USD"),
                limit_value: Money::zero("USD"),
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check buying power
        let order_cost = self.calculate_order_cost(order);
        if order_cost.amount > account_state.buying_power.amount {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::InsufficientFunds,
                message: "Insufficient buying power".to_string(),
                current_value: order_cost,
                limit_value: account_state.buying_power.clone(),
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check position size limits
        if order_cost.amount > limits.max_position_size.amount {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::PositionSize,
                message: "Position size exceeds limit".to_string(),
                current_value: order_cost,
                limit_value: limits.max_position_size.clone(),
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check portfolio concentration
        let concentration = (order_cost.amount / account_state.equity.amount) * Decimal::new(100, 0);
        if concentration > limits.max_portfolio_concentration {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::PortfolioConcentration,
                message: "Position concentration exceeds limit".to_string(),
                current_value: Money::new(concentration, "USD"),
                limit_value: Money::new(limits.max_portfolio_concentration, "USD"),
                severity: ViolationSeverity::Warning,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check daily loss limit
        if account_state.daily_pnl.amount < -limits.max_daily_loss.amount {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::DailyLoss,
                message: "Daily loss limit exceeded".to_string(),
                current_value: account_state.daily_pnl.clone(),
                limit_value: Money::new(-limits.max_daily_loss.amount, "USD"),
                severity: ViolationSeverity::KillSwitch,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check drawdown
        let drawdown = self.calculate_drawdown(account_state);
        if drawdown > limits.max_drawdown {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::Drawdown,
                message: "Maximum drawdown exceeded".to_string(),
                current_value: Money::new(drawdown, "USD"),
                limit_value: Money::new(limits.max_drawdown, "USD"),
                severity: ViolationSeverity::KillSwitch,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check pattern day trader
        if account_state.pattern_day_trader && account_state.day_trade_count >= limits.pattern_day_trader_limit {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: order.user_id.clone(),
                violation_type: ViolationType::PatternDayTrader,
                message: "Pattern day trader limit exceeded".to_string(),
                current_value: Money::new(Decimal::from(account_state.day_trade_count), "USD"),
                limit_value: Money::new(Decimal::from(limits.pattern_day_trader_limit), "USD"),
                severity: ViolationSeverity::Critical,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Calculate risk score
        risk_score = self.calculate_risk_score(&violations, account_state, limits);

        // Determine if order is allowed
        let allowed = violations.iter().all(|v| {
            matches!(v.severity, ViolationSeverity::Warning)
        });

        RiskCheckResult {
            allowed,
            violations,
            warnings,
            risk_score,
            margin_requirement: order_cost,
        }
    }

    pub async fn check_position_risk(&self, position: &PositionUpdatedEvent,
                                   account_state: &AccountState) -> RiskCheckResult {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        let limits = match self.limits.get(&position.user_id) {
            Some(limits) => limits,
            None => return RiskCheckResult {
                allowed: true,
                violations,
                warnings: vec!["No risk limits configured".to_string()],
                risk_score: 0.5,
                margin_requirement: Money::zero("USD"),
            },
        };

        // Check position size
        if position.market_value.amount > limits.max_position_size.amount {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: position.user_id.clone(),
                violation_type: ViolationType::PositionSize,
                message: "Position size exceeds limit".to_string(),
                current_value: position.market_value.clone(),
                limit_value: limits.max_position_size.clone(),
                severity: ViolationSeverity::Warning,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        // Check portfolio concentration
        let concentration = (position.market_value.amount / account_state.equity.amount) * Decimal::new(100, 0);
        if concentration > limits.max_portfolio_concentration {
            violations.push(RiskViolation {
                id: uuid::Uuid::new_v4().to_string(),
                user_id: position.user_id.clone(),
                violation_type: ViolationType::PortfolioConcentration,
                message: "Position concentration exceeds limit".to_string(),
                current_value: Money::new(concentration, "USD"),
                limit_value: Money::new(limits.max_portfolio_concentration, "USD"),
                severity: ViolationSeverity::Warning,
                timestamp: Utc::now(),
                resolved: false,
            });
        }

        let risk_score = self.calculate_risk_score(&violations, account_state, limits);

        RiskCheckResult {
            allowed: violations.is_empty(),
            violations,
            warnings,
            risk_score,
            margin_requirement: position.market_value.clone(),
        }
    }

    fn calculate_order_cost(&self, order: &OrderRequestedEvent) -> Money {
        let quantity = order.quantity;
        let price = order.price.as_ref().map(|p| p.amount).unwrap_or(Decimal::ZERO);
        Money::new(quantity * price, "USD")
    }

    fn calculate_drawdown(&self, account_state: &AccountState) -> Decimal {
        if account_state.max_equity.amount == Decimal::ZERO {
            return Decimal::ZERO;
        }
        let drawdown = (account_state.equity.amount - account_state.max_equity.amount) / account_state.max_equity.amount;
        -drawdown * Decimal::new(100, 0)  // Convert to percentage
    }

    fn calculate_risk_score(&self, violations: &[RiskViolation], 
                           account_state: &AccountState, 
                           limits: &RiskLimits) -> f64 {
        let mut score = 0.0;
        
        for violation in violations {
            match violation.severity {
                ViolationSeverity::Warning => score += 0.2,
                ViolationSeverity::Critical => score += 0.5,
                ViolationSeverity::KillSwitch => score += 1.0,
            }
        }

        // Add risk based on account state
        if account_state.equity.amount < account_state.max_equity.amount * Decimal::new(90, 0) / Decimal::new(100, 0) {
            score += 0.3; // 10% drawdown
        }

        if account_state.day_trade_count > limits.pattern_day_trader_limit / 2 {
            score += 0.2; // Approaching PDT limit
        }

        score.min(1.0)
    }

    pub fn get_violations(&self, user_id: &str) -> Vec<&RiskViolation> {
        self.violations.get(user_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn resolve_violation(&mut self, user_id: &str, violation_id: &str) {
        if let Some(violations) = self.violations.get_mut(user_id) {
            for violation in violations {
                if violation.id == violation_id {
                    violation.resolved = true;
                    break;
                }
            }
        }
    }
}

// ============================================================================
// MARKET HOURS
// ============================================================================

pub struct MarketHours {
    pub open_time: (u32, u32),  // (hour, minute)
    pub close_time: (u32, u32),
    pub timezone: String,
}

impl MarketHours {
    pub fn new() -> Self {
        Self {
            open_time: (9, 30),   // 9:30 AM ET
            close_time: (16, 0),  // 4:00 PM ET
            timezone: "America/New_York".to_string(),
        }
    }

    pub fn is_market_open(&self) -> bool {
        let now = Local::now();
        let open_time = now.date_naive().and_hms_opt(self.open_time.0, self.open_time.1, 0).unwrap();
        let close_time = now.date_naive().and_hms_opt(self.close_time.0, self.close_time.1, 0).unwrap();
        let current_time = now.time();

        current_time >= open_time.time() && current_time <= close_time.time()
    }

    pub fn is_weekend(&self) -> bool {
        let now = Local::now();
        let weekday = now.weekday();
        weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
    }

    pub fn is_holiday(&self) -> bool {
        // TODO: Implement holiday calendar
        false
    }
}

// ============================================================================
// CIRCUIT BREAKERS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    pub symbol: String,
    pub level: CircuitBreakerLevel,
    pub trigger_price: Money,
    pub current_price: Money,
    pub triggered_at: DateTime<Utc>,
    pub duration_minutes: u32,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CircuitBreakerLevel {
    Level1,  // 7% drop - 15 minute halt
    Level2,  // 13% drop - 15 minute halt
    Level3,  // 20% drop - Trading halted for the day
}

impl CircuitBreaker {
    pub fn new(symbol: String, trigger_price: Money, current_price: Money) -> Self {
        let level = Self::determine_level(trigger_price.amount, current_price.amount);
        let duration_minutes = match level {
            CircuitBreakerLevel::Level1 => 15,
            CircuitBreakerLevel::Level2 => 15,
            CircuitBreakerLevel::Level3 => 1440, // 24 hours
        };

        Self {
            symbol,
            level,
            trigger_price,
            current_price,
            triggered_at: Utc::now(),
            duration_minutes,
            active: true,
        }
    }

    fn determine_level(trigger_price: Decimal, current_price: Decimal) -> CircuitBreakerLevel {
        let drop_percentage = ((trigger_price - current_price) / trigger_price) * Decimal::new(100, 0);
        
        if drop_percentage >= Decimal::new(20, 0) {
            CircuitBreakerLevel::Level3
        } else if drop_percentage >= Decimal::new(13, 0) {
            CircuitBreakerLevel::Level2
        } else {
            CircuitBreakerLevel::Level1
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let expiry = self.triggered_at + chrono::Duration::minutes(self.duration_minutes as i64);
        now > expiry
    }

    pub fn should_halt_trading(&self) -> bool {
        self.active && !self.is_expired()
    }
}

// ============================================================================
// RISK MONITORING
// ============================================================================

pub struct RiskMonitor {
    risk_manager: RiskManager,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl RiskMonitor {
    pub fn new() -> Self {
        Self {
            risk_manager: RiskManager::new(),
            circuit_breakers: HashMap::new(),
        }
    }

    pub async fn monitor_order(&mut self, order: OrderRequestedEvent, 
                             account_state: AccountState) -> RiskCheckResult {
        // Check circuit breakers
        if let Some(breaker) = self.circuit_breakers.get(&order.symbol) {
            if breaker.should_halt_trading() {
                return RiskCheckResult {
                    allowed: false,
                    violations: vec![RiskViolation {
                        id: uuid::Uuid::new_v4().to_string(),
                        user_id: order.user_id.clone(),
                        violation_type: ViolationType::CircuitBreaker,
                        message: format!("Circuit breaker active for {}", order.symbol),
                        current_value: Money::zero("USD"),
                        limit_value: Money::zero("USD"),
                        severity: ViolationSeverity::Critical,
                        timestamp: Utc::now(),
                        resolved: false,
                    }],
                    warnings: vec![],
                    risk_score: 1.0,
                    margin_requirement: Money::zero("USD"),
                };
            }
        }

        // Check risk limits
        self.risk_manager.check_order_risk(&order, &account_state).await
    }

    pub fn update_circuit_breaker(&mut self, symbol: String, current_price: Money) {
        // Check if circuit breaker should be triggered
        if let Some(existing) = self.circuit_breakers.get(&symbol) {
            if existing.is_expired() {
                self.circuit_breakers.remove(&symbol);
            }
        }

        // TODO: Implement circuit breaker logic based on price movements
        // This would track previous prices and trigger breakers on significant drops
    }

    pub fn get_risk_summary(&self, user_id: &str) -> RiskSummary {
        let account_state = self.risk_manager.get_account_state(user_id);
        let limits = self.risk_manager.get_risk_limits(user_id);
        let violations = self.risk_manager.get_violations(user_id);

        RiskSummary {
            user_id: user_id.to_string(),
            risk_score: violations.iter().map(|v| {
                match v.severity {
                    ViolationSeverity::Warning => 0.2,
                    ViolationSeverity::Critical => 0.5,
                    ViolationSeverity::KillSwitch => 1.0,
                }
            }).sum::<f64>().min(1.0),
            active_violations: violations.len(),
            account_state: account_state.cloned(),
            limits: limits.cloned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummary {
    pub user_id: String,
    pub risk_score: f64,
    pub active_violations: usize,
    pub account_state: Option<AccountState>,
    pub limits: Option<RiskLimits>,
}
