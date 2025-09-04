use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::EntityMappingConfig;
use crate::types::EntityMapping;

pub struct EntityLinker {
    config: EntityMappingConfig,
    ticker_mappings: HashMap<String, EntityMapping>,
    cik_mappings: HashMap<String, String>, // CIK -> Ticker
}

impl EntityLinker {
    pub async fn new(config: &EntityMappingConfig) -> Result<Self> {
        let mut linker = Self {
            config: config.clone(),
            ticker_mappings: HashMap::new(),
            cik_mappings: HashMap::new(),
        };

        linker.load_entity_mappings().await?;
        Ok(linker)
    }

    pub async fn extract_tickers(&self, title: &str, content: &str) -> Result<Vec<String>> {
        debug!("Extracting tickers from: {}", title);
        
        let text = format!("{} {}", title, content);
        let mut tickers = Vec::new();

        // Direct ticker mentions (e.g., $AAPL, AAPL)
        let ticker_pattern = Regex::new(r"\$?([A-Z]{1,5})\b")?;
        for cap in ticker_pattern.captures_iter(&text) {
            if let Some(ticker) = cap.get(1) {
                let ticker_str = ticker.as_str();
                if self.is_valid_ticker(ticker_str) {
                    tickers.push(ticker_str.to_string());
                }
            }
        }

        // Company name mentions
        for (ticker, mapping) in &self.ticker_mappings {
            for keyword in &mapping.keywords {
                if text.to_lowercase().contains(&keyword.to_lowercase()) {
                    tickers.push(ticker.clone());
                }
            }
            
            for alias in &mapping.aliases {
                if text.to_lowercase().contains(&alias.to_lowercase()) {
                    tickers.push(ticker.clone());
                }
            }
        }

        // Remove duplicates
        tickers.sort();
        tickers.dedup();

        info!("Extracted tickers: {:?}", tickers);
        Ok(tickers)
    }

    pub async fn get_company_info(&self, ticker: &str) -> Option<&EntityMapping> {
        self.ticker_mappings.get(ticker)
    }

    pub async fn get_cik_code(&self, ticker: &str) -> Option<&String> {
        self.cik_mappings.get(ticker)
    }

    async fn load_entity_mappings(&mut self) -> Result<()> {
        info!("Loading entity mappings");
        
        // TODO: Load from actual data sources
        // For now, populate with common tickers
        self.populate_common_tickers();
        
        info!("Loaded {} entity mappings", self.ticker_mappings.len());
        Ok(())
    }

    fn populate_common_tickers(&mut self) {
        let common_tickers = vec![
            ("AAPL", "Apple Inc.", vec!["apple", "iphone", "ipad", "mac"], vec!["Apple Computer"]),
            ("MSFT", "Microsoft Corporation", vec!["microsoft", "windows", "azure", "office"], vec!["Microsoft Corp"]),
            ("GOOGL", "Alphabet Inc.", vec!["google", "alphabet", "youtube", "android"], vec!["Google"]),
            ("AMZN", "Amazon.com Inc.", vec!["amazon", "aws", "prime", "alexa"], vec!["Amazon"]),
            ("TSLA", "Tesla Inc.", vec!["tesla", "electric", "autopilot", "cybertruck"], vec!["Tesla Motors"]),
            ("META", "Meta Platforms Inc.", vec!["meta", "facebook", "instagram", "whatsapp"], vec!["Facebook"]),
            ("NVDA", "NVIDIA Corporation", vec!["nvidia", "gpu", "ai", "cuda"], vec!["NVIDIA Corp"]),
            ("NFLX", "Netflix Inc.", vec!["netflix", "streaming", "movies"], vec!["Netflix"]),
            ("AMD", "Advanced Micro Devices", vec!["amd", "ryzen", "radeon", "processors"], vec!["AMD Inc"]),
            ("INTC", "Intel Corporation", vec!["intel", "processors", "chips", "xeon"], vec!["Intel Corp"]),
        ];

        for (ticker, name, keywords, aliases) in common_tickers {
            let mapping = EntityMapping {
                ticker: ticker.to_string(),
                company_name: name.to_string(),
                cik_code: format!("cik_{}", ticker.to_lowercase()),
                sector: "Technology".to_string(),
                keywords: keywords.iter().map(|s| s.to_string()).collect(),
                aliases: aliases.iter().map(|s| s.to_string()).collect(),
            };
            
            self.ticker_mappings.insert(ticker.to_string(), mapping);
            self.cik_mappings.insert(ticker.to_string(), format!("cik_{}", ticker.to_lowercase()));
        }
    }

    fn is_valid_ticker(&self, ticker: &str) -> bool {
        // Basic validation: 1-5 uppercase letters
        if ticker.len() < 1 || ticker.len() > 5 {
            return false;
        }
        
        if !ticker.chars().all(|c| c.is_ascii_uppercase()) {
            return false;
        }

        // Check against known tickers
        self.ticker_mappings.contains_key(ticker)
    }
}
