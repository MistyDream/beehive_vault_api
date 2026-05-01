-- Revert UUID PKs back to SERIAL/BIGSERIAL.
--
-- WARNING: this destroys the UUID identifiers — rolling back means any client
-- that captured a UUID (frontend, external integration) loses its references.
-- The new INT ids are reassigned in row order, no preservation guarantee.

ALTER TABLE transactions DROP CONSTRAINT transactions_portfolio_id_fkey;
ALTER TABLE transactions DROP CONSTRAINT transactions_pkey;
ALTER TABLE portfolios   DROP CONSTRAINT portfolios_pkey;

ALTER TABLE portfolios   DROP COLUMN id;
ALTER TABLE transactions DROP COLUMN portfolio_id;
ALTER TABLE transactions DROP COLUMN id;

ALTER TABLE portfolios   ADD COLUMN id SERIAL PRIMARY KEY;
ALTER TABLE transactions ADD COLUMN id BIGSERIAL PRIMARY KEY;
ALTER TABLE transactions ADD COLUMN portfolio_id INTEGER;

-- We cannot rebuild the original FK relation: the int → UUID join was lossy
-- the moment we dropped the source columns. Set portfolio_id to the first
-- portfolio id so the table stays referentially valid; manual repair needed.
UPDATE transactions
SET portfolio_id = (SELECT id FROM portfolios ORDER BY id LIMIT 1)
WHERE portfolio_id IS NULL;

ALTER TABLE transactions ALTER COLUMN portfolio_id SET NOT NULL;
ALTER TABLE transactions
    ADD CONSTRAINT transactions_portfolio_id_fkey
    FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE;
