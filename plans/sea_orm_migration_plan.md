# Sea-ORM Migration Plan

## Overview

This document outlines the strategy for migrating the current database access layer from SQLx to Sea-ORM. The migration will involve changes to dependencies, connection management, entity definitions, repository implementations, and migration handling.

## Current Architecture

The application currently uses:

- **SQLx** for database access with raw SQL queries
- Custom `Database` wrapper for connection pooling
- Model structs with `#[derive(FromRow)]` for query results
- Repository pattern with imperative SQL query building
- SQL-based migrations

## 1. Required Dependencies

Add the following dependencies to `Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...

# Sea-ORM
sea-orm = { version = "0.12", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
    "macros",
    "with-chrono",
    "with-uuid",
    "with-json",
    "with-bigdecimal",
    "postgres-array",
] }
sea-orm-migration = "0.12"
```

## 2. Database Connection Layer Changes

### Current Implementation

The current `db.rs` provides:

- A `Database` struct wrapping `PgPool`
- Connection pooling via `PgPoolOptions`
- Transaction management
- Helper function `now()` for timestamps

### Sea-ORM Implementation

Update `src/db.rs` to:

```rust
use anyhow::Result;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr, TransactionTrait};
use std::time::Duration;

/// Database connection wrapper
#[derive(Debug, Clone)]
pub struct AppDatabase {
    conn: DatabaseConnection,
}

impl AppDatabase {
    /// Create a new database connection
    pub async fn connect(database_url: &str) -> Result<Self> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(5)
           .min_connections(1)
           .connect_timeout(Duration::from_secs(3))
           .idle_timeout(Duration::from_secs(60))
           .sqlx_logging(true);

        let conn = Database::connect(opt).await?;
        Ok(Self { conn })
    }
    
    /// Get the database connection
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }
    
    /// Create a new database instance from an existing connection
    pub fn from_connection(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Execute a transaction
    pub async fn transaction<F, T, E>(&self, f: F) -> Result<T, E> 
    where
        F: for<'a> FnOnce(&'a DatabaseConnection) -> BoxFuture<'a, Result<T, E>>,
        E: From<DbErr> + std::error::Error,
        T: Send + 'static,
    {
        self.conn
            .transaction(|txn| Box::pin(f(txn)))
            .await
            .map_err(Into::into)
    }
}

/// Get current timestamp for database updates
pub fn now() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}
```

## 3. Entity Generation and Management

### Strategy

1. Generate Sea-ORM entities from the existing database schema
2. Create an `entity` module to organize and expose these entities
3. Map existing model types to Sea-ORM entities

### Implementation Steps

1. Install Sea-ORM CLI:
   ```bash
   cargo install sea-orm-cli
   ```

2. Generate entities:
   ```bash
   sea-orm-cli generate entity -o src/entity
   ```

3. Create a new module in `src/entity.rs`:
   ```rust
   pub mod prelude;
   pub mod product;
   pub mod category;
   pub mod product_category;
   
   pub use prelude::*;
   ```

## 4. Repository Implementation Changes

### General Pattern

Convert each repository to use Sea-ORM query builders instead of raw SQL:

1. Replace SQL queries with Sea-ORM's query DSL
2. Use Sea-ORM's ActiveModel pattern for inserts and updates
3. Use relations to handle foreign key relationships
4. Use transactions through Sea-ORM's transaction API

### Product Repository Example

```rust
use crate::db::AppDatabase;
use crate::entity::{product, product_category, category};
use crate::entity::prelude::{Product, Category, ProductCategory};
use crate::error::ApiError;
use crate::models::product::{
    CategoryBrief, CreateProductRequest, ProductListResponse, 
    ProductQueryParams, ProductResponse, UpdateProductRequest,
};
use anyhow::Result;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, 
    EntityTrait, ModelTrait, QueryFilter, Set, TransactionTrait,
    RelationTrait, PaginatorTrait,
};

#[derive(Clone)]
pub struct ProductRepository {
    db: AppDatabase,
}

impl ProductRepository {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    // Example of converted create_product method
    pub async fn create_product(
        &self, 
        req: CreateProductRequest
    ) -> Result<ProductResponse, ApiError> {
        let db = self.db.conn();
        
        let result = db.transaction::<_, ProductResponse, ApiError>(|txn| {
            Box::pin(async move {
                // Create product
                let product = product::ActiveModel {
                    name: Set(req.name),
                    description: Set(req.description),
                    price: Set(req.price),
                    sku: Set(req.sku),
                    ..Default::default()
                };
                
                let product_model = product.insert(txn).await?;
                
                // Insert product categories
                for category_id in &req.category_ids {
                    let product_category = product_category::ActiveModel {
                        product_id: Set(product_model.id),
                        category_id: Set(*category_id),
                        ..Default::default()
                    };
                    product_category.insert(txn).await?;
                }
                
                // Load categories for response
                let categories = self.get_product_categories(product_model.id, txn).await?;
                
                Ok(ProductResponse {
                    id: product_model.id,
                    name: product_model.name,
                    description: product_model.description,
                    price: product_model.price,
                    sku: product_model.sku,
                    categories,
                    created_at: product_model.created_at,
                    updated_at: product_model.updated_at,
                })
            })
        }).await?;
        
        Ok(result)
    }
    
    // Other methods would be similarly converted...
    
    async fn get_product_categories<'c>(
        &self,
        product_id: i32,
        db: &DatabaseConnection,
    ) -> Result<Vec<CategoryBrief>, DbErr> {
        // Using relations to fetch related categories
        let product = Product::find_by_id(product_id)
            .find_with_related(Category)
            .all(db)
            .await?;
            
        let categories = product
            .into_iter()
            .flat_map(|(_, categories)| categories)
            .map(|c| CategoryBrief {
                id: c.id,
                name: c.name,
            })
            .collect();
            
        Ok(categories)
    }
}
```

## 5. Migration Management Changes

### Current Implementation

- SQL files in the `migrations` directory
- Manual SQL schema definitions
- Using SQLx migration tools

### Sea-ORM Migration Implementation

1. Create a separate migrator crate or integration:

```rust
// src/migrator/mod.rs
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260117_000001_products_and_categories::Migration),
        ]
    }
}

mod m20260117_000001_products_and_categories;
```

2. Implement the migration:

```rust
// src/migrator/m20260117_000001_products_and_categories.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Products::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Products::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Products::Description).text())
                    .col(ColumnDef::new(Products::Price).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Products::Sku).string_len(50).unique_key())
                    .col(
                        ColumnDef::new(Products::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Products::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;
            
        // Add categories and product_categories tables...
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProductCategories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    Name,
    Description,
    Price,
    Sku,
    CreatedAt,
    UpdatedAt,
}

// Define other tables...
```

## 6. API Impact Assessment

### Expected API Changes

- **No changes to external API signatures**: The repository interfaces will remain unchanged from an API consumer perspective
- **No changes to request/response model structures**: The existing model structures can be retained

### Internal Implementation Changes

- Repository methods will need to convert between domain models and Sea-ORM entities
- Transaction handling will use the Sea-ORM approach
- Query construction will use Sea-ORM's DSL instead of raw SQL

## 7. Implementation Strategy

### Phase 1: Environment Setup

1. Add Sea-ORM dependencies to Cargo.toml
2. Generate entities from the existing database schema
3. Update database connection management in db.rs
4. Implement a prototype for a single method to validate approach

### Phase 2: Repository Migration

1. Migrate ProductRepository methods one by one
2. Migrate CategoryRepository methods one by one
3. Add tests for each converted method
4. Run existing tests against new implementation

### Phase 3: Migration Integration

1. Set up Sea-ORM migration system
2. Convert existing SQL migrations to Sea-ORM format
3. Test migration functionality

### Phase 4: Validation and Cleanup

1. Ensure all tests pass with the new implementation
2. Refactor common patterns into helper functions
3. Remove any legacy SQLx-specific code
4. Update documentation

## Conclusion

This migration will modernize the database layer by:

1. Utilizing Sea-ORM's type-safe query building
2. Improving maintainability with the ActiveModel pattern
3. Reducing boilerplate through relationship handling
4. Making future schema changes easier to manage
5. Preserving existing API contracts and business logic

The expected benefits include reduced error-prone manual SQL, better compile-time checking, improved database access patterns, and easier integration with ecosystem libraries that work with Sea-ORM.