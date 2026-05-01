CREATE TABLE score_details (
    id          SERIAL PRIMARY KEY,
    snapshot_id INTEGER NOT NULL REFERENCES score_snapshots (id) ON DELETE CASCADE,
    category    VARCHAR NOT NULL,
    score       DOUBLE PRECISION NOT NULL,
    weight      DOUBLE PRECISION NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT score_details_category_check CHECK (
        category IN ('valuation', 'profitability', 'growth', 'solidity', 'shareholder_return', 'tactical')
    ),
    CONSTRAINT score_details_score_check CHECK (score >= 0.0 AND score <= 100.0),
    CONSTRAINT score_details_unique UNIQUE (snapshot_id, category),
    CONSTRAINT score_details_weight_check CHECK (weight > 0.0 AND weight <= 1.0)
);

CREATE INDEX idx_score_details_snapshot_id ON score_details (snapshot_id);
CREATE INDEX idx_score_details_category    ON score_details (category);
