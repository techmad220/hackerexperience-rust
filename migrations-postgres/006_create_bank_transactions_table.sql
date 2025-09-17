-- Create bank transactions table (PostgreSQL version)

CREATE TABLE IF NOT EXISTS bank_transactions (
    id BIGSERIAL PRIMARY KEY,
    from_user_id BIGINT REFERENCES users(id) ON DELETE SET NULL,
    to_account VARCHAR(20) NOT NULL,
    amount BIGINT NOT NULL, -- In cents
    type VARCHAR(20) NOT NULL, -- 'transfer', 'deposit', 'withdrawal', 'hack'
    status VARCHAR(20) NOT NULL DEFAULT 'completed',
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bank_transactions_from_user_id ON bank_transactions(from_user_id);
CREATE INDEX idx_bank_transactions_to_account ON bank_transactions(to_account);
CREATE INDEX idx_bank_transactions_created_at ON bank_transactions(created_at);