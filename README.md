# Beehive Vault

Beehive Vault est un centre de pilotage financier personnel. Son objectif est de centraliser les finances d'un foyer afin de suivre son patrimoine, comprendre ses flux d'argent et prendre de meilleures décisions.

Le projet est actuellement conçu pour un usage personnel. Il doit toutefois rester suffisamment simple à étendre pour pouvoir être partagé, plus tard, avec la famille ou des proches.

## État du projet

Le produit est en cours de recentrage autour d'un premier socle universel :

1. comptes financiers ;
2. transactions et virements ;
3. soldes ;
4. patrimoine net ;
5. analyse mensuelle des revenus et dépenses.

Les fonctions spécialisées comme le suivi boursier avancé, le scoring d'entreprises, les budgets et la synchronisation bancaire viendront après la validation de ce socle.

## Documentation produit

- [Fondation produit](docs/product-foundation.md)
- [Modèle métier initial](docs/domain-model.md)
- [Feuille de route](docs/roadmap.md)
- [Architecture technique](docs/architecture.md)
- [Journal des décisions d'architecture](docs/adr/README.md)

## Socle technique

Le nouveau socle utilise Rust 2024, Axum, SQLx et PostgreSQL. L'application est
un monolithe modulaire organisé par fonctionnalité et n'utilise pas d'ORM.

Après avoir installé Rust et préparé une base PostgreSQL :

```bash
cp .env.example .env
cargo run
```

Les routes techniques initiales sont `GET /healthz` pour la vivacité du
processus et `GET /readyz` pour la disponibilité de PostgreSQL.

## Code existant

La version `0.1.0` et les branches historiques restent disponibles comme référence. La branche `reboot/mvp-foundation` porte la reconstruction du MVP.
