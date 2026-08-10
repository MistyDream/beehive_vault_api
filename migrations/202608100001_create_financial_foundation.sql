CREATE TABLE households (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (char_length(trim(name)) BETWEEN 1 AND 100),
    base_currency TEXT NOT NULL CHECK (
        char_length(base_currency) = 3
        AND base_currency = upper(base_currency)
    ),
    timezone TEXT NOT NULL CHECK (char_length(trim(timezone)) BETWEEN 1 AND 100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE institutions (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (char_length(trim(name)) BETWEEN 1 AND 100),
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (household_id, id)
);

CREATE UNIQUE INDEX institutions_active_name_unique
    ON institutions (household_id, lower(name))
    WHERE archived_at IS NULL;

CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    institution_id UUID,
    name TEXT NOT NULL CHECK (char_length(trim(name)) BETWEEN 1 AND 100),
    kind TEXT NOT NULL CHECK (kind IN (
        'checking',
        'savings',
        'cash',
        'investment',
        'credit_card',
        'loan',
        'other_asset',
        'other_liability'
    )),
    currency TEXT NOT NULL CHECK (
        char_length(currency) = 3
        AND currency = upper(currency)
    ),
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (household_id, institution_id)
        REFERENCES institutions(household_id, id)
);

CREATE INDEX accounts_household_active_idx
    ON accounts (household_id, created_at)
    WHERE archived_at IS NULL;

CREATE INDEX accounts_institution_idx
    ON accounts (institution_id)
    WHERE institution_id IS NOT NULL;

CREATE TABLE account_balance_snapshots (
    id UUID PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    amount NUMERIC(20, 4) NOT NULL,
    balance_date DATE NOT NULL,
    source TEXT NOT NULL DEFAULT 'manual' CHECK (source IN (
        'manual',
        'import',
        'synchronization',
        'reconciliation'
    )),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, balance_date)
);

CREATE INDEX account_balance_snapshots_latest_idx
    ON account_balance_snapshots (account_id, balance_date DESC);
