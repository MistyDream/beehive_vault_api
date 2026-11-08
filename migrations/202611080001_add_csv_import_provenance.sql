ALTER TABLE transactions
    ADD COLUMN csv_statement_digest BYTEA,
    ADD COLUMN csv_source_line INTEGER,
    ADD COLUMN csv_file_name TEXT,
    ADD COLUMN csv_signed_amount NUMERIC(20, 4),
    ADD COLUMN csv_source_record TEXT,
    ADD CONSTRAINT transactions_csv_provenance_check CHECK (
        (
            csv_statement_digest IS NULL
            AND csv_source_line IS NULL
            AND csv_file_name IS NULL
            AND csv_signed_amount IS NULL
            AND csv_source_record IS NULL
        )
        OR
        (
            source = 'import'
            AND nature IN ('income', 'expense')
            AND csv_statement_digest IS NOT NULL
            AND octet_length(csv_statement_digest) = 32
            AND csv_source_line IS NOT NULL
            AND csv_source_line > 0
            AND csv_file_name IS NOT NULL
            AND char_length(csv_file_name) BETWEEN 1 AND 255
            AND csv_signed_amount IS NOT NULL
            AND abs(csv_signed_amount) = abs(amount)
            AND csv_source_record IS NOT NULL
            AND octet_length(csv_source_record) BETWEEN 1 AND 1048576
        )
    );

CREATE UNIQUE INDEX transactions_csv_row_unique
    ON transactions (account_id, csv_statement_digest, csv_source_line)
    WHERE csv_statement_digest IS NOT NULL;
