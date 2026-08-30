CREATE TABLE global_institutions (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (
        name = trim(name)
        AND char_length(name) BETWEEN 1 AND 100
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX global_institutions_name_unique
    ON global_institutions (lower(name));

INSERT INTO global_institutions (id, name, created_at, updated_at)
SELECT DISTINCT ON (lower(trim(name)))
    id,
    trim(name),
    created_at,
    updated_at
FROM institutions
ORDER BY lower(trim(name)), created_at, id;

ALTER TABLE accounts
    DROP CONSTRAINT accounts_household_id_institution_id_fkey;

UPDATE accounts AS account
SET institution_id = global_institution.id
FROM institutions AS household_institution
JOIN global_institutions AS global_institution
    ON lower(global_institution.name) = lower(trim(household_institution.name))
WHERE account.institution_id = household_institution.id;

DROP TABLE institutions;

ALTER TABLE global_institutions RENAME TO institutions;
ALTER INDEX global_institutions_name_unique RENAME TO institutions_name_unique;

ALTER TABLE accounts
    ADD CONSTRAINT accounts_institution_id_fkey
    FOREIGN KEY (institution_id)
    REFERENCES institutions(id)
    ON DELETE SET NULL;
