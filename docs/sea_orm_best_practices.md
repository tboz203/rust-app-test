# Sea ORM Migration Best Practices

## 1. Introduction to Sea ORM Migrations

Sea ORM provides a robust migration framework through the `sea-orm-migration` crate, allowing developers to manage database schema changes in a controlled, version-controlled manner. Migrations in Sea ORM are designed to:

- Track changes to database schema over time
- Ensure consistent database states across different environments
- Support both forward (up) and rollback (down) operations
- Provide type safety through Rust's static typing system
- Allow for testing migrations before applying them to production

The migration framework is independent of the main Sea ORM library, allowing for flexible integration with different application architectures.

## 2. Best Practices for Implementing the Migrator Type

The `Migrator` type is the central component of Sea ORM's migration system. Here are best practices for implementing it:

### 2.1. Standard Implementation

```rust
use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // List all migrations in chronological order
        vec![
            Box::new(m20220101_000001_create_users_table::Migration),
            Box::new(m20220115_000001_create_posts_table::Migration),
            // Add future migrations here
        ]
    }
}
```

### 2.2. Key Best Practices

- **Complete Migration List**: Always include all migrations in the `migrations()` method, even historical ones
- **Chronological Order**: List migrations in chronological order to ensure they are applied correctly
- **No Dynamic Loading**: Avoid dynamically loading migrations at runtime; explicitly list them for reliability
- **Avoid Custom Methods**: Stick to the standard `MigratorTrait` implementation; custom methods may lead to inconsistencies
- **Version Tracking**: Let Sea ORM handle migration versioning through the standard mechanisms

### 2.3. Migration Manager Table

Sea ORM automatically creates and manages a `seaql_migrations` table to track which migrations have been applied. This table includes:

- `version`: The migration identifier
- `applied_at`: Timestamp when the migration was applied

## 3. How to Structure and Organize Migrations

### 3.1. Migration Module Naming

Follow this naming convention for migration modules:

```
m{date}_{time}_{description}
```

Example: `m20260117_000001_create_products_table`

- `m` - Prefix for all migrations
- `date` - YYYYMMDD format
- `time` - HHMMSS format
- `description` - Brief description using snake_case

### 3.2. Migration Structure

Each migration should implement the `MigrationTrait` and provide both `up()` and `down()` methods:

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table, add columns, etc.
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert the changes in the up method
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

// Define entity for better type safety
#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    CreatedAt,
}
```

### 3.3. Organization Recommendations

- **One Change Per Migration**: Each migration should focus on one logical change
- **Related Changes Together**: Group related changes in a single migration when appropriate
- **Directory Structure**: Store migrations in a dedicated `migration` directory, separate from application code
- **Module Organization**: Create a `mod.rs` file to export all migrations for the `Migrator`

## 4. Recommendations for Applying Migrations

### 4.1. Development Environment

In development environments, you have several options for applying migrations:

**Option 1: Use sea-orm-cli (Recommended)**

```bash
sea-orm-cli migrate up
```

**Option 2: Integrate with application startup**

```rust
use sea_orm_migration::MigratorTrait;

async fn main() -> Result<(), DbErr> {
    let db = Database::connect("database_url").await?;
    
    // Apply pending migrations
    Migrator::up(&db, None).await?;
    
    // Start application
    // ...
    
    Ok(())
}
```

**Option 3: Apply migrations in tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_with_migrations() {
        let db = Database::connect("test_database_url").await.unwrap();
        
        // Apply migrations for test database
        Migrator::up(&db, None).await.unwrap();
        
        // Perform test
        // ...
    }
}
```

### 4.2. Production Environment

For production deployments:

- **Pre-Deployment Migrations**: Apply migrations before deploying the new application version
- **Database Backup**: Always back up production databases before applying migrations
- **Downtime Consideration**: Plan for potential downtime during major schema changes
- **Explicit Version Control**: Use the explicit version parameter to control which migrations are applied

```bash
# Apply migrations up to a specific version
sea-orm-cli migrate up -v m20260117_000001_create_products_table
```

### 4.3. Managing Rollbacks

Be prepared for rollbacks in case of issues:

- **Test Rollbacks**: Verify that `down()` methods work correctly before deploying
- **Transaction Support**: Wrap complex migrations in transactions when possible
- **Limited Scope**: Keep migrations focused to minimize rollback complexity

## 5. Common sea-orm-cli Commands and Their Usage

The `sea-orm-cli` command line tool provides several commands for managing migrations:

### 5.1. Installation

```bash
cargo install sea-orm-cli
```

### 5.2. Generate a New Migration

```bash
sea-orm-cli migrate generate create_users_table
```

This generates a new migration file with the naming convention `m{date}_{time}_create_users_table`.

### 5.3. Apply Pending Migrations

```bash
sea-orm-cli migrate up
```

### 5.4. Rollback the Latest Migration

```bash
sea-orm-cli migrate down
```

### 5.5. Apply Migrations to a Specific Version

```bash
sea-orm-cli migrate up -v m20260117_000001_create_products_table
```

### 5.6. Fresh Start (Drop All Tables and Reapply)

```bash
sea-orm-cli migrate fresh
```

### 5.7. Status Check

```bash
sea-orm-cli migrate status
```

### 5.8. Generate Entity Files from Database

```bash
sea-orm-cli generate entity -o src/entity
```

## 6. Comparison of SQL Files vs. Sea ORM Migration Modules

### 6.1. SQL Files

**Advantages:**
- Direct SQL control
- Familiar syntax for SQL experts
- No abstraction layer
- Can leverage database-specific features

**Disadvantages:**
- No type safety
- Harder to manage rollbacks
- Requires manual tracking of which migrations have been applied
- No integration with Sea ORM's entity generation
- Limited programmatic control
- Environment-specific SQL might be needed

### 6.2. Sea ORM Migration Modules

**Advantages:**
- Type safety through Rust
- Cross-database compatibility
- Automatic version tracking
- Integration with Sea ORM entities
- Programmatic control for complex migrations
- Testable migrations
- Consistent up/down methods

**Disadvantages:**
- Learning curve for the schema builder syntax
- Some complex SQL features might require raw SQL

### 6.3. Recommendation

Sea ORM migration modules are strongly recommended over SQL files for the following reasons:

1. **Type Safety**: Catch errors at compile time
2. **Maintainability**: Consistent structure and organization
3. **Integration**: Better integration with the rest of the Sea ORM ecosystem
4. **Version Control**: Built-in version tracking
5. **Testing**: Ability to test migrations before applying them

## 7. Implementation Recommendations for Our Project

Based on the analysis of the current implementation in our project, here are specific recommendations:

### 7.1. Replace SQL Files with Migration Modules

Convert the existing `20260117000001_products_and_categories.sql` file into a proper Sea ORM migration module:

```rust
// migrations/src/m20260117_000001_products_and_categories.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create products table
        manager
            .create_table(
                Table::create()
                    .table(Products::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Products::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Products::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Products::Description).text())
                    .col(ColumnDef::new(Products::Price).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(Products::Sku).string_len(50).unique_key())
                    .col(ColumnDef::new(Products::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Products::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
            
        // Create categories table
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Categories::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Categories::Name).string_len(100).not_null().unique_key())
                    .col(ColumnDef::new(Categories::Description).text())
                    .col(ColumnDef::new(Categories::CreatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Categories::UpdatedAt).timestamp().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;
            
        // Create product_categories table
        manager
            .create_table(
                Table::create()
                    .table(ProductCategories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ProductCategories::ProductId).integer().not_null())
                    .col(ColumnDef::new(ProductCategories::CategoryId).integer().not_null())
                    .primary_key(Index::create().col(ProductCategories::ProductId).col(ProductCategories::CategoryId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategories::Table, ProductCategories::ProductId)
                            .to(Products::Table, Products::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategories::Table, ProductCategories::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;
            
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
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

// Define entities for better type safety
#[derive(Iden)]
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

#[derive(Iden)]
enum Categories {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ProductCategories {
    Table,
    ProductId,
    CategoryId,
}
```

### 7.2. Update the Migrator Implementation

Modify `src/migration.rs` to use the standard Sea ORM migration approach:

```rust
use sea_orm_migration::prelude::*;

mod m20260117_000001_products_and_categories;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260117_000001_products_and_categories::Migration),
            // Add future migrations here
        ]
    }
}
```

### 7.3. Integration with Application Startup

Consider adding migration application at application startup for development environments:

```rust
// In src/main.rs

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize tracing
    // ...

    // Load configuration
    let config = Config::from_env()?;

    // Set up database connection
    let db = Database::connect(&config.database_url).await?;
    
    // Apply migrations in development environments
    #[cfg(debug_assertions)]
    {
        use crate::migration::Migrator;
        use sea_orm_migration::MigratorTrait;
        
        Migrator::up(&db, None).await?;
    }

    // Build our application with routes
    // ...
    
    Ok(())
}
```

### 7.4. Setup Migration Project Structure

For larger projects, consider creating a dedicated migration crate:

```
my-project/
├── migration/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── m20260117_000001_products_and_categories.rs
│       └── (other migration files)
├── entity/
│   ├── Cargo.toml
│   └── src/
│       └── (entity files)
└── api/
    ├── Cargo.toml
    └── src/
        └── (api implementation)
```

### 7.5. Continuous Integration

Add migration checks to CI pipelines:

```yaml
# Example GitHub Actions workflow
jobs:
  migration-check:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_USER: postgres
          POSTGRES_DB: test_db
        ports:
          - 5432:5432
    steps:
      - uses: actions/checkout@v3
      - name: Install sea-orm-cli
        run: cargo install sea-orm-cli
      - name: Run migrations
        run: sea-orm-cli migrate up
        env:
          DATABASE_URL: postgres://postgres:postgres@localhost:5432/test_db
```

## Conclusion

Following these best practices for Sea ORM migrations will lead to a more maintainable, type-safe, and reliable database schema management system. By leveraging Rust's strong type system and Sea ORM's migration framework, we can ensure consistent database states across different environments and track changes effectively over time.

For our specific project, transitioning from SQL files to Sea ORM migration modules will provide significant benefits in terms of type safety, integration with the rest of the codebase, and better handling of schema versioning.