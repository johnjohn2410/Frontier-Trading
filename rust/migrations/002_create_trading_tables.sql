-- Create trading tables for the end-to-end flow
-- This migration creates the essential tables for positions, orders, and suggestions

-- Positions table
CREATE TABLE positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    symbol TEXT NOT NULL,
    qty NUMERIC(20,8) NOT NULL,
    avg_price NUMERIC(20,8) NOT NULL,
    unrealized NUMERIC(20,8) NOT NULL DEFAULT 0,
    realized NUMERIC(20,8) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    UNIQUE(user_id, symbol),
    INDEX idx_positions_user_id (user_id),
    INDEX idx_positions_symbol (symbol)
);

-- Orders table
CREATE TABLE orders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('buy','sell')),
    type TEXT NOT NULL CHECK (type IN ('market','limit')),
    qty NUMERIC(20,8) NOT NULL,
    limit_price NUMERIC(20,8),
    status TEXT NOT NULL CHECK (status IN ('requested','accepted','filled','rejected','canceled')),
    filled_qty NUMERIC(20,8) DEFAULT 0,
    filled_avg_price NUMERIC(20,8),
    correlation_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    INDEX idx_orders_user_id (user_id),
    INDEX idx_orders_symbol (symbol),
    INDEX idx_orders_status (status),
    INDEX idx_orders_correlation_id (correlation_id)
);

-- Suggestions table
CREATE TABLE suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    symbol TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','accepted','dismissed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    INDEX idx_suggestions_user_id (user_id),
    INDEX idx_suggestions_symbol (symbol),
    INDEX idx_suggestions_status (status),
    INDEX idx_suggestions_created_at (created_at)
);

-- Accounts table for user account state
CREATE TABLE accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE,
    cash NUMERIC(20,8) NOT NULL DEFAULT 0,
    equity NUMERIC(20,8) NOT NULL DEFAULT 0,
    buying_power NUMERIC(20,8) NOT NULL DEFAULT 0,
    day_trade_count INTEGER NOT NULL DEFAULT 0,
    pattern_day_trader BOOLEAN NOT NULL DEFAULT false,
    max_equity NUMERIC(20,8) NOT NULL DEFAULT 0,
    daily_pnl NUMERIC(20,8) NOT NULL DEFAULT 0,
    total_pnl NUMERIC(20,8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    INDEX idx_accounts_user_id (user_id)
);

-- Risk limits table
CREATE TABLE risk_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE,
    max_position_size NUMERIC(20,8) NOT NULL DEFAULT 10000,
    max_portfolio_concentration NUMERIC(5,2) NOT NULL DEFAULT 5.0,
    max_daily_loss NUMERIC(20,8) NOT NULL DEFAULT 1000,
    max_drawdown NUMERIC(5,2) NOT NULL DEFAULT 10.0,
    max_leverage NUMERIC(5,2) NOT NULL DEFAULT 1.0,
    allow_short_selling BOOLEAN NOT NULL DEFAULT false,
    allow_options BOOLEAN NOT NULL DEFAULT false,
    allow_futures BOOLEAN NOT NULL DEFAULT false,
    pattern_day_trader_limit INTEGER NOT NULL DEFAULT 3,
    market_hours_only BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    INDEX idx_risk_limits_user_id (user_id)
);

-- Market data cache table for storing latest prices
CREATE TABLE market_data_cache (
    symbol TEXT PRIMARY KEY,
    price NUMERIC(20,8) NOT NULL,
    volume NUMERIC(20,8),
    bid NUMERIC(20,8),
    ask NUMERIC(20,8),
    high NUMERIC(20,8),
    low NUMERIC(20,8),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT now(),
    
    -- Indexes for performance
    INDEX idx_market_data_cache_last_updated (last_updated)
);

-- Create triggers to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_positions_updated_at 
    BEFORE UPDATE ON positions 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_orders_updated_at 
    BEFORE UPDATE ON orders 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_accounts_updated_at 
    BEFORE UPDATE ON accounts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_risk_limits_updated_at 
    BEFORE UPDATE ON risk_limits 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default risk limits for new users
CREATE OR REPLACE FUNCTION create_default_risk_limits()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO risk_limits (user_id) VALUES (NEW.user_id);
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER create_default_risk_limits_trigger
    AFTER INSERT ON accounts
    FOR EACH ROW EXECUTE FUNCTION create_default_risk_limits();

-- Insert some sample data for testing
INSERT INTO accounts (user_id, cash, equity, buying_power, max_equity) VALUES 
    ('550e8400-e29b-41d4-a716-446655440000', 100000, 100000, 100000, 100000);

INSERT INTO risk_limits (user_id) VALUES 
    ('550e8400-e29b-41d4-a716-446655440000');

INSERT INTO market_data_cache (symbol, price, volume) VALUES 
    ('AAPL', 192.34, 50000000),
    ('SPY', 485.67, 80000000),
    ('BTCUSD', 43250.0, 1000000),
    ('GOOGL', 142.89, 20000000),
    ('TSLA', 245.12, 30000000),
    ('MSFT', 378.45, 25000000);
