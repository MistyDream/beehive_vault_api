# ADR-0005 — Modéliser les flux financiers par des transactions signées

- Statut : Accepté
- Date : 2026-08-12

La décision `limit/offset` de la section « Consultation et pagination » est
remplacée par l'[ADR-0006](0006-page-based-pagination.md). Les autres décisions
de cet ADR restent applicables.

## Contexte

Le socle financier représente actuellement les comptes, les établissements et des soldes de rapprochement datés. La phase suivante doit permettre de saisir des revenus et dépenses, de catégoriser les flux, de représenter les transferts internes et de recalculer les soldes sans introduire plusieurs sources concurrentes de vérité.

Le signe d'une écriture ne suffit pas à déterminer sa nature économique. Une dépense diminue un compte courant mais augmente une dette de carte de crédit. De même, un transfert entre un actif et une dette ne produit pas nécessairement deux montants bruts opposés.

Le modèle doit également préserver les écritures supprimées et importées, garantir l'appartenance des données à un même foyer et rester suffisamment simple pour le MVP.

## Décision

### Catégories rattachées au foyer

Une catégorie appartient à un foyer, pas à un compte. Elle peut ainsi être utilisée sur plusieurs comptes et dans les statistiques consolidées.

Lors de la création d'un foyer, l'application copie automatiquement un petit catalogue initial. Les catégories copiées deviennent propres au foyer et suivent exactement les mêmes règles que les catégories créées par l'utilisateur. Aucun champ ne distingue leur origine tant qu'aucun comportement métier ne le nécessite.

Une catégorie possède une nature `income` ou `expense`. Son nom est unique parmi les catégories actives du foyer sans distinction de casse. Elle peut être renommée ou archivée. Une catégorie archivée reste attachée aux transactions historiques mais ne peut plus être sélectionnée pour une nouvelle transaction.

Les sous-catégories sont reportées après le MVP.

Le catalogue initial est volontairement évolutif :

| Revenus | Dépenses |
|---|---|
| Salaire | Logement |
| Revenus professionnels | Alimentation |
| Revenus locatifs | Restaurants |
| Intérêts et dividendes | Transport |
| Prestations et pensions | Santé |
| Autres revenus | Assurances |
| | Abonnements |
| | Loisirs |
| | Achats personnels |
| | Voyages |
| | Impôts et taxes |
| | Frais bancaires |
| | Autres dépenses |

« Épargne » et « Investissement » ne sont pas des dépenses : déplacer de l'argent vers un autre compte patrimonial constitue un transfert. Un remboursement conserve la catégorie de la dépense initiale avec un effet économique inverse afin d'en réduire le total.

### Nature et montant d'une transaction

Une transaction possède une nature explicite :

- `income` pour un revenu ;
- `expense` pour une dépense ;
- `transfer` pour un mouvement interne entre deux comptes du foyer.

La nature reste obligatoire en l'absence de catégorie. Lorsqu'une catégorie est présente, sa nature doit correspondre à celle de la transaction. Un transfert ne possède pas de catégorie.

Le montant décimal signé représente uniquement l'effet sur le solde du compte :

- un montant positif augmente le solde ;
- un montant négatif diminue le solde ;
- un montant nul est interdit.

Cette convention est identique pour les actifs et les dettes. La devise n'est pas répétée sur la transaction : elle est celle du compte.

### Transferts représentés par deux transactions

Un transfert possède une identité dans une table `transfers`, mais ses deux transactions financières restent la source de vérité des montants :

- un mouvement `source` ;
- un mouvement `destination`.

La table `transfers` ne stocke aucun montant. La création du transfert et de ses deux mouvements est atomique dans PostgreSQL.

Le montant brut de chaque mouvement est calculé selon le rôle du compte :

| Rôle | Compte actif | Compte de dette |
|---|---:|---:|
| Source | `-montant` | `+montant` |
| Destination | `+montant` | `-montant` |

Les deux comptes doivent être distincts et appartenir au même foyer. Les effets économiques des mouvements doivent avoir la même valeur absolue et des signes opposés.

Un mouvement de transfert ne peut pas être modifié ou supprimé par les opérations génériques des transactions. Toute modification ou suppression passe par le transfert et traite les deux mouvements dans une même transaction PostgreSQL.

Les comptes, le montant, les dates, les libellés et les notes peuvent être corrigés par l'opération dédiée au transfert. Les dates source et destination peuvent différer lorsqu'un transfert met plusieurs jours à parvenir au compte de destination.

### Suppression logique

La suppression renseigne `deleted_at` au lieu de supprimer physiquement une transaction.

Une transaction supprimée :

- disparaît des listes ordinaires ;
- est exclue des soldes, revenus et dépenses ;
- ne peut plus être modifiée par les opérations ordinaires ;
- conserve sa trace et sa future clé de déduplication d'import.

La suppression d'un transfert marque atomiquement le transfert et ses deux mouvements comme supprimés. La restauration et un historique détaillé des modifications sont reportés.

### Règles de modification

Une transaction manuelle ordinaire peut corriger son compte, sa date de comptabilisation, son libellé, son montant, sa nature, sa catégorie et sa note. Chaque modification réapplique les invariants de création.

Son identifiant, son foyer, son origine et ses dates techniques restent immuables.

Une transaction importée peut modifier sa nature, sa catégorie et sa note. Son compte, son montant, sa date et son libellé bancaire restent inchangés afin de préserver la donnée du relevé. Une donnée bancaire incorrecte est supprimée logiquement puis remplacée par une transaction manuelle. La phase d'import conservera aussi la ligne source brute.

### Rapprochement en fin de journée

Un solde de rapprochement représente le solde à la fin de sa journée. Le solde courant est calculé ainsi :

```text
dernier solde de rapprochement
+ somme des transactions non supprimées dont la date de comptabilisation
  est strictement postérieure à la date du rapprochement
```

Les transactions de la même date sont considérées comme déjà incluses dans le rapprochement. Corriger ou supprimer une écriture antérieure à celui-ci modifie l'historique, mais pas le solde courant déjà recalé.

L'ordre des opérations d'une même journée ne change pas le solde de fin de journée. Le MVP n'invente pas d'heure lorsque la source fournit uniquement une date. Une future position d'import pourra préserver l'ordre d'affichage du relevé sans prétendre représenter un ordre d'exécution bancaire réel.

### Dates futures refusées

La date de comptabilisation d'une transaction doit être antérieure ou égale à la date actuelle dans le fuseau horaire du foyer. Le fuseau du foyer doit donc être validé.

Les dates passées restent autorisées. Une opération future relève d'un modèle distinct d'opération planifiée ou récurrente et ne devient une transaction qu'au moment de sa comptabilisation.

### Consultation et pagination

La collection principale appartient au foyer :

```text
POST   /v1/households/{household_id}/transactions
GET    /v1/households/{household_id}/transactions
GET    /v1/households/{household_id}/transactions/{transaction_id}
PATCH  /v1/households/{household_id}/transactions/{transaction_id}
DELETE /v1/households/{household_id}/transactions/{transaction_id}
```

La liste accepte les filtres `accountId`, `dateFrom`, `dateTo`, `nature`, `categoryId`, `source` et `search`. Son ordre par défaut est `booking_date DESC, created_at DESC, id DESC`.

Le MVP utilise une pagination `limit/offset` : limite de 50 par défaut, maximum de 200 et offset nul par défaut. Le nombre total n'est pas calculé systématiquement. Une pagination par curseur ne sera introduite que si le volume réel le justifie.

Les transactions supprimées ne sont pas exposées tant qu'aucune restauration ou vue d'audit n'existe.

Les transferts possèdent leur propre collection `/v1/households/{household_id}/transfers`, mais leurs mouvements apparaissent dans la liste consolidée des transactions.

### Calcul de l'effet économique et des flux

L'effet économique est dérivé du montant brut et du type de compte, sans être stocké :

```text
compte actif    : effet économique = montant
compte de dette : effet économique = -montant
```

Les revenus sont la somme des effets économiques des transactions `income`. Les dépenses sont la somme de l'opposé des effets économiques des transactions `expense`.

Les signes inverses restent autorisés. Un remboursement catégorisé comme dépense contribue négativement aux dépenses et réduit le total de sa catégorie. Une correction de revenu suit la règle symétrique.

Les transferts sont exclus des revenus et dépenses. Une transaction sans catégorie reste incluse dans les totaux et apparaît dans un groupe virtuel « Non catégorisé », qui n'est pas stocké en base.

Les rapports historiques incluent les comptes et catégories archivés, mais excluent les transactions supprimées.

## Modèle retenu

### Category

```text
Category
├── id: CategoryId
├── household_id: HouseholdId
├── name: String
├── kind: income | expense
├── archived_at: timestamp optional
├── created_at: timestamp
└── updated_at: timestamp
```

### Transaction

```text
Transaction
├── id: TransactionId
├── household_id: HouseholdId
├── account_id: AccountId
├── booking_date: date
├── label: String
├── amount: TransactionAmount
├── nature: income | expense | transfer
├── category_id: CategoryId optional
├── transfer_id: TransferId optional
├── transfer_role: source | destination, optional
├── note: String optional
├── source: manual | import
├── created_at: timestamp
├── updated_at: timestamp
└── deleted_at: timestamp optional
```

`household_id` est conservé même si le compte appartient déjà au foyer afin que PostgreSQL puisse garantir l'appartenance commune du compte, de la catégorie, du transfert et de la transaction.

### Transfer

```text
Transfer
├── id: TransferId
├── household_id: HouseholdId
├── created_at: timestamp
├── updated_at: timestamp
└── deleted_at: timestamp optional
```

## Contraintes PostgreSQL

La migration devra garantir :

- une clé unique composite `(household_id, id)` sur les comptes, catégories et transferts référencés par une transaction ;
- une clé étrangère composite entre la transaction et son compte ;
- une clé étrangère composite `(household_id, category_id, nature)` vers `(household_id, id, kind)` afin de garantir simultanément l'appartenance et la nature de la catégorie ;
- une clé étrangère composite entre la transaction et son transfert ;
- un montant `NUMERIC(20, 4)` non nul et différent de zéro ;
- une nature parmi `income`, `expense` et `transfer` ;
- une origine parmi `manual` et `import` ;
- un rôle parmi `source` et `destination` lorsqu'il est présent ;
- l'absence de catégorie et la présence d'un transfert et d'un rôle pour toute transaction `transfer` ;
- l'absence de transfert et de rôle pour toute transaction `income` ou `expense` ;
- au maximum un mouvement par rôle et par transfert grâce à un index unique partiel ;
- l'unicité insensible à la casse du nom d'une catégorie active dans un foyer.

PostgreSQL ne garantit pas qu'un transfert possède exactement deux mouvements sans ajouter un déclencheur différé. La complétude de la paire est garantie par le repository et la transaction PostgreSQL atomique, puis vérifiée par des tests d'intégration.

Les index initiaux couvrent :

- la liste consolidée par foyer et date ;
- le calcul du solde par compte et date ;
- l'historique par catégorie et date.

Aucun index textuel n'est ajouté avant qu'un volume réel ne le justifie.

## Validations applicatives

Rust garantit les règles qui dépendent de plusieurs agrégats ou du temps :

- le fuseau horaire du foyer est valide ;
- la date de comptabilisation n'est pas future dans ce fuseau ;
- le compte et la catégorie sont actifs lors d'une création ;
- les comptes d'un transfert sont distincts ;
- les deux mouvements d'un transfert sont créés ensemble ;
- leurs signes sont calculés selon les types de comptes ;
- leurs effets économiques sont opposés ;
- un mouvement de transfert ne peut pas être modifié isolément ;
- les champs bancaires d'une transaction importée restent immuables.

## Données reportées

- date de valeur ;
- bénéficiaire structuré ;
- géolocalisation ;
- pièce jointe ;
- opération récurrente ;
- état bancaire en attente ;
- ventilation entre plusieurs catégories ;
- détails d'import, empreinte de déduplication et position source ;
- taux de change.

## Conséquences positives

- les transactions financières restent l'unique source de vérité des mouvements ;
- les soldes, revenus, dépenses et remboursements sont calculables sans montant économique dupliqué ;
- les transferts entre actifs et dettes suivent la même convention ;
- PostgreSQL garantit directement l'appartenance au foyer et la cohérence des catégories ;
- la suppression logique préserve la traçabilité et la future déduplication des imports ;
- le catalogue de catégories reste personnalisable par foyer.

## Conséquences négatives

- la nature d'une transaction doit être stockée séparément de son montant ;
- le calcul des flux doit connaître si un compte représente un actif ou une dette ;
- la complétude d'un transfert repose sur le repository et ses tests plutôt que sur une contrainte déclarative unique ;
- les modifications d'un transfert nécessitent une opération dédiée ;
- la pagination par offset peut devenir coûteuse à très grand volume.

## Alternatives considérées

### Déduire la nature du signe

Rejetée car le même signe n'a pas la même signification économique sur un actif et une dette.

### Stocker un seul mouvement pour un transfert

Rejetée car chaque compte doit conserver son propre mouvement et son historique de solde.

### Stocker le montant sur la table des transferts

Rejetée car cela introduirait une source concurrente de vérité par rapport aux deux transactions financières.

### Partager des catégories globales entre tous les foyers

Rejetée car les renommages, archivages et personnalisations exigeraient alors un système de surcharges propre à chaque foyer.

### Supprimer physiquement les transactions

Rejetée afin de préserver la traçabilité, la cohérence historique et les futures clés de déduplication d'import.

### Utiliser immédiatement une pagination par curseur

Rejetée pour le MVP car le volume personnel attendu ne justifie pas encore sa complexité supplémentaire.
