# API du socle financier

Toutes les routes métier sont préfixées par `/v1` et utilisent JSON. Les UUID sont générés par l'application et les montants décimaux sont transmis sous forme de chaînes afin de préserver leur précision.

## Pagination

Les collections paginées acceptent `page` et `limit`. `page` commence à 1 et vaut 1 par défaut. `limit` vaut 50 par défaut et accepte une valeur comprise entre 1 et 200. L'offset PostgreSQL est calculé en interne et n'appartient pas au contrat HTTP.

Le nombre total d'éléments n'est pas retourné systématiquement.

Le nouveau client Web utilisera un chargement explicite « Afficher plus » sans afficher le nombre total. Avant son intégration, les collections concernées doivent néanmoins indiquer si une page suivante existe, avec `hasMore` ou une information équivalente. Le format commun de cette métadonnée reste à stabiliser.

## Erreurs

L'API retourne actuellement les erreurs sous la forme JSON `{ code, message }`.
L'[ADR-0007](adr/0007-rfc-9457-problem-details.md) retient leur migration vers le format Problem Details de la RFC 9457 avec le type de contenu `application/problem+json`. Cette migration doit être implémentée avant que le
nouveau client web dépende de ce contrat.

## Foyers

- `POST /v1/households` crée un foyer financier ;
- `GET /v1/households/{household_id}` consulte un foyer.

## Établissements

Le contrat actuellement implémenté reste :

- `POST /v1/households/{household_id}/institutions` crée un établissement ;
- `GET /v1/households/{household_id}/institutions` liste les établissements ;
- `PATCH /v1/households/{household_id}/institutions/{institution_id}` le renomme ;
- `DELETE /v1/households/{household_id}/institutions/{institution_id}` l'archive.

L'[ADR-0008](adr/0008-global-financial-institution-catalog.md) remplace ce modèle par un catalogue global configuré côté serveur et exposé à terme avec `GET /v1/institutions`. Cette migration doit précéder l'intégration des comptes
par le nouveau client web.

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
- `calculatedBalance`, ce montant augmenté des transactions non supprimées strictement postérieures à `balanceDate`.

Les transactions du jour du rapprochement sont considérées comme déjà incluses dans `latestBalance`.

Le type peut changer au sein d'une même famille actif ou dette. Le passage entre ces deux familles est refusé dès que le compte possède une transaction, y compris supprimée logiquement.

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

La liste accepte `accountId`, `dateFrom`, `dateTo`, `nature`, `categoryId`, `uncategorized`, `source`, `search`, `page` et `limit`. `uncategorized=true` sélectionne les revenus et dépenses sans catégorie, exclut les transferts et ne peut pas être combiné avec `categoryId`. Les mouvements d'un transfert apparaissent dans cette liste, mais ne peuvent pas être modifiés ou supprimés isolément.

Le champ `amount` actuellement exposé représente le montant signé stocké sur le compte. Cette représentation reste le contrat effectif tant que le contrat de saisie du nouveau client Web n'est pas stabilisé.

### Besoins du nouveau client Web

La conception Web de la liste, du détail et des formulaires met en évidence les évolutions suivantes, qui ne sont pas encore implémentées :

- accompagner la pagination d'une information indiquant l'existence d'une page suivante, sans rendre le total obligatoire ;
- exposer l'effet économique affichable d'un mouvement sans demander au client de recalculer les règles propres aux comptes d'actif et de dette ;
- enrichir un mouvement de transfert avec le compte opposé et les informations nécessaires à un résumé « compte source → compte destination » ;
- rendre résolubles les libellés des comptes et catégories archivés encore référencés par les transactions historiques ;
- définir une sémantique de saisie qui accepte un montant nominal compréhensible tout en préservant les remboursements et corrections de signe inverse ;
- stabiliser la précision maximale acceptée et affichable pour chaque devise sans demander au client de convertir les décimaux en nombres binaires ;
- stabiliser une métadonnée d'icône pour les catégories si le catalogue doit porter ce choix, le client conservant dans tous les cas une icône neutre de repli.

Le choix entre des résumés incorporés aux réponses et des référentiels incluant les éléments archivés reste ouvert. Il devra éviter les appels supplémentaires par transaction et ne pas faire dépendre l'affichage historique d'un libellé courant introuvable.

## Transferts

- `POST /v1/households/{household_id}/transfers` crée atomiquement les mouvements source et destination ;
- `GET /v1/households/{household_id}/transfers` liste les transferts ;
- `GET /v1/households/{household_id}/transfers/{transfer_id}` consulte un transfert ;
- `PATCH /v1/households/{household_id}/transfers/{transfer_id}` modifie atomiquement ses deux mouvements ;
- `DELETE /v1/households/{household_id}/transfers/{transfer_id}` supprime logiquement l'ensemble.

Le montant nominal est strictement positif. Les montants signés des deux mouvements sont calculés selon leurs rôles et selon que chaque compte représente un actif ou une dette. Les dates, libellés et notes peuvent différer entre les deux mouvements.

## Patrimoine

- `GET /v1/households/{household_id}/summary` retourne `assets`, `liabilities`, `netWorth` et `currency`.

Le résumé utilise `calculatedBalance` pour chaque compte actif. Le montant brut d'un compte de dette est inversé pour obtenir son effet économique, puis les valeurs sont réparties entre `assets`, `liabilities` et `netWorth`.

## Flux mensuels

- `GET /v1/households/{household_id}/monthly-flows/{month}` retourne le rapport d'un mois civil, où `month` respecte le format `YYYY-MM`.

Le rapport expose les bornes du mois, la devise du foyer, les revenus, les dépenses et le flux net. Les revenus et dépenses contiennent chacun un total, un nombre de transactions et une ventilation par catégorie. Une catégorie nulle représente le groupe virtuel « Non catégorisé ».

L'effet économique inverse le montant brut des transactions portées par un compte de dette. Les remboursements et corrections conservent leur signe et peuvent donc réduire le total d'une section ou d'une catégorie. Les transferts et les transactions supprimées sont exclus. Les comptes et catégories archivés restent inclus dans l'historique.

Les transactions sources d'un total sont consultées avec la collection des transactions et les mêmes bornes, nature et catégorie. Le filtre `uncategorized=true` permet de retrouver les sources du groupe sans catégorie.
