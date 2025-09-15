-- Create comprehensive banking system tables
-- Handles banks, accounts, transactions, and financial operations

-- Banks table - Different banks in the game world
CREATE TABLE IF NOT EXISTS banks (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    short_name VARCHAR(10) NOT NULL UNIQUE,       -- Abbreviated name
    swift_code VARCHAR(11) UNIQUE,                -- International bank code
    
    -- Bank properties
    bank_type VARCHAR(20) NOT NULL DEFAULT 'commercial', -- commercial, investment, central, crypto
    security_level INTEGER NOT NULL DEFAULT 5,    -- Security rating (1-10)
    reputation INTEGER NOT NULL DEFAULT 50,       -- Bank reputation (0-100)
    
    -- Geographic information
    country_code VARCHAR(2) NOT NULL,
    city VARCHAR(50) NOT NULL,
    headquarters_address TEXT,
    
    -- Financial information
    total_assets BIGINT NOT NULL DEFAULT 0,       -- Total bank assets
    total_deposits BIGINT NOT NULL DEFAULT 0,     -- Total customer deposits  
    interest_rate_savings DECIMAL(5,4) DEFAULT 0.0250, -- 2.5% default savings rate
    interest_rate_loan DECIMAL(5,4) DEFAULT 0.0800,    -- 8% default loan rate
    
    -- Banking features and limits
    min_account_balance BIGINT NOT NULL DEFAULT 0,     -- Minimum balance requirement
    max_daily_transfer BIGINT NOT NULL DEFAULT 100000, -- Daily transfer limit
    transfer_fee BIGINT NOT NULL DEFAULT 50,           -- Transfer fee
    atm_fee BIGINT NOT NULL DEFAULT 25,               -- ATM withdrawal fee
    overdraft_limit BIGINT NOT NULL DEFAULT 0,        -- Overdraft allowance
    
    -- Status and flags
    is_active BOOLEAN NOT NULL DEFAULT TRUE,      -- Bank is operational
    accepts_new_accounts BOOLEAN NOT NULL DEFAULT TRUE,
    requires_identity_verification BOOLEAN NOT NULL DEFAULT TRUE,
    supports_international BOOLEAN NOT NULL DEFAULT TRUE,
    has_mobile_banking BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Constraints
    CONSTRAINT chk_bank_security_level CHECK (security_level >= 1 AND security_level <= 10),
    CONSTRAINT chk_bank_reputation CHECK (reputation >= 0 AND reputation <= 100),
    CONSTRAINT chk_bank_assets CHECK (total_assets >= 0),
    CONSTRAINT chk_bank_deposits CHECK (total_deposits >= 0),
    CONSTRAINT chk_bank_interest_rates CHECK (
        interest_rate_savings >= 0 AND interest_rate_savings <= 1.0 AND
        interest_rate_loan >= 0 AND interest_rate_loan <= 1.0
    ),
    CONSTRAINT chk_bank_fees CHECK (transfer_fee >= 0 AND atm_fee >= 0),
    CONSTRAINT chk_bank_limits CHECK (
        min_account_balance >= 0 AND 
        max_daily_transfer > 0 AND 
        overdraft_limit >= 0
    )
);

-- Bank accounts table  
CREATE TABLE IF NOT EXISTS bank_accounts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    bank_id BIGINT NOT NULL,
    
    -- Account identification
    account_number VARCHAR(20) NOT NULL UNIQUE,
    account_name VARCHAR(100) NOT NULL,
    account_type VARCHAR(20) NOT NULL DEFAULT 'checking', -- checking, savings, investment, crypto
    
    -- Account status and properties
    balance BIGINT NOT NULL DEFAULT 0,            -- Current balance in cents
    available_balance BIGINT NOT NULL DEFAULT 0,  -- Available balance (balance - holds)
    pending_balance BIGINT NOT NULL DEFAULT 0,    -- Pending transactions
    
    -- Account settings
    account_status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, frozen, closed, suspended
    overdraft_protection BOOLEAN NOT NULL DEFAULT FALSE,
    requires_pin BOOLEAN NOT NULL DEFAULT TRUE,
    daily_transfer_limit BIGINT,                  -- Custom daily limit (NULL = bank default)
    
    -- Security and access
    pin_hash VARCHAR(60),                         -- Hashed PIN for account access
    security_questions JSONB DEFAULT '[]',       -- Security questions and answers
    last_accessed TIMESTAMPTZ,
    access_attempts INTEGER NOT NULL DEFAULT 0,   -- Failed access attempts
    locked_until TIMESTAMPTZ,                    -- Account lock expiry
    
    -- Interest and fees
    interest_rate DECIMAL(5,4),                   -- Custom interest rate (NULL = bank default)
    monthly_fee BIGINT NOT NULL DEFAULT 0,       -- Monthly maintenance fee
    last_interest_payment TIMESTAMPTZ,           -- Last interest calculation
    accrued_interest BIGINT NOT NULL DEFAULT 0,  -- Interest earned but not paid
    
    -- Account history and statistics
    total_deposits BIGINT NOT NULL DEFAULT 0,    -- Lifetime deposits
    total_withdrawals BIGINT NOT NULL DEFAULT 0, -- Lifetime withdrawals
    total_transfers_sent BIGINT NOT NULL DEFAULT 0,
    total_transfers_received BIGINT NOT NULL DEFAULT 0,
    transaction_count INTEGER NOT NULL DEFAULT 0,
    
    -- Timestamps
    opened_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    closed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (bank_id) REFERENCES banks(id) ON DELETE RESTRICT,
    
    -- Check constraints
    CONSTRAINT chk_account_balance CHECK (balance >= 0 OR overdraft_protection = TRUE),
    CONSTRAINT chk_account_available_balance CHECK (available_balance >= 0),
    CONSTRAINT chk_account_pending_balance CHECK (pending_balance >= 0),
    CONSTRAINT chk_account_access_attempts CHECK (access_attempts >= 0),
    CONSTRAINT chk_account_statistics CHECK (
        total_deposits >= 0 AND total_withdrawals >= 0 AND
        total_transfers_sent >= 0 AND total_transfers_received >= 0 AND
        transaction_count >= 0
    ),
    CONSTRAINT chk_account_interest_rate CHECK (interest_rate IS NULL OR (interest_rate >= 0 AND interest_rate <= 1.0)),
    CONSTRAINT chk_account_fees CHECK (monthly_fee >= 0 AND accrued_interest >= 0),
    CONSTRAINT chk_account_closed CHECK (closed_at IS NULL OR closed_at >= opened_at)
);

-- Transactions table
CREATE TABLE IF NOT EXISTS bank_transactions (
    id BIGSERIAL PRIMARY KEY,
    
    -- Transaction identification  
    transaction_id VARCHAR(50) NOT NULL UNIQUE,  -- External transaction ID
    reference_number VARCHAR(50),                 -- Reference/memo
    
    -- Account information
    from_account_id BIGINT,                       -- Source account (NULL for deposits)
    to_account_id BIGINT,                        -- Destination account (NULL for withdrawals)
    
    -- Transaction details
    transaction_type VARCHAR(20) NOT NULL,       -- transfer, deposit, withdrawal, fee, interest
    amount BIGINT NOT NULL,                      -- Transaction amount in cents
    fee BIGINT NOT NULL DEFAULT 0,              -- Transaction fee
    exchange_rate DECIMAL(10,6) DEFAULT 1.0,    -- Currency exchange rate
    
    -- Transaction status and processing
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, completed, failed, cancelled, reversed
    processing_status VARCHAR(20) DEFAULT 'queued', -- queued, processing, processed
    failure_reason TEXT,                         -- Reason if failed
    
    -- Security and validation
    authorization_code VARCHAR(50),             -- Authorization/confirmation code
    ip_address INET,                            -- IP address of transaction origin
    device_fingerprint VARCHAR(128),            -- Device fingerprint
    requires_manual_approval BOOLEAN NOT NULL DEFAULT FALSE,
    approved_by_user_id BIGINT,                 -- If manual approval required
    
    -- Timing information
    initiated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    reversed_at TIMESTAMPTZ,
    
    -- Batch and grouping
    batch_id VARCHAR(50),                       -- For batch processing
    parent_transaction_id BIGINT,               -- For split/grouped transactions
    
    -- Additional metadata
    description TEXT,
    metadata JSONB DEFAULT '{}',
    
    -- Balances after transaction (for audit trail)
    from_account_balance_before BIGINT,
    from_account_balance_after BIGINT,
    to_account_balance_before BIGINT,
    to_account_balance_after BIGINT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Foreign key constraints
    FOREIGN KEY (from_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (to_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT,
    FOREIGN KEY (approved_by_user_id) REFERENCES users(id) ON DELETE SET NULL,
    FOREIGN KEY (parent_transaction_id) REFERENCES bank_transactions(id) ON DELETE SET NULL,
    
    -- Check constraints
    CONSTRAINT chk_transaction_amount CHECK (amount > 0),
    CONSTRAINT chk_transaction_fee CHECK (fee >= 0),
    CONSTRAINT chk_transaction_exchange_rate CHECK (exchange_rate > 0),
    CONSTRAINT chk_transaction_accounts CHECK (
        from_account_id IS NOT NULL OR to_account_id IS NOT NULL
    ),
    CONSTRAINT chk_transaction_timing CHECK (
        (processed_at IS NULL OR processed_at >= initiated_at) AND
        (completed_at IS NULL OR completed_at >= initiated_at) AND
        (reversed_at IS NULL OR reversed_at >= initiated_at)
    )
);

-- Create comprehensive indexes
CREATE INDEX IF NOT EXISTS idx_banks_name ON banks(name);
CREATE INDEX IF NOT EXISTS idx_banks_country ON banks(country_code);
CREATE INDEX IF NOT EXISTS idx_banks_active ON banks(is_active);
CREATE INDEX IF NOT EXISTS idx_banks_security_level ON banks(security_level);

CREATE INDEX IF NOT EXISTS idx_bank_accounts_user_id ON bank_accounts(user_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_bank_id ON bank_accounts(bank_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_number ON bank_accounts(account_number);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_status ON bank_accounts(account_status);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_type ON bank_accounts(account_type);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_balance ON bank_accounts(balance);

CREATE INDEX IF NOT EXISTS idx_bank_transactions_from_account ON bank_transactions(from_account_id);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_to_account ON bank_transactions(to_account_id);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_type ON bank_transactions(transaction_type);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_status ON bank_transactions(status);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_initiated_at ON bank_transactions(initiated_at);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_transaction_id ON bank_transactions(transaction_id);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_batch_id ON bank_transactions(batch_id);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_parent ON bank_transactions(parent_transaction_id);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_bank_accounts_security_questions ON bank_accounts USING GIN(security_questions);
CREATE INDEX IF NOT EXISTS idx_bank_transactions_metadata ON bank_transactions USING GIN(metadata);

-- Create triggers for updated_at
CREATE TRIGGER banks_updated_at_trigger
    BEFORE UPDATE ON banks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER bank_accounts_updated_at_trigger
    BEFORE UPDATE ON bank_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER bank_transactions_updated_at_trigger
    BEFORE UPDATE ON bank_transactions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to create a new bank transfer
CREATE OR REPLACE FUNCTION create_bank_transfer(
    from_account_id_param BIGINT,
    to_account_id_param BIGINT,
    amount_param BIGINT,
    description_param TEXT DEFAULT NULL,
    reference_param VARCHAR(50) DEFAULT NULL
) RETURNS BIGINT AS $$
DECLARE
    transaction_id BIGINT;
    from_account RECORD;
    to_account RECORD;
    transfer_fee BIGINT;
    transaction_ref VARCHAR(50);
BEGIN
    -- Get account details
    SELECT * INTO from_account FROM bank_accounts WHERE id = from_account_id_param;
    SELECT * INTO to_account FROM bank_accounts WHERE id = to_account_id_param;
    
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Account not found';
    END IF;
    
    -- Check account status
    IF from_account.account_status != 'active' THEN
        RAISE EXCEPTION 'Source account is not active';
    END IF;
    
    IF to_account.account_status != 'active' THEN
        RAISE EXCEPTION 'Destination account is not active';
    END IF;
    
    -- Calculate transfer fee
    SELECT banks.transfer_fee INTO transfer_fee
    FROM banks WHERE id = from_account.bank_id;
    
    -- Check sufficient balance
    IF from_account.available_balance < (amount_param + transfer_fee) THEN
        RAISE EXCEPTION 'Insufficient funds. Available: %, Required: %', 
            from_account.available_balance, (amount_param + transfer_fee);
    END IF;
    
    -- Generate transaction reference
    transaction_ref := COALESCE(reference_param, 'TXN-' || TO_CHAR(CURRENT_TIMESTAMP, 'YYYYMMDDHH24MISS') || '-' || EXTRACT(MICROSECONDS FROM CURRENT_TIMESTAMP)::TEXT);
    
    -- Create transaction record
    INSERT INTO bank_transactions (
        transaction_id, from_account_id, to_account_id, transaction_type,
        amount, fee, description, reference_number,
        from_account_balance_before, to_account_balance_before
    ) VALUES (
        transaction_ref, from_account_id_param, to_account_id_param, 'transfer',
        amount_param, transfer_fee, description_param, reference_param,
        from_account.balance, to_account.balance
    ) RETURNING id INTO transaction_id;
    
    -- Update account balances
    UPDATE bank_accounts SET
        balance = balance - amount_param - transfer_fee,
        available_balance = available_balance - amount_param - transfer_fee,
        total_transfers_sent = total_transfers_sent + amount_param,
        transaction_count = transaction_count + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = from_account_id_param;
    
    UPDATE bank_accounts SET
        balance = balance + amount_param,
        available_balance = available_balance + amount_param,
        total_transfers_received = total_transfers_received + amount_param,
        transaction_count = transaction_count + 1,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = to_account_id_param;
    
    -- Update transaction with final balances and mark as completed
    UPDATE bank_transactions SET
        status = 'completed',
        processing_status = 'processed',
        processed_at = CURRENT_TIMESTAMP,
        completed_at = CURRENT_TIMESTAMP,
        from_account_balance_after = from_account.balance - amount_param - transfer_fee,
        to_account_balance_after = to_account.balance + amount_param,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = transaction_id;
    
    RETURN transaction_id;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate and pay interest
CREATE OR REPLACE FUNCTION calculate_account_interest(account_id_param BIGINT)
RETURNS BIGINT AS $$
DECLARE
    account_record RECORD;
    bank_record RECORD;
    interest_rate DECIMAL(5,4);
    interest_amount BIGINT;
    days_since_last_payment INTEGER;
BEGIN
    -- Get account details
    SELECT * INTO account_record FROM bank_accounts WHERE id = account_id_param;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'Account % not found', account_id_param;
    END IF;
    
    -- Get bank details for default interest rate
    SELECT * INTO bank_record FROM banks WHERE id = account_record.bank_id;
    
    -- Determine interest rate (account-specific or bank default)
    interest_rate := COALESCE(account_record.interest_rate, bank_record.interest_rate_savings);
    
    -- Calculate days since last interest payment
    days_since_last_payment := COALESCE(
        EXTRACT(DAYS FROM CURRENT_DATE - account_record.last_interest_payment::DATE)::INTEGER,
        30  -- Default to 30 days if no previous payment
    );
    
    -- Calculate daily interest (annual rate / 365)
    interest_amount := (account_record.balance * interest_rate / 365 * days_since_last_payment)::BIGINT;
    
    IF interest_amount > 0 THEN
        -- Add interest to account
        UPDATE bank_accounts SET
            balance = balance + interest_amount,
            available_balance = available_balance + interest_amount,
            accrued_interest = accrued_interest + interest_amount,
            last_interest_payment = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = account_id_param;
        
        -- Create interest transaction record
        INSERT INTO bank_transactions (
            transaction_id, to_account_id, transaction_type, amount,
            description, status, processing_status, processed_at, completed_at,
            to_account_balance_before, to_account_balance_after
        ) VALUES (
            'INT-' || account_id_param || '-' || TO_CHAR(CURRENT_TIMESTAMP, 'YYYYMMDDHH24MISS'),
            account_id_param, 'interest', interest_amount,
            'Interest payment for ' || days_since_last_payment || ' days',
            'completed', 'processed', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP,
            account_record.balance, account_record.balance + interest_amount
        );
    END IF;
    
    RETURN interest_amount;
END;
$$ LANGUAGE plpgsql;

-- Create view for account summaries
CREATE OR REPLACE VIEW account_summaries AS
SELECT 
    ba.id,
    ba.account_number,
    ba.account_name,
    ba.account_type,
    ba.balance / 100.0 AS balance_dollars,
    ba.available_balance / 100.0 AS available_balance_dollars,
    ba.account_status,
    b.name AS bank_name,
    b.short_name AS bank_code,
    u.login AS owner_username,
    ba.transaction_count,
    ba.total_deposits / 100.0 AS total_deposits_dollars,
    ba.total_withdrawals / 100.0 AS total_withdrawals_dollars,
    ba.last_accessed,
    ba.opened_at,
    COALESCE(ba.interest_rate, b.interest_rate_savings) AS effective_interest_rate
FROM bank_accounts ba
JOIN banks b ON ba.bank_id = b.id
JOIN users u ON ba.user_id = u.id
WHERE ba.account_status = 'active';

-- Add comprehensive comments
COMMENT ON TABLE banks IS 'Banks available in the game world with their properties and features';
COMMENT ON TABLE bank_accounts IS 'Player bank accounts with balances, security, and transaction history';
COMMENT ON TABLE bank_transactions IS 'Comprehensive transaction log with audit trail and status tracking';

COMMENT ON COLUMN banks.security_level IS 'Bank security rating affecting hack difficulty (1-10)';
COMMENT ON COLUMN bank_accounts.balance IS 'Account balance in cents to avoid floating point issues';
COMMENT ON COLUMN bank_accounts.pin_hash IS 'Hashed PIN for account access authentication';
COMMENT ON COLUMN bank_transactions.device_fingerprint IS 'Device fingerprint for fraud detection';

COMMENT ON VIEW account_summaries IS 'Summary view of active accounts with bank and user information';
COMMENT ON FUNCTION create_bank_transfer IS 'Creates a bank transfer between accounts with validation and fee calculation';
COMMENT ON FUNCTION calculate_account_interest IS 'Calculates and pays interest on savings accounts';