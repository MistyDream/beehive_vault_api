-- Take ownership of stock-related constraints from the legacy PHP migrations
-- so the Rust schema is the single source of truth.
--
-- 1. UNIQUE(symbol) and UNIQUE(isin) on `stocks` — idempotent because the
--    legacy PHP migrations may already have created them (constraint names:
--    stocks_symbol_unique, stocks_isin_unique).
-- 2. transactions.stock_id FK switched from ON DELETE SET NULL to RESTRICT —
--    SET NULL silently orphaned historical buys/sells and broke wallet
--    integrity when an admin deleted a referenced stock.

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'stocks_symbol_unique'
    ) THEN
        ALTER TABLE stocks ADD CONSTRAINT stocks_symbol_unique UNIQUE (symbol);
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'stocks_isin_unique'
    ) THEN
        ALTER TABLE stocks ADD CONSTRAINT stocks_isin_unique UNIQUE (isin);
    END IF;
END $$;

ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_stock_id_fkey;
ALTER TABLE transactions
    ADD CONSTRAINT transactions_stock_id_fkey
    FOREIGN KEY (stock_id) REFERENCES stocks(id) ON DELETE RESTRICT;
