CREATE TABLE stock_prices (
    id         BIGSERIAL PRIMARY KEY,
    stock_id   INTEGER NOT NULL REFERENCES stocks (id) ON DELETE CASCADE,
    price_date DATE NOT NULL,
    close      NUMERIC NOT NULL,
    source     VARCHAR NOT NULL,
    fetched_at TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT stock_prices_stock_date_unique UNIQUE (stock_id, price_date)
);

CREATE INDEX idx_stock_prices_stock_date_desc ON stock_prices (stock_id, price_date DESC);
