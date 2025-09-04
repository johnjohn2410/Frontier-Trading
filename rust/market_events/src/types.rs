use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketEvent {
    pub id: Uuid,
    pub incident_id: String, // For deduplication and threading
    pub title: String,
    pub description: String,
    pub category: EventCategory,
    pub severity: SeverityLevel,
    pub confidence: f64, // 0.0 to 1.0
    pub tickers: Vec<String>,
    pub cik_codes: Vec<String>,
    pub sources: Vec<EventSource>,
    pub status: EventStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub price_impact: Option<PriceImpact>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventCategory {
    CriticalIncident,    // Accidents, safety events, production outages
    RegulatoryFiling,    // SEC 8-K, material 6-Ks, guidance changes
    TradingStatus,       // Halts/resumptions, material short-sale restrictions
    EarningsSurprise,    // Material beats/misses, guidance revisions
    LegalRegulatory,     // Major lawsuits, FTC/DoJ/EU actions
    ProductRecall,       // Recalls, FDA actions, withdrawals
    Leadership,          // CEO/CFO departures/appointments
    CryptoIncident,      // Protocol exploits, exchange incidents, depegs
    MarketHalt,          // Trading halts and resumptions
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    A, // Loss of life/catastrophic accident, regulatory shutdown, massive breach
    B, // Major recall, guidance withdrawal, CEO resignation, significant litigation
    C, // Plant outage, product delay, localized incident, exec reshuffle
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSource {
    pub name: String,
    pub url: String,
    pub credibility_score: f64, // 0.0 to 1.0
    pub is_official: bool,      // SEC, exchange, government source
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    Developing,  // Initial report, needs confirmation
    Confirmed,   // Verified by multiple sources
    Update,      // Additional information
    Correction,  // Previous information was incorrect
    Resolved,    // Incident is closed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceImpact {
    pub ticker: String,
    pub price_change_pct: f64,
    pub volume_change_pct: f64,
    pub market_relative_change: f64, // vs S&P 500
    pub timeframe_minutes: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPost {
    pub id: Uuid,
    pub event_id: Uuid,
    pub incident_id: String,
    pub content: String,
    pub thread_position: i32, // 0 = initial post, 1+ = updates
    pub posted_at: Option<DateTime<Utc>>,
    pub platform: AlertPlatform,
    pub status: PostStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertPlatform {
    Twitter,
    Discord,
    Slack,
    Webhook,
    Email,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PostStatus {
    Pending,
    Posted,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetrics {
    pub total_events_processed: u64,
    pub events_by_category: HashMap<String, u64>,
    pub events_by_severity: HashMap<String, u64>,
    pub average_detection_latency_ms: f64,
    pub precision_rate: f64,
    pub recall_rate: f64,
    pub alerts_posted: u64,
    pub corrections_made: u64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMapping {
    pub ticker: String,
    pub company_name: String,
    pub cik_code: String,
    pub sector: String,
    pub keywords: Vec<String>,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityScore {
    pub base_score: f64,
    pub source_credibility: f64,
    pub incident_magnitude: f64,
    pub price_impact: f64,
    pub novelty_factor: f64,
    pub final_score: f64,
    pub level: SeverityLevel,
}

impl MarketEvent {
    pub fn new(
        title: String,
        description: String,
        category: EventCategory,
        sources: Vec<EventSource>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            incident_id: format!("incident_{}", now.timestamp()),
            title,
            description,
            category,
            severity: SeverityLevel::C, // Default, will be calculated
            confidence: 0.5, // Default, will be calculated
            tickers: Vec::new(),
            cik_codes: Vec::new(),
            sources,
            status: EventStatus::Developing,
            created_at: now,
            updated_at: now,
            price_impact: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_ticker(&mut self, ticker: String) {
        if !self.tickers.contains(&ticker) {
            self.tickers.push(ticker);
        }
    }

    pub fn add_source(&mut self, source: EventSource) {
        self.sources.push(source);
        self.updated_at = Utc::now();
    }

    pub fn update_status(&mut self, status: EventStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    pub fn set_severity(&mut self, severity: SeverityLevel, confidence: f64) {
        self.severity = severity;
        self.confidence = confidence;
        self.updated_at = Utc::now();
    }
}

impl AlertPost {
    pub fn new(event_id: Uuid, incident_id: String, content: String, platform: AlertPlatform) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_id,
            incident_id,
            content,
            thread_position: 0,
            posted_at: None,
            platform,
            status: PostStatus::Pending,
        }
    }

    pub fn mark_posted(&mut self) {
        self.status = PostStatus::Posted;
        self.posted_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self) {
        self.status = PostStatus::Failed;
    }
}
