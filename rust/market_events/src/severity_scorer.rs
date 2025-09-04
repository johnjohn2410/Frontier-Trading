use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

use crate::config::SeverityScoringConfig;
use crate::types::*;

pub struct SeverityScorer {
    config: SeverityScoringConfig,
}

impl SeverityScorer {
    pub fn new(config: &SeverityScoringConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    pub async fn calculate_severity(&self, event: &MarketEvent) -> Result<SeverityScore> {
        debug!("Calculating severity for event: {}", event.title);
        
        let source_credibility = self.calculate_source_credibility(event);
        let incident_magnitude = self.calculate_incident_magnitude(event);
        let price_impact = self.calculate_price_impact(event);
        let novelty_factor = self.calculate_novelty_factor(event);
        
        let base_score = self.get_category_base_score(&event.category);
        
        let final_score = (base_score * 0.3) + 
                         (source_credibility * 0.25) + 
                         (incident_magnitude * 0.25) + 
                         (price_impact * 0.15) + 
                         (novelty_factor * 0.05);
        
        let level = self.score_to_level(final_score);
        
        let severity_score = SeverityScore {
            base_score,
            source_credibility,
            incident_magnitude,
            price_impact,
            novelty_factor,
            final_score,
            level,
        };
        
        info!("Severity calculated: {:?} (score: {:.2})", level, final_score);
        Ok(severity_score)
    }

    fn calculate_source_credibility(&self, event: &MarketEvent) -> f64 {
        if event.sources.is_empty() {
            return 0.0;
        }

        let mut total_credibility = 0.0;
        let mut official_sources = 0;
        
        for source in &event.sources {
            let weight = self.config.source_weights
                .get(&source.name)
                .copied()
                .unwrap_or(0.5);
            
            total_credibility += source.credibility_score * weight;
            
            if source.is_official {
                official_sources += 1;
            }
        }

        // Boost for official sources
        let official_boost = if official_sources > 0 { 0.2 } else { 0.0 };
        
        (total_credibility / event.sources.len() as f64 + official_boost).min(1.0)
    }

    fn calculate_incident_magnitude(&self, event: &MarketEvent) -> f64 {
        let text = format!("{} {}", event.title, event.description).to_lowercase();
        
        let mut magnitude = 0.0;
        
        // Critical incident indicators
        if text.contains("death") || text.contains("fatality") || text.contains("killed") {
            magnitude += 1.0;
        } else if text.contains("injury") || text.contains("injured") {
            magnitude += 0.7;
        }
        
        if text.contains("explosion") || text.contains("fire") || text.contains("explosion") {
            magnitude += 0.8;
        }
        
        if text.contains("recall") {
            magnitude += 0.6;
        }
        
        if text.contains("breach") || text.contains("hack") {
            magnitude += 0.7;
        }
        
        if text.contains("lawsuit") || text.contains("settlement") {
            magnitude += 0.5;
        }
        
        if text.contains("ceo") || text.contains("cfo") {
            magnitude += 0.4;
        }
        
        if text.contains("halt") || text.contains("suspended") {
            magnitude += 0.6;
        }
        
        // Financial impact indicators
        if text.contains("billion") {
            magnitude += 0.8;
        } else if text.contains("million") {
            magnitude += 0.5;
        }
        
        magnitude.min(1.0)
    }

    fn calculate_price_impact(&self, event: &MarketEvent) -> f64 {
        if let Some(price_impact) = &event.price_impact {
            let abs_change = price_impact.price_change_pct.abs();
            
            if abs_change >= 10.0 {
                1.0
            } else if abs_change >= 5.0 {
                0.8
            } else if abs_change >= 2.0 {
                0.6
            } else if abs_change >= 1.0 {
                0.4
            } else {
                0.2
            }
        } else {
            0.0
        }
    }

    fn calculate_novelty_factor(&self, event: &MarketEvent) -> f64 {
        // TODO: Implement novelty detection based on historical events
        // For now, return a default value
        0.5
    }

    fn get_category_base_score(&self, category: &EventCategory) -> f64 {
        let category_name = match category {
            EventCategory::CriticalIncident => "CriticalIncident",
            EventCategory::RegulatoryFiling => "RegulatoryFiling",
            EventCategory::TradingStatus => "TradingStatus",
            EventCategory::EarningsSurprise => "EarningsSurprise",
            EventCategory::LegalRegulatory => "LegalRegulatory",
            EventCategory::ProductRecall => "ProductRecall",
            EventCategory::Leadership => "Leadership",
            EventCategory::CryptoIncident => "CryptoIncident",
            EventCategory::MarketHalt => "TradingStatus",
        };
        
        self.config.category_weights
            .get(category_name)
            .copied()
            .unwrap_or(0.5)
    }

    fn score_to_level(&self, score: f64) -> SeverityLevel {
        if score >= 0.8 {
            SeverityLevel::A
        } else if score >= 0.6 {
            SeverityLevel::B
        } else {
            SeverityLevel::C
        }
    }

    pub fn should_post_alert(&self, event: &MarketEvent, severity_score: &SeverityScore) -> bool {
        // Always post A and B level events
        if matches!(severity_score.level, SeverityLevel::A | SeverityLevel::B) {
            return true;
        }
        
        // Post C level events if they meet certain criteria
        if matches!(severity_score.level, SeverityLevel::C) {
            // Post if price impact is significant
            if let Some(price_impact) = &event.price_impact {
                if price_impact.price_change_pct.abs() >= self.config.price_impact_threshold {
                    return true;
                }
            }
            
            // Post if confidence is high
            if event.confidence >= self.config.confidence_threshold {
                return true;
            }
        }
        
        false
    }

    pub fn requires_two_sources(&self, event: &MarketEvent) -> bool {
        if !self.config.two_source_rule_enabled {
            return false;
        }
        
        let category_name = match event.category {
            EventCategory::CriticalIncident => "CriticalIncident",
            EventCategory::LegalRegulatory => "LegalRegulatory",
            _ => return false,
        };
        
        self.config.two_source_required_for.contains(&category_name.to_string())
    }
}
