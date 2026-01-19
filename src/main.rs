#![allow(unused)]

mod api;
mod config;
mod db;
mod entity;
mod error;
mod models;
mod repository;
mod validation;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;

use axum::{Router, routing::get};
use config::Config;
use db::Database;
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
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

    // Run database migrations
    tracing::info!("Running database migrations");
    Migrator::up(&db, None).await?;
    tracing::info!("Database migrations completed successfully");

    // Build our application with routes
    let app = Router::new()
        .nest("/api", api::routes(db))
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
