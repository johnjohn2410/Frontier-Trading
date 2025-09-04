use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, error, debug};

use crate::config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecFiling {
    pub title: String,
    pub summary: String,
    pub url: String,
    pub filed_at: DateTime<Utc>,
    pub company: String,
    pub filing_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsArticle {
    pub title: String,
    pub content: String,
    pub url: String,
    pub source: String,
    pub published_at: DateTime<Utc>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingHalt {
    pub symbol: String,
    pub reason: String,
    pub halt_time: DateTime<Utc>,
    pub exchange: String,
    pub url: String,
    pub halt_code: String,
}

pub struct SecEdgarIngestor {
    config: SecEdgarConfig,
    client: reqwest::Client,
}

impl SecEdgarIngestor {
    pub fn new(config: &SecEdgarConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MarketEventsBot/1.0 (compliance monitoring)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: config.clone(),
            client,
        }
    }

    pub async fn fetch_latest_filings(&self) -> Result<Vec<SecFiling>> {
        debug!("Fetching latest SEC filings");
        
        let mut filings = Vec::new();
        
        for ticker in &self.config.company_tickers {
            match self.fetch_company_filings(ticker).await {
                Ok(mut company_filings) => {
                    filings.append(&mut company_filings);
                }
                Err(e) => {
                    error!("Failed to fetch filings for {}: {}", ticker, e);
                }
            }
            
            // Rate limiting
            tokio::time::sleep(std::time::Duration::from_millis(
                1000 / self.config.rate_limit_per_second as u64
            )).await;
        }

        info!("Fetched {} SEC filings", filings.len());
        Ok(filings)
    }

    async fn fetch_company_filings(&self, ticker: &str) -> Result<Vec<SecFiling>> {
        // TODO: Implement actual SEC EDGAR API calls
        // For now, return mock data
        Ok(vec![
            SecFiling {
                title: format!("8-K Filing for {}", ticker),
                summary: format!("Material event reported for {}", ticker),
                url: format!("https://www.sec.gov/Archives/edgar/data/123456/{}/index.htm", ticker),
                filed_at: Utc::now(),
                company: ticker.to_string(),
                filing_type: "8-K".to_string(),
            }
        ])
    }
}

pub struct NewsFeedIngestor {
    config: NewsFeedsConfig,
    client: reqwest::Client,
}

impl NewsFeedIngestor {
    pub fn new(config: &NewsFeedsConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MarketEventsBot/1.0 (news monitoring)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: config.clone(),
            client,
        }
    }

    pub async fn fetch_feed(&self, feed_url: &str) -> Result<Vec<NewsArticle>> {
        debug!("Fetching news feed: {}", feed_url);
        
        let response = self.client.get(feed_url).send().await?;
        let content = response.text().await?;
        
        // Parse RSS feed
        let channel = rss::Channel::read_from(&content.as_bytes())?;
        
        let mut articles = Vec::new();
        for item in channel.items {
            if let Some(title) = item.title {
                if let Some(link) = item.link {
                    let content = item.description.unwrap_or_default();
                    let pub_date = item.pub_date
                        .and_then(|date| DateTime::parse_from_rfc2822(&date).ok())
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(Utc::now);

                    articles.push(NewsArticle {
                        title,
                        content,
                        url: link,
                        source: channel.title().to_string(),
                        published_at: pub_date,
                        author: item.author,
                    });
                }
            }
        }

        info!("Fetched {} articles from {}", articles.len(), feed_url);
        Ok(articles)
    }
}

pub struct HaltIngestor {
    config: HaltMonitoringConfig,
    client: reqwest::Client,
}

impl HaltIngestor {
    pub fn new(config: &HaltMonitoringConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("MarketEventsBot/1.0 (halt monitoring)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config: config.clone(),
            client,
        }
    }

    pub async fn fetch_halts(&self, halt_feed_url: &str) -> Result<Vec<TradingHalt>> {
        debug!("Fetching trading halts from: {}", halt_feed_url);
        
        // TODO: Implement actual halt feed parsing
        // For now, return mock data
        Ok(vec![
            TradingHalt {
                symbol: "TEST".to_string(),
                reason: "Volatility pause".to_string(),
                halt_time: Utc::now(),
                exchange: "NASDAQ".to_string(),
                url: halt_feed_url.to_string(),
                halt_code: "LUDP".to_string(),
            }
        ])
    }
}
