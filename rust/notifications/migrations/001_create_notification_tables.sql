-- Create enum types
CREATE TYPE alert_type AS ENUM (
    'price_above',
    'price_below', 
    'percentage_gain',
    'percentage_loss',
    'volume_spike',
    'news_mention',
    'earnings_announcement',
    'technical_breakout'
);

CREATE TYPE notification_type AS ENUM (
    'price_alert',
    'news_alert',
    'technical_alert',
    'earnings_alert',
    'volume_alert',
    'system_alert'
);

CREATE TYPE notification_priority AS ENUM (
    'low',
    'medium',
    'high',
    'critical'
);

CREATE TYPE notification_channel AS ENUM (
    'in_app',
    'email',
    'push',
    'sms',
    'websocket'
);

-- Create stock_alerts table
CREATE TABLE stock_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    alert_type alert_type NOT NULL,
    price_target DECIMAL(10, 2),
    percentage_change DECIMAL(5, 2),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create news_alerts table
CREATE TABLE news_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    keywords TEXT[] NOT NULL DEFAULT '{}',
    sources TEXT[] NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create news_articles table
CREATE TABLE news_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    summary TEXT NOT NULL,
    url TEXT NOT NULL,
    source VARCHAR(100) NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL,
    symbols TEXT[] NOT NULL DEFAULT '{}',
    sentiment DECIMAL(3, 2), -- -1.00 to 1.00
    relevance_score DECIMAL(3, 2) NOT NULL DEFAULT 0.5, -- 0.00 to 1.00
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create notifications table
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    notification_type notification_type NOT NULL,
    priority notification_priority NOT NULL DEFAULT 'medium',
    channels notification_channel[] NOT NULL DEFAULT '{in_app}',
    metadata JSONB,
    is_read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create price_alert_triggers table
CREATE TABLE price_alert_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id UUID NOT NULL REFERENCES stock_alerts(id) ON DELETE CASCADE,
    symbol VARCHAR(10) NOT NULL,
    current_price DECIMAL(10, 2) NOT NULL,
    target_price DECIMAL(10, 2) NOT NULL,
    alert_type alert_type NOT NULL,
    triggered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create user_notification_preferences table
CREATE TABLE user_notification_preferences (
    user_id UUID PRIMARY KEY,
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    push_enabled BOOLEAN NOT NULL DEFAULT true,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    in_app_enabled BOOLEAN NOT NULL DEFAULT true,
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    timezone VARCHAR(50) NOT NULL DEFAULT 'UTC',
    daily_digest BOOLEAN NOT NULL DEFAULT false,
    weekly_summary BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_stock_alerts_user_id ON stock_alerts(user_id);
CREATE INDEX idx_stock_alerts_symbol ON stock_alerts(symbol);
CREATE INDEX idx_stock_alerts_active ON stock_alerts(is_active);

CREATE INDEX idx_news_alerts_user_id ON news_alerts(user_id);
CREATE INDEX idx_news_alerts_symbol ON news_alerts(symbol);
CREATE INDEX idx_news_alerts_active ON news_alerts(is_active);

CREATE INDEX idx_news_articles_symbols ON news_articles USING GIN(symbols);
CREATE INDEX idx_news_articles_published_at ON news_articles(published_at);
CREATE INDEX idx_news_articles_source ON news_articles(source);

CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_created_at ON notifications(created_at);
CREATE INDEX idx_notifications_is_read ON notifications(is_read);
CREATE INDEX idx_notifications_type ON notifications(notification_type);

CREATE INDEX idx_price_alert_triggers_alert_id ON price_alert_triggers(alert_id);
CREATE INDEX idx_price_alert_triggers_symbol ON price_alert_triggers(symbol);
CREATE INDEX idx_price_alert_triggers_triggered_at ON price_alert_triggers(triggered_at);

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers to automatically update updated_at
CREATE TRIGGER update_stock_alerts_updated_at 
    BEFORE UPDATE ON stock_alerts 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_notification_preferences_updated_at 
    BEFORE UPDATE ON user_notification_preferences 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
