# Fondation produit

## Vision

Beehive Vault centralise les finances d'un foyer afin de suivre son patrimoine, comprendre ses flux d'argent et prendre de meilleures décisions.

À terme, le produit pourra couvrir l'ensemble du parcours financier : comptes, budget, épargne, crédits, investissements, objectifs, projections et analyses.
Cette ambition guide les choix de conception, mais ne définit pas le périmètre du premier MVP.

## Utilisateur initial

Le premier utilisateur est le créateur du projet. Le produit privilégie donc :
- la valeur personnelle immédiate plutôt que les fonctions commerciales ;
- la saisie et l'import de données plutôt que les intégrations coûteuses ;
- une interface compréhensible plutôt qu'une configuration exhaustive ;
- la maîtrise et la confidentialité des données financières.

Le modèle prévoit un **foyer financier** pouvant contenir plusieurs membres, mais le premier MVP ne développe ni invitation ni gestion avancée des droits.

## Problème à résoudre en premier

Les informations financières sont dispersées entre plusieurs établissements, types de comptes et fichiers. Il est difficile d'obtenir une réponse rapide et fiable à ces questions :
- Combien est-ce que je possède réellement aujourd'hui ?
- Où se trouve mon argent ?
- Combien ai-je gagné et dépensé ce mois-ci ?
- Quelles catégories expliquent mes dépenses ?
- Comment mon patrimoine évolue-t-il ?

## Proposition de valeur du MVP

Le MVP permet de représenter tous les comptes d'un foyer, d'enregistrer ou d'importer leurs transactions, puis d'afficher une vue consolidée du patrimoine et des flux mensuels.

## Principes produit

1. **Une donnée explicable.** Chaque total doit pouvoir être relié aux comptes et transactions qui le composent.
2. **Une saisie progressive.** Une information inconnue ne doit pas empêcher d'utiliser le reste du produit.
3. **Les transferts ne sont pas des dépenses.** Un mouvement entre deux comptes du foyer ne modifie ni les revenus ni les dépenses consolidés.
4. **Le présent avant la prédiction.** Le MVP restitue correctement les données connues avant de proposer simulations ou recommandations.
5. **Un usage personnel sans impasse.** L'expérience commence avec un seul utilisateur, tandis que les données restent rattachées à un foyer.
6. **La simplicité avant l'automatisation.** La saisie manuelle et l'import CSV précèdent la synchronisation bancaire.

## Périmètre du premier MVP

### Inclus

- créer un foyer financier initial ;
- créer, modifier, archiver et consulter des comptes ;
- gérer les principaux types de comptes, actifs et dettes ;
- saisir manuellement des transactions ;
- importer des transactions depuis un fichier CSV ;
- catégoriser les revenus et dépenses ;
- représenter un virement entre deux comptes sans le compter comme une dépense ;
- consulter les soldes actuels ;
- consulter le patrimoine net consolidé ;
- consulter les revenus et dépenses d'un mois ;
- visualiser la répartition des dépenses par catégorie.

### Explicitement exclu

- synchronisation automatique avec les banques ;
- paiement ou mouvement réel d'argent ;
- invitations et permissions multi-utilisateurs ;
- budgets et alertes ;
- objectifs et projections ;
- fiscalité ;
- valorisation automatique de l'immobilier ;
- analyse et scoring d'entreprises ;
- recommandations financières automatisées ;
- application mobile native.

Ces exclusions sont des reports, pas des abandons.

## Parcours principal

1. L'utilisateur crée son foyer financier.
2. Il ajoute ses comptes courants, d'épargne, d'investissement ou de crédit.
3. Il renseigne un solde initial.
4. Il saisit quelques transactions ou importe un relevé CSV.
5. Il corrige les catégories si nécessaire.
6. Il consulte son patrimoine net et le résumé du mois.

## Critères de réussite

Le MVP est validé lorsque l'utilisateur peut :

- représenter tous ses comptes réels sans contournement majeur ;
- retrouver un patrimoine net cohérent avec ses établissements ;
- importer un mois de transactions sans créer de doublons ;
- distinguer correctement revenus, dépenses et transferts ;
- expliquer chaque chiffre affiché dans le tableau de bord ;
- utiliser le produit régulièrement pour suivre ses finances.

## Hypothèses à valider

- L'import CSV couvre suffisamment le besoin avant une synchronisation bancaire.
- Un solde initial suivi de transactions permet un démarrage simple et fiable.
- Une devise principale par foyer suffit pour le premier MVP.
- Les comptes d'investissement peuvent d'abord être suivis par leur valeur
  globale, sans détail de chaque titre.
