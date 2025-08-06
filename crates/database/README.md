# Database Crate

This crate provides database access for the Dokusho backends using SQLx with compile-time checked queries.

## Features

- **Compile-time SQL verification**: All SQL queries are checked at compile time against the database schema
- **Type-safe queries**: Query results are automatically mapped to Rust types
- **Migration management**: Database schema is managed through SQLx migrations
- **Offline mode support**: Can build without database connection using prepared query metadata

## Setup

### Prerequisites

- Docker and Docker Compose installed
- Rust with cargo-sqlx CLI tool: `cargo install sqlx-cli --no-default-features --features postgres`

### Initial Setup

1. Start the PostgreSQL database:
```bash
docker compose up -d postgres
```

2. Run migrations:
```bash
cd crates/database
cargo sqlx migrate run
```

3. Prepare offline mode data (for CI/CD):
```bash
cargo sqlx prepare
```

Or use the convenience script:
```bash
./scripts/prepare-db.sh
```

## Development Workflow

### Adding New Queries

When adding new compile-time checked queries:

1. Write your query using `sqlx::query!` or `sqlx::query_as!` macros
2. Ensure the database is running with the latest schema
3. Run `cargo sqlx prepare` to update offline mode data
4. Commit the changes to `.sqlx/` directory

### Creating Migrations

```bash
cd crates/database
cargo sqlx migrate add <migration_name>
```

This creates a new migration file in `migrations/` directory.

### Running Migrations

```bash
cargo sqlx migrate run
```

### Reverting Migrations

```bash
cargo sqlx migrate revert
```

## Query Macros

### Basic Query
```rust
let result = sqlx::query!(
    "DELETE FROM users WHERE id = $1",
    user_id
)
.execute(&pool)
.await?;
```

### Query with Type Mapping
```rust
let user = sqlx::query_as!(
    User,
    "SELECT * FROM users WHERE id = $1",
    user_id
)
.fetch_one(&pool)
.await?;
```

## Building in CI/CD

The crate can be built without database connection using the prepared query metadata:

```bash
SQLX_OFFLINE=true cargo build
```

The `.sqlx/` directory contains query metadata and must be committed to version control.

## Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required for development)
- `SQLX_OFFLINE`: Set to `true` to build without database connection (uses `.sqlx/` metadata)