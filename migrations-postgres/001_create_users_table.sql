-- Create users table (PostgreSQL version)
-- Based on original HackerExperience schema

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    login VARCHAR(15) NOT NULL UNIQUE,
    password VARCHAR(60) NOT NULL,  -- BCrypt/Argon2 hash
    email VARCHAR(50) NOT NULL UNIQUE,
    game_pass VARCHAR(8) NOT NULL,
    game_ip BIGINT NOT NULL,
    real_ip BIGINT NOT NULL,
    home_ip BIGINT NOT NULL,
    learning BOOLEAN NOT NULL DEFAULT FALSE,
    premium BOOLEAN NOT NULL DEFAULT FALSE,
    last_login TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_users_game_ip ON users(game_ip);
CREATE INDEX idx_users_last_login ON users(last_login);
CREATE INDEX idx_users_premium ON users(premium);

-- Create updated_at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at BEFORE UPDATE
    ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();