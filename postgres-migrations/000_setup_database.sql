-- HackerExperience PostgreSQL Database Setup Script
-- This script sets up the complete database environment for the HackerExperience game
-- Run this script first to create the database and user

-- Create the main database
CREATE DATABASE hackerexperience_rust 
WITH 
    ENCODING = 'UTF8'
    LC_COLLATE = 'en_US.UTF-8'
    LC_CTYPE = 'en_US.UTF-8'
    CONNECTION LIMIT = -1;

-- Create application user
CREATE ROLE he_app WITH
    LOGIN
    NOSUPERUSER
    NOCREATEDB
    NOCREATEROLE
    INHERIT
    NOREPLICATION
    CONNECTION LIMIT -1
    PASSWORD 'he_secure_password_change_in_production';

-- Grant database privileges
GRANT ALL PRIVILEGES ON DATABASE hackerexperience_rust TO he_app;

-- Connect to the database
\c hackerexperience_rust;

-- Create extensions that we'll need
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";      -- UUID generation
CREATE EXTENSION IF NOT EXISTS "postgis";        -- Geographic data support
CREATE EXTENSION IF NOT EXISTS "pg_trgm";        -- Trigram matching for text search
CREATE EXTENSION IF NOT EXISTS "btree_gin";      -- GIN indexes for btree operations
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements"; -- Query performance monitoring

-- Grant schema privileges
GRANT ALL ON SCHEMA public TO he_app;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO he_app;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO he_app;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO he_app;

-- Set default privileges for future objects
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO he_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO he_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON FUNCTIONS TO he_app;

-- Configure PostgreSQL settings for optimal performance
-- These settings should be adjusted based on your hardware
ALTER SYSTEM SET shared_preload_libraries = 'pg_stat_statements';
ALTER SYSTEM SET max_connections = 200;
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
ALTER SYSTEM SET random_page_cost = 1.1;
ALTER SYSTEM SET effective_io_concurrency = 200;

-- Create schema version tracking table
CREATE TABLE IF NOT EXISTS schema_migrations (
    version VARCHAR(50) PRIMARY KEY,
    description TEXT,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    execution_time_ms INTEGER,
    checksum VARCHAR(64)
);

-- Insert initial migration record
INSERT INTO schema_migrations (version, description) VALUES 
('000', 'Database setup and configuration');

-- Success message
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE 'HackerExperience Database Setup Complete';
    RAISE NOTICE '========================================';
    RAISE NOTICE 'Database: hackerexperience_rust';
    RAISE NOTICE 'User: he_app';
    RAISE NOTICE 'Extensions: uuid-ossp, postgis, pg_trgm, btree_gin, pg_stat_statements';
    RAISE NOTICE 'Ready to run migrations 001-014';
    RAISE NOTICE '========================================';
END $$;