# Catalogue des fonctionnalités

Ce document indique ce que le produit sait faire aujourd'hui, ce qui est en cours de développement et ce qui pourrait être ajouté. La [feuille de route](roadmap.md) reste la référence pour l'ordre de réalisation.

## Statuts

- **Disponible** : présent dans le code et utilisable par l'API ;
- **En développement** : implémentation commencée mais non terminée ;
- **Prévue** : retenue dans le périmètre du MVP ou dans une prochaine étape ;
- **Idée** : possibilité envisagée, sans engagement de réalisation.

## Disponibles

| Domaine | Fonctionnalité | Limites actuelles |
|---|---|---|
| Technique | Vérification de vivacité et de disponibilité de PostgreSQL | API locale sans authentification |
| Technique | Erreurs HTTP Problem Details RFC 9457 | Textes humains en anglais, sans identifiant d'occurrence |
| Foyer | Création, liste et consultation individuelle des foyers financiers | Un seul utilisateur, sans invitations ni permissions |
| Établissements | Consultation du catalogue global configuré côté serveur | Administration explicite côté serveur, aucune écriture par l'API HTTP |
| Comptes | Création, consultation, collections actives ou archivées et modification | Devise identique à celle du foyer |
| Comptes | Gestion des comptes d'actif et de dette | Pas de positions détaillées pour les investissements |
| Comptes | Sous-totaux par rôle, archivage à solde nul et restauration | Les comptes archivés restent principalement en lecture seule |
| Soldes | Solde initial, rapprochements datés et corrections | Dates non futures, nouveaux rapprochements strictement chronologiques |
| Soldes | Solde courant calculé | Ajoute les transactions non supprimées strictement postérieures au rapprochement |
| Patrimoine | Total des actifs, dettes et patrimoine net | Calculé depuis les soldes courants des comptes actifs |
| Catégories | Catalogue initial propre à chaque foyer | Pas de sous-catégories |
| Catégories | Création, filtrage, renommage et archivage | Une catégorie possède une nature immuable |
| Transactions | Création manuelle d'un revenu ou d'une dépense | Montant nominal positif et effet `standard` ou `reversal` |
| Transactions | Collection consolidée, filtres et pagination exacte | Un transfert compte comme une seule opération ; page 1 et limite 50 par défaut |
| Transactions | Résumés incorporés et montants dérivés | Compte et catégorie archivés conservés dans l'historique |
| Transactions | Modification et suppression logique | Les mouvements de transfert sont protégés des modifications isolées |
| Transferts | Création et consultation des deux mouvements liés | Les montants signés sont calculés selon les types de comptes |
| Transferts | Modification et suppression atomiques | Les deux mouvements sont toujours traités ensemble |
| Flux mensuels | Revenus, dépenses, flux net et ventilation par catégorie | Un rapport porte sur un seul mois civil |
| Flux mensuels | Accès aux transactions sources de chaque total | Réutilise la liste paginée des transactions et ses filtres |

## En développement

Aucune fonctionnalité produit n'est actuellement en développement. Le prochain lot de l'API est la prévisualisation CSV.

## Prévues

| Étape | Fonctionnalité | Objectif |
|---|---|---|
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

## Synchronisation bancaire différée

Le MVP conserve l'ajout manuel des comptes et des transactions. L'import CSV
constitue la première automatisation prévue afin de valider la qualité des
données, la prévisualisation et la déduplication avant d'introduire une
intégration externe.

Une évolution ultérieure pourra s'appuyer sur un prestataire d'agrégation
bancaire pour récupérer, après consentement de l'utilisateur, les comptes, leurs
soldes et leurs transactions. Cette synchronisation complétera toujours l'ajout
manuel, nécessaire pour les espèces, les établissements non pris en charge et
certains comptes d'investissement ou de dette.

Cette évolution devra traiter explicitement :

- l'identité de l'utilisateur auprès du prestataire et le rattachement de chaque
  connexion à un foyer ;
- les identifiants externes et l'idempotence des synchronisations ;
- la distinction entre soldes bancaires, soldes de rapprochement et soldes
  calculés par Beehive Vault ;
- les transactions ajoutées, modifiées ou supprimées par la banque ;
- les erreurs de synchronisation, les webhooks et le renouvellement du
  consentement bancaire ;
- la conservation des secrets et appels au prestataire exclusivement côté
  serveur.

La connexion bancaire ne déplacera jamais d'argent. Elle restera une source de
données importées dont Beehive Vault maîtrise la normalisation et la traçabilité.

Une idée ne passe dans les fonctionnalités prévues qu'après une décision sur son périmètre et sa priorité.
