# Product Catalog API

A RESTful API for managing product catalog data built with Rust and Axum.

## Overview

This API provides endpoints to manage a product catalog with categories and products. It supports standard CRUD operations for both products and categories, with features like pagination, filtering, and relationships between entities.

## Features

- **Category Management**: Create, read, update, and delete product categories
- **Product Management**: Create, read, update, and delete products with category associations
- **Validation**: Input validation for all API requests
- **Error Handling**: Comprehensive error handling with appropriate HTTP status codes
- **Database Integration**: PostgreSQL database with SQLx for type-safe queries
- **Pagination**: Support for paginated responses
- **Documentation**: API documentation with examples

## Requirements

- Rust 1.70 or later
- PostgreSQL 14 or later
- Cargo package manager

## Dependencies

Major dependencies include:

- **Axum**: Web framework for building the API
- **SQLx**: Database toolkit with compile-time checked queries
- **Tokio**: Async runtime
- **Serde**: Serialization/deserialization
- **Validator**: Request validation
- **Tracing**: Logging and instrumentation

See `Cargo.toml` for the complete list of dependencies.

## Setup Instructions

### 1. Clone the repository

```bash
git clone <repository-url>
cd product-catalog-api
```

### 2. Set up the database

Create a PostgreSQL database and user:

```bash
psql -U postgres
```

```sql
CREATE DATABASE product_catalog;
CREATE USER catalog_user WITH ENCRYPTED PASSWORD 'your_password';
GRANT ALL PRIVILEGES ON DATABASE product_catalog TO catalog_user;
```

### 3. Configure environment variables

Create a `.env` file in the project root with the following variables:

```
# Database configuration
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=catalog_user
POSTGRES_PASSWORD=your_password
POSTGRES_DB=product_catalog

# Server configuration
SERVER_HOST=localhost
SERVER_PORT=3000
RUST_LOG=info
```

### 4. Run database migrations

```bash
sqlx migrate run
```

### 5. Build and run the application

```bash
cargo build --release
cargo run --release
```

The API will be available at `http://localhost:3000` (or the port specified in your .env file).

## Configuration Options

The application can be configured using environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_HOST` | Database server hostname | localhost |
| `POSTGRES_PORT` | Database server port | 5432 |
| `POSTGRES_USER` | Database username | postgres |
| `POSTGRES_PASSWORD` | Database password | postgres |
| `POSTGRES_DB` | Database name | product_catalog |
| `SERVER_HOST` | Host to bind the server to | 127.0.0.1 |
| `SERVER_PORT` | Port for the HTTP server | 3000 |
| `RUST_LOG` | Log level (error, warn, info, debug, trace) | info |
| `DATABASE_MAX_CONNECTIONS` | Maximum database connections | 5 |
| `CORS_ALLOWED_ORIGINS` | Comma-separated list of allowed origins | * |

## Testing

Run the test suite with:

```bash
cargo test
```

The test suite includes integration tests for all API endpoints.

## API Endpoints

See the [API Documentation](docs/api.md) for detailed information about available endpoints, request/response formats, and examples.

## License

This project is licensed under the MIT License - see the LICENSE file for details.