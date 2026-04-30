-- Revert the FK back to ON DELETE SET NULL and drop the unique constraints.

ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_stock_id_fkey;
ALTER TABLE transactions
    ADD CONSTRAINT transactions_stock_id_fkey
    FOREIGN KEY (stock_id) REFERENCES stocks(id) ON DELETE SET NULL;

ALTER TABLE stocks DROP CONSTRAINT IF EXISTS stocks_isin_unique;
ALTER TABLE stocks DROP CONSTRAINT IF EXISTS stocks_symbol_unique;
