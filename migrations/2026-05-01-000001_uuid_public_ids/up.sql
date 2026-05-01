-- Migrate `portfolios.id` and `transactions.id`/`transactions.portfolio_id`
-- from SERIAL/BIGSERIAL to UUID.
--
-- Existing rows get random v4 UUIDs (gen_random_uuid). New rows will be
-- assigned UUIDv7 values from Rust at insert time. The DB-level DEFAULT is
-- kept on `id` columns so partial inserts (admin SQL, fixtures) still work.
--
-- Done in a single transaction: the diesel CLI wraps each migration in BEGIN…
-- COMMIT, so any failure rolls everything back.

-- 1. Add new UUID columns alongside the existing INT/BIGINT ones.
ALTER TABLE portfolios
    ADD COLUMN id_new UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE transactions
    ADD COLUMN id_new UUID NOT NULL DEFAULT gen_random_uuid();
ALTER TABLE transactions
    ADD COLUMN portfolio_id_new UUID;

-- 2. Backfill the FK by joining on the existing INT relation.
UPDATE transactions t
SET portfolio_id_new = p.id_new
FROM portfolios p
WHERE t.portfolio_id = p.id;

-- 3. Drop old constraints (FK first, then PKs) so we can drop the int columns.
ALTER TABLE transactions DROP CONSTRAINT transactions_portfolio_id_fkey;
ALTER TABLE transactions DROP CONSTRAINT transactions_pkey;
ALTER TABLE portfolios   DROP CONSTRAINT portfolios_pkey;

ALTER TABLE transactions DROP COLUMN portfolio_id;
ALTER TABLE transactions DROP COLUMN id;
ALTER TABLE portfolios   DROP COLUMN id;

-- 4. Rename UUID columns to take over the canonical names.
ALTER TABLE transactions RENAME COLUMN portfolio_id_new TO portfolio_id;
ALTER TABLE transactions RENAME COLUMN id_new TO id;
ALTER TABLE portfolios   RENAME COLUMN id_new TO id;

-- 5. Recreate PKs and the FK.
ALTER TABLE portfolios   ADD PRIMARY KEY (id);
ALTER TABLE transactions ADD PRIMARY KEY (id);
ALTER TABLE transactions ALTER COLUMN portfolio_id SET NOT NULL;
ALTER TABLE transactions
    ADD CONSTRAINT transactions_portfolio_id_fkey
    FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE;
