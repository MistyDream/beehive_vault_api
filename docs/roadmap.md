# Feuille de route

La feuille de route suit des tranches verticales : chaque étape doit produire un
résultat utilisable, de PostgreSQL jusqu'à l'API ou à l'interface.

## État actuel

- **Phase active :** phase 2 — Transactions manuelles ;
- **Dernière étape terminée :** 2.4 — Transferts atomiques ;
- **Prochaine étape :** 2.5 — Soldes calculés.

Légende :

- ✅ terminé ;
- 🚧 en cours ;
- ⬜ à faire.

## Phase 0 — Recentrage ✅

### 0.1 — Fondation produit ✅

- définir la vision, l'utilisateur initial et le périmètre du MVP ;
- documenter les parcours principaux et les critères de réussite ;
- conserver la version `0.1.0` comme référence historique.

### 0.2 — Fondation technique ✅

- retenir Rust, Axum, SQLx et PostgreSQL sans ORM ;
- adopter un monolithe modulaire organisé par fonctionnalité ;
- documenter les décisions structurantes dans des ADR ;
- préparer l'environnement PostgreSQL de développement et d'intégration ;
- définir les vérifications locales et la stratégie de tests.

**Résultat :** le projet dispose d'un périmètre clair et d'un socle technique
simple sur lequel reconstruire le MVP.

## Phase 1 — Comptes et patrimoine initial ✅

La phase 1 est terminée. Le patrimoine exposé à ce stade correspond à une
photographie fondée sur le dernier solde de rapprochement de chaque compte. La
prise en compte des transactions postérieures appartient à l'étape 2.5.

### 1.1 — Foyer et établissements ✅

- créer et consulter le foyer financier ;
- créer, lister, renommer et archiver ses établissements ;
- garantir l'appartenance des établissements au foyer.

### 1.2 — Comptes ✅

- gérer les principaux comptes d'actif et de dette ;
- créer un compte avec un établissement facultatif ;
- consulter, modifier et archiver un compte ;
- limiter le MVP à la devise principale du foyer.

### 1.3 — Soldes de rapprochement ✅

- créer atomiquement le compte et son solde initial ;
- ajouter des soldes de rapprochement datés ;
- consulter leur historique du plus récent au plus ancien ;
- conserver la source de chaque solde.

### 1.4 — Première vue consolidée ✅

- utiliser le dernier solde connu de chaque compte actif ;
- distinguer les actifs et les dettes ;
- calculer le patrimoine net ;
- exposer le résultat par l'API du foyer ;
- couvrir le parcours complet par un test d'intégration PostgreSQL.

**Résultat :** l'utilisateur peut représenter la photographie actuelle de son
patrimoine et obtenir un total consolidé par l'API.

## Phase 2 — Transactions manuelles 🚧

### 2.1 — Conception et stockage ✅

- définir les transactions signées et leurs invariants dans un ADR ;
- rattacher les catégories au foyer ;
- modéliser les transactions ordinaires et les transferts ;
- créer le schéma PostgreSQL et ses contraintes ;
- introduire les identifiants et types métier nécessaires.

### 2.2 — Catégories ✅

- créer automatiquement le catalogue initial de chaque foyer ;
- créer et lister les catégories actives ;
- filtrer les catégories par nature ;
- renommer et archiver une catégorie ;
- préserver les catégories archivées sur l'historique existant.

### 2.3 — Transactions ordinaires ✅

- créer une transaction manuelle de revenu ou de dépense ;
- consulter une transaction et lister celles du foyer ;
- filtrer et paginer la liste ;
- modifier les champs autorisés ;
- supprimer logiquement une transaction ;
- valider le compte, la catégorie et la date de comptabilisation ;
- protéger les mouvements de transfert et les champs bancaires importés ;
- couvrir les règles par des tests unitaires et un test d'intégration PostgreSQL.

### 2.4 — Transferts atomiques ✅

- créer un transfert et ses deux mouvements dans une transaction PostgreSQL ;
- calculer le signe de chaque mouvement selon le rôle du compte ;
- consulter et lister les transferts ;
- modifier atomiquement le transfert et ses deux mouvements ;
- supprimer logiquement le transfert et ses deux mouvements ;
- tester les transferts entre comptes actifs et comptes de dette.

### 2.5 — Soldes calculés ⬜

- partir du dernier solde de rapprochement de chaque compte ;
- ajouter les transactions non supprimées strictement postérieures ;
- exposer le solde calculé de chaque compte ;
- recalculer les actifs, les dettes et le patrimoine net.

### 2.6 — Flux mensuels ⬜

- calculer les revenus et dépenses selon le type du compte ;
- exclure les transferts des revenus et dépenses ;
- regrouper les flux par catégorie ;
- regrouper les transactions sans catégorie sous « Non catégorisé » ;
- permettre de retrouver les transactions sources d'un total.

**Résultat attendu :** l'utilisateur peut maintenir ses finances au fil du
temps et comprendre ses flux mensuels.

## Phase 3 — Import CSV ⬜

### 3.1 — Configuration et prévisualisation ⬜

- configurer le format d'un relevé ;
- analyser le fichier sans modifier les données ;
- présenter les lignes valides et rejetées avant confirmation.

### 3.2 — Import et déduplication ⬜

- importer les transactions validées atomiquement ;
- conserver les données bancaires sources ;
- détecter les doublons ;
- permettre la catégorisation après import.

**Résultat attendu :** l'utilisateur peut intégrer un relevé réel sans tout
ressaisir et sans créer de doublons.

## Phase 4 — Tableau de bord du MVP ⬜

### 4.1 — Patrimoine ⬜

- afficher le patrimoine net actuel ;
- afficher sa répartition par compte et établissement ;
- afficher son évolution dans le temps.

### 4.2 — Revenus et dépenses ⬜

- afficher les revenus et dépenses mensuels ;
- afficher leur répartition par catégorie ;
- permettre de remonter de chaque total aux transactions sources.

### 4.3 — Validation en usage réel ⬜

- préparer des données de démonstration représentatives ;
- vérifier le parcours complet du foyer au tableau de bord ;
- utiliser le produit régulièrement avec des données personnelles ;
- ajuster la suite selon les difficultés réellement rencontrées.

**Résultat attendu :** le socle du MVP répond aux questions définies dans la
fondation produit.

## Après validation du MVP

L'ordre sera déterminé par l'usage réel plutôt que par la quantité de fonctions
possibles. Les candidats principaux sont :

- budgets et opérations récurrentes ;
- objectifs d'épargne et projections ;
- suivi détaillé des investissements ;
- synchronisation bancaire ;
- partage familial ;
- suivi immobilier et des crédits ;
- analyses financières avancées.
