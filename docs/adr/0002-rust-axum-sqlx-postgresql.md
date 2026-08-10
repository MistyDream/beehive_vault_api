# ADR-0002 — Utiliser Rust, Axum, SQLx et PostgreSQL sans ORM

- Statut : Accepté
- Date : 2026-08-10

## Contexte

Beehive Vault est d'abord un projet personnel et un support d'apprentissage de Rust. La version `0.1.0` utilisait Actix Web, Diesel et PostgreSQL. Diesel effectuait des opérations synchrones depuis une application asynchrone, ce qui nécessitait un pont d'exécution bloquant et des modèles de persistance dédiés.

Le produit manipule des montants financiers, des transactions atomiques et des agrégations qui doivent rester explicites. PostgreSQL est aussi destiné à rester la base principale si le projet est partagé avec plusieurs proches.

## Décision

Le backend utilise :

- Rust 2024 ;
- Axum pour l'API HTTP ;
- Tokio comme environnement asynchrone ;
- SQLx pour les connexions, migrations et requêtes SQL asynchrones ;
- PostgreSQL comme unique base de données prise en charge.

Aucun ORM traditionnel n'est utilisé. Les requêtes sont écrites en SQL et mappées vers des structures Rust. Les macros de vérification SQLx sont utilisées lorsqu'elles améliorent la sûreté sans dégrader inutilement l'environnement de développement.

## Conséquences positives

- le projet continue de servir l'apprentissage de Rust ;
- le chemin asynchrone ne nécessite plus de pont pour un ORM synchrone ;
- les requêtes et agrégations financières restent lisibles en SQL ;
- les types PostgreSQL peuvent être contrôlés explicitement ;
- le nombre de modèles et de conversions techniques diminue.

## Conséquences négatives

- l'équipe doit écrire et maîtriser davantage de SQL ;
- les requêtes peuvent être spécifiques à PostgreSQL ;
- certaines vérifications SQLx nécessitent un schéma de développement ou des
  métadonnées préparées hors ligne ;
- Rust ralentira parfois le prototypage par rapport à une stack dynamique ;
- changer de base de données représentera une migration significative.

Ces coûts sont acceptés car l'apprentissage de Rust et l'utilisation durable de
PostgreSQL font partie des objectifs explicites du projet.

## Alternatives considérées

### Conserver Actix Web et Diesel

Rejetée pour le redémarrage car cette combinaison préserverait une partie de la
complexité de persistance et du pont synchrone que le projet souhaite retirer.
Actix Web reste néanmoins une solution viable en soi.

### Utiliser un autre langage pour accélérer le MVP

Rejetée car elle supprimerait l'objectif d'apprentissage de Rust. Cette décision
pourra être réévaluée si la livraison du produit devient prioritaire sur cet
apprentissage.

### Utiliser SQLite

Rejetée comme base principale. SQLite simplifierait une application strictement
locale, mais PostgreSQL correspond mieux au déploiement serveur et à l'évolution
éventuelle vers plusieurs utilisateurs.
