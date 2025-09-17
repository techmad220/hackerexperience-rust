-- Create servers table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS servers (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    ip_address INET NOT NULL UNIQUE,
    hostname VARCHAR(255),
    cpu_total INTEGER NOT NULL DEFAULT 500,   -- MHz
    ram_total INTEGER NOT NULL DEFAULT 256,   -- MB
    hdd_total INTEGER NOT NULL DEFAULT 10000, -- MB
    net_total INTEGER NOT NULL DEFAULT 100,   -- Mbps
    cpu_used INTEGER NOT NULL DEFAULT 0,
    ram_used INTEGER NOT NULL DEFAULT 0,
    hdd_used INTEGER NOT NULL DEFAULT 0,
    net_used INTEGER NOT NULL DEFAULT 0,
    is_npc BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_servers_user_id ON servers(user_id);
CREATE INDEX idx_servers_ip_address ON servers(ip_address);
CREATE INDEX idx_servers_is_npc ON servers(is_npc);

CREATE TRIGGER update_servers_updated_at BEFORE UPDATE
    ON servers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();