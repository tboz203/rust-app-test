use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// Get the connection pool
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Execute a transaction
    pub async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: for<'c> FnOnce(&'c mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
        E: From<sqlx::Error> + Send + 'static,
    {
        let pool = self.pool.clone();
        let mut tx = pool.begin().await?;
        
        match f(&mut tx).await {
            Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                let _ = tx.rollback().await;
                Err(e)
            }
        }
    }
}

/// Get current timestamp for database updates
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}