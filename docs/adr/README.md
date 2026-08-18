# Architecture Decision Records

Ce répertoire conserve les décisions d'architecture importantes de Beehive Vault. Un ADR explique le contexte d'une décision, le choix effectué, les alternatives considérées et ses conséquences.

## Cycle de vie

Un ADR possède l'un des statuts suivants :

- `Proposé` : la décision est en discussion ;
- `Accepté` : la décision guide le projet ;
- `Remplacé` : un nouvel ADR prend sa place ;
- `Abandonné` : la décision n'est plus applicable sans remplacement direct.

Un ADR accepté n'est pas réécrit pour refléter une nouvelle décision. Un nouvel
ADR est créé et référence celui qu'il remplace afin de préserver l'historique.

## Index

- [ADR-0001 — Utiliser un monolithe modulaire organisé par fonctionnalité](0001-modular-monolith.md)
- [ADR-0002 — Utiliser Rust, Axum, SQLx et PostgreSQL sans ORM](0002-rust-axum-sqlx-postgresql.md)
- [ADR-0003 — Utiliser des soldes de rapprochement comme points d'ancrage](0003-reconciliation-balance-snapshots.md)
- [ADR-0004 — Composer explicitement les modules Axum](0004-explicit-module-composition.md)
- [ADR-0005 — Modéliser les flux financiers par des transactions signées](0005-model-financial-transactions.md)
- [ADR-0006 — Exposer une pagination par page](0006-page-based-pagination.md)
- [ADR-0007 — Standardiser les erreurs HTTP avec RFC 9457](0007-rfc-9457-problem-details.md)
