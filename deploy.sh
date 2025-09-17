#!/bin/bash

# HackerExperience Production Deployment Script
set -euo pipefail

echo "🚀 Starting HackerExperience deployment..."

# Load environment
if [ -f .env.production ]; then
    export $(cat .env.production | grep -v '^#' | xargs)
else
    echo "L .env.production not found!"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "=� Checking prerequisites..."
if ! command_exists docker; then
    echo "L Docker not installed"
    exit 1
fi

if ! command_exists docker-compose; then
    echo "L Docker Compose not installed"
    exit 1
fi

# Build containers
echo "=( Building containers..."
docker-compose build --parallel

# Run database migrations
echo "=� Running database migrations..."
docker-compose run --rm api /app/he-api migrate

# Start services
echo "<� Starting services..."
docker-compose up -d

# Wait for services to be healthy
echo "� Waiting for services to be healthy..."
sleep 10

# Check service health
echo "<� Checking service health..."
docker-compose ps

# Run smoke tests
echo ">� Running smoke tests..."
curl -f http://localhost:3000/health || exit 1
curl -f http://localhost:8080 || exit 1

echo " Deployment complete!"
echo ""
echo "Services running at:"
echo "  - Frontend: http://localhost:8080"
echo "  - API: http://localhost:3000"
echo "  - WebSocket: ws://localhost:3001"
echo ""
echo "To view logs: docker-compose logs -f"
echo "To stop services: docker-compose down"