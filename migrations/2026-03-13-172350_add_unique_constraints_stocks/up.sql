ALTER TABLE stocks ADD CONSTRAINT stocks_isin_unique UNIQUE (isin);
ALTER TABLE stocks ADD CONSTRAINT stocks_symbol_unique UNIQUE (symbol);
