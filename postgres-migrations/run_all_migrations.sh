#!/bin/bash

# HackerExperience PostgreSQL Migration Runner
# This script runs all database migrations in the correct order

set -e  # Exit on any error

# Configuration
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-hackerexperience_rust}"
DB_USER="${DB_USER:-he_app}"
DB_PASSWORD="${DB_PASSWORD:-he_secure_password_change_in_production}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if PostgreSQL is available
check_postgres() {
    log "Checking PostgreSQL connection..."
    if ! pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" >/dev/null 2>&1; then
        error "Cannot connect to PostgreSQL at $DB_HOST:$DB_PORT"
        error "Please ensure PostgreSQL is running and accessible"
        exit 1
    fi
    success "PostgreSQL connection verified"
}

# Function to run a single migration
run_migration() {
    local migration_file="$1"
    local migration_name=$(basename "$migration_file" .sql)
    
    log "Running migration: $migration_name"
    
    # Record start time
    local start_time=$(date +%s%3N)
    
    # Run the migration
    if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$migration_file" -v ON_ERROR_STOP=1 -q; then
        # Calculate execution time
        local end_time=$(date +%s%3N)
        local execution_time=$((end_time - start_time))
        
        # Record migration in tracking table
        PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "
            INSERT INTO schema_migrations (version, description, execution_time_ms) 
            VALUES ('$migration_name', 'Migration: $migration_name', $execution_time)
            ON CONFLICT (version) DO UPDATE SET 
                applied_at = CURRENT_TIMESTAMP,
                execution_time_ms = $execution_time;
        " -q
        
        success "Migration $migration_name completed in ${execution_time}ms"
    else
        error "Migration $migration_name failed"
        return 1
    fi
}

# Check if migration was already applied
is_migration_applied() {
    local migration_name="$1"
    local count=$(PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c "
        SELECT COUNT(*) FROM schema_migrations WHERE version = '$migration_name';
    " 2>/dev/null || echo "0")
    
    [ "$count" -gt 0 ]
}

# Main function
main() {
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}  HackerExperience Database Migration Runner    ${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo
    
    # Get the directory where this script is located
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    
    log "Migration directory: $SCRIPT_DIR"
    log "Database: $DB_NAME @ $DB_HOST:$DB_PORT"
    log "User: $DB_USER"
    echo
    
    # Check PostgreSQL connection
    check_postgres
    echo
    
    # Check if we need to run the initial setup
    if ! PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -l | grep -q "$DB_NAME"; then
        warning "Database $DB_NAME does not exist. Running setup first..."
        if ! PGPASSWORD="postgres" psql -h "$DB_HOST" -p "$DB_PORT" -U "postgres" -f "$SCRIPT_DIR/000_setup_database.sql" -v ON_ERROR_STOP=1; then
            error "Failed to run database setup. Please run 000_setup_database.sql manually as postgres superuser"
            exit 1
        fi
        success "Database setup completed"
        echo
    fi
    
    # Find all migration files
    migration_files=($(find "$SCRIPT_DIR" -name "*.sql" -not -name "000_setup_database.sql" | sort))
    
    if [ ${#migration_files[@]} -eq 0 ]; then
        error "No migration files found in $SCRIPT_DIR"
        exit 1
    fi
    
    log "Found ${#migration_files[@]} migration files"
    echo
    
    # Run migrations
    local applied_count=0
    local skipped_count=0
    local failed_count=0
    
    for migration_file in "${migration_files[@]}"; do
        migration_name=$(basename "$migration_file" .sql)
        
        if is_migration_applied "$migration_name"; then
            warning "Migration $migration_name already applied, skipping"
            ((skipped_count++))
        else
            if run_migration "$migration_file"; then
                ((applied_count++))
            else
                ((failed_count++))
                error "Migration failed, stopping"
                break
            fi
        fi
        echo
    done
    
    # Summary
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}              Migration Summary                 ${NC}"
    echo -e "${BLUE}================================================${NC}"
    success "Applied: $applied_count"
    if [ $skipped_count -gt 0 ]; then
        warning "Skipped: $skipped_count"
    fi
    if [ $failed_count -gt 0 ]; then
        error "Failed: $failed_count"
    fi
    echo
    
    if [ $failed_count -eq 0 ]; then
        success "All migrations completed successfully!"
        
        # Initialize sample data if requested
        if [ "$1" = "--with-sample-data" ]; then
            log "Initializing sample data..."
            PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT initialize_sample_data();" -q
            success "Sample data initialized"
        fi
        
        # Run database maintenance
        log "Running initial database maintenance..."
        PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT * FROM perform_database_maintenance();" -q
        success "Database maintenance completed"
        
        echo
        echo -e "${GREEN}🚀 HackerExperience database is ready!${NC}"
        echo -e "${GREEN}   Connection string: postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME${NC}"
        echo
    else
        error "Some migrations failed. Please check the errors above."
        exit 1
    fi
}

# Handle command line arguments
case "$1" in
    --help|-h)
        echo "Usage: $0 [options]"
        echo ""
        echo "Options:"
        echo "  --with-sample-data    Initialize with sample data for testing"
        echo "  --help, -h           Show this help message"
        echo ""
        echo "Environment variables:"
        echo "  DB_HOST              Database host (default: localhost)"
        echo "  DB_PORT              Database port (default: 5432)"
        echo "  DB_NAME              Database name (default: hackerexperience_rust)"
        echo "  DB_USER              Database user (default: he_app)"
        echo "  DB_PASSWORD          Database password"
        echo ""
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac