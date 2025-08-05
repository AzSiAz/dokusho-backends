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

Copy `.env.development` to `.env` and configure:

```bash
DATABASE_URL=postgres://dokusho:dokusho@localhost/dokusho
FLARESOLVER_URL=http://localhost:8191
JWT_SECRET=your-secret-key
```

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