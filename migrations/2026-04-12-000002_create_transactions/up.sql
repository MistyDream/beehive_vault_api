CREATE TABLE transactions (
    id               BIGSERIAL PRIMARY KEY,
    portfolio_id     INTEGER NOT NULL REFERENCES portfolios (id) ON DELETE CASCADE,
    stock_id         INTEGER REFERENCES stocks (id) ON DELETE SET NULL,
    transaction_type VARCHAR NOT NULL,
    executed_at      DATE NOT NULL,
    quantity         DOUBLE PRECISION,
    unit_price       DOUBLE PRECISION,
    amount           DOUBLE PRECISION,
    fees             DOUBLE PRECISION NOT NULL DEFAULT 0,
    tax              DOUBLE PRECISION NOT NULL DEFAULT 0,
    split_from       INTEGER,
    split_to         INTEGER,
    currency         VARCHAR(3) NOT NULL,
    exchange_rate    DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    notes            TEXT,
    created_at       TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT transactions_type_check CHECK (
        transaction_type IN ('buy', 'sell', 'dividend', 'fee', 'split', 'deposit', 'withdrawal')
    )
);

CREATE INDEX idx_transactions_portfolio_id ON transactions (portfolio_id);
CREATE INDEX idx_transactions_stock_id ON transactions (stock_id);
CREATE INDEX idx_transactions_portfolio_type ON transactions (portfolio_id, transaction_type);
CREATE INDEX idx_transactions_portfolio_executed ON transactions (portfolio_id, executed_at DESC);
