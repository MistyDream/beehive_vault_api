# ADR-0006 — Exposer une pagination par page

- Statut : Accepté
- Date : 2026-08-17
- Remplace partiellement : ADR-0005, section « Consultation et pagination »

## Contexte

L'ADR-0005 retenait des paramètres HTTP `limit` et `offset` pour les listes de
transactions. Les transferts utilisent désormais la même pagination et les
futures collections auront probablement le même besoin.

Un offset correspond directement au fonctionnement de PostgreSQL, mais il est
moins naturel pour un client ou une interface qui présente des pages. Le
contrat HTTP ne doit pas exposer ce détail de stockage lorsqu'une notion plus
compréhensible suffit.

## Décision

Les collections paginées utilisent deux paramètres HTTP partagés :

- `page`, indexé à partir de 1 et égal à 1 par défaut ;
- `limit`, égal à 50 par défaut et compris entre 1 et 200.

L'application valide ces valeurs puis calcule l'offset PostgreSQL avec la
formule suivante :

```text
offset = (page - 1) * limit
```

Le calcul refuse les dépassements numériques. L'offset reste un détail interne
des repositories et n'est pas exposé dans le contrat HTTP.

Le nombre total d'éléments n'est pas calculé systématiquement. Une pagination
par curseur pourra remplacer cette stratégie si le volume réel le justifie.

## Conséquences positives

- le contrat est plus naturel pour l'interface et les clients de l'API ;
- transactions, transferts et futures collections partagent les mêmes règles ;
- la validation et les valeurs par défaut sont centralisées ;
- PostgreSQL conserve une requête simple avec `LIMIT` et `OFFSET`.

## Conséquences négatives

- accéder à une page éloignée conserve le coût d'un grand offset SQL ;
- l'ajout ou la suppression d'éléments entre deux requêtes peut décaler le
  contenu des pages ;
- l'absence de total empêche de connaître à l'avance la dernière page.

## Alternatives considérées

### Exposer directement un offset

Rejetée car l'offset est un détail technique moins pratique pour les clients.

### Introduire immédiatement une pagination par curseur

Rejetée pour le MVP car le volume personnel attendu ne justifie pas encore sa
complexité supplémentaire.
