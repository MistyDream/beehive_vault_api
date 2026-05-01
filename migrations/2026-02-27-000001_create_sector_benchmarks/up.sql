CREATE TABLE sector_benchmarks (
    id          SERIAL PRIMARY KEY,
    sector      VARCHAR NOT NULL,
    industry    VARCHAR,
    metric_key  VARCHAR NOT NULL REFERENCES metrics_catalog (key) ON DELETE RESTRICT,
    value       DOUBLE PRECISION NOT NULL,
    source      VARCHAR NOT NULL DEFAULT 'gurufocus',
    period_end  DATE NOT NULL,
    fetched_at  TIMESTAMP NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT sector_benchmarks_unique
        UNIQUE (sector, industry, metric_key, period_end)
);

CREATE INDEX idx_sector_benchmarks_sector ON sector_benchmarks (sector);
CREATE INDEX idx_sector_benchmarks_sector_industry ON sector_benchmarks (sector, industry);
CREATE INDEX idx_sector_benchmarks_metric_key ON sector_benchmarks (metric_key);
CREATE INDEX idx_sector_benchmarks_lookup ON sector_benchmarks (sector, industry, metric_key, period_end DESC);
