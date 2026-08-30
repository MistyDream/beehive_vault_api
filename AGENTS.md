# Beehive Vault API agent guidance

## Project scope

- This repository contains the Beehive Vault API built with Rust 2024, Axum, SQLx, and PostgreSQL.
- Organize production code by feature. Keep HTTP extraction and serialization in handlers and DTOs, business orchestration in services, and SQL in repositories.
- Keep product documentation and user-facing text in French. Keep source code, technical identifiers, test data, and code comments in English.

## Domain and implementation rules

- Preserve monetary values with `rust_decimal` and PostgreSQL `NUMERIC`; never introduce binary floating-point arithmetic for financial values.
- Evaluate business dates in the household time zone when the rule depends on the current day.
- Keep operations that validate and mutate related financial state atomic. Use PostgreSQL transactions and locks when concurrent writes could violate an invariant.
- Keep financial domain rules independent from Axum request and response types.
- Validate external input at the API boundary and expose expected failures as RFC 9457 Problem Details without leaking database internals.
- Treat migrations as immutable after they have been shared. Add a new migration instead of rewriting published database history.
- Keep `docs/api.md` aligned with behavior that is currently available. Keep future contracts in `docs/client-contracts.md` until implementation and tests make them effective.
- Preserve unrelated user changes in a dirty worktree.

## Verification

- Run `cargo fmt --all -- --check`.
- Run `cargo check --all-targets`.
- Run `cargo clippy --all-targets -- -D warnings`.
- Run `cargo test --all-targets`.
- Run every affected ignored PostgreSQL integration test against the isolated database configured by `TEST_DATABASE_URL`.
- Add regression coverage for every new financial invariant, migration, route, and Problem Details code.

## Code Review Rules

- Flag financial invariants that can be bypassed by concurrent requests. The safe path is to validate and write under the same PostgreSQL transaction and lock.
- Flag current-day validation based on UTC when the rule belongs to a household. The safe path is to resolve the date in the household time zone.
- Flag monetary conversions to floating-point values or client-dependent rounding. Preserve exact decimal representations end to end.
- Flag divergences between implemented routes, Problem Details, `docs/api.md`, `docs/client-contracts.md`, and the web repository contract documentation.
- Flag migrations that can discard data, orphan references, or choose unstable canonical identities without an explicit migration rule and regression test.
- Do not report formatting or lint preferences already enforced by rustfmt or Clippy.
