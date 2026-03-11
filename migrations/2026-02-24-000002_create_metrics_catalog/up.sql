CREATE TABLE metrics_catalog (
    id               SERIAL PRIMARY KEY,
    key              VARCHAR NOT NULL,
    name             VARCHAR NOT NULL,
    category         VARCHAR NOT NULL,
    data_type        VARCHAR NOT NULL,
    unit             VARCHAR,
    frequency        VARCHAR,
    higher_is_better BOOLEAN NOT NULL DEFAULT TRUE,
    min_plausible    DOUBLE PRECISION,
    max_plausible    DOUBLE PRECISION,
    notes            TEXT,
    created_at       TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT metrics_catalog_key_unique UNIQUE (key),
    CONSTRAINT metrics_catalog_category_check CHECK (
        category IN ('valuation', 'profitability', 'growth', 'solidity', 'shareholder_return', 'tactical')
    ),
    CONSTRAINT metrics_catalog_data_type_check CHECK (
        data_type IN ('percent', 'multiple', 'currency', 'bool')
    )
);

CREATE INDEX idx_metrics_catalog_category ON metrics_catalog (category);

SELECT diesel_manage_updated_at('metrics_catalog');
