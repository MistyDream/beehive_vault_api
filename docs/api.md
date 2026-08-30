# API du socle financier

Toutes les routes métier sont préfixées par `/v1` et utilisent JSON. Les UUID sont générés par l'application et les montants décimaux sont transmis sous forme de chaînes afin de préserver leur précision.

Les [contrats du nouveau client Web](client-contracts.md) décrivent séparément les représentations cibles validées mais pas encore entièrement implémentées.
Le présent document reste la référence du comportement actuellement disponible.

## Pagination

Les collections paginées acceptent `page` et `limit`. `page` commence à 1 et vaut 1 par défaut. `limit` vaut 50 par défaut et accepte une valeur comprise entre 1 et 200. L'offset PostgreSQL est calculé en interne et n'appartient pas au contrat HTTP.

Dans le contrat actuellement implémenté, le nombre total d'éléments n'est pas retourné systématiquement.

Les réponses actuelles sont encore des tableaux JSON. La cible validée pour le nouveau client utilise une enveloppe `items`, `page`, `limit` et `total`. Le client déduira l'existence d'une page suivante ; aucun membre `hasMore` ne sera retourné. Ce changement est détaillé dans les [contrats du client](client-contracts.md#pagination).

## Erreurs

Les routes métier retournent leurs erreurs au format Problem Details de la RFC 9457 avec le type de contenu `application/problem+json`. Les routes techniques `/healthz` et `/readyz` conservent leur représentation dédiée.

Chaque réponse contient :

- `type`, l'identité canonique et stable du problème sous la forme `urn:beehive-vault:problem:<type>` ;
- `title`, son résumé stable en anglais ;
- `status`, identique au statut HTTP ;
- `code`, une extension stable en `snake_case` qui simplifie le routage côté client ;
- `detail`, uniquement lorsqu'une explication sûre et utile existe pour cette occurrence ;
- `errors`, uniquement pour détailler une ou plusieurs valeurs invalides.

`instance` est omis tant que l'API ne produit pas d'identifiant d'occurrence exploitable. Le client utilise `type` comme identité canonique, peut utiliser `code` comme raccourci et n'interprète jamais `title` ou `detail` comme des identifiants.

Chaque entrée de `errors` contient `location` (`body`, `path` ou `query`), `pointer` sous forme de fragment JSON Pointer, `code` et `detail`. Par exemple :

```json
{
  "type": "urn:beehive-vault:problem:validation-error",
  "title": "Request validation failed",
  "status": 422,
  "detail": "One or more request values are invalid.",
  "code": "validation_error",
  "errors": [
    {
      "location": "body",
      "pointer": "#/baseCurrency",
      "code": "invalid_value",
      "detail": "The value has an invalid format or type."
    }
  ]
}
```

Un JSON mal formé ou un paramètre de chemin illisible produit `400`. Une valeur bien formée mais invalide sémantiquement produit `422`. Les ressources absentes produisent `404`, y compris lorsqu'un foyer parent d'une collection imbriquée n'existe pas. Les conflits métier produisent `409`, un type de contenu non pris en charge `415`, un corps trop volumineux `413`, une méthode non prise en charge `405` et une route inconnue `404`.

Le catalogue stable des problèmes généraux et métier est le suivant. La valeur complète de `type` s'obtient en préfixant son suffixe par `urn:beehive-vault:problem:`.

| Statut | Suffixe de `type` | `code` |
| ---: | --- | --- |
| 400 | `invalid-request` | `invalid_request` |
| 404 | `route-not-found` | `route_not_found` |
| 404 | `household-not-found` | `household_not_found` |
| 404 | `institution-not-found` | `institution_not_found` |
| 404 | `account-not-found` | `account_not_found` |
| 404 | `category-not-found` | `category_not_found` |
| 404 | `transaction-not-found` | `transaction_not_found` |
| 404 | `transfer-not-found` | `transfer_not_found` |
| 405 | `method-not-allowed` | `method_not_allowed` |
| 409 | `duplicate-institution-name` | `duplicate_institution_name` |
| 409 | `duplicate-category-name` | `duplicate_category_name` |
| 409 | `duplicate-balance-date` | `duplicate_balance_date` |
| 409 | `account-kind-change-forbidden` | `account_kind_change_forbidden` |
| 409 | `transfer-movement-update-forbidden` | `transfer_movement_update_forbidden` |
| 409 | `transfer-movement-delete-forbidden` | `transfer_movement_delete_forbidden` |
| 409 | `imported-transaction-fields-immutable` | `imported_transaction_fields_immutable` |
| 413 | `payload-too-large` | `payload_too_large` |
| 415 | `unsupported-media-type` | `unsupported_media_type` |
| 422 | `validation-error` | `validation_error` |
| 500 | `internal-error` | `internal_error` |

Les corps JSON ignorent les membres inconnus afin de préserver la compatibilité ascendante. Les détails SQL et autres informations internes ne sont jamais exposés. Le catalogue et les principes de stabilité sont définis dans l'[ADR-0007](adr/0007-rfc-9457-problem-details.md).

## Foyers

- `POST /v1/households` crée un foyer financier ;
- `GET /v1/households` liste les foyers disponibles ;
- `GET /v1/households/{household_id}` consulte un foyer.

La liste n'est pas paginée afin que le client puisse résoudre le foyer actif en une seule requête. Elle retourne un tableau JSON, vide lorsqu'aucun foyer n'existe, ordonné par nom sans distinction de casse, puis par date de création et identifiant.

L'API ne possède pas encore d'authentification ni d'autorisation : la liste contient donc actuellement tous les foyers de l'installation locale. Une future autorisation limitera cette même collection aux foyers accessibles à l'utilisateur authentifié.

## Établissements

`GET /v1/institutions` retourne le catalogue global configuré côté serveur. La
collection n'est pas paginée, peut être vide et contient uniquement `id` et
`name`. Elle est ordonnée par nom sans distinction de casse, puis par nom et
identifiant.

Un compte référence facultativement une entrée globale avec `institutionId`.
Le client ne peut ni créer, ni renommer, ni archiver un établissement. Les
anciennes routes propres au foyer ont été retirées conformément à
l'[ADR-0008](adr/0008-global-financial-institution-catalog.md).

La maintenance du catalogue reste hors de l'API HTTP. Le binaire serveur `beehive_vault_admin` permet de lister, ajouter, renommer et importer des établissements. L'import initial est additif, réentrant et déclenché explicitement ; il ne supprime ni ne renomme les entrées déjà stockées. Son utilisation est décrite dans le [guide de développement](development.md#administrer-le-catalogue-détablissements).

La migration regroupe les anciennes entrées par nom normalisé sans distinction de casse, conserve la plus ancienne comme identité canonique et rattache les comptes existants à celle-ci. La représentation est également décrite dans les [contrats du client](client-contracts.md#catalogue-global-détablissements).

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

La cible du nouveau client ajoute les sous-totaux de ses trois groupes, la consultation des comptes archivés et leur restauration. Elle interdit également l'archivage d'un compte dont le solde calculé n'est pas nul. Ces comportements restent à implémenter et sont définis dans les [contrats du client](client-contracts.md#collection-des-comptes).

## Soldes de rapprochement

- `POST /v1/households/{household_id}/accounts/{account_id}/balances` ajoute un solde daté ;
- `GET /v1/households/{household_id}/accounts/{account_id}/balances` retourne l'historique du plus récent au plus ancien.

Les sources acceptées sont `manual`, `import`, `synchronization` et `reconciliation`.

La correction d'un solde existant, le refus des dates futures et l'obligation qu'un nouveau rapprochement soit postérieur au dernier appartiennent au [contrat cible](client-contracts.md#soldes-de-rapprochement).

## Transactions

- `POST /v1/households/{household_id}/transactions` crée un revenu ou une dépense manuelle ;
- `GET /v1/households/{household_id}/transactions` liste les mouvements du foyer ;
- `GET /v1/households/{household_id}/transactions/{transaction_id}` consulte un mouvement ;
- `PATCH /v1/households/{household_id}/transactions/{transaction_id}` modifie un mouvement ordinaire ;
- `DELETE /v1/households/{household_id}/transactions/{transaction_id}` le supprime logiquement.

La liste accepte `accountId`, `dateFrom`, `dateTo`, `nature`, `categoryId`, `uncategorized`, `source`, `search`, `page` et `limit`. `uncategorized=true` sélectionne les revenus et dépenses sans catégorie, exclut les transferts et ne peut pas être combiné avec `categoryId`. Les mouvements d'un transfert apparaissent dans cette liste, mais ne peuvent pas être modifiés ou supprimés isolément.

Le champ `amount` actuellement exposé représente le montant signé stocké sur le compte. Cette représentation reste le contrat effectif jusqu'à l'implémentation du contrat cible du nouveau client Web.

### Cible du nouveau client Web

La conception de la liste, du détail et des formulaires a stabilisé une collection consolidée où un transfert apparaît une seule fois, une pagination avec `total`, des résumés compacts incorporés et une représentation distincte du montant nominal, de l'effet économique et du montant du compte.

Ces évolutions ne sont pas encore implémentées. Leur contrat complet, notamment la saisie `standard` ou `reversal`, les références archivées et la précision maximale de quatre décimales, est défini dans les [contrats du client](client-contracts.md#collection-consolidée-des-opérations).
Le MVP retient un pictogramme neutre côté Web plutôt qu'une métadonnée d'icône dans l'API.

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
