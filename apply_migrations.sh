#!/bin/bash

# Database Migration Script for HackerExperience
# This script applies all pending migrations to the PostgreSQL database

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}HackerExperience Database Migration Tool${NC}"
echo "========================================="
echo ""

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
elif [ -f .env.production ]; then
    export $(cat .env.production | grep -v '^#' | xargs)
else
    echo -e "${YELLOW}Warning: No .env file found. Using defaults.${NC}"
fi

# Set database URL
DATABASE_URL=${DATABASE_URL:-"postgres://hackerexp:password@localhost:5432/hackerexperience"}
echo -e "Database URL: ${DATABASE_URL}"
echo ""

# Function to run a migration
run_migration() {
    local migration_file=$1
    local migration_name=$(basename "$migration_file")

    echo -e "${YELLOW}Applying migration: ${migration_name}${NC}"

    if psql "$DATABASE_URL" -f "$migration_file" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Applied: ${migration_name}${NC}"

        # Record migration in history
        psql "$DATABASE_URL" -c "INSERT INTO _sqlx_migrations (version, description, installed_on) VALUES ('$(date +%s)', '${migration_name}', NOW()) ON CONFLICT DO NOTHING;" > /dev/null 2>&1
        return 0
    else
        echo -e "${RED}✗ Failed: ${migration_name}${NC}"
        return 1
    fi
}

# Create migrations table if it doesn't exist
echo -e "${YELLOW}Ensuring migrations table exists...${NC}"
psql "$DATABASE_URL" <<EOF > /dev/null 2>&1 || true
CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL DEFAULT true,
    checksum BYTEA,
    execution_time BIGINT
);
EOF

# Apply migrations in order
MIGRATION_DIRS=(
    "migrations"
    "migrations-postgres"
    "postgres-migrations"
)

echo -e "${YELLOW}Searching for migrations...${NC}"
echo ""

MIGRATIONS_APPLIED=0
MIGRATIONS_FAILED=0

for dir in "${MIGRATION_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "${GREEN}Found migration directory: ${dir}${NC}"

        # Get all SQL files and sort them
        for migration in $(ls "$dir"/*.sql 2>/dev/null | sort); do
            if run_migration "$migration"; then
                ((MIGRATIONS_APPLIED++))
            else
                ((MIGRATIONS_FAILED++))
                echo -e "${RED}Migration failed. Stopping execution.${NC}"
                exit 1
            fi
        done
    fi
done

# Apply the critical performance indexes we just created
CRITICAL_INDEX_FILE="migrations-postgres/20240918_critical_performance_indexes.sql"
if [ -f "$CRITICAL_INDEX_FILE" ]; then
    echo ""
    echo -e "${YELLOW}Applying critical performance indexes...${NC}"
    if run_migration "$CRITICAL_INDEX_FILE"; then
        ((MIGRATIONS_APPLIED++))
        echo -e "${GREEN}✓ Critical performance indexes applied successfully!${NC}"
    else
        ((MIGRATIONS_FAILED++))
        echo -e "${RED}✗ Failed to apply critical indexes. Manual intervention may be required.${NC}"
    fi
fi

# Summary
echo ""
echo "========================================="
echo -e "${GREEN}Migration Summary${NC}"
echo "========================================="
echo -e "Applied: ${GREEN}${MIGRATIONS_APPLIED}${NC} migrations"
if [ $MIGRATIONS_FAILED -gt 0 ]; then
    echo -e "Failed: ${RED}${MIGRATIONS_FAILED}${NC} migrations"
    exit 1
else
    echo -e "${GREEN}✓ All migrations applied successfully!${NC}"
fi

# Verify database health
echo ""
echo -e "${YELLOW}Verifying database health...${NC}"

# Check if we can connect and run a simple query
if psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" > /dev/null 2>&1; then
    TABLE_COUNT=$(psql -t -A "$DATABASE_URL" -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';")
    echo -e "${GREEN}✓ Database is healthy. Found ${TABLE_COUNT} tables.${NC}"
else
    echo -e "${RED}✗ Could not verify database health.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Database migration completed successfully!${NC}"