#!/bin/bash

# Initialize PostgreSQL database for HackerExperience

set -e

echo "🔧 Initializing HackerExperience Database..."

# Database configuration
DB_NAME=${DATABASE_NAME:-hackerexperience}
DB_USER=${DATABASE_USER:-heuser}
DB_PASS=${DATABASE_PASSWORD:-hepass}
DB_HOST=${DATABASE_HOST:-localhost}
DB_PORT=${DATABASE_PORT:-5432}

# Check if PostgreSQL is running
if ! pg_isready -h $DB_HOST -p $DB_PORT > /dev/null 2>&1; then
    echo "❌ PostgreSQL is not running on $DB_HOST:$DB_PORT"
    echo "Please start PostgreSQL first"
    exit 1
fi

echo "✅ PostgreSQL is running"

# Create database and user (requires superuser privileges)
echo "Creating database and user..."

sudo -u postgres psql <<EOF
-- Create user if not exists
DO
\$do\$
BEGIN
   IF NOT EXISTS (
      SELECT FROM pg_catalog.pg_roles
      WHERE  rolname = '$DB_USER') THEN
      CREATE USER $DB_USER WITH PASSWORD '$DB_PASS';
   END IF;
END
\$do\$;

-- Create database if not exists
SELECT 'CREATE DATABASE $DB_NAME OWNER $DB_USER'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = '$DB_NAME')\gexec

-- Grant all privileges
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF

echo "✅ Database and user created"

# Export connection string
export DATABASE_URL="postgresql://$DB_USER:$DB_PASS@$DB_HOST:$DB_PORT/$DB_NAME"
echo "✅ DATABASE_URL set: $DATABASE_URL"

# Run migrations
echo "Running migrations..."
cd "$(dirname "$0")/.."

# Use sqlx-cli if available, otherwise use the binary
if command -v sqlx &> /dev/null; then
    sqlx migrate run --source migrations-postgres
else
    cargo sqlx migrate run --source migrations-postgres
fi

echo "✅ Migrations completed"

# Create initial admin user (optional)
psql $DATABASE_URL <<EOF
-- Create admin user
INSERT INTO users (login, password, email, game_pass, game_ip, real_ip, home_ip, premium)
VALUES (
    'admin',
    '\$argon2id\$v=19\$m=4096,t=3,p=1\$salt\$hash', -- Replace with actual hash
    'admin@hackerexperience.com',
    'ADMIN123',
    3232235777,  -- 192.168.1.1
    3232235777,
    3232235777,
    true
) ON CONFLICT (login) DO NOTHING;

-- Create admin's server
INSERT INTO servers (user_id, ip_address, hostname, cpu_total, ram_total, hdd_total, net_total)
SELECT id, '192.168.1.1', 'admin-server', 10000, 16384, 1000000, 10000
FROM users WHERE login = 'admin'
ON CONFLICT (ip_address) DO NOTHING;

-- Create initial bank account for admin
INSERT INTO bank_accounts (user_id, account_number, routing_number, balance)
SELECT id, '1000000001', '123456789', 1000000
FROM users WHERE login = 'admin'
ON CONFLICT (account_number) DO NOTHING;

EOF

echo "✅ Initial data created"

echo "
========================================
   HackerExperience Database Ready!
========================================
Database: $DB_NAME
User: $DB_USER
Host: $DB_HOST:$DB_PORT

To connect:
psql $DATABASE_URL

To start the server:
DATABASE_URL=$DATABASE_URL cargo run --bin he-api
========================================
"