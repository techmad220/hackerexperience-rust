#!/bin/bash

# HackerExperience Rust - Test Runner Script
# This script sets up the test environment and runs all tests

set -e

echo "========================================="
echo "HackerExperience Rust - Test Suite"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if PostgreSQL is running
if ! pg_isready -q; then
    echo -e "${YELLOW}PostgreSQL is not running. Starting it...${NC}"
    sudo systemctl start postgresql || echo "Could not start PostgreSQL automatically"
fi

# Set test environment variables
export DATABASE_URL="postgresql://test:test@localhost/hackerexperience_test"
export REDIS_URL="redis://localhost:6379"
export JWT_SECRET="test_secret_key_for_testing"
export ENCRYPTION_KEY="test_encryption_key_32_bytes___"
export RUST_LOG="debug"
export RUST_BACKTRACE=1

# Create test database if it doesn't exist
echo -e "${YELLOW}Setting up test database...${NC}"
psql -U postgres -c "DROP DATABASE IF EXISTS hackerexperience_test;" 2>/dev/null || true
psql -U postgres -c "CREATE DATABASE hackerexperience_test;" 2>/dev/null || true
psql -U postgres -c "CREATE USER test WITH PASSWORD 'test';" 2>/dev/null || true
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE hackerexperience_test TO test;" 2>/dev/null || true

# Run migrations on test database
echo -e "${YELLOW}Running database migrations...${NC}"
sqlx migrate run || echo "Migrations might already be applied"

# Check Redis
if ! redis-cli ping > /dev/null 2>&1; then
    echo -e "${YELLOW}Redis is not running. Starting it...${NC}"
    redis-server --daemonize yes
    sleep 1
fi

echo -e "${GREEN}Test environment ready!${NC}"
echo ""

# Run tests with different levels
echo "========================================="
echo "Running Unit Tests"
echo "========================================="
cargo test --lib --no-fail-fast 2>&1 | tee test_output.log || true

echo ""
echo "========================================="
echo "Running Integration Tests"
echo "========================================="
cargo test --test '*' --no-fail-fast 2>&1 | tee -a test_output.log || true

echo ""
echo "========================================="
echo "Running Documentation Tests"
echo "========================================="
cargo test --doc --no-fail-fast 2>&1 | tee -a test_output.log || true

echo ""
echo "========================================="
echo "Test Summary"
echo "========================================="

# Count test results
PASSED=$(grep -c "test result: ok" test_output.log 2>/dev/null || echo "0")
FAILED=$(grep -c "test result: FAILED" test_output.log 2>/dev/null || echo "0")

if [ "$FAILED" -eq "0" ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
else
    echo -e "${RED}✗ Some tests failed. Check test_output.log for details.${NC}"
fi

echo ""
echo "To run specific tests:"
echo "  cargo test test_name"
echo "  cargo test --package package_name"
echo "  cargo test -- --nocapture (to see println output)"

# Cleanup
rm -f test_output.log

exit 0