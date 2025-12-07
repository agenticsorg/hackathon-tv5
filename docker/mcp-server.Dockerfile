# MCP Server Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/core ./crates/core
COPY crates/mcp-server ./crates/mcp-server

# Build release binary
RUN cargo build --release --bin mcp-server

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/mcp-server /usr/local/bin/mcp-server

# Create non-root user
RUN useradd -m -u 1000 mcp && \
    chown -R mcp:mcp /app

USER mcp

# Expose MCP port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["mcp-server"]
