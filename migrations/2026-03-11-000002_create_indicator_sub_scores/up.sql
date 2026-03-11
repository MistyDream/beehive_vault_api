CREATE TABLE indicator_sub_scores (
    id                  SERIAL PRIMARY KEY,
    indicator_score_id  INTEGER NOT NULL REFERENCES indicator_scores (id) ON DELETE CASCADE,
    sub_score_type      VARCHAR NOT NULL,
    score               DOUBLE PRECISION NOT NULL,
    created_at          TIMESTAMP NOT NULL DEFAULT NOW(),

    CONSTRAINT indicator_sub_scores_score_check CHECK (score >= 0.0 AND score <= 100.0),
    CONSTRAINT indicator_sub_scores_unique UNIQUE (indicator_score_id, sub_score_type)
);

CREATE INDEX idx_indicator_sub_scores_indicator_score_id ON indicator_sub_scores (indicator_score_id);
