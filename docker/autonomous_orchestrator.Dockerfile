# Autonomous Orchestrator Dockerfile
# Multi-stage build for optimized image size

# Stage 1: Build
FROM rust:1.85 as builder

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies first (for caching)
RUN cargo build --release --package autonomous_orchestrator --locked

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/autonomous_orchestrator /app/autonomous_orchestrator

# Copy configuration
COPY config/autonomous_orchestrator.toml /app/config/autonomous_orchestrator.toml

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 8090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8090/api/v1/health || exit 1

# Run the application
ENTRYPOINT ["/app/autonomous_orchestrator"]
