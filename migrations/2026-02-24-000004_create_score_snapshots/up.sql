CREATE TABLE score_snapshots (
    id           SERIAL PRIMARY KEY,
    stock_id     INTEGER NOT NULL REFERENCES stocks (id) ON DELETE CASCADE,
    scored_at    TIMESTAMP NOT NULL,
    global_score DOUBLE PRECISION NOT NULL,
    created_at   TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT score_snapshots_unique UNIQUE (stock_id, scored_at),
    CONSTRAINT score_snapshots_global_score_check CHECK (global_score >= 0.0 AND global_score <= 100.0)
);

CREATE INDEX idx_score_snapshots_stock_id   ON score_snapshots (stock_id);
CREATE INDEX idx_score_snapshots_stock_time ON score_snapshots (stock_id, scored_at DESC);
CREATE INDEX idx_score_snapshots_scored_at  ON score_snapshots (scored_at DESC);
