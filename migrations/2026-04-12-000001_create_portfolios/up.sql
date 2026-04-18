CREATE TABLE portfolios (
    id           SERIAL PRIMARY KEY,
    name         VARCHAR NOT NULL,
    kind         VARCHAR NOT NULL,
    currency     VARCHAR(3) NOT NULL DEFAULT 'EUR',
    description  TEXT,
    created_at   TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT portfolios_kind_check CHECK (kind IN ('real', 'virtual'))
);

SELECT diesel_manage_updated_at('portfolios');
