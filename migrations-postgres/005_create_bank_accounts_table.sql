-- Create bank accounts table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS bank_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_number VARCHAR(20) NOT NULL UNIQUE,
    routing_number VARCHAR(20) NOT NULL,
    balance BIGINT NOT NULL DEFAULT 0, -- In cents to avoid float issues
    account_type VARCHAR(20) NOT NULL DEFAULT 'checking',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX idx_bank_accounts_account_number ON bank_accounts(account_number);

CREATE TRIGGER update_bank_accounts_updated_at BEFORE UPDATE
    ON bank_accounts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();