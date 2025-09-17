# Multi-stage Dockerfile for HackerExperience

# Base builder stage
FROM rust:1.75-slim AS builder
WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY he-*/ ./

# Build all binaries in release mode
RUN cargo build --release

# API service stage
FROM debian:bookworm-slim AS api
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/he-api /app/he-api

# Create non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 3000
CMD ["./he-api"]

# WebSocket service stage
FROM debian:bookworm-slim AS websocket
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/he-websocket /app/he-websocket

# Create non-root user
RUN useradd -m -u 1000 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 3001
CMD ["./he-websocket"]

# Frontend build stage
FROM node:20-slim AS frontend-builder
WORKDIR /app

# Install wasm-pack
RUN npm install -g wasm-pack

# Copy frontend source
COPY crates/he-leptos-frontend/ ./

# Build WASM frontend
RUN wasm-pack build --target web --out-dir pkg
RUN npm install && npm run build

# Frontend serve stage
FROM nginx:alpine AS frontend
COPY --from=frontend-builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80