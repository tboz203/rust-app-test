use std::time::Duration;

use anyhow::{Result, anyhow};
// Re-export Sea-ORM types for future use
pub use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, TransactionTrait};

/// Create a new database connection pool
pub async fn connect(database_url: &str) -> Result<DatabaseConnection> {
    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(5)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60))
        .sqlx_logging(true);

    Database::connect(opt)
        .await
        .map_err(|sea_err| anyhow!("Database connection error: {:?}", sea_err))
}
