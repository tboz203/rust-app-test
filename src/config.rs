use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Get database connection parameters
        let postgres_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let postgres_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
        let postgres_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let postgres_password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let postgres_db = env::var("POSTGRES_DB").unwrap_or_else(|_| "product_catalog".to_string());
        
        // Construct database URL from individual parameters
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
        );

        Ok(Self {
            database_url,
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }
}