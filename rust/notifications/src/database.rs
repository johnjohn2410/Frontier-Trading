use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use anyhow::Result;
use crate::models::*;

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Stock Alert Operations
    pub async fn create_stock_alert(&self, alert: &StockAlert) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO stock_alerts (id, user_id, symbol, alert_type, price_target, percentage_change, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            alert.id,
            alert.user_id,
            alert.symbol,
            alert.alert_type as i32,
            alert.price_target,
            alert.percentage_change,
            alert.is_active,
            alert.created_at,
            alert.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_stock_alerts(&self, user_id: Uuid) -> Result<Vec<StockAlert>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, symbol, alert_type, price_target, percentage_change, is_active, created_at, updated_at
            FROM stock_alerts
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let alerts = rows
            .into_iter()
            .map(|row| StockAlert {
                id: row.id,
                user_id: row.user_id,
                symbol: row.symbol,
                alert_type: match row.alert_type {
                    0 => AlertType::PriceAbove,
                    1 => AlertType::PriceBelow,
                    2 => AlertType::PercentageGain,
                    3 => AlertType::PercentageLoss,
                    4 => AlertType::VolumeSpike,
                    5 => AlertType::NewsMention,
                    6 => AlertType::EarningsAnnouncement,
                    7 => AlertType::TechnicalBreakout,
                    _ => AlertType::PriceAbove,
                },
                price_target: row.price_target,
                percentage_change: row.percentage_change,
                is_active: row.is_active,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(alerts)
    }

    pub async fn get_active_stock_alerts(&self) -> Result<Vec<StockAlert>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, symbol, alert_type, price_target, percentage_change, is_active, created_at, updated_at
            FROM stock_alerts
            WHERE is_active = true
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let alerts = rows
            .into_iter()
            .map(|row| StockAlert {
                id: row.id,
                user_id: row.user_id,
                symbol: row.symbol,
                alert_type: match row.alert_type {
                    0 => AlertType::PriceAbove,
                    1 => AlertType::PriceBelow,
                    2 => AlertType::PercentageGain,
                    3 => AlertType::PercentageLoss,
                    4 => AlertType::VolumeSpike,
                    5 => AlertType::NewsMention,
                    6 => AlertType::EarningsAnnouncement,
                    7 => AlertType::TechnicalBreakout,
                    _ => AlertType::PriceAbove,
                },
                price_target: row.price_target,
                percentage_change: row.percentage_change,
                is_active: row.is_active,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(alerts)
    }

    pub async fn deactivate_stock_alert(&self, alert_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE stock_alerts
            SET is_active = false, updated_at = $2
            WHERE id = $1
            "#,
            alert_id,
            Utc::now()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // News Alert Operations
    pub async fn create_news_alert(&self, alert: &NewsAlert) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO news_alerts (id, user_id, symbol, keywords, sources, is_active, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            alert.id,
            alert.user_id,
            alert.symbol,
            &alert.keywords,
            &alert.sources,
            alert.is_active,
            alert.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_news_alerts(&self) -> Result<Vec<NewsAlert>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, symbol, keywords, sources, is_active, created_at
            FROM news_alerts
            WHERE is_active = true
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let alerts = rows
            .into_iter()
            .map(|row| NewsAlert {
                id: row.id,
                user_id: row.user_id,
                symbol: row.symbol,
                keywords: row.keywords,
                sources: row.sources,
                is_active: row.is_active,
                created_at: row.created_at,
            })
            .collect();

        Ok(alerts)
    }

    // News Article Operations
    pub async fn create_news_article(&self, article: &NewsArticle) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO news_articles (id, title, content, summary, url, source, published_at, symbols, sentiment, relevance_score)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            article.id,
            article.title,
            article.content,
            article.summary,
            article.url,
            article.source,
            article.published_at,
            &article.symbols,
            article.sentiment,
            article.relevance_score
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_news_for_symbol(&self, symbol: &str, limit: usize) -> Result<Vec<NewsArticle>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, title, content, summary, url, source, published_at, symbols, sentiment, relevance_score
            FROM news_articles
            WHERE $1 = ANY(symbols)
            ORDER BY published_at DESC
            LIMIT $2
            "#,
            symbol,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        let articles = rows
            .into_iter()
            .map(|row| NewsArticle {
                id: row.id,
                title: row.title,
                content: row.content,
                summary: row.summary,
                url: row.url,
                source: row.source,
                published_at: row.published_at,
                symbols: row.symbols,
                sentiment: row.sentiment,
                relevance_score: row.relevance_score,
            })
            .collect();

        Ok(articles)
    }

    // Notification Operations
    pub async fn create_notification(&self, notification: &Notification) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO notifications (id, user_id, title, message, notification_type, priority, channels, metadata, is_read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            notification.id,
            notification.user_id,
            notification.title,
            notification.message,
            notification.notification_type as i32,
            notification.priority as i32,
            &notification.channels.iter().map(|c| *c as i32).collect::<Vec<_>>(),
            notification.metadata,
            notification.is_read,
            notification.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_notifications(&self, user_id: Uuid, limit: Option<usize>) -> Result<Vec<Notification>> {
        let limit = limit.unwrap_or(50) as i64;
        
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, title, message, notification_type, priority, channels, metadata, is_read, created_at
            FROM notifications
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        let notifications = rows
            .into_iter()
            .map(|row| Notification {
                id: row.id,
                user_id: row.user_id,
                title: row.title,
                message: row.message,
                notification_type: match row.notification_type {
                    0 => NotificationType::PriceAlert,
                    1 => NotificationType::NewsAlert,
                    2 => NotificationType::TechnicalAlert,
                    3 => NotificationType::EarningsAlert,
                    4 => NotificationType::VolumeAlert,
                    5 => NotificationType::SystemAlert,
                    _ => NotificationType::SystemAlert,
                },
                priority: match row.priority {
                    0 => NotificationPriority::Low,
                    1 => NotificationPriority::Medium,
                    2 => NotificationPriority::High,
                    3 => NotificationPriority::Critical,
                    _ => NotificationPriority::Medium,
                },
                channels: row.channels
                    .into_iter()
                    .map(|c| match c {
                        0 => NotificationChannel::InApp,
                        1 => NotificationChannel::Email,
                        2 => NotificationChannel::Push,
                        3 => NotificationChannel::SMS,
                        4 => NotificationChannel::WebSocket,
                        _ => NotificationChannel::InApp,
                    })
                    .collect(),
                metadata: row.metadata,
                is_read: row.is_read,
                created_at: row.created_at,
            })
            .collect();

        Ok(notifications)
    }

    pub async fn mark_notification_read(&self, notification_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE notifications
            SET is_read = true
            WHERE id = $1
            "#,
            notification_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn mark_all_notifications_read(&self, user_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE notifications
            SET is_read = true
            WHERE user_id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_notification(&self, notification_id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM notifications
            WHERE id = $1
            "#,
            notification_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Price Alert Trigger Operations
    pub async fn create_price_alert_trigger(&self, trigger: &PriceAlertTrigger) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO price_alert_triggers (alert_id, symbol, current_price, target_price, alert_type, triggered_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            trigger.alert_id,
            trigger.symbol,
            trigger.current_price,
            trigger.target_price,
            trigger.alert_type as i32,
            trigger.triggered_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // User Preferences Operations
    pub async fn get_user_notification_preferences(&self, user_id: Uuid) -> Result<Option<UserNotificationPreferences>> {
        let row = sqlx::query!(
            r#"
            SELECT user_id, email_enabled, push_enabled, sms_enabled, in_app_enabled, quiet_hours_start, quiet_hours_end, timezone, daily_digest, weekly_summary
            FROM user_notification_preferences
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| UserNotificationPreferences {
            user_id: r.user_id,
            email_enabled: r.email_enabled,
            push_enabled: r.push_enabled,
            sms_enabled: r.sms_enabled,
            in_app_enabled: r.in_app_enabled,
            quiet_hours_start: r.quiet_hours_start,
            quiet_hours_end: r.quiet_hours_end,
            timezone: r.timezone,
            daily_digest: r.daily_digest,
            weekly_summary: r.weekly_summary,
        }))
    }

    pub async fn update_user_notification_preferences(&self, preferences: &UserNotificationPreferences) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_notification_preferences (user_id, email_enabled, push_enabled, sms_enabled, in_app_enabled, quiet_hours_start, quiet_hours_end, timezone, daily_digest, weekly_summary)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (user_id) DO UPDATE SET
                email_enabled = EXCLUDED.email_enabled,
                push_enabled = EXCLUDED.push_enabled,
                sms_enabled = EXCLUDED.sms_enabled,
                in_app_enabled = EXCLUDED.in_app_enabled,
                quiet_hours_start = EXCLUDED.quiet_hours_start,
                quiet_hours_end = EXCLUDED.quiet_hours_end,
                timezone = EXCLUDED.timezone,
                daily_digest = EXCLUDED.daily_digest,
                weekly_summary = EXCLUDED.weekly_summary
            "#,
            preferences.user_id,
            preferences.email_enabled,
            preferences.push_enabled,
            preferences.sms_enabled,
            preferences.in_app_enabled,
            preferences.quiet_hours_start,
            preferences.quiet_hours_end,
            preferences.timezone,
            preferences.daily_digest,
            preferences.weekly_summary
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
