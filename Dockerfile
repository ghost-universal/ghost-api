# Ghost API Server
#
# Multi-stage build for optimized production image

# ============================================
# Build stage
# ============================================
FROM rust:1.78-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ghost-core/Cargo.toml ./crates/ghost-core/
COPY crates/ghost-schema/Cargo.toml ./crates/ghost-schema/
COPY crates/ghost-bridge/Cargo.toml ./crates/ghost-bridge/
COPY crates/ghost-vault/Cargo.toml ./crates/ghost-vault/
COPY crates/ghost-server/Cargo.toml ./crates/ghost-server/
COPY crates/ghost-adapters/x-adapter/Cargo.toml ./crates/ghost-adapters/x-adapter/
COPY crates/ghost-adapters/threads-adapter/Cargo.toml ./crates/ghost-adapters/threads-adapter/

# Create dummy main.rs to build dependencies
RUN mkdir -p crates/ghost-server/src && \
    echo "fn main() {}" > crates/ghost-server/src/main.rs && \
    mkdir -p crates/ghost-core/src && touch crates/ghost-core/src/lib.rs && \
    mkdir -p crates/ghost-schema/src && touch crates/ghost-schema/src/lib.rs && \
    mkdir -p crates/ghost-bridge/src && touch crates/ghost-bridge/src/lib.rs && \
    mkdir -p crates/ghost-vault/src && touch crates/ghost-vault/src/lib.rs && \
    mkdir -p crates/ghost-adapters/x-adapter/src && touch crates/ghost-adapters/x-adapter/src/lib.rs && \
    mkdir -p crates/ghost-adapters/threads-adapter/src && touch crates/ghost-adapters/threads-adapter/src/lib.rs

# Build dependencies
RUN cargo build --release -p ghost-server

# Copy actual source code
COPY crates/ ./crates/

# Build the application
# TODO: Uncomment when actual code is ready
# RUN cargo build --release -p ghost-server

# ============================================
# Runtime stage
# ============================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false ghost

# Copy the binary from builder
# TODO: Uncomment when binary is built
# COPY --from=builder /app/target/release/ghost-server /usr/local/bin/

# Copy configuration
COPY config/ /etc/ghost-api/

# Create directories for data
RUN mkdir -p /var/lib/ghost-api && chown ghost:ghost /var/lib/ghost-api

# Switch to non-root user
USER ghost

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV GHOST_API_CONFIG=/etc/ghost-api/ghost.toml

# Run the server
# TODO: Uncomment when binary is ready
# CMD ["ghost-server"]
CMD ["echo", "Ghost API - Build not yet complete. Implement the application first."]
