# Environnement de développement

## Prérequis

- Rust stable avec `rustfmt` et Clippy ;
- PostgreSQL 18 ;
- les outils clients `psql`, `createdb` et `pg_isready`.

PostgreSQL 18.4 est utilisé pour le développement local. Une version mineure plus récente de PostgreSQL 18 peut être adoptée sans changer le schéma du projet.

## Préparer PostgreSQL

Démarrer le serveur PostgreSQL local, puis exécuter :

```bash
./scripts/setup-postgres.sh
```

Le script est réentrant. Il crée, si nécessaire :

- le rôle non privilégié `beehive_vault` ;
- la base de développement `beehive_vault` ;
- la base d'intégration `beehive_vault_test` ;
- une configuration UTC pour les deux bases.

Par défaut, le script se connecte à `127.0.0.1:5432` avec l'utilisateur du système. Ces valeurs peuvent être adaptées avec `BEEHIVE_DB_HOST`, `BEEHIVE_DB_PORT`, `BEEHIVE_DB_ADMIN_DATABASE` et `BEEHIVE_DB_ADMIN_USER`.

Le cluster Homebrew local accepte actuellement les connexions de localhost sans mot de passe. Cette configuration est réservée au développement. Un déploiement distant doit utiliser un secret distinct et une authentification PostgreSQL.

## Configurer l'API

Créer le fichier local ignoré par Git :

```bash
cp .env.example .env
```

La valeur de développement attendue est :

```dotenv
DATABASE_URL=postgres://beehive_vault@127.0.0.1:5432/beehive_vault
```

## Lancer l'application

```bash
cargo run
```

L'application applique les migrations SQLx en attente avant d'ouvrir le serveur HTTP. Les routes techniques sont ensuite disponibles :

- `GET http://127.0.0.1:8080/healthz` vérifie le processus ;
- `GET http://127.0.0.1:8080/readyz` vérifie PostgreSQL.

## Administrer le catalogue d'établissements

L'API HTTP expose le catalogue global en lecture seule. Le binaire `beehive_vault_admin` utilise la même variable `DATABASE_URL` pour les opérations de maintenance :

```bash
cargo run --bin beehive_vault_admin -- institutions list
cargo run --bin beehive_vault_admin -- institutions add --name "Établissement"
cargo run --bin beehive_vault_admin -- institutions rename \
  --id 019c0000-0000-7000-8000-000000000000 \
  --name "Nouveau nom"
cargo run --bin beehive_vault_admin -- institutions import \
  --file catalog/institutions.json
```

Le fichier fourni contient un catalogue initial adapté à une installation française. L'import retire les espaces superflus, refuse les noms invalides ou dupliqués dans le fichier, ajoute les entrées absentes et laisse les entrées existantes inchangées. Il peut donc être rejoué sans produire de doublons.

Une absence dans le fichier ne supprime jamais une entrée de la base et un changement de nom dans le fichier ne la renomme pas. Utiliser la commande `rename` avec l'UUID existant pour conserver les références des comptes. Aucun import n'est exécuté automatiquement au démarrage de l'API.

## Vérifications locales

```bash
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo test --test household_listing -- --ignored
cargo test --test financial_foundation -- --ignored
cargo test --test category_management -- --ignored
cargo test --test transaction_management -- --ignored
cargo test --test transfer_management -- --ignored
cargo test --test calculated_balance -- --ignored
cargo test --test monthly_flows -- --ignored
cargo test --test institution_catalog -- --ignored
```

Les tests d'intégration utilisent exclusivement `beehive_vault_test` afin de ne jamais modifier les données de développement.
