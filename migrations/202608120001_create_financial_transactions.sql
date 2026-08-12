ALTER TABLE accounts
    ADD CONSTRAINT accounts_household_id_id_unique
    UNIQUE (household_id, id);

CREATE TABLE categories (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    name TEXT NOT NULL CHECK (char_length(trim(name)) BETWEEN 1 AND 100),
    kind TEXT NOT NULL CHECK (kind IN ('income', 'expense')),
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (household_id, id, kind)
);

CREATE UNIQUE INDEX categories_active_name_unique
    ON categories (household_id, lower(name))
    WHERE archived_at IS NULL;

INSERT INTO categories (id, household_id, name, kind)
SELECT gen_random_uuid(), households.id, initial_categories.name, initial_categories.kind
FROM households
CROSS JOIN (
    VALUES
        ('Salaire', 'income'),
        ('Revenus professionnels', 'income'),
        ('Revenus locatifs', 'income'),
        ('Intérêts et dividendes', 'income'),
        ('Prestations et pensions', 'income'),
        ('Autres revenus', 'income'),
        ('Logement', 'expense'),
        ('Alimentation', 'expense'),
        ('Restaurants', 'expense'),
        ('Transport', 'expense'),
        ('Santé', 'expense'),
        ('Assurances', 'expense'),
        ('Abonnements', 'expense'),
        ('Loisirs', 'expense'),
        ('Achats personnels', 'expense'),
        ('Voyages', 'expense'),
        ('Impôts et taxes', 'expense'),
        ('Frais bancaires', 'expense'),
        ('Autres dépenses', 'expense')
) AS initial_categories(name, kind);

CREATE TABLE transfers (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL REFERENCES households(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    UNIQUE (household_id, id)
);

CREATE TABLE transactions (
    id UUID PRIMARY KEY,
    household_id UUID NOT NULL,
    account_id UUID NOT NULL,
    booking_date DATE NOT NULL,
    label TEXT NOT NULL CHECK (char_length(trim(label)) BETWEEN 1 AND 500),
    amount NUMERIC(20, 4) NOT NULL CHECK (amount <> 0),
    nature TEXT NOT NULL CHECK (nature IN ('income', 'expense', 'transfer')),
    category_id UUID,
    transfer_id UUID,
    transfer_role TEXT CHECK (transfer_role IN ('source', 'destination')),
    note TEXT CHECK (note IS NULL OR char_length(note) <= 2000),
    source TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual', 'import')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (household_id, account_id)
        REFERENCES accounts(household_id, id)
        ON DELETE CASCADE,
    FOREIGN KEY (household_id, category_id, nature)
        REFERENCES categories(household_id, id, kind),
    FOREIGN KEY (household_id, transfer_id)
        REFERENCES transfers(household_id, id),
    CHECK (
        (
            nature = 'transfer'
            AND category_id IS NULL
            AND transfer_id IS NOT NULL
            AND transfer_role IS NOT NULL
        )
        OR
        (
            nature IN ('income', 'expense')
            AND transfer_id IS NULL
            AND transfer_role IS NULL
        )
    )
);

CREATE UNIQUE INDEX transactions_transfer_role_unique
    ON transactions (transfer_id, transfer_role)
    WHERE transfer_id IS NOT NULL;

CREATE INDEX transactions_household_list_idx
    ON transactions (
        household_id,
        booking_date DESC,
        created_at DESC,
        id DESC
    )
    WHERE deleted_at IS NULL;

CREATE INDEX transactions_account_balance_idx
    ON transactions (account_id, booking_date)
    WHERE deleted_at IS NULL;

CREATE INDEX transactions_category_history_idx
    ON transactions (household_id, category_id, booking_date DESC)
    WHERE deleted_at IS NULL AND category_id IS NOT NULL;
