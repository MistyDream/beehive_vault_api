CREATE TABLE stocks (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    isin VARCHAR NOT NULL,
    currency VARCHAR,
    market VARCHAR,
    sector VARCHAR,
    industry VARCHAR,
    country VARCHAR,
    updated_at TIMESTAMP
);
