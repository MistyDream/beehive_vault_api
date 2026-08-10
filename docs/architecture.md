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
├── config.rs
├── main.rs
└── features/
    ├── health.rs
    ├── households/
    ├── accounts/
    ├── transactions/
    └── dashboard/
```

Chaque fonctionnalité ne crée que les éléments dont elle a besoin : routes, types, requêtes et éventuellement un service lorsqu'une véritable orchestration métier existe.

## Frontières conservées

- les règles financières ne dépendent pas des types HTTP ;
- les données entrantes sont validées à la frontière de l'API ;
- les opérations atomiques utilisent des transactions PostgreSQL ;
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
