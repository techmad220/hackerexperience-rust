-- Create software table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS software (
    id BIGSERIAL PRIMARY KEY,
    server_id BIGINT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL, -- 'cracker', 'hasher', 'firewall', 'antivirus', 'spam', 'warez', 'bitcoin_miner'
    version DECIMAL(10,2) NOT NULL DEFAULT 1.0,
    size INTEGER NOT NULL, -- MB
    is_installed BOOLEAN NOT NULL DEFAULT FALSE,
    is_running BOOLEAN NOT NULL DEFAULT FALSE,
    effectiveness INTEGER NOT NULL DEFAULT 10, -- 1-100
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_software_server_id ON software(server_id);
CREATE INDEX idx_software_type ON software(type);
CREATE INDEX idx_software_is_installed ON software(is_installed);
CREATE INDEX idx_software_is_running ON software(is_running);

CREATE TRIGGER update_software_updated_at BEFORE UPDATE
    ON software FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();