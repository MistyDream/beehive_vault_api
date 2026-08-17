# API du socle financier

Toutes les routes métier sont préfixées par `/v1` et utilisent JSON. Les UUID sont générés par l'application et les montants décimaux sont transmis sous forme de chaînes afin de préserver leur précision.

## Pagination

Les collections paginées acceptent `page` et `limit`. `page` commence à 1 et
vaut 1 par défaut. `limit` vaut 50 par défaut et accepte une valeur comprise
entre 1 et 200. L'offset PostgreSQL est calculé en interne et n'appartient pas
au contrat HTTP.

Le nombre total d'éléments n'est pas retourné systématiquement.

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

Les réponses d'un compte distinguent :

- `latestBalance`, le montant du rapprochement le plus récent ;
- `balanceDate`, la date de ce rapprochement ;
- `calculatedBalance`, ce montant augmenté des transactions non supprimées
  strictement postérieures à `balanceDate`.

Les transactions du jour du rapprochement sont considérées comme déjà incluses
dans `latestBalance`.

Le type peut changer au sein d'une même famille actif ou dette. Le passage entre
ces deux familles est refusé dès que le compte possède une transaction, y
compris supprimée logiquement.

## Soldes de rapprochement

- `POST /v1/households/{household_id}/accounts/{account_id}/balances` ajoute un solde daté ;
- `GET /v1/households/{household_id}/accounts/{account_id}/balances` retourne l'historique du plus récent au plus ancien.

Les sources acceptées sont `manual`, `import`, `synchronization` et `reconciliation`.

## Transactions

- `POST /v1/households/{household_id}/transactions` crée un revenu ou une dépense manuelle ;
- `GET /v1/households/{household_id}/transactions` liste les mouvements du foyer ;
- `GET /v1/households/{household_id}/transactions/{transaction_id}` consulte un mouvement ;
- `PATCH /v1/households/{household_id}/transactions/{transaction_id}` modifie un mouvement ordinaire ;
- `DELETE /v1/households/{household_id}/transactions/{transaction_id}` le supprime logiquement.

La liste accepte `accountId`, `dateFrom`, `dateTo`, `nature`, `categoryId`,
`source`, `search`, `page` et `limit`. Les mouvements d'un transfert apparaissent
dans cette liste, mais ne peuvent pas être modifiés ou supprimés isolément.

## Transferts

- `POST /v1/households/{household_id}/transfers` crée atomiquement les mouvements source et destination ;
- `GET /v1/households/{household_id}/transfers` liste les transferts ;
- `GET /v1/households/{household_id}/transfers/{transfer_id}` consulte un transfert ;
- `PATCH /v1/households/{household_id}/transfers/{transfer_id}` modifie atomiquement ses deux mouvements ;
- `DELETE /v1/households/{household_id}/transfers/{transfer_id}` supprime logiquement l'ensemble.

Le montant nominal est strictement positif. Les montants signés des deux
mouvements sont calculés selon leurs rôles et selon que chaque compte représente
un actif ou une dette. Les dates, libellés et notes peuvent différer entre les
deux mouvements.

## Patrimoine

- `GET /v1/households/{household_id}/summary` retourne `assets`, `liabilities`, `netWorth` et `currency`.

Le résumé utilise `calculatedBalance` pour chaque compte actif. Le montant brut
d'un compte de dette est inversé pour obtenir son effet économique, puis les
valeurs sont réparties entre `assets`, `liabilities` et `netWorth`.
