#!/bin/bash
# HackerExperience Database Setup Script

set -e

echo "🗄️  Setting up HackerExperience PostgreSQL databases..."

# Check if Docker is available
if command -v docker &> /dev/null && command -v docker-compose &> /dev/null; then
    echo "📦 Using Docker setup..."
    
    # Start PostgreSQL with Docker Compose
    docker-compose up -d postgres redis
    
    # Wait for PostgreSQL to be ready
    echo "⏳ Waiting for PostgreSQL to be ready..."
    timeout=60
    while ! docker-compose exec -T postgres pg_isready -U postgres; do
        timeout=$((timeout - 1))
        if [ $timeout -le 0 ]; then
            echo "❌ Timeout waiting for PostgreSQL"
            exit 1
        fi
        sleep 1
    done
    
    echo "✅ PostgreSQL is ready!"
    echo "✅ Redis is ready!"
    
    # Copy environment file
    if [ ! -f .env ]; then
        cp .env.example .env
        echo "📝 Created .env file from .env.example"
        echo "⚠️  Please update the passwords in .env for production!"
    fi
    
    echo ""
    echo "🎯 Database setup complete!"
    echo "🔗 Connection URLs:"
    echo "   Main DB: postgresql://he_app:secure_password_change_in_production@localhost:5432/hackerexperience"
    echo "   Redis:   redis://localhost:6379"
    echo ""
    echo "🚀 Ready to run: export DATABASE_URL='postgresql://he_app:secure_password_change_in_production@localhost:5432/hackerexperience'"
    echo "🚀 Then run: cargo build"
    
elif command -v psql &> /dev/null; then
    echo "🐘 Using local PostgreSQL installation..."
    
    # Check if PostgreSQL is running
    if ! pg_isready &> /dev/null; then
        echo "❌ PostgreSQL is not running. Please start it first:"
        echo "   sudo systemctl start postgresql"
        echo "   # or on macOS: brew services start postgresql"
        exit 1
    fi
    
    # Run the setup SQL
    echo "📝 Creating databases and tables..."
    psql -h localhost -U postgres -f database-setup.sql
    
    # Copy environment file and update for local setup
    if [ ! -f .env ]; then
        cp .env.example .env
        # Update for local PostgreSQL (assuming default postgres user)
        sed -i 's/he_app:secure_password_change_in_production@localhost/postgres@localhost/g' .env
        echo "📝 Created .env file configured for local PostgreSQL"
    fi
    
    echo ""
    echo "🎯 Database setup complete!"
    echo "🔗 Using local PostgreSQL connection"
    echo "🚀 Ready to run: export DATABASE_URL='postgresql://postgres@localhost:5432/hackerexperience'"
    echo "🚀 Then run: cargo build"
    
else
    echo "❌ Neither Docker nor PostgreSQL found!"
    echo ""
    echo "Please install one of the following:"
    echo "1. Docker & Docker Compose:"
    echo "   curl -fsSL https://get.docker.com | sh"
    echo "   sudo usermod -aG docker $USER"
    echo ""
    echo "2. PostgreSQL locally:"
    echo "   # Ubuntu/Debian:"
    echo "   sudo apt install postgresql postgresql-contrib"
    echo "   # macOS:"
    echo "   brew install postgresql"
    echo "   # Arch:"
    echo "   sudo pacman -S postgresql"
    echo ""
    echo "Then run this script again."
    exit 1
fi