# API du socle financier

Toutes les routes métier sont préfixées par `/v1` et utilisent JSON. Les UUID
sont générés par l'application et les montants décimaux sont transmis sous forme
de chaînes afin de préserver leur précision.

## Foyers

- `POST /v1/households` crée un foyer financier ;
- `GET /v1/households/{household_id}` consulte un foyer.

## Établissements

- `POST /v1/households/{household_id}/institutions` crée un établissement ;
- `GET /v1/households/{household_id}/institutions` liste les établissements ;
- `PATCH /v1/households/{household_id}/institutions/{institution_id}` le renomme ;
- `DELETE /v1/households/{household_id}/institutions/{institution_id}` l'archive.

## Comptes

- `POST /v1/households/{household_id}/accounts` crée un compte et son premier
  solde de rapprochement dans une même transaction PostgreSQL ;
- `GET /v1/households/{household_id}/accounts` liste les comptes actifs ;
- `GET /v1/households/{household_id}/accounts/{account_id}` consulte un compte ;
- `PATCH /v1/households/{household_id}/accounts/{account_id}` modifie son nom,
  son type ou son établissement ;
- `DELETE /v1/households/{household_id}/accounts/{account_id}` l'archive.

Le corps de création contient `name`, `kind`, `currency`, `initialBalance` et
`balanceDate`. `institutionId` est facultatif.

## Soldes de rapprochement

- `POST /v1/households/{household_id}/accounts/{account_id}/balances` ajoute un
  solde daté ;
- `GET /v1/households/{household_id}/accounts/{account_id}/balances` retourne
  l'historique du plus récent au plus ancien.

Les sources acceptées sont `manual`, `import`, `synchronization` et
`reconciliation`.

## Patrimoine

- `GET /v1/households/{household_id}/summary` retourne `assets`, `liabilities`,
  `netWorth` et `currency`.

Le résumé utilise actuellement le dernier solde de chaque compte. Lorsque les
transactions seront introduites, un compte bancaire utilisera le dernier solde
de rapprochement augmenté des transactions postérieures.
