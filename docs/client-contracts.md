# Contrats API du nouveau client Web

- Statut : disponible
- Date : 2026-08-31

Ce document décrit les contrats stabilisés et disponibles pour les premiers parcours du nouveau client Web. Il complète la [vue d'ensemble de l'API](api.md), qui reste la référence synthétique du comportement réel.

## Principes communs

- les montants sont transmis sous forme de chaînes décimales en base 10, sans notation exponentielle ;
- une chaîne monétaire accepte au maximum 16 chiffres avant le séparateur et 4 chiffres après celui-ci, conformément à `NUMERIC(20, 4)` ;
- les zéros décimaux finaux ne font pas partie du contrat : deux chaînes qui représentent la même valeur peuvent avoir une échelle différente ;
- les erreurs utilisent les Problem Details RFC 9457 déjà disponibles ;
- les réponses de lecture incorporent uniquement les résumés nécessaires à leur affichage immédiat ;
- les requêtes de mutation continuent de référencer les ressources par leur identifiant ;
- les membres JSON inconnus restent ignorés pour préserver la compatibilité ascendante.

## Pagination

Les collections paginées conservent les paramètres `page` et `limit` définis par l'[ADR-0006](adr/0006-page-based-pagination.md). L'enveloppe et son `total` exact suivent l'[ADR-0009](adr/0009-stabilize-web-client-contracts.md) :

```json
{
  "items": [],
  "page": 1,
  "limit": 50,
  "total": 127
}
```

`total` est un entier positif ou nul qui compte les éléments correspondant aux filtres avant pagination. Une page située après la dernière page retourne `items: []` tout en conservant le même total. Le client détermine l'existence d'une page suivante avec `page * limit < total` ; l'API ne duplique donc pas cette information dans un membre `hasMore`.

Cette enveloppe s'applique aux collections des transactions et des transferts. Dans la collection consolidée des transactions, un transfert compte comme une seule opération.

## Résumés incorporés

Une transaction ordinaire incorpore un résumé compact de son compte :

```json
{
  "id": "019c…",
  "name": "Compte courant",
  "kind": "checking",
  "archived": false
}
```

Sa catégorie facultative utilise un résumé compact distinct :

```json
{
  "id": "019c…",
  "name": "Alimentation",
  "kind": "expense",
  "archived": false
}
```

Ces résumés utilisent le nom actuel de la ressource. Ils restent produits lorsque le compte ou la catégorie est archivé, mais ne constituent pas un instantané historique de son ancien nom. Ils n'incluent ni dates techniques, ni soldes, ni autres données de la ressource complète.

Le serveur construit ces résumés avec les jointures de la requête de lecture.
Le contrat ne demande aucune requête supplémentaire par transaction.

## Collection consolidée des opérations

`GET /v1/households/{household_id}/transactions` est la collection chronologique des opérations financières du foyer. Chaque élément possède un discriminateur `operationType` égal à `transaction` ou `transfer`.

### Transaction ordinaire

Une transaction ordinaire est représentée ainsi :

```json
{
  "operationType": "transaction",
  "id": "019c…",
  "householdId": "019c…",
  "bookingDate": "2026-08-22",
  "label": "Courses",
  "nature": "expense",
  "amount": "42.00",
  "effect": "standard",
  "economicAmount": "-42.00",
  "accountAmount": "-42.00",
  "account": {
    "id": "019c…",
    "name": "Compte courant",
    "kind": "checking",
    "archived": false
  },
  "category": {
    "id": "019c…",
    "name": "Alimentation",
    "kind": "expense",
    "archived": false
  },
  "origin": "manual",
  "note": null,
  "createdAt": "2026-08-22T10:00:00Z",
  "updatedAt": "2026-08-22T10:00:00Z"
}
```

`amount` est toujours le montant nominal strictement positif. `economicAmount` est son effet signé sur le patrimoine et `accountAmount` le montant signé appliqué au solde du compte. `effect` vaut :

- `standard` pour un revenu qui augmente le patrimoine ou une dépense qui le diminue ;
- `reversal` pour une correction de revenu, un remboursement ou une autre opération dont l'effet inverse la nature habituelle.

La conversion est définie par la table suivante :

| `nature` | `effect` | `economicAmount` |
| --- | --- | ---: |
| `income` | `standard` | `+amount` |
| `income` | `reversal` | `-amount` |
| `expense` | `standard` | `-amount` |
| `expense` | `reversal` | `+amount` |

Pour un compte d'actif, `accountAmount` est égal à `economicAmount`. Pour un compte de dette, il est égal à son opposé.

La catégorie vaut `null` lorsque la transaction n'est pas catégorisée. `origin` vaut `manual` ou `import` et ne partage ainsi pas le nom du mouvement `source` d'un transfert dans l'union des opérations.

### Transfert

Un transfert apparaît une seule fois et incorpore ses deux mouvements :

```json
{
  "operationType": "transfer",
  "id": "019c…",
  "householdId": "019c…",
  "bookingDate": "2026-08-21",
  "amount": "250.00",
  "source": {
    "transactionId": "019c…",
    "bookingDate": "2026-08-21",
    "label": "Virement épargne",
    "accountAmount": "-250.00",
    "account": {
      "id": "019c…",
      "name": "Compte courant",
      "kind": "checking",
      "archived": false
    },
    "note": null
  },
  "destination": {
    "transactionId": "019c…",
    "bookingDate": "2026-08-22",
    "label": "Virement reçu",
    "accountAmount": "250.00",
    "account": {
      "id": "019c…",
      "name": "Livret A",
      "kind": "savings",
      "archived": false
    },
    "note": null
  },
  "createdAt": "2026-08-21T10:00:00Z",
  "updatedAt": "2026-08-21T10:00:00Z"
}
```

Le montant du transfert est nominal, strictement positif et économiquement neutre pour le foyer. La date du mouvement source est sa `bookingDate` canonique et détermine son classement dans la collection consolidée. Les deux dates détaillées restent disponibles.

Le filtre `accountId` sélectionne un transfert lorsque l'un de ses deux comptes correspond. Les filtres de date utilisent sa date canonique. La recherche porte sur les libellés et notes des deux mouvements. Un filtre de catégorie exclut les transferts.

La consultation, la modification et la suppression d'un transfert continuent d'utiliser `/transfers/{transfer_id}`. Les identifiants de ses mouvements ne sont pas des destinations éditables dans le contrat du client Web.

## Saisie des montants ordinaires

La création d'un revenu ou d'une dépense exige un `amount` nominal strictement positif et un `effect` :

```json
{
  "accountId": "019c…",
  "bookingDate": "2026-08-22",
  "label": "Courses",
  "amount": "42.00",
  "effect": "standard",
  "nature": "expense",
  "categoryId": "019c…",
  "note": null
}
```

L'API dérive l'effet économique depuis `nature` et `effect`, puis le montant brut du compte depuis sa famille actif ou dette. Le client ne calcule jamais ces signes. Pendant une modification, `amount` et `effect` sont facultatifs et conservent séparément leur valeur actuelle lorsqu'ils sont absents. Une transaction ordinaire ne peut toujours pas devenir un transfert, ni l'inverse.

## Collection des comptes

`GET /v1/households/{household_id}/accounts` reste non paginé et accepte `status=active` ou `status=archived`. La valeur par défaut est `active`.

La réponse contient les comptes et les sous-totaux calculés par l'API :

```json
{
  "items": [],
  "totals": {
    "daily": "0.00",
    "savings": "0.00",
    "liabilities": "0.00"
  }
}
```

Les groupes correspondent aux types suivants :

- `daily` : `checking` et `cash` ;
- `savings` : `savings`, `investment` et `other_asset` ;
- `liabilities` : `credit_card`, `loan` et `other_liability`.

Le client classe les lignes selon `kind`, mais ne recalcule pas les sous-totaux.
Chaque sous-total additionne les `calculatedBalance` de son groupe. Un solde de dette positif représente un montant dû ; un solde négatif représente une position créditrice et conserve son signe.
Le résumé global `GET /v1/households/{household_id}/summary` reste séparé.
Chaque compte conserve `institutionId` ; le nom est résolu depuis le catalogue global stable des établissements.

La consultation individuelle retrouve un compte actif ou archivé :

```text
GET /v1/households/{household_id}/accounts/{account_id}
```

## Archivage et restauration des comptes

`DELETE /v1/households/{household_id}/accounts/{account_id}` archive uniquement un compte dont `calculatedBalance` est exactement nul. Un autre solde produit un conflit `account_balance_not_zero`.

La restauration utilise :

```text
POST /v1/households/{household_id}/accounts/{account_id}/restore
```

Elle retourne le compte actif. Restaurer un compte déjà actif retourne aussi ce compte afin que l'opération soit idempotente.

## Soldes de rapprochement

Le solde initial et tout nouveau solde doivent être antérieurs ou égaux à la date actuelle dans le fuseau horaire du foyer. Un nouveau solde ajouté à un compte existant doit également être strictement postérieur au solde le plus récent.

Une correction utilise :

```text
PATCH /v1/households/{household_id}/accounts/{account_id}/balances/{balance_id}
```

Le corps accepte `amount`, `balanceDate` ou les deux :

```json
{
  "amount": "1250.00",
  "balanceDate": "2026-08-22"
}
```

Au moins un champ doit être fourni. La correction conserve la source originale et accepte toute date non future qui ne crée pas de doublon pour le compte. Une correction peut donc changer le solde qui sert de point d'ancrage le plus récent.

Les problèmes propres aux soldes et à l'archivage sont disponibles :

| Disponibilité | Statut | Suffixe de `type`          | `code`                     |
| ------------- | -----: | -------------------------- | -------------------------- |
| Disponible    |    404 | `balance-not-found`        | `balance_not_found`        |
| Disponible    |    409 | `account-balance-not-zero` | `account_balance_not_zero` |
| Disponible    |    409 | `duplicate-balance-date`   | `duplicate_balance_date`   |

Les dates futures et les dates qui ne sont pas postérieures au dernier solde utilisent `validation-error` avec respectivement les codes de champ `balance_date_in_future` et `balance_date_not_after_latest`.

## Catalogue global d'établissements

Le contrat de l'[ADR-0008](adr/0008-global-financial-institution-catalog.md) est désormais disponible :

```text
GET /v1/institutions
```

La réponse est une collection non paginée et ordonnée par nom :

```json
[
  {
    "id": "019c…",
    "name": "Établissement exemple"
  }
]
```

Les routes de création, modification et archivage propres au foyer ont disparu.
Les entrées existantes sont regroupées sans distinction de casse et les comptes sont rattachés à leur entrée globale. Une installation neuve commence avec un catalogue vide tant que l'administrateur n'a pas appliqué le catalogue initial fourni côté serveur ; l'absence d'établissement ne bloque jamais un compte.

## Catégories et icônes

Le contrat des catégories ne reçoit pas de métadonnée d'icône pendant le MVP.
Le client garantit un pictogramme neutre pour toute catégorie. Cette décision évite de rendre publique une identité d'icône avant qu'un catalogue stable ou un besoin d'administration existe.

## État de l'implémentation

Tous les lots du contrat sont disponibles :

1. catalogue global d'établissements : disponible ;
2. validations de date et correction des soldes : disponibles ;
3. comptes archivés, restauration et sous-totaux : disponibles ;
4. collection consolidée, montants, résumés incorporés et pagination : disponible.
