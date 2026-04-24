# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build                          # Build the project
cargo run                            # Run the API server
cargo check                          # Type-check without building
diesel migration run                 # Apply pending migrations
diesel migration revert              # Rollback last migration
diesel migration redo                # Revert + reapply last migration
diesel print-schema > src/schema.rs  # Regenerate schema after migration changes
cargo audit                          # Check Cargo.lock against RustSec advisories
cargo deny check                     # Check licenses, duplicates, advisories (deny.toml)
```

**Environment variables** (via `.env`): `API_ADDR`, `API_PORT`, `DATABASE_URL`

**System build dependencies**: `protoc` (e.g. `apt install protobuf-compiler`) — required by `yfinance-rs` at build time.

## Architecture

DDD layered architecture with strict one-way dependency: `infrastructure` → `application` → `domain`.

- **`domain/`** — Pure business entities and enums. No framework imports (no Diesel, no Actix). Uses only `serde` for serialization.
- **`application/`** — Cross-cutting concerns. Currently contains `AppError` (the boundary error type).
- **`infrastructure/http/`** — Actix-web server, routes, controllers, `AppState` (DI container wrapping `Db`), `ApiError` (HTTP error responses).
- **`infrastructure/persistence/`** — Diesel ORM layer: `Db` struct with async bridge (`spawn_blocking`), connection pool, ORM models, error types.

## Key Patterns

**ORM Model Pattern**: Each entity has two structs in `persistence/models/`:
- `XxxRow` (`Queryable`, `Selectable`) — for SELECT queries, includes DB-only fields like `created_at`
- `NewXxxRow<'a>` (`Insertable`) — for INSERT, borrows strings from caller
- `TryFrom<XxxRow> for DomainEntity` — converts to domain type, returns `DbError` on enum parse failure

**Enum Handling**: VARCHAR columns in PostgreSQL + CHECK constraints. Rust enums with manual `as_str()` / `TryFrom<&str>` conversions. No `diesel-derive-enum` — avoids friction with `diesel print-schema` workflow.

**Async DB Bridge**: All Diesel queries run through `Db::exec()` which dispatches closures to `tokio::task::spawn_blocking`.

**Error Chain**: `diesel::Error` / `PoolError` / `JoinError` → `DbError` → `AppError` → `ApiError` → HTTP response. Each layer uses `From`/`Into` conversions.

## Database

PostgreSQL, shared with a legacy PHP app. **`diesel.toml`** uses `filter.only_tables` to scope `print-schema` to only this project's tables. Always maintain this filter when adding new tables.

After any migration change: `diesel migration run && diesel print-schema > src/schema.rs`

## External Data Providers

Market data is fetched through the `PriceFetcher` port (`application/ports/price_fetcher.rs`). The only adapter today is `YFinancePriceFetcher` (`infrastructure/market/`), backed by `yfinance-rs`.

- **`yfinance-rs` is a young single-author crate** (first release 2025-08) that pulls the entire `paft` ecosystem (7 transitive crates) and requires `protoc` at build time. The `PriceFetcher` port exists specifically to contain this risk — application code must never import `yfinance_rs::*` directly, so swapping to another provider (EODHD, Alpha Vantage, ...) stays a one-file change.
- Any new market-data source must be introduced as a new adapter behind the same port.

## Code Conventions

- All code, comments, and identifiers must be in **English**
- Domain entities exclude infrastructure timestamps (`created_at`) except when semantically meaningful (`updated_at` on mutable entities like `MetricCatalog`)
- `SERIAL` (i32) for low-volume PKs, `BIGSERIAL` (i64) for high-volume tables (`metric_values`)
- Migrations use `IF EXISTS` in `down.sql` for safe re-entrant rollback
