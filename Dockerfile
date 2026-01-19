FROM rust:1.92-slim-bullseye AS builder

WORKDIR /app

# Install required dependencies for building
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install sea-orm-cli for migrations
RUN cargo install sea-orm-cli

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

# ================================================================================

# Runtime stage
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y libssl-dev libpq-dev ca-certificates postgresql-client && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/debug/product-catalog-api /app/product-catalog-api
COPY --from=builder /usr/local/cargo/bin/sea-orm-cli /usr/local/bin/sea-orm-cli

CMD ["/app/product-catalog-api"]
