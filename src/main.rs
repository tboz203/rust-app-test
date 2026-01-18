#![allow(unused)]

pub mod api;
pub mod config;
pub mod db;
pub mod entity;
pub mod error;
pub mod models;
pub mod repository;
pub mod validation;

// #[cfg(test)]
// mod tests;

use std::net::SocketAddr;

use axum::{routing::get, Router};
use config::Config;
use db::Database;
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let config = Config::from_env()?;

    // Set up database connection
    let db = Database::connect(&config.database_url).await?;

    // Build our application with routes
    let app = Router::new()
        .nest("/api", api::routes(db))
        .route("/health", get(health_check));

    // Run our application
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Listening on {}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
