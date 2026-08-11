# ADR-0004 — Composer explicitement les modules Axum

- Statut : Accepté
- Date : 2026-08-11

## Contexte

Le monolithe modulaire retenu par l'ADR-0001 organise le code par fonctionnalité, mais ne précise pas comment construire les dépendances ni comment déclarer les routes. Une composition centralisée dans un `AppState` global obligerait `app.rs` à connaître les services et repositories internes de chaque module. Elle conduirait également tous les handlers à dépendre d'un état commun qui grandirait avec l'application.

L'accès à PostgreSQL doit rester explicite. Les services contenant une orchestration métier ne doivent toutefois pas manipuler directement `PgPool` ou les transactions SQLx.

## Décision

Chaque module fonctionnel :

- reçoit un `Database` lors de sa configuration ;
- construit lui-même ses repositories et services internes ;
- possède un type d'état Axum dédié ;
- expose une fonction `routes(module)` retournant son routeur configuré.

Le type `Database` encapsule `PgPool` et expose l'accès au pool ainsi que l'ouverture d'une transaction. Il ne cherche pas à abstraire PostgreSQL et ne constitue pas un `UnitOfWork` générique.

`app.rs` crée `Database`, appelle la configuration de chaque module, fusionne leurs routeurs et applique les éléments globaux comme le préfixe de version. `main.rs` initialise l'infrastructure et appelle uniquement le constructeur de l'application.

La structure interne d'un module reste adaptée à sa complexité. Un handler peut utiliser directement `Database` pour une opération simple. Lorsqu'un cas d'utilisation contient des validations, une orchestration ou plusieurs écritures atomiques, le module peut introduire un service et un repository concrets sans trait obligatoire.

## Conséquences positives

- chaque fonctionnalité contrôle ses dépendances et ses routes ;
- `app.rs` ne dépend pas des handlers, services ou repositories internes ;
- les états Axum restent petits et propres à leur module ;
- les services métier peuvent rester indépendants de SQLx ;
- l'ajout ou le retrait d'un module modifie principalement la composition de l'application ;
- la construction des dépendances reste explicite et vérifiée par Rust.

## Conséquences négatives

- chaque module possède un peu de code de configuration répétitif ;
- `Database` reste directement disponible dans les modules simples ;
- partager ultérieurement une transaction entre plusieurs modules nécessitera une conception supplémentaire ;
- les routes représentant plusieurs domaines devront être rattachées explicitement à un module propriétaire.

## Alternatives considérées

### Utiliser un `AppState` global

Rejetée car sa construction doit connaître les détails internes de tous les modules et parce que chaque handler reçoit alors davantage de dépendances que nécessaire.

### Utiliser un conteneur d'injection de dépendances dynamique

Rejetée car la composition explicite est courte, lisible et vérifiée à la compilation. Un conteneur ajouterait de la résolution dynamique sans besoin actuel.

### Introduire un `UnitOfWork` générique

Rejetée pour le moment. Les transactions propres à un cas d'utilisation sont gérées par le repository concerné. Cette décision pourra être réévaluée lorsqu'une opération atomique devra coordonner plusieurs modules.
