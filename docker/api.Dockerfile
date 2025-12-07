# Multi-stage build for Media Gateway API
FROM rust:1.74-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build dependencies (cached layer)
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin media-gateway-api
RUN rm -rf src

# Build application
COPY src/ ./src/
RUN touch src/main.rs && cargo build --release --bin media-gateway-api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/media-gateway-api /usr/local/bin/media-gateway-api

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set entrypoint
ENTRYPOINT ["media-gateway-api"]
