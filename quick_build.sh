#!/bin/bash

# ================================================
# HackerExperience Rust - Quick Build Script
# ================================================
# Fast build for development

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

echo -e "${CYAN}${BOLD}HackerExperience - Quick Build${NC}"
echo "================================"
echo ""

# Set environment
if [ -f .env ]; then
    source .env
elif [ -f .env.production ]; then
    source .env.production
else
    export DATABASE_URL="postgresql://heuser:hepass@localhost:5432/hedb"
    export SQLX_OFFLINE=true
fi

# Quick checks
echo -e "${CYAN}Running quick checks...${NC}"
~/.cargo/bin/cargo check --bin he-api 2>&1 | grep -E "error\[" || true

# Build main binary
echo ""
echo -e "${CYAN}Building he-api...${NC}"
~/.cargo/bin/cargo build --bin he-api

# Report
echo ""
echo -e "${GREEN}${BOLD}✓ Build Complete!${NC}"
echo ""
echo "Run with:"
echo "  ~/.cargo/bin/cargo run --bin he-api"
echo ""
echo "Or directly:"
echo "  ./target/debug/he-api"
echo ""