FROM rust:1.77-slim as builder

WORKDIR /app

# Install required dependencies for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the Cargo files for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create a dummy main.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build && \
    rm -rf src

# Copy the real source code
COPY . .

# Build the application
RUN cargo build

# Install sqlx-cli for migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

# ================================================================================

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl-dev libpq-dev ca-certificates postgresql-client && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/debug/product-catalog-api /app/product-catalog-api

# Copy sqlx-cli for migrations
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx

# Copy migrations for SQLx
COPY --from=builder /app/migrations /app/migrations

# Set the entrypoint
CMD ["/app/product-catalog-api"]
