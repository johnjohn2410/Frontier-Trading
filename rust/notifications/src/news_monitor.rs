use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use anyhow::Result;
use tracing::{info, warn, error};
use reqwest::Client;
use feed_rs::parser;
use html2text::from_read;

use crate::models::*;
use crate::database::Database;

pub struct NewsMonitor {
    db: Arc<Database>,
    http_client: Client,
    news_sources: Vec<NewsSource>,
    active_news_alerts: Arc<RwLock<HashMap<Uuid, NewsAlert>>>,
    last_fetch: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

#[derive(Debug, Clone)]
struct NewsSource {
    name: String,
    url: String,
    rss_feed: String,
    reliability_score: f64,
}

impl NewsMonitor {
    pub fn new(db: Arc<Database>) -> Self {
        let mut sources = vec![
            NewsSource {
                name: "Yahoo Finance".to_string(),
                url: "https://finance.yahoo.com".to_string(),
                rss_feed: "https://feeds.finance.yahoo.com/rss/2.0/headline".to_string(),
                reliability_score: 0.8,
            },
            NewsSource {
                name: "Reuters".to_string(),
                url: "https://www.reuters.com".to_string(),
                rss_feed: "https://feeds.reuters.com/reuters/businessNews".to_string(),
                reliability_score: 0.9,
            },
            NewsSource {
                name: "Bloomberg".to_string(),
                url: "https://www.bloomberg.com".to_string(),
                rss_feed: "https://feeds.bloomberg.com/markets/news.rss".to_string(),
                reliability_score: 0.9,
            },
            NewsSource {
                name: "MarketWatch".to_string(),
                url: "https://www.marketwatch.com".to_string(),
                rss_feed: "https://feeds.marketwatch.com/marketwatch/topstories/".to_string(),
                reliability_score: 0.7,
            },
        ];

        Self {
            db,
            http_client: Client::new(),
            news_sources: sources,
            active_news_alerts: Arc::new(RwLock::new(HashMap::new())),
            last_fetch: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_monitoring(&self) -> Result<()> {
        info!("Starting news monitoring service");
        
        // Load active news alerts
        self.load_active_news_alerts().await?;
        
        // Start periodic news fetching
        let monitor = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = monitor.fetch_and_process_news().await {
                    error!("Error fetching news: {}", e);
                }
                
                // Wait 15 minutes before next fetch
                tokio::time::sleep(tokio::time::Duration::from_secs(900)).await;
            }
        });

        Ok(())
    }

    async fn fetch_and_process_news(&self) -> Result<()> {
        for source in &self.news_sources {
            if let Err(e) = self.fetch_source_news(source).await {
                warn!("Failed to fetch news from {}: {}", source.name, e);
                continue;
            }
        }

        Ok(())
    }

    async fn fetch_source_news(&self, source: &NewsSource) -> Result<()> {
        let response = self.http_client.get(&source.rss_feed)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let content = response.bytes().await?;
        let feed = parser::parse(&content[..])?;

        for entry in feed.entries {
            if let Err(e) = self.process_news_entry(&entry, source).await {
                warn!("Error processing news entry: {}", e);
            }
        }

        Ok(())
    }

    async fn process_news_entry(&self, entry: &feed_rs::model::Entry, source: &NewsSource) -> Result<()> {
        let title = entry.title.as_ref().map(|t| t.content.clone()).unwrap_or_default();
        let content = entry.content.as_ref().map(|c| c.body.clone()).unwrap_or_default();
        let url = entry.links.first().map(|l| l.href.clone()).unwrap_or_default();
        let published_at = entry.published.unwrap_or_else(Utc::now);

        // Extract plain text from HTML content
        let plain_text = if !content.is_empty() {
            from_read(content.as_bytes(), 80)
        } else {
            title.clone()
        };

        // Extract mentioned symbols
        let symbols = self.extract_stock_symbols(&title, &plain_text);
        
        if symbols.is_empty() {
            return Ok(());
        }

        // Calculate sentiment and relevance
        let sentiment = self.analyze_sentiment(&title, &plain_text);
        let relevance_score = self.calculate_relevance_score(&title, &plain_text, source);

        // Create news article
        let article = NewsArticle {
            id: Uuid::new_v4(),
            title,
            content: plain_text.clone(),
            summary: self.generate_summary(&plain_text),
            url,
            source: source.name.clone(),
            published_at,
            symbols,
            sentiment,
            relevance_score,
        };

        // Store in database
        self.db.create_news_article(&article).await?;

        // Check if any users have alerts for these symbols
        self.check_news_alerts(&article).await?;

        Ok(())
    }

    fn extract_stock_symbols(&self, title: &str, content: &str) -> Vec<String> {
        let mut symbols = Vec::new();
        let text = format!("{} {}", title, content);
        
        // Simple regex pattern for stock symbols (1-5 uppercase letters)
        // In a real implementation, you'd use a more sophisticated approach
        let re = regex::Regex::new(r"\b[A-Z]{1,5}\b").unwrap();
        
        for cap in re.find_iter(&text) {
            let symbol = cap.as_str();
            // Filter out common words that aren't stock symbols
            if !self.is_common_word(symbol) {
                symbols.push(symbol.to_string());
            }
        }

        // Remove duplicates
        symbols.sort();
        symbols.dedup();
        symbols
    }

    fn is_common_word(&self, word: &str) -> bool {
        let common_words = [
            "THE", "AND", "FOR", "ARE", "BUT", "NOT", "YOU", "ALL", "CAN", "HER", "WAS", "ONE",
            "OUR", "OUT", "DAY", "GET", "HAS", "HIM", "HIS", "HOW", "MAN", "NEW", "NOW", "OLD",
            "SEE", "TWO", "WAY", "WHO", "BOY", "DID", "ITS", "LET", "PUT", "SAY", "SHE", "TOO", "USE"
        ];
        common_words.contains(&word)
    }

    fn analyze_sentiment(&self, title: &str, content: &str) -> Option<f64> {
        let text = format!("{} {}", title, content).to_lowercase();
        
        // Simple sentiment analysis based on keyword counting
        // In a real implementation, you'd use a proper NLP library
        let positive_words = [
            "up", "rise", "gain", "positive", "growth", "profit", "earnings", "beat", "surge",
            "rally", "bullish", "strong", "increase", "higher", "success", "win"
        ];
        
        let negative_words = [
            "down", "fall", "drop", "negative", "loss", "decline", "miss", "crash", "plunge",
            "bearish", "weak", "decrease", "lower", "fail", "lose", "risk"
        ];

        let positive_count = positive_words.iter()
            .map(|word| text.matches(word).count())
            .sum::<usize>();
            
        let negative_count = negative_words.iter()
            .map(|word| text.matches(word).count())
            .sum::<usize>();

        let total = positive_count + negative_count;
        if total == 0 {
            None
        } else {
            let sentiment = (positive_count as f64 - negative_count as f64) / total as f64;
            Some(sentiment.clamp(-1.0, 1.0))
        }
    }

    fn calculate_relevance_score(&self, title: &str, content: &str, source: &NewsSource) -> f64 {
        let mut score = source.reliability_score;
        
        // Boost score for financial keywords
        let financial_keywords = [
            "earnings", "revenue", "profit", "quarterly", "annual", "financial", "trading",
            "market", "stock", "shares", "dividend", "merger", "acquisition", "ipo"
        ];
        
        let text = format!("{} {}", title, content).to_lowercase();
        let keyword_matches = financial_keywords.iter()
            .map(|word| text.matches(word).count())
            .sum::<usize>();
            
        score += (keyword_matches as f64 * 0.1).min(0.3);
        
        score.clamp(0.0, 1.0)
    }

    fn generate_summary(&self, content: &str) -> String {
        // Simple summary generation - take first 200 characters
        let words: Vec<&str> = content.split_whitespace().collect();
        let summary_words: Vec<&str> = words.into_iter().take(30).collect();
        summary_words.join(" ")
    }

    async fn check_news_alerts(&self, article: &NewsArticle) -> Result<()> {
        let alerts = self.active_news_alerts.read().await;
        
        for alert in alerts.values() {
            if !alert.is_active {
                continue;
            }

            // Check if article mentions the alert symbol
            if !article.symbols.contains(&alert.symbol) {
                continue;
            }

            // Check if article matches keywords
            if !alert.keywords.is_empty() {
                let text = format!("{} {}", article.title, article.content).to_lowercase();
                let has_keyword = alert.keywords.iter()
                    .any(|keyword| text.contains(&keyword.to_lowercase()));
                
                if !has_keyword {
                    continue;
                }
            }

            // Check if article is from preferred sources
            if !alert.sources.is_empty() && !alert.sources.contains(&article.source) {
                continue;
            }

            // Create notification
            self.create_news_notification(alert, article).await?;
        }

        Ok(())
    }

    async fn create_news_notification(&self, alert: &NewsAlert, article: &NewsArticle) -> Result<()> {
        let notification = Notification {
            id: Uuid::new_v4(),
            user_id: alert.user_id,
            title: format!("News Alert: {}", article.title),
            message: format!("New article about {}: {}", alert.symbol, article.summary),
            notification_type: NotificationType::NewsAlert,
            priority: NotificationPriority::Medium,
            channels: vec![NotificationChannel::InApp, NotificationChannel::Push],
            metadata: serde_json::json!({
                "alert_id": alert.id,
                "symbol": alert.symbol,
                "article_id": article.id,
                "article_url": article.url,
                "source": article.source,
                "sentiment": article.sentiment,
                "relevance_score": article.relevance_score
            }),
            is_read: false,
            created_at: Utc::now(),
        };

        self.db.create_notification(&notification).await?;
        
        info!("Created news notification for user {} about {}", 
              alert.user_id, alert.symbol);

        Ok(())
    }

    async fn load_active_news_alerts(&self) -> Result<()> {
        let alerts = self.db.get_active_news_alerts().await?;
        let mut active_alerts = self.active_news_alerts.write().await;
        active_alerts.clear();
        
        for alert in alerts {
            active_alerts.insert(alert.id, alert);
        }
        
        info!("Loaded {} active news alerts", active_alerts.len());
        Ok(())
    }

    pub async fn get_recent_news(&self, symbol: &str, limit: usize) -> Result<Vec<NewsArticle>> {
        self.db.get_recent_news_for_symbol(symbol, limit).await
    }
}

impl Clone for NewsMonitor {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            http_client: self.http_client.clone(),
            news_sources: self.news_sources.clone(),
            active_news_alerts: self.active_news_alerts.clone(),
            last_fetch: self.last_fetch.clone(),
        }
    }
}
