# Multi-stage Docker build for Neo CRM Backend

# Build stage
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install dependencies for SSL (PostgreSQL via sqlx)
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer - speeds up rebuilds)
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/hash_password.rs && \
    cargo build --release --bin crm-backend && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy compiled binary from builder
COPY --from=builder /app/target/release/crm-backend /app/crm-backend

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Create necessary directories
RUN mkdir -p /app/data /app/uploads

# Expose ports
# 3000 - API server
# 9090 - Metrics endpoint (Prometheus)
EXPOSE 3000 9090

# Set environment variables
# Overridden by docker-compose (hostname `postgres`)
ENV DATABASE_URL=postgresql://crm_app:crm_app@postgres:5432/crm \
    RUST_LOG=info \
    UPLOAD_DIR=/app/uploads

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["/app/crm-backend"]
