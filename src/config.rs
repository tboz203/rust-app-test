use std::env;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Configuration for rust-app-test
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
        let postgres_host = env_var_verbose("POSTGRES_HOST")?;
        let postgres_port = env_var_verbose("POSTGRES_PORT")?;
        let postgres_user = env_var_verbose("POSTGRES_USER")?;
        let postgres_password = env_var_verbose("POSTGRES_PASSWORD")?;
        let postgres_db = env_var_verbose("POSTGRES_DB")?;

        // Construct database URL from individual parameters
        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
        );

        Ok(Self {
            database_url,
            server_host: env_var_verbose("SERVER_HOST")?,
            server_port: env_var_verbose("SERVER_PORT")?.parse()?,
            rust_log: env_var_verbose("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        })
    }
}

/// Read an environment variable, or return an error with context
fn env_var_verbose(variable: &str) -> Result<String> {
    env::var(variable).with_context(|| format!("{variable} not found"))
}
