use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{info, error, debug};
use uuid::Uuid;

use crate::config::AlertPostingConfig;
use crate::types::*;

pub struct AlertPoster {
    config: AlertPostingConfig,
    client: reqwest::Client,
}

impl AlertPoster {
    pub async fn new(config: &AlertPostingConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("MarketEventsBot/1.0 (alert posting)")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            config: config.clone(),
            client,
        })
    }

    pub async fn post_alert(&self, event: &MarketEvent) -> Result<Vec<AlertPost>> {
        debug!("Posting alert for event: {}", event.title);
        
        let mut posts = Vec::new();
        
        // Generate alert content
        let content = self.generate_alert_content(event)?;
        
        // Post to configured platforms
        if let Some(twitter_config) = &self.config.twitter {
            match self.post_to_twitter(&content, twitter_config).await {
                Ok(post) => posts.push(post),
                Err(e) => error!("Twitter post failed: {}", e),
            }
        }
        
        if let Some(discord_config) = &self.config.discord {
            match self.post_to_discord(&content, discord_config).await {
                Ok(post) => posts.push(post),
                Err(e) => error!("Discord post failed: {}", e),
            }
        }
        
        if let Some(slack_config) = &self.config.slack {
            match self.post_to_slack(&content, slack_config).await {
                Ok(post) => posts.push(post),
                Err(e) => error!("Slack post failed: {}", e),
            }
        }
        
        // Post to webhooks
        for webhook_config in &self.config.webhooks {
            match self.post_to_webhook(&content, webhook_config).await {
                Ok(post) => posts.push(post),
                Err(e) => error!("Webhook post failed for {}: {}", webhook_config.name, e),
            }
        }
        
        if let Some(email_config) = &self.config.email {
            match self.post_to_email(&content, email_config).await {
                Ok(post) => posts.push(post),
                Err(e) => error!("Email post failed: {}", e),
            }
        }
        
        info!("Posted {} alerts for event: {}", posts.len(), event.title);
        Ok(posts)
    }

    fn generate_alert_content(&self, event: &MarketEvent) -> Result<String> {
        let mut content = String::new();
        
        // Add tickers
        if !event.tickers.is_empty() {
            let ticker_list = event.tickers.iter()
                .map(|t| format!("${}", t))
                .collect::<Vec<_>>()
                .join(" ");
            content.push_str(&format!("{} — ", ticker_list));
        }
        
        // Add category and status
        let category = self.format_category(&event.category);
        let status = self.format_status(&event.status);
        content.push_str(&format!("[{}]: {} [{}]\n", category, event.title, status));
        
        // Add sources
        if !event.sources.is_empty() {
            content.push_str("• Source: ");
            let source_links = event.sources.iter()
                .map(|s| format!("<{}>", s.url))
                .collect::<Vec<_>>()
                .join(" + ");
            content.push_str(&source_links);
            content.push('\n');
        }
        
        // Add price impact if available
        if let Some(price_impact) = &event.price_impact {
            content.push_str(&format!(
                "• Early move ({}m): {:.1}% vs S&P {:.1}%\n",
                price_impact.timeframe_minutes,
                price_impact.price_change_pct,
                price_impact.market_relative_change
            ));
        }
        
        // Add potential impact
        content.push_str(&format!(
            "• Potential impact: {}\n",
            self.get_impact_description(&event.category)
        ));
        
        // Add disclaimer
        content.push_str("Not investment advice. Time: ");
        content.push_str(&Utc::now().format("%H:%M ET").to_string());
        
        Ok(content)
    }

    fn format_category(&self, category: &EventCategory) -> &'static str {
        match category {
            EventCategory::CriticalIncident => "CRITICAL INCIDENT",
            EventCategory::RegulatoryFiling => "FILING",
            EventCategory::TradingStatus => "HALT",
            EventCategory::EarningsSurprise => "EARNINGS",
            EventCategory::LegalRegulatory => "LEGAL",
            EventCategory::ProductRecall => "RECALL",
            EventCategory::Leadership => "LEADERSHIP",
            EventCategory::CryptoIncident => "CRYPTO",
            EventCategory::MarketHalt => "HALT",
        }
    }

    fn format_status(&self, status: &EventStatus) -> &'static str {
        match status {
            EventStatus::Developing => "Developing",
            EventStatus::Confirmed => "Confirmed",
            EventStatus::Update => "Update",
            EventStatus::Correction => "Correction",
            EventStatus::Resolved => "Resolved",
        }
    }

    fn get_impact_description(&self, category: &EventCategory) -> &'static str {
        match category {
            EventCategory::CriticalIncident => "operations disruption & liability exposure; monitoring",
            EventCategory::RegulatoryFiling => "regulatory compliance & disclosure requirements",
            EventCategory::TradingStatus => "trading suspension & market access",
            EventCategory::EarningsSurprise => "financial performance & guidance revision",
            EventCategory::LegalRegulatory => "legal exposure & regulatory scrutiny",
            EventCategory::ProductRecall => "product safety & brand reputation",
            EventCategory::Leadership => "management transition & strategic direction",
            EventCategory::CryptoIncident => "protocol security & market confidence",
            EventCategory::MarketHalt => "trading suspension & market access",
        }
    }

    async fn post_to_twitter(&self, content: &str, config: &crate::config::TwitterConfig) -> Result<AlertPost> {
        debug!("Posting to Twitter");
        
        // TODO: Implement actual Twitter API posting
        // For now, create a mock post
        let post = AlertPost::new(
            Uuid::new_v4(),
            "mock_incident".to_string(),
            content.to_string(),
            AlertPlatform::Twitter,
        );
        
        info!("Mock Twitter post created: {}", content);
        Ok(post)
    }

    async fn post_to_discord(&self, content: &str, config: &crate::config::DiscordConfig) -> Result<AlertPost> {
        debug!("Posting to Discord");
        
        let payload = serde_json::json!({
            "content": content,
            "username": "MarketEventBot",
            "avatar_url": "https://example.com/bot-avatar.png"
        });
        
        let response = self.client
            .post(&config.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            let post = AlertPost::new(
                Uuid::new_v4(),
                "mock_incident".to_string(),
                content.to_string(),
                AlertPlatform::Discord,
            );
            Ok(post)
        } else {
            Err(anyhow::anyhow!("Discord webhook failed: {}", response.status()))
        }
    }

    async fn post_to_slack(&self, content: &str, config: &crate::config::SlackConfig) -> Result<AlertPost> {
        debug!("Posting to Slack");
        
        let payload = serde_json::json!({
            "text": content,
            "channel": config.channel,
            "username": "MarketEventBot"
        });
        
        let response = self.client
            .post(&config.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            let post = AlertPost::new(
                Uuid::new_v4(),
                "mock_incident".to_string(),
                content.to_string(),
                AlertPlatform::Slack,
            );
            Ok(post)
        } else {
            Err(anyhow::anyhow!("Slack webhook failed: {}", response.status()))
        }
    }

    async fn post_to_webhook(&self, content: &str, config: &crate::config::WebhookConfig) -> Result<AlertPost> {
        debug!("Posting to webhook: {}", config.name);
        
        let payload = serde_json::json!({
            "content": content,
            "timestamp": Utc::now(),
            "source": "MarketEventBot"
        });
        
        let mut request = self.client.post(&config.url).json(&payload);
        
        // Add custom headers
        for (key, value) in &config.headers {
            request = request.header(key, value);
        }
        
        let response = request.send().await?;
        
        if response.status().is_success() {
            let post = AlertPost::new(
                Uuid::new_v4(),
                "mock_incident".to_string(),
                content.to_string(),
                AlertPlatform::Webhook,
            );
            Ok(post)
        } else {
            Err(anyhow::anyhow!("Webhook failed: {}", response.status()))
        }
    }

    async fn post_to_email(&self, content: &str, config: &crate::config::EmailConfig) -> Result<AlertPost> {
        debug!("Posting to email");
        
        // TODO: Implement actual email sending
        // For now, create a mock post
        let post = AlertPost::new(
            Uuid::new_v4(),
            "mock_incident".to_string(),
            content.to_string(),
            AlertPlatform::Email,
        );
        
        info!("Mock email post created: {}", content);
        Ok(post)
    }

    pub async fn post_update(&self, event: &MarketEvent, update_content: &str) -> Result<Vec<AlertPost>> {
        debug!("Posting update for event: {}", event.title);
        
        let mut content = String::new();
        content.push_str("[Update] ");
        
        if !event.tickers.is_empty() {
            let ticker_list = event.tickers.iter()
                .map(|t| format!("${}", t))
                .collect::<Vec<_>>()
                .join(" ");
            content.push_str(&format!("{} — ", ticker_list));
        }
        
        content.push_str(update_content);
        content.push_str(&format!(". Time: {}", Utc::now().format("%H:%M ET")));
        
        self.post_alert(&MarketEvent {
            title: update_content.to_string(),
            description: update_content.to_string(),
            ..event.clone()
        }).await
    }

    pub async fn post_correction(&self, event: &MarketEvent, correction: &str) -> Result<Vec<AlertPost>> {
        debug!("Posting correction for event: {}", event.title);
        
        let mut content = String::new();
        content.push_str("[Correction] ");
        
        if !event.tickers.is_empty() {
            let ticker_list = event.tickers.iter()
                .map(|t| format!("${}", t))
                .collect::<Vec<_>>()
                .join(" ");
            content.push_str(&format!("{} — ", ticker_list));
        }
        
        content.push_str(correction);
        content.push_str(&format!(". Time: {}", Utc::now().format("%H:%M ET")));
        
        self.post_alert(&MarketEvent {
            title: correction.to_string(),
            description: correction.to_string(),
            ..event.clone()
        }).await
    }
}
