# Repository Guidelines

## Project Structure & Module Organization

- Rust workspace with apps and crates.
- API: `apps/dokusho` (Axum REST + OpenAPI), binary `dokusho`.
- Worker: `apps/worker`, binary `worker`.
- Libraries in `crates/`: `core`, `database` (SQLx, migrations under `crates/database/migrations`), `auth`, `clients`, `config`, `jobs`, `sources`.
- Configuration via root `.env` and the `dokusho-config` crate.

## Build, Test, and Development Commands

- Start deps (PostgreSQL, RabbitMQ, Flaresolverr): `make docker-up`; stop: `make docker-down`; logs: `make logs`.
- Run API with hot reload: `make dev`. Run worker: `make dev-worker`.
- Run migrations: `cd crates/database && cargo sqlx migrate run`.
- All tests: `make test` or `cargo test --workspace`.
- Lint/format: `make check`; format only: `make fmt`; clean: `make clean`.
- Offline checks (no DB): `SQLX_OFFLINE=true cargo check --workspace`.

## Coding Style & Naming Conventions

- Rust 2024 edition; use `rustfmt` defaults (4-space indent). Keep imports organized.
- Clippy must pass with `-D warnings` (use `make check`).
- Crate names follow `dokusho-*`; binaries are short and lowercase (`dokusho`, `worker`).
- Visibility: prefer `pub(crate)` over `pub` unless needed across crates.
- Keep small, cohesive modules; put `mod tests` at file end.

## Testing Guidelines

- Use the Rust test harness; async tests with `#[tokio::test]`.
- Place unit tests inline (`mod tests`) or in per-crate `tests/` for integration tests.
- For DB tests, ensure migrations are applied and `DATABASE_URL` is set. For CI or local checks without DB, use `SQLX_OFFLINE=true`.
- Use `wiremock` for HTTP mocking in client code where applicable.

## Commit & Pull Request Guidelines

- Commits: short, imperative, and scoped (e.g., `fix lint`, `add job crate`, `improve sources endpoint`).
- PRs: include a clear description, linked issues, test/run steps, and call out schema changes with required migration commands. Add screenshots/URLs when changing docs or API surface (Swagger at `/docs`).

## Security & Configuration Tips

- Do not commit secrets. Use `.env` locally (e.g., `DATABASE_URL`, `AUTH_*`, `BASE_URL`).
- Configure CORS via env; prefer explicit origins outside development.
- When adding SQLx queries, update offline data: `cargo sqlx prepare` and commit `.sqlx/` changes.
