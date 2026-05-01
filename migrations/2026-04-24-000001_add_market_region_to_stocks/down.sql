ALTER TABLE stocks DROP CONSTRAINT IF EXISTS stocks_market_region_check;
ALTER TABLE stocks DROP COLUMN IF EXISTS market_region;
