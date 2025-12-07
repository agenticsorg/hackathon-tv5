# Multi-stage build for Ingestion Service
FROM rust:1.74-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin media-gateway-ingestion
RUN rm -rf src

COPY src/ ./src/
RUN touch src/main.rs && cargo build --release --bin media-gateway-ingestion

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

COPY --from=builder /app/target/release/media-gateway-ingestion /usr/local/bin/media-gateway-ingestion

RUN mkdir -p /app/data && \
    chown -R appuser:appuser /app

USER appuser

EXPOSE 8085 9096

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8085/health || exit 1

ENTRYPOINT ["media-gateway-ingestion"]
