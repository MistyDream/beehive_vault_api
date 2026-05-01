CREATE TABLE metric_values (
    id          BIGSERIAL PRIMARY KEY,
    stock_id    INTEGER NOT NULL REFERENCES stocks (id) ON DELETE CASCADE,
    metric_key  VARCHAR NOT NULL REFERENCES metrics_catalog (key) ON DELETE RESTRICT,
    period      VARCHAR NOT NULL,
    period_end  DATE NOT NULL,
    value       DOUBLE PRECISION NOT NULL,
    unit        VARCHAR,
    currency    VARCHAR,
    source      VARCHAR NOT NULL DEFAULT 'gurufocus',
    fetched_at  TIMESTAMP NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT metric_values_period_check CHECK (
        period IN ('FY', 'TTM', 'Q')
    ),
    CONSTRAINT metric_values_unique UNIQUE (stock_id, metric_key, period, period_end)
);

CREATE INDEX idx_metric_values_stock_id        ON metric_values (stock_id);
CREATE INDEX idx_metric_values_stock_metric    ON metric_values (stock_id, metric_key);
CREATE INDEX idx_metric_values_stock_period_end ON metric_values (stock_id, period_end DESC);
CREATE INDEX idx_metric_values_metric_period   ON metric_values (metric_key, period_end DESC);
CREATE INDEX idx_metric_values_fetched_at      ON metric_values (fetched_at DESC);
