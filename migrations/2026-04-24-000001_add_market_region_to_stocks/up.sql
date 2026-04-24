ALTER TABLE stocks ADD COLUMN market_region VARCHAR;

UPDATE stocks
SET market_region = CASE
    WHEN symbol LIKE '%.TO' OR symbol LIKE '%.V' THEN 'americas'
    WHEN symbol LIKE '%.PA' OR symbol LIKE '%.AS' OR symbol LIKE '%.BR'
      OR symbol LIKE '%.DE' OR symbol LIKE '%.F'  OR symbol LIKE '%.L'
      OR symbol LIKE '%.SW' OR symbol LIKE '%.MI' THEN 'europe'
    WHEN symbol LIKE '%.T'  OR symbol LIKE '%.HK'
      OR symbol LIKE '%.AX' OR symbol LIKE '%.SI' THEN 'asia_pacific'
    WHEN position('.' IN symbol) = 0 THEN 'americas'
    ELSE 'other'
END;

ALTER TABLE stocks ALTER COLUMN market_region SET DEFAULT 'other';
ALTER TABLE stocks ALTER COLUMN market_region SET NOT NULL;
ALTER TABLE stocks ADD CONSTRAINT stocks_market_region_check
    CHECK (market_region IN ('americas', 'europe', 'asia_pacific', 'other'));
