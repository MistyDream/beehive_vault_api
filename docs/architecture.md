# Architecture technique

Ce document décrit l'architecture actuellement retenue. L'historique et les raisons des décisions sont conservés dans les [Architecture Decision Records](adr/README.md).

## Décision

Beehive Vault utilise un monolithe modulaire composé de :

- Rust 2024 ;
- Axum pour l'API HTTP ;
- Tokio pour l'exécution asynchrone ;
- SQLx pour l'accès SQL asynchrone ;
- PostgreSQL comme base de données.

SQLx est utilisé sans ORM : les requêtes restent écrites en SQL et sont mappées vers des structures Rust.

## Organisation

Le code est organisé par fonctionnalité plutôt que par couche technique :

```text
src/
├── app.rs
├── bin/
│   └── beehive_vault_admin.rs
├── config.rs
├── database.rs
├── main.rs
├── types.rs
└── features/
    ├── health/
    ├── households/
    ├── institutions/
    ├── accounts/
    └── net_worth/
```

Chaque module fonctionnel possède sa configuration, son état Axum et ses routes. Il expose `configure(database)` pour construire ses dépendances et `routes(module)` pour produire son routeur. `app.rs` assemble ces routeurs et applique les préoccupations globales telles que le préfixe `/v1`.

Le binaire `beehive_vault_admin` porte les opérations serveur qui ne doivent pas être exposées par HTTP. Il réutilise les services des modules fonctionnels et la même configuration PostgreSQL que l'API.

Le type `Database` encapsule le pool PostgreSQL. Il fournit l'accès au pool pour les requêtes simples et l'ouverture explicite d'une transaction pour les opérations atomiques. Chaque module reçoit un clone léger de `Database` lors de sa configuration ; aucun état global Axum n'est partagé entre tous les modules.

La structure interne reste proportionnelle à la complexité de la fonctionnalité. Un module simple peut contenir uniquement `mod.rs` et `handlers.rs`. Le module `accounts`, qui porte des validations et une création atomique, sépare actuellement :

```text
accounts/
├── domain.rs
├── dto.rs
├── handlers.rs
├── mod.rs
├── repository.rs
└── service.rs
```

Le handler traite HTTP, le service orchestre le cas d'utilisation et le repository contient SQLx ainsi que les transactions PostgreSQL. Aucun trait de repository n'est ajouté tant qu'une seconde implémentation ou un besoin de substitution précis ne le justifie.

## Frontières conservées

- les règles financières ne dépendent pas des types HTTP ;
- les données entrantes sont validées à la frontière de l'API ;
- les opérations atomiques utilisent des transactions PostgreSQL ;
- les services métier ne manipulent ni `PgPool` ni les transactions SQLx ;
- chaque module construit ses dépendances et possède son routeur Axum ;
- les services externes sont isolés derrière un trait lorsqu'une substitution est réellement utile ;
- les calculs purs sont testés sans base de données ;
- les requêtes et migrations sont testées avec PostgreSQL.

## Ce qui n'est pas introduit par défaut

- repository générique pour chaque table ;
- séparation systématique `domain/application/infrastructure` ;
- modèle de persistance distinct lorsqu'il est identique au type retourné ;
- trait ne possédant qu'une seule implémentation sans besoin de test précis ;
- abstraction destinée à remplacer PostgreSQL.

Une abstraction est ajoutée lorsqu'un besoin concret apparaît, pas uniquement pour préserver une possibilité théorique.
