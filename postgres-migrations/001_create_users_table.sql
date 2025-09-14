-- Create users table - PostgreSQL port from original HackerExperience game.sql
-- This is the core player data table that all other game entities reference

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    login VARCHAR(15) NOT NULL UNIQUE,
    password VARCHAR(60) NOT NULL,  -- BCrypt hash
    email VARCHAR(50) NOT NULL UNIQUE,
    game_pass VARCHAR(8) NOT NULL,  -- Game authentication token
    game_ip BIGINT NOT NULL,        -- Player's game IP address 
    real_ip BIGINT NOT NULL,        -- Player's real IP address for security
    home_ip BIGINT NOT NULL,        -- Player's starting/home IP address
    learning BOOLEAN NOT NULL DEFAULT FALSE,  -- Tutorial mode flag
    premium BOOLEAN NOT NULL DEFAULT FALSE,   -- Premium account status
    last_login TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_login ON users(login);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_game_ip ON users(game_ip);
CREATE INDEX IF NOT EXISTS idx_users_real_ip ON users(real_ip);
CREATE INDEX IF NOT EXISTS idx_users_last_login ON users(last_login);
CREATE INDEX IF NOT EXISTS idx_users_premium ON users(premium);

-- Create trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at_trigger
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE users IS 'Core user/player table containing authentication and basic profile information';
COMMENT ON COLUMN users.id IS 'Primary key - unique user identifier';
COMMENT ON COLUMN users.login IS 'Unique player username/login';
COMMENT ON COLUMN users.password IS 'BCrypt hashed password';
COMMENT ON COLUMN users.email IS 'Unique email address for account recovery';
COMMENT ON COLUMN users.game_pass IS '8-character game authentication token';
COMMENT ON COLUMN users.game_ip IS 'Players current in-game IP address';
COMMENT ON COLUMN users.real_ip IS 'Players real-world IP for security tracking';
COMMENT ON COLUMN users.home_ip IS 'Players starting/home IP address in game';
COMMENT ON COLUMN users.learning IS 'Flag indicating if player is in tutorial mode';
COMMENT ON COLUMN users.premium IS 'Flag indicating premium account status';