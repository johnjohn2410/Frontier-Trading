use anyhow::Result;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, error, warn, debug};
use uuid::Uuid;

use crate::config::MarketEventsConfig;
use crate::types::*;
use crate::ingestors::*;
use crate::entity_linker::EntityLinker;
use crate::severity_scorer::SeverityScorer;
use crate::alert_poster::AlertPoster;

pub struct EventProcessor {
    config: MarketEventsConfig,
    entity_linker: EntityLinker,
    severity_scorer: SeverityScorer,
    alert_poster: AlertPoster,
    metrics: EventMetrics,
}

impl EventProcessor {
    pub async fn new(config: MarketEventsConfig) -> Result<Self> {
        let entity_linker = EntityLinker::new(&config.entity_mapping).await?;
        let severity_scorer = SeverityScorer::new(&config.severity_scoring);
        let alert_poster = AlertPoster::new(&config.alert_posting).await?;
        
        let metrics = EventMetrics {
            total_events_processed: 0,
            events_by_category: std::collections::HashMap::new(),
            events_by_severity: std::collections::HashMap::new(),
            average_detection_latency_ms: 0.0,
            precision_rate: 0.0,
            recall_rate: 0.0,
            alerts_posted: 0,
            corrections_made: 0,
            last_updated: chrono::Utc::now(),
        };

        Ok(Self {
            config,
            entity_linker,
            severity_scorer,
            alert_poster,
            metrics,
        })
    }

    pub async fn start_sec_edgar_monitoring(&self) -> Result<()> {
        if !self.config.sec_edgar.enabled {
            info!("SEC EDGAR monitoring disabled");
            return Ok(());
        }

        info!("Starting SEC EDGAR monitoring");
        let edgar_ingestor = SecEdgarIngestor::new(&self.config.sec_edgar);

        loop {
            match edgar_ingestor.fetch_latest_filings().await {
                Ok(filings) => {
                    for filing in filings {
                        if let Err(e) = self.process_filing(filing).await {
                            error!("Failed to process filing: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("SEC EDGAR fetch failed: {}", e);
                }
            }

            sleep(Duration::from_secs(60)).await; // Check every minute
        }
    }

    pub async fn start_news_feed_monitoring(&self) -> Result<()> {
        if !self.config.news_feeds.enabled {
            info!("News feed monitoring disabled");
            return Ok(());
        }

        info!("Starting news feed monitoring");
        let news_ingestor = NewsFeedIngestor::new(&self.config.news_feeds);

        loop {
            for feed in &self.config.news_feeds.feeds {
                match news_ingestor.fetch_feed(&feed.url).await {
                    Ok(articles) => {
                        for article in articles {
                            if let Err(e) = self.process_news_article(article).await {
                                error!("Failed to process news article: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("News feed fetch failed for {}: {}", feed.name, e);
                    }
                }
            }

            sleep(Duration::from_secs(300)).await; // Check every 5 minutes
        }
    }

    pub async fn start_halt_monitoring(&self) -> Result<()> {
        if !self.config.halt_monitoring.enabled {
            info!("Halt monitoring disabled");
            return Ok(());
        }

        info!("Starting halt monitoring");
        let halt_ingestor = HaltIngestor::new(&self.config.halt_monitoring);

        loop {
            for exchange in &self.config.halt_monitoring.exchanges {
                match halt_ingestor.fetch_halts(&exchange.halt_feed_url).await {
                    Ok(halts) => {
                        for halt in halts {
                            if let Err(e) = self.process_trading_halt(halt).await {
                                error!("Failed to process trading halt: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Halt fetch failed for {}: {}", exchange.name, e);
                    }
                }
            }

            sleep(Duration::from_secs(30)).await; // Check every 30 seconds
        }
    }

    pub async fn process_events_loop(&self) -> Result<()> {
        info!("Starting event processing loop");
        
        loop {
            // Process events from Redis queue
            match self.process_pending_events().await {
                Ok(processed) => {
                    if processed > 0 {
                        info!("Processed {} pending events", processed);
                    }
                }
                Err(e) => {
                    error!("Event processing failed: {}", e);
                }
            }

            sleep(Duration::from_secs(10)).await; // Process every 10 seconds
        }
    }

    pub async fn post_alerts_loop(&self) -> Result<()> {
        info!("Starting alert posting loop");
        
        loop {
            // Post pending alerts
            match self.post_pending_alerts().await {
                Ok(posted) => {
                    if posted > 0 {
                        info!("Posted {} alerts", posted);
                    }
                }
                Err(e) => {
                    error!("Alert posting failed: {}", e);
                }
            }

            sleep(Duration::from_secs(30)).await; // Post every 30 seconds
        }
    }

    async fn process_filing(&self, filing: SecFiling) -> Result<()> {
        debug!("Processing SEC filing: {}", filing.title);
        
        let mut event = MarketEvent::new(
            filing.title,
            filing.summary,
            EventCategory::RegulatoryFiling,
            vec![EventSource {
                name: "SEC EDGAR".to_string(),
                url: filing.url,
                credibility_score: 1.0,
                is_official: true,
                timestamp: filing.filed_at,
            }],
        );

        // Link to tickers
        let tickers = self.entity_linker.extract_tickers(&event.title, &event.description).await?;
        for ticker in tickers {
            event.add_ticker(ticker);
        }

        // Calculate severity
        let severity_score = self.severity_scorer.calculate_severity(&event).await?;
        event.set_severity(severity_score.level, severity_score.final_score);

        // Store event
        self.store_event(event).await?;
        
        Ok(())
    }

    async fn process_news_article(&self, article: NewsArticle) -> Result<()> {
        debug!("Processing news article: {}", article.title);
        
        // Check if article contains relevant keywords
        if !self.contains_relevant_keywords(&article.title, &article.content) {
            return Ok(());
        }

        let mut event = MarketEvent::new(
            article.title,
            article.content,
            self.categorize_article(&article),
            vec![EventSource {
                name: article.source,
                url: article.url,
                credibility_score: 0.8, // Default, will be updated based on source
                is_official: false,
                timestamp: article.published_at,
            }],
        );

        // Link to tickers
        let tickers = self.entity_linker.extract_tickers(&event.title, &event.description).await?;
        for ticker in tickers {
            event.add_ticker(ticker);
        }

        // Calculate severity
        let severity_score = self.severity_scorer.calculate_severity(&event).await?;
        event.set_severity(severity_score.level, severity_score.final_score);

        // Store event
        self.store_event(event).await?;
        
        Ok(())
    }

    async fn process_trading_halt(&self, halt: TradingHalt) -> Result<()> {
        debug!("Processing trading halt: {} - {}", halt.symbol, halt.reason);
        
        let mut event = MarketEvent::new(
            format!("Trading HALTED: {} - {}", halt.symbol, halt.reason),
            format!("Trading halted for {} at {} due to: {}", halt.symbol, halt.halt_time, halt.reason),
            EventCategory::TradingStatus,
            vec![EventSource {
                name: halt.exchange,
                url: halt.url,
                credibility_score: 1.0,
                is_official: true,
                timestamp: halt.halt_time,
            }],
        );

        event.add_ticker(halt.symbol);
        event.status = EventStatus::Confirmed; // Trading halts are always confirmed

        // Calculate severity
        let severity_score = self.severity_scorer.calculate_severity(&event).await?;
        event.set_severity(severity_score.level, severity_score.final_score);

        // Store event
        self.store_event(event).await?;
        
        Ok(())
    }

    async fn process_pending_events(&self) -> Result<u32> {
        // TODO: Implement Redis queue processing
        // For now, return 0
        Ok(0)
    }

    async fn post_pending_alerts(&self) -> Result<u32> {
        // TODO: Implement alert posting queue
        // For now, return 0
        Ok(0)
    }

    async fn store_event(&self, event: MarketEvent) -> Result<()> {
        debug!("Storing event: {} - {}", event.id, event.title);
        
        // TODO: Store in database and Redis
        // For now, just log
        info!("Event stored: {} - {} - {:?}", event.id, event.title, event.severity);
        
        Ok(())
    }

    fn contains_relevant_keywords(&self, title: &str, content: &str) -> bool {
        let text = format!("{} {}", title, content).to_lowercase();
        self.config.news_feeds.keywords.iter().any(|keyword| {
            text.contains(&keyword.to_lowercase())
        })
    }

    fn categorize_article(&self, article: &NewsArticle) -> EventCategory {
        let text = format!("{} {}", article.title, article.content).to_lowercase();
        
        if text.contains("accident") || text.contains("explosion") || text.contains("fire") {
            EventCategory::CriticalIncident
        } else if text.contains("recall") || text.contains("withdrawal") {
            EventCategory::ProductRecall
        } else if text.contains("lawsuit") || text.contains("settlement") || text.contains("fine") {
            EventCategory::LegalRegulatory
        } else if text.contains("ceo") || text.contains("cfo") || text.contains("resign") {
            EventCategory::Leadership
        } else if text.contains("earnings") || text.contains("guidance") {
            EventCategory::EarningsSurprise
        } else if text.contains("breach") || text.contains("hack") || text.contains("exploit") {
            EventCategory::CryptoIncident
        } else {
            EventCategory::CriticalIncident // Default
        }
    }

    pub async fn get_metrics(&self) -> EventMetrics {
        self.metrics.clone()
    }
}
