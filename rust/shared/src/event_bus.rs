use redis::{Client, Connection, Commands, RedisResult, RedisError};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, error, warn, debug};

use crate::types::{Event, EventPayload};

// ============================================================================
// REDIS STREAMS EVENT BUS
// ============================================================================

#[derive(Debug, Clone)]
pub struct EventBus {
    redis_client: Arc<Client>,
    streams: Vec<String>,
    consumer_group: String,
    consumer_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    pub id: String,
    pub event: Event,
    pub timestamp: DateTime<Utc>,
    pub retry_count: u32,
}

#[derive(Debug, Clone)]
pub struct EventBusConfig {
    pub redis_url: String,
    pub streams: Vec<String>,
    pub consumer_group: String,
    pub consumer_name: String,
    pub batch_size: usize,
    pub poll_interval_ms: u64,
    pub max_retries: u32,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://127.0.0.1:6379".to_string(),
            streams: vec![
                "market_ticks".to_string(),
                "orders".to_string(),
                "fills".to_string(),
                "positions".to_string(),
                "alerts".to_string(),
                "copilot_suggestions".to_string(),
                "news".to_string(),
            ],
            consumer_group: "frontier_trading".to_string(),
            consumer_name: format!("consumer_{}", Uuid::new_v4()),
            batch_size: 10,
            poll_interval_ms: 100,
            max_retries: 3,
        }
    }
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> RedisResult<Self> {
        let client = Client::open(config.redis_url)?;
        
        Ok(Self {
            redis_client: Arc::new(client),
            streams: config.streams,
            consumer_group: config.consumer_group,
            consumer_name: config.consumer_name,
        })
    }

    pub async fn publish(&self, stream: &str, event: Event) -> RedisResult<String> {
        let mut conn = self.redis_client.get_connection()?;
        
        let message = StreamMessage {
            id: Uuid::new_v4().to_string(),
            event,
            timestamp: Utc::now(),
            retry_count: 0,
        };
        
        let event_json = serde_json::to_string(&message).map_err(|e| {
            RedisError::from((
                redis::ErrorKind::InvalidArgument,
                "Failed to serialize event",
                e.to_string(),
            ))
        })?;
        
        let message_id = conn.xadd(stream, "*", &[("event", &event_json)])?;
        
        info!("Published event to stream {}: {}", stream, message_id);
        Ok(message_id)
    }

    pub async fn publish_batch(&self, stream: &str, events: Vec<Event>) -> RedisResult<Vec<String>> {
        let mut conn = self.redis_client.get_connection()?;
        let mut message_ids = Vec::new();
        
        for event in events {
            let message = StreamMessage {
                id: Uuid::new_v4().to_string(),
                event,
                timestamp: Utc::now(),
                retry_count: 0,
            };
            
            let event_json = serde_json::to_string(&message).map_err(|e| {
                RedisError::from((
                    redis::ErrorKind::InvalidArgument,
                    "Failed to serialize event",
                    e.to_string(),
                ))
            })?;
            
            let message_id = conn.xadd(stream, "*", &[("event", &event_json)])?;
            message_ids.push(message_id);
        }
        
        info!("Published {} events to stream {}", events.len(), stream);
        Ok(message_ids)
    }

    pub async fn subscribe(
        &self,
        stream: &str,
        last_id: Option<String>,
    ) -> mpsc::Receiver<StreamMessage> {
        let (tx, rx) = mpsc::channel(100);
        let redis_client = self.redis_client.clone();
        let consumer_group = self.consumer_group.clone();
        let consumer_name = self.consumer_name.clone();
        
        tokio::spawn(async move {
            if let Err(e) = Self::subscribe_worker(
                redis_client,
                stream,
                consumer_group,
                consumer_name,
                last_id,
                tx,
            ).await {
                error!("Subscription worker failed for stream {}: {}", stream, e);
            }
        });
        
        rx
    }

    async fn subscribe_worker(
        redis_client: Arc<Client>,
        stream: &str,
        consumer_group: String,
        consumer_name: String,
        last_id: Option<String>,
        tx: mpsc::Sender<StreamMessage>,
    ) -> RedisResult<()> {
        let mut conn = redis_client.get_connection()?;
        
        // Create consumer group if it doesn't exist
        if let Err(_) = conn.xgroup_create(stream, &consumer_group, "$", true) {
            debug!("Consumer group {} already exists for stream {}", consumer_group, stream);
        }
        
        let start_id = last_id.unwrap_or_else(|| "0".to_string());
        
        loop {
            let messages: Vec<(String, Vec<(String, Vec<(String, String)>)>)> = conn
                .xreadgroup(
                    &consumer_group,
                    &consumer_name,
                    &[(stream, ">")],
                    Some(100),
                    Some(1000),
                )?;
            
            for (stream_name, stream_messages) in messages {
                for (message_id, fields) in stream_messages {
                    if let Some(event_field) = fields.iter().find(|(k, _)| k == "event") {
                        if let Ok(message) = serde_json::from_str::<StreamMessage>(&event_field.1) {
                            if tx.send(message).await.is_err() {
                                error!("Failed to send message to receiver");
                                break;
                            }
                        } else {
                            error!("Failed to deserialize message from stream {}", stream_name);
                        }
                    }
                    
                    // Acknowledge the message
                    if let Err(e) = conn.xack(&stream_name, &consumer_group, &message_id) {
                        error!("Failed to acknowledge message {}: {}", message_id, e);
                    }
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    pub async fn get_stream_length(&self, stream: &str) -> RedisResult<u64> {
        let mut conn = self.redis_client.get_connection()?;
        conn.xlen(stream)
    }

    pub async fn trim_stream(&self, stream: &str, max_len: usize) -> RedisResult<u64> {
        let mut conn = self.redis_client.get_connection()?;
        conn.xtrim(stream, max_len, false)
    }

    pub async fn get_pending_messages(&self, stream: &str) -> RedisResult<Vec<(String, String, u64, u64)>> {
        let mut conn = self.redis_client.get_connection()?;
        conn.xpending(stream, &self.consumer_group, None, None, None, None)
    }

    pub async fn claim_pending_messages(
        &self,
        stream: &str,
        min_idle_time_ms: u64,
    ) -> RedisResult<Vec<(String, Vec<(String, Vec<(String, String)>)>)>> {
        let mut conn = self.redis_client.get_connection()?;
        conn.xclaim(
            stream,
            &self.consumer_group,
            &self.consumer_name,
            min_idle_time_ms,
            &[],
            None,
            None,
            false,
        )
    }
}

// ============================================================================
// EVENT BUS MANAGER
// ============================================================================

#[derive(Debug, Clone)]
pub struct EventBusManager {
    event_bus: EventBus,
    handlers: HashMap<String, Box<dyn EventHandler + Send + Sync>>,
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Event) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

impl EventBusManager {
    pub fn new(config: EventBusConfig) -> RedisResult<Self> {
        Ok(Self {
            event_bus: EventBus::new(config)?,
            handlers: HashMap::new(),
        })
    }

    pub fn register_handler(&mut self, event_type: &str, handler: Box<dyn EventHandler + Send + Sync>) {
        self.handlers.insert(event_type.to_string(), handler);
    }

    pub async fn start_processing(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut handles = Vec::new();
        
        for stream in &self.event_bus.streams {
            let event_bus = self.event_bus.clone();
            let handlers = self.handlers.clone();
            
            let handle = tokio::spawn(async move {
                if let Err(e) = Self::process_stream(event_bus, stream, handlers).await {
                    error!("Stream processing failed for {}: {}", stream, e);
                }
            });
            
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await?;
        }
        
        Ok(())
    }

    async fn process_stream(
        event_bus: EventBus,
        stream: &str,
        handlers: HashMap<String, Box<dyn EventHandler + Send + Sync>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut rx = event_bus.subscribe(stream, None).await;
        
        while let Some(message) = rx.recv().await {
            let event_type = message.event.event_type.clone();
            
            if let Some(handler) = handlers.get(&event_type) {
                if let Err(e) = handler.handle(message.event).await {
                    error!("Handler failed for event type {}: {}", event_type, e);
                    
                    // Retry logic for failed messages
                    if message.retry_count < 3 {
                        let retry_message = StreamMessage {
                            id: message.id,
                            event: message.event,
                            timestamp: Utc::now(),
                            retry_count: message.retry_count + 1,
                        };
                        
                        // Publish to retry stream
                        if let Err(e) = event_bus.publish(&format!("{}_retry", stream), retry_message.event).await {
                            error!("Failed to publish retry message: {}", e);
                        }
                    }
                }
            } else {
                warn!("No handler registered for event type: {}", event_type);
            }
        }
        
        Ok(())
    }

    pub async fn publish_event(&self, stream: &str, event: Event) -> RedisResult<String> {
        self.event_bus.publish(stream, event).await
    }

    pub async fn publish_events(&self, stream: &str, events: Vec<Event>) -> RedisResult<Vec<String>> {
        self.event_bus.publish_batch(stream, events).await
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

pub async fn create_event_bus_manager() -> Result<EventBusManager, Box<dyn std::error::Error + Send + Sync>> {
    let config = EventBusConfig::default();
    EventBusManager::new(config).map_err(|e| e.into())
}

pub fn get_stream_name_for_event(event: &Event) -> String {
    match &event.payload {
        EventPayload::MarketTick(_) => "market_ticks".to_string(),
        EventPayload::OrderRequested(_) | EventPayload::OrderAccepted(_) | 
        EventPayload::OrderRejected(_) | EventPayload::OrderFilled(_) | 
        EventPayload::OrderCancelled(_) => "orders".to_string(),
        EventPayload::PositionUpdated(_) | EventPayload::PositionClosed(_) => "positions".to_string(),
        EventPayload::AccountUpdated(_) | EventPayload::CashUpdated(_) => "account".to_string(),
        EventPayload::AlertRaised(_) | EventPayload::AlertTriggered(_) => "alerts".to_string(),
        EventPayload::CopilotSuggestion(_) | EventPayload::CopilotAnalysis(_) => "copilot_suggestions".to_string(),
        EventPayload::NewsArticle(_) | EventPayload::SentimentUpdate(_) => "news".to_string(),
        EventPayload::PriceAlert(_) => "price_alerts".to_string(),
    }
}
