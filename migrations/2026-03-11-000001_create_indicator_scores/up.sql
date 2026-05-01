CREATE TABLE indicator_scores (
    id          SERIAL PRIMARY KEY,
    detail_id   INTEGER NOT NULL REFERENCES score_details (id) ON DELETE CASCADE,
    metric_key  VARCHAR NOT NULL,
    score       DOUBLE PRECISION NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT indicator_scores_score_check CHECK (score >= 0.0 AND score <= 100.0),
    CONSTRAINT indicator_scores_unique UNIQUE (detail_id, metric_key)
);

CREATE INDEX idx_indicator_scores_detail_id ON indicator_scores (detail_id);
