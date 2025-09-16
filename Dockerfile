# Multi-stage build for optimal image size
# Stage 1: Builder
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/app

# Copy Cargo files first for better caching
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build dependencies first (for caching)
RUN cargo build --release --workspace

# Copy source code
COPY . .

# Build the application
RUN cargo build --release --bin he-api

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash hacker

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/he-api /usr/local/bin/he-api

# Copy static files and migrations
COPY --from=builder /usr/src/app/frontend /app/frontend
COPY --from=builder /usr/src/app/migrations /app/migrations
COPY --from=builder /usr/src/app/.sqlx /app/.sqlx

# Set working directory
WORKDIR /app

# Change ownership
RUN chown -R hacker:hacker /app

# Switch to non-root user
USER hacker

# Environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=""
ENV JWT_SECRET=""
ENV REDIS_URL="redis://localhost:6379"
ENV PORT=3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Expose port
EXPOSE 3000

# Run the application
CMD ["he-api"]