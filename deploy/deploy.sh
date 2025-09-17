#!/bin/bash

# Automated Deployment Script for HackerExperience
# Supports multiple environments and zero-downtime deployment

set -e

# Configuration
ENVIRONMENT=${1:-staging}
VERSION=${2:-latest}
DEPLOY_USER=${DEPLOY_USER:-deploy}
HEALTH_CHECK_RETRIES=30
HEALTH_CHECK_INTERVAL=2

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "
╔══════════════════════════════════════════════╗
║     HACKEREXPERIENCE AUTOMATED DEPLOYMENT    ║
╚══════════════════════════════════════════════╝
"

echo "Environment: $ENVIRONMENT"
echo "Version: $VERSION"
echo ""

# Load environment config
if [ -f "deploy/config/$ENVIRONMENT.env" ]; then
    source deploy/config/$ENVIRONMENT.env
else
    echo -e "${RED}Error: Configuration file for $ENVIRONMENT not found${NC}"
    exit 1
fi

# Pre-deployment checks
pre_deployment_checks() {
    echo "Running pre-deployment checks..."

    # Check Docker is running
    if ! docker info > /dev/null 2>&1; then
        echo -e "${RED}Docker is not running${NC}"
        exit 1
    fi

    # Check database connectivity
    if ! docker-compose exec -T postgres pg_isready -U heuser > /dev/null 2>&1; then
        echo -e "${YELLOW}Warning: Database not accessible${NC}"
    fi

    # Check disk space
    DISK_USAGE=$(df -h / | awk 'NR==2 {print int($5)}')
    if [ $DISK_USAGE -gt 90 ]; then
        echo -e "${RED}Error: Disk usage is above 90%${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Pre-deployment checks passed${NC}"
}

# Backup database
backup_database() {
    echo "Backing up database..."

    BACKUP_DIR="/backups/hackerexperience"
    BACKUP_FILE="$BACKUP_DIR/backup_${ENVIRONMENT}_$(date +%Y%m%d_%H%M%S).sql"

    mkdir -p $BACKUP_DIR

    docker-compose exec -T postgres pg_dump -U heuser hackerexperience > $BACKUP_FILE

    if [ -f $BACKUP_FILE ]; then
        gzip $BACKUP_FILE
        echo -e "${GREEN}✓ Database backed up to $BACKUP_FILE.gz${NC}"

        # Keep only last 7 backups
        ls -t $BACKUP_DIR/*.gz | tail -n +8 | xargs -r rm
    else
        echo -e "${RED}Error: Database backup failed${NC}"
        exit 1
    fi
}

# Deploy backend with zero downtime
deploy_backend() {
    echo "Deploying backend..."

    # Pull new image
    docker pull hackerexperience/backend:$VERSION

    # Start new container alongside old one
    docker-compose up -d --no-deps --scale backend=2 backend

    # Wait for new container to be healthy
    echo "Waiting for new backend to be healthy..."
    for i in $(seq 1 $HEALTH_CHECK_RETRIES); do
        if curl -f http://localhost:3005/health > /dev/null 2>&1; then
            echo -e "${GREEN}✓ New backend is healthy${NC}"
            break
        fi
        echo -n "."
        sleep $HEALTH_CHECK_INTERVAL
    done

    # Remove old container
    docker-compose up -d --no-deps --remove-orphans backend

    echo -e "${GREEN}✓ Backend deployed${NC}"
}

# Deploy frontend
deploy_frontend() {
    echo "Deploying frontend..."

    docker pull hackerexperience/frontend:$VERSION
    docker-compose up -d --no-deps frontend

    echo -e "${GREEN}✓ Frontend deployed${NC}"
}

# Run migrations
run_migrations() {
    echo "Running database migrations..."

    docker-compose exec -T backend /app/he-api migrate

    echo -e "${GREEN}✓ Migrations completed${NC}"
}

# Post-deployment checks
post_deployment_checks() {
    echo "Running post-deployment checks..."

    # Check backend health
    if ! curl -f http://localhost:3005/health > /dev/null 2>&1; then
        echo -e "${RED}Error: Backend health check failed${NC}"
        rollback
        exit 1
    fi

    # Check frontend
    if ! curl -f http://localhost:8080 > /dev/null 2>&1; then
        echo -e "${RED}Error: Frontend not accessible${NC}"
        rollback
        exit 1
    fi

    # Check database connectivity
    if ! docker-compose exec -T backend curl -f http://localhost:3005/health > /dev/null 2>&1; then
        echo -e "${RED}Error: Backend cannot connect to database${NC}"
        rollback
        exit 1
    fi

    echo -e "${GREEN}✓ Post-deployment checks passed${NC}"
}

# Rollback on failure
rollback() {
    echo -e "${YELLOW}Rolling back deployment...${NC}"

    # Restore previous version
    docker-compose down
    docker-compose up -d

    echo -e "${YELLOW}Rollback completed${NC}"
}

# Send notification
send_notification() {
    local status=$1
    local message="Deployment to $ENVIRONMENT $status (Version: $VERSION)"

    # Slack notification
    if [ ! -z "$SLACK_WEBHOOK" ]; then
        curl -X POST -H 'Content-type: application/json' \
            --data "{\"text\":\"$message\"}" \
            $SLACK_WEBHOOK
    fi

    # Email notification
    if [ ! -z "$NOTIFY_EMAIL" ]; then
        echo "$message" | mail -s "HackerExperience Deployment" $NOTIFY_EMAIL
    fi
}

# Main deployment flow
main() {
    # Set error trap
    trap 'send_notification "FAILED"' ERR

    pre_deployment_checks
    backup_database

    # Stop cron jobs during deployment
    docker-compose exec -T backend supervisorctl stop cron || true

    # Deploy services
    deploy_backend
    deploy_frontend
    run_migrations

    # Restart cron jobs
    docker-compose exec -T backend supervisorctl start cron || true

    post_deployment_checks

    # Clean up old images
    docker system prune -af

    send_notification "SUCCESSFUL"

    echo "
╔══════════════════════════════════════════════╗
║          DEPLOYMENT SUCCESSFUL! 🚀           ║
╠══════════════════════════════════════════════╣
║  Environment: $ENVIRONMENT                   ║
║  Version: $VERSION                           ║
║  Status: LIVE                                ║
╚══════════════════════════════════════════════╝
"
}

# Run deployment
main