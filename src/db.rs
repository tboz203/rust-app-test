use anyhow::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

// Re-export Sea-ORM types for future use
pub use sea_orm::{ConnectOptions, Database as SeaORMDatabase, DatabaseConnection, DbErr, TransactionTrait};

/// Database connection pool wrapper
#[derive(Debug, Clone)]
pub struct Database {
    // Keep SQLx pool for backward compatibility
    pool: PgPool,
    // Add Sea-ORM connection
    conn: DatabaseConnection,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(database_url: &str) -> Result<Self> {
        // Initialize SQLx connection
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(3))
            .connect(database_url)
            .await?;

        // Initialize Sea-ORM connection
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(5)
           .min_connections(1)
           .connect_timeout(Duration::from_secs(3))
           .idle_timeout(Duration::from_secs(60))
           .sqlx_logging(true);
        let conn = SeaORMDatabase::connect(opt).await?;

        Ok(Self { pool, conn })
    }

    /// Create a new database instance from an existing pool
    pub fn from_pool(pool: PgPool) -> Self {
        // For backward compatibility, we create a database instance without Sea-ORM connection
        // This is not ideal but allows existing code to keep working
        Self {
            pool,
            conn: DatabaseConnection::Disconnected
        }
    }

    /// Get the connection pool (backward compatibility)
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
    
    /// Get the Sea-ORM connection
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// Execute a transaction using SQLx (backward compatibility)
    pub async fn transaction<F, Fut, R, E>(&self, f: F) -> Result<R, E>
    where
        F: for<'c> FnOnce(&'c mut sqlx::Transaction<'_, sqlx::Postgres>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, E>> + Send,
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