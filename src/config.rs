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
        let postgres_host = env::var("POSTGRES_HOST")?;
        let postgres_port = env::var("POSTGRES_PORT")?;
        let postgres_user = env::var("POSTGRES_USER")?;
        let postgres_password = env::var("POSTGRES_PASSWORD")?;
        let postgres_db = env::var("POSTGRES_DB")?;

        // Construct database URL from individual parameters
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
        );

        Ok(Self {
            database_url,
            server_host: env::var("SERVER_HOST")?,
            server_port: env::var("SERVER_PORT")?.parse()?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }
}
