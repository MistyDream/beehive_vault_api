# API du socle financier

Toutes les routes métier sont préfixées par `/v1` et utilisent JSON. Les UUID sont générés par l'application et les montants décimaux sont transmis sous forme de chaînes afin de préserver leur précision.

## Foyers

- `POST /v1/households` crée un foyer financier ;
- `GET /v1/households/{household_id}` consulte un foyer.

## Établissements

- `POST /v1/households/{household_id}/institutions` crée un établissement ;
- `GET /v1/households/{household_id}/institutions` liste les établissements ;
- `PATCH /v1/households/{household_id}/institutions/{institution_id}` le renomme ;
- `DELETE /v1/households/{household_id}/institutions/{institution_id}` l'archive.

## Catégories

- `POST /v1/households/{household_id}/categories` crée une catégorie personnalisée ;
- `GET /v1/households/{household_id}/categories` liste les catégories actives ;
- `PATCH /v1/households/{household_id}/categories/{category_id}` renomme une catégorie ;
- `DELETE /v1/households/{household_id}/categories/{category_id}` l'archive.

La création d'un foyer crée atomiquement son catalogue initial de 19 catégories.
Une catégorie possède un `kind` égal à `income` ou `expense`. La liste accepte le filtre facultatif `kind`. Le type d'une catégorie est immuable après sa création.

## Comptes

- `POST /v1/households/{household_id}/accounts` crée un compte et son premier solde de rapprochement dans une même transaction PostgreSQL ;
- `GET /v1/households/{household_id}/accounts` liste les comptes actifs ;
- `GET /v1/households/{household_id}/accounts/{account_id}` consulte un compte ;
- `PATCH /v1/households/{household_id}/accounts/{account_id}` modifie son nom, son type ou son établissement ;
- `DELETE /v1/households/{household_id}/accounts/{account_id}` l'archive.

Le corps de création contient `name`, `kind`, `currency`, `initialBalance` et `balanceDate`. `institutionId` est facultatif.

## Soldes de rapprochement

- `POST /v1/households/{household_id}/accounts/{account_id}/balances` ajoute un solde daté ;
- `GET /v1/households/{household_id}/accounts/{account_id}/balances` retourne l'historique du plus récent au plus ancien.

Les sources acceptées sont `manual`, `import`, `synchronization` et `reconciliation`.

## Patrimoine

- `GET /v1/households/{household_id}/summary` retourne `assets`, `liabilities`, `netWorth` et `currency`.

Le résumé utilise actuellement le dernier solde de chaque compte. Les transactions sont disponibles, mais leur prise en compte dans le solde calculé sera ajoutée à l'étape 2.5 de la feuille de route : le dernier solde de rapprochement sera alors augmenté des transactions strictement postérieures.
