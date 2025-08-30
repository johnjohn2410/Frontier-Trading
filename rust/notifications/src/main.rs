use std::sync::Arc;
use tokio::net::TcpListener;
use axum::{
    Router,
    http::Method,
};
use tower_http::cors::{CorsLayer, Any};
use sqlx::postgres::PgPoolOptions;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use notifications::{
    NotificationService,
    AlertManager,
    NewsMonitor,
    PriceMonitor,
    database::Database,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Frontier Trading Notification Service");

    // Load configuration
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/frontier_trading".to_string());
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8001".to_string())
        .parse::<u16>()
        .expect("Invalid PORT");

    // Set up database connection
    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    info!("Database connected successfully");

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;
    
    info!("Database migrations completed");

    // Initialize components
    let db = Arc::new(Database::new(pool.clone()));
    let alert_manager = Arc::new(AlertManager::new(db.clone()));
    let news_monitor = Arc::new(NewsMonitor::new(db.clone()));
    let price_monitor = Arc::new(PriceMonitor::new(db.clone(), alert_manager.clone()));

    // Create notification service
    let notification_service = NotificationService::new(
        db.clone(),
        alert_manager.clone(),
        news_monitor.clone(),
        price_monitor.clone(),
    );

    // Set up CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create router
    let app = Router::new()
        .nest("/", notification_service.router())
        .layer(cors);

    // Start monitoring services
    info!("Starting monitoring services...");
    notification_service.start_monitoring().await?;

    // Start server
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Notification service listening on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
