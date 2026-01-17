# Project Implementation Templates

This document provides templates for the key files needed to implement the product catalog API.

## Cargo.toml Template

```toml
[package]
name = "product-catalog-api"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "A RESTful API for managing product catalog data"

[dependencies]
# Web framework
axum = "0.7.2"
tower = "0.4.13"
tower-http = { version = "0.5.0", features = ["trace", "cors"] }

# Async runtime
tokio = { version = "1.34.0", features = ["full"] }

# Database
sqlx = { version = "0.7.3", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "json", "migrate", "macros", "decimal"] }
rust_decimal = "1.33.1"
rust_decimal_macros = "1.33.1"

# Error handling
anyhow = "1.0.75"
thiserror = "1.0.50"

# Serialization/deserialization
serde = { version = "1.0.193", features = ["derive"] }
serde_json = "1.0.108"

# Validation
validator = { version = "0.16.1", features = ["derive"] }

# Utilities
chrono = { version = "0.4.31", features = ["serde"] }
uuid = { version = "1.6.1", features = ["v4", "serde"] }

# Configuration
config = "0.13.4"
dotenv = "0.15.0"

# Logging
tracing = "0.1.40"
tracing-subscriber = { version = "0.3.18", features = ["env-filter", "json"] }

[dev-dependencies]
reqwest = { version = "0.11.22", features = ["json"] }
mockall = "0.12.1"
tokio-test = "0.4.3"

[profile.dev]
opt-level = 0

[profile.release]
opt-level = 3
```

## .env Template

```
# Database configuration
DATABASE_URL=postgres://username:password@localhost:5432/product_catalog
POSTGRES_USER=username
POSTGRES_PASSWORD=password
POSTGRES_DB=product_catalog

# Server configuration
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
RUST_LOG=info
```

## Database Migrations

### migrations/20260117000001_create_categories_table.sql
```sql
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### migrations/20260117000002_create_products_table.sql
```sql
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL,
    sku VARCHAR(50) UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE product_categories (
    product_id INTEGER REFERENCES products(id) ON DELETE CASCADE,
    category_id INTEGER REFERENCES categories(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, category_id)
);
```

## Key Source Files

### src/main.rs
```rust
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
```

### src/config.rs
```rust
use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
        })
    }
}
```

### src/error.rs
```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Database(ref e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::NotFound(ref message) => (StatusCode::NOT_FOUND, message.clone()),
            Self::BadRequest(ref message) => (StatusCode::BAD_REQUEST, message.clone()),
            Self::Internal(ref message) => (StatusCode::INTERNAL_SERVER_ERROR, message.clone()),
            Self::Validation(ref message) => (StatusCode::UNPROCESSABLE_ENTITY, message.clone()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "status": status.as_u16(),
            }
        }));

        (status, body).into_response()
    }
}
```

### src/models/product.rs
```rust
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub sku: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub sku: Option<String>,
    pub category_ids: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub sku: Option<String>,
    pub category_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub sku: Option<String>,
    pub categories: Vec<CategoryBrief>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CategoryBrief {
    pub id: i32,
    pub name: String,
}