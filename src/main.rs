use std::net::SocketAddr;

use axum::{
    routing::get,
    Router,
};
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod config;
mod db;
mod error;
mod models;
mod repository;
mod validation;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();
    
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = config::Config::from_env()?;
    
    // Set up database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Build our application with routes
    let app = Router::new()
        .nest("/api", api::routes(pool.clone()))
        .route("/health", get(health_check));
    
    // Run our application
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}