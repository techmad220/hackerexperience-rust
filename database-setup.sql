-- HackerExperience Database Setup
-- Creates all 13 databases used by the game

-- Main game database
CREATE DATABASE hackerexperience;

-- Specialized databases for different game systems
CREATE DATABASE he_account;      -- User accounts and authentication
CREATE DATABASE he_server;       -- Server management and hardware
CREATE DATABASE he_software;     -- Software and processes
CREATE DATABASE he_network;      -- Network connections and topology
CREATE DATABASE he_bank;         -- Financial transactions
CREATE DATABASE he_clan;         -- Clan management
CREATE DATABASE he_mission;      -- Mission and quest system
CREATE DATABASE he_log;          -- Game logs and history
CREATE DATABASE he_cache;        -- Caching and temporary data
CREATE DATABASE he_universe;     -- Game world and entities
CREATE DATABASE he_story;        -- Storyline and narrative
CREATE DATABASE he_factor;       -- Game balance and factors

-- Create a user for the application
CREATE USER he_app WITH PASSWORD 'secure_password_change_in_production';

-- Grant permissions to all databases
GRANT ALL PRIVILEGES ON DATABASE hackerexperience TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_account TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_server TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_software TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_network TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_bank TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_clan TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_mission TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_log TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_cache TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_universe TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_story TO he_app;
GRANT ALL PRIVILEGES ON DATABASE he_factor TO he_app;

-- Connect to main database and create initial tables
\c hackerexperience;

-- Users table (core player data)
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    level INTEGER DEFAULT 1,
    experience BIGINT DEFAULT 0,
    money BIGINT DEFAULT 10000,
    reputation INTEGER DEFAULT 0,
    clan_id INTEGER,
    last_login TIMESTAMP,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    is_online BOOLEAN DEFAULT false,
    storage_used INTEGER DEFAULT 0,
    storage_total INTEGER DEFAULT 1024,
    cpu_power INTEGER DEFAULT 100,
    ram INTEGER DEFAULT 512
);

-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id SERIAL PRIMARY KEY,
    owner_id INTEGER REFERENCES users(id),
    ip VARCHAR(15) UNIQUE NOT NULL,
    name VARCHAR(100),
    type VARCHAR(20) DEFAULT 'desktop',
    cpu_power INTEGER DEFAULT 100,
    ram INTEGER DEFAULT 512,
    storage INTEGER DEFAULT 1024,
    firewall_level INTEGER DEFAULT 0,
    visible BOOLEAN DEFAULT true,
    requires_password BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Software table
CREATE TABLE IF NOT EXISTS software (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    type VARCHAR(20) NOT NULL,
    version INTEGER DEFAULT 1,
    size INTEGER NOT NULL,
    installed BOOLEAN DEFAULT false,
    running BOOLEAN DEFAULT false,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Processes table
CREATE TABLE IF NOT EXISTS processes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    type VARCHAR(20) NOT NULL,
    target VARCHAR(100),
    target_id INTEGER,
    status VARCHAR(20) DEFAULT 'running',
    progress INTEGER DEFAULT 0,
    duration INTEGER NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Bank accounts table
CREATE TABLE IF NOT EXISTS bank_accounts (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    account_number VARCHAR(20) UNIQUE NOT NULL,
    bank VARCHAR(50) NOT NULL,
    type VARCHAR(20) DEFAULT 'checking',
    balance BIGINT DEFAULT 0,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    from_account_id INTEGER REFERENCES bank_accounts(id),
    to_account_id INTEGER REFERENCES bank_accounts(id),
    amount BIGINT NOT NULL,
    fee BIGINT DEFAULT 0,
    type VARCHAR(20) NOT NULL,
    status VARCHAR(20) DEFAULT 'completed',
    created_at TIMESTAMP DEFAULT NOW()
);

-- Clans table
CREATE TABLE IF NOT EXISTS clans (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    leader_id INTEGER REFERENCES users(id),
    created_at TIMESTAMP DEFAULT NOW()
);

-- Clan members table
CREATE TABLE IF NOT EXISTS clan_members (
    id SERIAL PRIMARY KEY,
    clan_id INTEGER REFERENCES clans(id),
    user_id INTEGER REFERENCES users(id),
    role VARCHAR(20) DEFAULT 'member',
    joined_at TIMESTAMP DEFAULT NOW(),
    active BOOLEAN DEFAULT true
);

-- Missions table
CREATE TABLE IF NOT EXISTS missions (
    id SERIAL PRIMARY KEY,
    title VARCHAR(100) NOT NULL,
    description TEXT,
    difficulty INTEGER DEFAULT 100,
    reward_money BIGINT DEFAULT 1000,
    exp_reward INTEGER DEFAULT 100,
    time_limit INTEGER DEFAULT 3600,
    status VARCHAR(20) DEFAULT 'available',
    created_at TIMESTAMP DEFAULT NOW()
);

-- User missions table
CREATE TABLE IF NOT EXISTS user_missions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    mission_id INTEGER REFERENCES missions(id),
    status VARCHAR(20) DEFAULT 'active',
    accepted_at TIMESTAMP DEFAULT NOW(),
    completed_at TIMESTAMP,
    deadline TIMESTAMP
);

-- Connections table
CREATE TABLE IF NOT EXISTS connections (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    server_id INTEGER REFERENCES servers(id),
    ip VARCHAR(15) NOT NULL,
    active BOOLEAN DEFAULT true,
    connected_at TIMESTAMP DEFAULT NOW()
);

-- Hack logs table
CREATE TABLE IF NOT EXISTS hack_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    server_id INTEGER REFERENCES servers(id),
    type VARCHAR(20) NOT NULL,
    success_chance REAL,
    started_at TIMESTAMP DEFAULT NOW()
);

-- User logs table
CREATE TABLE IF NOT EXISTS user_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    details TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Server logs table
CREATE TABLE IF NOT EXISTS server_logs (
    id SERIAL PRIMARY KEY,
    server_id INTEGER REFERENCES servers(id),
    user_id INTEGER,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Grant permissions on all tables to app user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO he_app;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO he_app;

-- Insert some sample data for testing
INSERT INTO users (username, email, password_hash, level, money) VALUES 
('admin', 'admin@hackerexperience.com', '$2b$12$hash', 50, 1000000),
('demo_user', 'demo@hackerexperience.com', '$2b$12$hash', 15, 50000),
('test_hacker', 'test@hackerexperience.com', '$2b$12$hash', 10, 25000);

INSERT INTO servers (owner_id, ip, name, type) VALUES 
(1, '192.168.1.100', 'Admin Server', 'server'),
(2, '10.0.0.5', 'Demo Desktop', 'desktop'),
(NULL, '203.0.113.1', 'Public Server', 'public');

INSERT INTO missions (title, description, difficulty, reward_money, exp_reward) VALUES
('Corporate Data Theft', 'Steal sensitive corporate data from MegaCorp servers', 150, 10000, 500),
('Bank System Infiltration', 'Break into First National Bank security system', 300, 50000, 2000),
('Virus Deployment', 'Deploy a custom virus to target network', 200, 25000, 1000);

-- Create indexes for better performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_servers_ip ON servers(ip);
CREATE INDEX idx_processes_user_id ON processes(user_id);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);
CREATE INDEX idx_user_missions_user_id ON user_missions(user_id);

\echo 'Database setup complete!'