use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEventsConfig {
    pub database_url: String,
    pub redis_url: String,
    pub sec_edgar: SecEdgarConfig,
    pub news_feeds: NewsFeedsConfig,
    pub halt_monitoring: HaltMonitoringConfig,
    pub alert_posting: AlertPostingConfig,
    pub entity_mapping: EntityMappingConfig,
    pub severity_scoring: SeverityScoringConfig,
    pub compliance: ComplianceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecEdgarConfig {
    pub enabled: bool,
    pub base_url: String,
    pub rate_limit_per_second: u32,
    pub company_tickers: Vec<String>,
    pub filing_types: Vec<String>, // 8-K, 10-K, 10-Q, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFeedsConfig {
    pub enabled: bool,
    pub feeds: Vec<NewsFeed>,
    pub rate_limit_per_second: u32,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsFeed {
    pub name: String,
    pub url: String,
    pub credibility_score: f64,
    pub is_official: bool,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaltMonitoringConfig {
    pub enabled: bool,
    pub exchanges: Vec<ExchangeConfig>,
    pub rate_limit_per_second: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    pub name: String,
    pub halt_feed_url: String,
    pub halt_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPostingConfig {
    pub twitter: Option<TwitterConfig>,
    pub discord: Option<DiscordConfig>,
    pub slack: Option<SlackConfig>,
    pub webhooks: Vec<WebhookConfig>,
    pub email: Option<EmailConfig>,
    pub rate_limit_per_minute: u32,
    pub max_updates_per_incident: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_token_secret: String,
    pub bearer_token: String,
    pub handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub webhook_url: String,
    pub channel_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub name: String,
    pub url: String,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMappingConfig {
    pub ticker_database_url: String,
    pub cik_mapping_url: String,
    pub update_interval_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityScoringConfig {
    pub source_weights: HashMap<String, f64>,
    pub category_weights: HashMap<String, f64>,
    pub price_impact_threshold: f64,
    pub confidence_threshold: f64,
    pub two_source_required_for: Vec<String>, // Categories requiring two sources
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub two_source_rule_enabled: bool,
    pub manual_hold_enabled: bool,
    pub correction_sla_minutes: u32,
    pub audit_log_enabled: bool,
    pub disclaimer_text: String,
}

impl MarketEventsConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        
        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?;

        let market_events_config: MarketEventsConfig = config.try_deserialize()?;
        Ok(market_events_config)
    }

    pub fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://frontier:frontier@localhost:5432/frontier".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            sec_edgar: SecEdgarConfig {
                enabled: true,
                base_url: "https://www.sec.gov/Archives/edgar/data".to_string(),
                rate_limit_per_second: 10,
                company_tickers: vec![
                    "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
                    "AMZN".to_string(), "TSLA".to_string(), "META".to_string(),
                ],
                filing_types: vec!["8-K".to_string(), "10-K".to_string(), "10-Q".to_string()],
            },
            news_feeds: NewsFeedsConfig {
                enabled: true,
                feeds: vec![
                    NewsFeed {
                        name: "Reuters Business".to_string(),
                        url: "https://feeds.reuters.com/reuters/businessNews".to_string(),
                        credibility_score: 0.9,
                        is_official: false,
                        keywords: vec!["accident".to_string(), "recall".to_string(), "lawsuit".to_string()],
                    },
                    NewsFeed {
                        name: "Bloomberg".to_string(),
                        url: "https://feeds.bloomberg.com/markets/news.rss".to_string(),
                        credibility_score: 0.95,
                        is_official: false,
                        keywords: vec!["halt".to_string(), "earnings".to_string(), "guidance".to_string()],
                    },
                ],
                rate_limit_per_second: 5,
                keywords: vec![
                    "accident".to_string(), "recall".to_string(), "lawsuit".to_string(),
                    "halt".to_string(), "earnings".to_string(), "guidance".to_string(),
                    "breach".to_string(), "exploit".to_string(), "depeg".to_string(),
                ],
            },
            halt_monitoring: HaltMonitoringConfig {
                enabled: true,
                exchanges: vec![
                    ExchangeConfig {
                        name: "NASDAQ".to_string(),
                        halt_feed_url: "https://www.nasdaqtrader.com/rss.aspx?feed=tradehalts".to_string(),
                        halt_codes: vec!["LUDP".to_string(), "T1".to_string(), "T2".to_string()],
                    },
                    ExchangeConfig {
                        name: "NYSE".to_string(),
                        halt_feed_url: "https://www.nyse.com/publicdocs/nyse/markets/nyse/NYSE_Halts.xml".to_string(),
                        halt_codes: vec!["T1".to_string(), "T2".to_string(), "T3".to_string()],
                    },
                ],
                rate_limit_per_second: 2,
            },
            alert_posting: AlertPostingConfig {
                twitter: None, // Configure with API keys
                discord: None, // Configure with webhook
                slack: None,   // Configure with webhook
                webhooks: vec![],
                email: None,
                rate_limit_per_minute: 10,
                max_updates_per_incident: 5,
            },
            entity_mapping: EntityMappingConfig {
                ticker_database_url: "https://api.polygon.io/v3/reference/tickers".to_string(),
                cik_mapping_url: "https://www.sec.gov/Archives/edgar/cik-lookup-data.txt".to_string(),
                update_interval_hours: 24,
            },
            severity_scoring: SeverityScoringConfig {
                source_weights: {
                    let mut weights = HashMap::new();
                    weights.insert("SEC".to_string(), 1.0);
                    weights.insert("Exchange".to_string(), 0.95);
                    weights.insert("Reuters".to_string(), 0.9);
                    weights.insert("Bloomberg".to_string(), 0.95);
                    weights.insert("AP".to_string(), 0.85);
                    weights
                },
                category_weights: {
                    let mut weights = HashMap::new();
                    weights.insert("CriticalIncident".to_string(), 1.0);
                    weights.insert("RegulatoryFiling".to_string(), 0.9);
                    weights.insert("TradingStatus".to_string(), 0.8);
                    weights.insert("EarningsSurprise".to_string(), 0.7);
                    weights.insert("LegalRegulatory".to_string(), 0.8);
                    weights.insert("ProductRecall".to_string(), 0.6);
                    weights.insert("Leadership".to_string(), 0.5);
                    weights.insert("CryptoIncident".to_string(), 0.8);
                    weights
                },
                price_impact_threshold: 2.0, // 2% price movement
                confidence_threshold: 0.7,
                two_source_required_for: vec![
                    "CriticalIncident".to_string(),
                    "LegalRegulatory".to_string(),
                ],
            },
            compliance: ComplianceConfig {
                two_source_rule_enabled: true,
                manual_hold_enabled: true,
                correction_sla_minutes: 10,
                audit_log_enabled: true,
                disclaimer_text: "Not investment advice. Automated alerts on material market events.".to_string(),
            },
        }
    }
}
