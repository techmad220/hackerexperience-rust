#!/bin/bash

# ================================================
# HackerExperience - Database Setup Verification
# ================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Configuration
DB_NAME="hackerexperience"
DB_USER="he_app"
DB_HOST="localhost"
DB_PORT="5432"

# Functions
log_header() {
    echo ""
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}${BOLD}  $1${NC}"
    echo -e "${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

# Main verification
log_header "Database Setup Verification"

# 1. Check PostgreSQL availability
echo -e "\n${BOLD}Checking PostgreSQL...${NC}"
if command -v psql &> /dev/null; then
    log_success "PostgreSQL client installed"
else
    log_error "PostgreSQL client not found"
    echo "    Install with: sudo apt-get install postgresql-client"
    exit 1
fi

# 2. Check if PostgreSQL is running
if command -v docker &> /dev/null; then
    if docker ps | grep -q postgres; then
        log_success "PostgreSQL container is running"
        USING_DOCKER=true
    else
        log_warning "No PostgreSQL container found"
        USING_DOCKER=false
    fi
else
    USING_DOCKER=false
fi

if ! $USING_DOCKER; then
    if pg_isready -h $DB_HOST -p $DB_PORT &> /dev/null; then
        log_success "PostgreSQL server is running on $DB_HOST:$DB_PORT"
    else
        log_error "PostgreSQL server not accessible"
        echo "    Start with: sudo service postgresql start"
        echo "    Or run: docker-compose up -d postgres"
        exit 1
    fi
fi

# 3. Check environment variables
echo -e "\n${BOLD}Checking environment configuration...${NC}"
if [ -f .env ]; then
    log_success ".env file exists"
    source .env

    if [ -n "$DATABASE_URL" ]; then
        log_success "DATABASE_URL is configured"
        # Extract connection details from DATABASE_URL
        if [[ "$DATABASE_URL" =~ postgresql://([^:]+):([^@]+)@([^:]+):([^/]+)/(.+) ]]; then
            DB_USER="${BASH_REMATCH[1]}"
            DB_PASS="${BASH_REMATCH[2]}"
            DB_HOST="${BASH_REMATCH[3]}"
            DB_PORT="${BASH_REMATCH[4]}"
            DB_NAME="${BASH_REMATCH[5]}"
            log_info "Connection: $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
        fi
    else
        log_warning "DATABASE_URL not set in .env"
        echo "    Add: DATABASE_URL=postgresql://he_app:password@localhost:5432/hackerexperience"
    fi
else
    log_error ".env file not found"
    if [ -f .env.example ]; then
        echo "    Create with: cp .env.example .env"
    else
        echo "    Create a .env file with DATABASE_URL configuration"
    fi
    exit 1
fi

# 4. Test database connection
echo -e "\n${BOLD}Testing database connection...${NC}"
if PGPASSWORD="${DB_PASS:-secure_password_change_in_production}" psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c '\l' &> /dev/null; then
    log_success "Successfully connected to database '$DB_NAME'"
else
    log_error "Failed to connect to database"
    echo "    Check your credentials in .env"
    echo "    Ensure database '$DB_NAME' exists"
    echo "    Create with: createdb -h $DB_HOST -U postgres $DB_NAME"
    exit 1
fi

# 5. Check for migrations
echo -e "\n${BOLD}Checking migrations...${NC}"
if [ -d "migrations" ] || [ -d "migrations-postgres" ]; then
    log_success "Migration directory found"

    # Check if sqlx is installed
    if ~/.cargo/bin/sqlx --version &> /dev/null; then
        log_success "sqlx-cli is installed"

        # Check migration status
        if SQLX_OFFLINE=true ~/.cargo/bin/sqlx migrate info --database-url "$DATABASE_URL" &> /dev/null; then
            log_success "Migrations are accessible"
        else
            log_warning "Could not check migration status"
            echo "    Run migrations with: ~/.cargo/bin/sqlx migrate run"
        fi
    else
        log_warning "sqlx-cli not installed"
        echo "    Install with: ~/.cargo/bin/cargo install sqlx-cli --features postgres"
    fi
else
    log_warning "No migration directory found"
fi

# 6. Check Redis (optional)
echo -e "\n${BOLD}Checking Redis (optional)...${NC}"
if command -v redis-cli &> /dev/null; then
    if redis-cli ping &> /dev/null; then
        log_success "Redis is running and accessible"
    else
        log_warning "Redis not running (optional for development)"
    fi
else
    log_info "Redis client not installed (optional)"
fi

# 7. Final summary
echo ""
log_header "Verification Summary"

READY=true

# Check essentials
if ! pg_isready -h $DB_HOST -p $DB_PORT &> /dev/null && ! docker ps | grep -q postgres; then
    log_error "PostgreSQL is not running"
    READY=false
fi

if [ ! -f .env ] || [ -z "$DATABASE_URL" ]; then
    log_error "Environment not configured"
    READY=false
fi

if $READY; then
    echo -e "${GREEN}${BOLD}✅ Database setup is ready!${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Run migrations: ~/.cargo/bin/sqlx migrate run"
    echo "  2. Build project: ~/.cargo/bin/cargo build --release"
    echo "  3. Run server: ~/.cargo/bin/cargo run --bin he-api"
else
    echo -e "${RED}${BOLD}❌ Database setup incomplete${NC}"
    echo ""
    echo "Fix the issues above and run this script again."
    exit 1
fi

echo ""
echo -e "${CYAN}Connection URL:${NC}"
echo "  $DATABASE_URL"
echo ""