# Catalogue des fonctionnalités

Ce document indique ce que le produit sait faire aujourd'hui, ce qui est en
cours de développement et ce qui pourrait être ajouté. La
[feuille de route](roadmap.md) reste la référence pour l'ordre de réalisation.

## Statuts

- **Disponible** : présent dans le code et utilisable par l'API ;
- **En développement** : implémentation commencée mais non terminée ;
- **Prévue** : retenue dans le périmètre du MVP ou dans une prochaine étape ;
- **Idée** : possibilité envisagée, sans engagement de réalisation.

## Disponibles

| Domaine | Fonctionnalité | Limites actuelles |
|---|---|---|
| Technique | Vérification de vivacité et de disponibilité de PostgreSQL | API locale sans authentification |
| Foyer | Création et consultation d'un foyer financier | Un seul utilisateur, sans invitations ni permissions |
| Établissements | Création, liste, renommage et archivage | Pas de synchronisation avec les établissements réels |
| Comptes | Création, consultation, liste, modification et archivage | Devise identique à celle du foyer |
| Comptes | Gestion des comptes d'actif et de dette | Pas de positions détaillées pour les investissements |
| Soldes | Solde initial et soldes de rapprochement datés | Le solde calculé n'inclut pas encore les transactions postérieures |
| Patrimoine | Total des actifs, dettes et patrimoine net | Calculé depuis le dernier solde de chaque compte |
| Catégories | Catalogue initial propre à chaque foyer | Pas de sous-catégories |
| Catégories | Création, filtrage, renommage et archivage | Une catégorie possède une nature immuable |
| Transactions | Création manuelle d'un revenu ou d'une dépense | Pas encore de création par import CSV |
| Transactions | Consultation, filtres et pagination | Pagination par limite et offset |
| Transactions | Modification et suppression logique | Les mouvements de transfert nécessiteront leur API dédiée |

## En développement

Aucune fonctionnalité produit n'est actuellement en développement. La prochaine
étape retenue est la gestion des transferts atomiques.

## Prévues

| Étape | Fonctionnalité | Objectif |
|---|---|---|
| 2.4 | Transferts atomiques | Créer, modifier et supprimer ensemble les deux mouvements liés |
| 2.5 | Soldes calculés | Ajouter au dernier rapprochement les transactions postérieures |
| 2.6 | Flux mensuels | Calculer revenus, dépenses et ventilation par catégorie |
| 3.1 | Prévisualisation CSV | Vérifier un relevé et ses erreurs avant import |
| 3.2 | Import CSV | Importer sans doublons et conserver les données bancaires sources |
| 4.1 | Tableau de bord du patrimoine | Afficher le patrimoine actuel et son évolution |
| 4.2 | Tableau de bord des flux | Afficher les revenus, dépenses et catégories du mois |
| 4.3 | Validation en usage réel | Tester le parcours complet avec des données représentatives |

## Idées après le MVP

- budgets et alertes ;
- opérations planifiées ou récurrentes ;
- suivi des abonnements et de l'évolution de leur coût ;
- détection automatique des opérations récurrentes ;
- calendrier de trésorerie et solde prévisionnel ;
- règles de catégorisation automatique ;
- rapprochement bancaire et identification des écarts ;
- détection des doublons, frais nouveaux et dépenses inhabituelles ;
- objectifs d'épargne et projections ;
- enveloppes financières françaises comme le PEA, le CTO, l'assurance-vie, le
  PEL, le Livret A et le LDDS ;
- suivi détaillé des titres et investissements ;
- suivi consolidé des frais bancaires, de courtage et de gestion ;
- synchronisation bancaire ;
- partage familial, invitations et permissions ;
- remboursements et dépenses partagées ;
- pièces jointes et justificatifs associés aux transactions ;
- tags personnalisés pour compléter les catégories ;
- historique des modifications et restauration des données supprimées ;
- export complet et portabilité des données ;
- gestion multidevise et taux de change ;
- suivi immobilier et détaillé des crédits ;
- fiscalité ;
- analyses financières avancées.

Une idée ne passe dans les fonctionnalités prévues qu'après une décision sur son
périmètre et sa priorité.
