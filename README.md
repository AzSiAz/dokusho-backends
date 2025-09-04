# Dokusho Backends (Rust)

A high-performance manga/comic aggregation backend written in Rust, providing a unified GraphQL API for multiple sources.

## Architecture

- **Core Types**: Type-safe domain models with NewType pattern
- **GraphQL API**: Async GraphQL server with authentication
- **Background Worker**: PGMQ-based job processing
- **Sources**: Pluggable scrapers (MangaDex API, WeebCentral HTML)

## Quick Start

### Prerequisites

- Rust 1.75+
- Docker & Docker Compose
- PostgreSQL 16+

### Development Setup

```bash
# Start dependencies
make docker-up

# Install tools
cargo install sqlx-cli cargo-watch

# Run migrations
make migrate

# Start API server
make dev

# Run tests
make test
```

### Environment Variables

Copy `.env` and adjust values as needed. Key settings:

- `DATABASE_URL`: PostgreSQL connection string.
- `SERVER_HOST` / `SERVER_PORT`: API bind host/port.
- `SOURCES_ENABLED_LANGUAGES`: Comma-separated list, e.g. `EN,FR`.
- `SOURCES_FLARESOLVERR_URL`: Optional, URL to Flaresolverr.
- `AUTH_ISSUER_URL`, `AUTH_PUBLIC_CLIENT_ID`, `AUTH_CLIENT_ID`, `AUTH_CLIENT_SECRET`: OpenID Connect settings.
- `BASE_URL`: Public base URL for this API (e.g. `http://localhost:8080`).
- `AUTH_TOKEN_CACHE_TTL_SECS` (optional, default `300`): Cache duration for validated access tokens.
- `AUTH_JWKS_CACHE_TTL_SECS` (optional, default `600`): Cache duration for JWKS keyset before refresh.

See the root `.env` file for a complete example.

## Project Structure

```
dokusho-backends/
├── apps/
│   ├── api/        # GraphQL API server
│   └── worker/     # Background job processor
├── crates/
│   ├── core/       # Core types and traits
│   ├── scrapers/   # Source implementations
│   ├── clients/    # HTTP clients
│   ├── database/   # Database layer
│   ├── auth/       # Authentication
│   └── jobs/       # Job queue abstractions
└── migrations/     # SQL migrations
```

## Development

### Running Tests

```bash
# All tests
cargo test

# Specific package
cargo test -p dokusho-core

# With logging
RUST_LOG=debug cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets --all-features

# Type check
cargo check
```

## License

MIT

## Adminboard

- Library: `crates/adminboard` — reusable Axum router serving the admin UI.
- Mounted in API at `/` (home). The REST API remains under `/api/v1` and Swagger UI under `/swagger-ui`.

Configuration (via API config/env):

- `AUTH_ISSUER_URL`, `AUTH_PUBLIC_CLIENT_ID` — OIDC settings used by the embedded Adminboard.
- `BASE_URL` — used to compute `redirect_url` for `/auth/callback`.

Notes:

- The Adminboard performs OIDC Authorization Code + PKCE server-side and sets an HttpOnly `access_token` cookie. The UI calls the API under `/api/v1` directly (same origin), and the API validates tokens per request.
