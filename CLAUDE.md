# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build & Run
```bash
# Start dependencies (PostgreSQL, Flaresolverr)
make docker-up

# Run database migrations
sea-orm-cli migrate up

# Generate entity code from database schema
make gen  # or: sea-orm-cli generate entity -o crates/database/src/entities --with-serde both --with-copy-enums --enum-extra-derives async_graphql::Enum --model-extra-derives async_graphql::SimpleObject

# Run API server with hot reload
make dev  # or: cargo watch -x "run --bin api"

# Run worker with hot reload  
make dev-worker  # or: cargo watch -x "run --bin worker"

# Run adminboard app (on port 8081)
cargo run -p dokusho-adminboard
```

### Testing
```bash
# Run all workspace tests
cargo test --workspace

# Run tests for specific crate
cargo test -p dokusho-core
cargo test -p sources

# Run specific test with debug output
RUST_LOG=debug cargo test test_name -- --nocapture

# Test with offline sqlx checks (when DB unavailable)
SQLX_OFFLINE=true cargo check --workspace
```

### Code Quality
```bash
# Format code
cargo fmt --all

# Run clippy linter  
cargo clippy --all-targets --all-features -- -D warnings

# Type check
cargo check --workspace

# Run both format and clippy checks
make check
```

## Architecture

### Workspace Structure
```
dokusho/
├── apps/
│   ├── api/         # GraphQL API server (Axum + async-graphql)
│   ├── worker/      # Background job processor (PGMQ-based)
│   └── adminboard/  # Admin UI server (proxies to API, OAuth flow)
├── crates/
│   ├── core/        # Core types, traits, and domain models
│   ├── database/    # SQLx database layer with repositories
│   ├── sources/     # Source scrapers (MangaDex API, WeebCentral HTML)
│   ├── auth/        # JWT & OpenID Connect authentication
│   ├── clients/     # HTTP clients with retry logic
│   ├── config/      # Shared configuration management
│   └── jobs/        # Job queue abstractions
└── migrations/      # SQL database migrations in crates/database/migrations/
```

### Key Architectural Patterns

1. **NewType Pattern**: Core types use strong typing (e.g., `SerieId(Uuid)`, `UserId(Uuid)`)
2. **Repository Pattern**: Database access through typed repositories (`UserRepository`, `SerieRepository`)
3. **Trait-based Sources**: All sources implement `SourceApi` trait for pluggable scrapers
4. **GraphQL Guards**: Authentication/authorization via async-graphql guards (`AuthGuard`, `AdminGuard`)
5. **JWT + OAuth**: OpenID Connect for login, JWT tokens for API access

### Database Layer
- Uses SQLX with compile-time query checking
- Migrations in `crates/database/migrations/`
- Repository pattern for all DB operations
- Connection pooling with configurable min/max connections

### SQL Query Optimization Patterns

#### 1. JSON Aggregation for One-to-Many Relations
- **Use `jsonb_agg` with `jsonb_build_object`** to aggregate related records into JSON arrays directly in PostgreSQL
- **Avoid N+1 queries** by fetching all related data in a single query instead of multiple queries
- **Pattern for aggregating related data:**
```sql
COALESCE(
  (SELECT jsonb_agg(
    jsonb_build_object(
      'id', t.id,
      'field1', t.field1,
      'field2', t.field2
    ) ORDER BY t.sort_field
  )
  FROM related_table t
  WHERE t.foreign_key = main_table.id),
  '[]'::jsonb
) as "field_name!: sqlx::types::Json<Vec<RelatedType>>"
```

#### 2. Type-Safe Database Row Structs
- **Create intermediate `Row` structs** in `models/` for complex queries (e.g., `SerieWithTitlesRow`, `SerieWithRelationsRow`)
- **Implement required traits:**
  - `#[derive(sqlx::FromRow)]` for automatic deserialization from database
  - `impl From<RowStruct> for DomainModel` for clean conversion
- **Benefits:** Separation of database concerns from domain models, reusable mappings, cleaner repository code
- **Example usage in repository:**
```rust
let rows = sqlx::query_as!(RowStruct, "SELECT ...").fetch_all(&pool).await?;
Ok(rows.into_iter().map(Into::into).collect())
```

#### 3. Efficient ID Handling  
- **Use newtype wrappers** for all ID fields (e.g., `SerieId(Uuid)`, `GenreId(Uuid)`)
- **Implement `Deref` trait** for seamless SQLx integration
- **Don't manually insert UUID fields** that have `DEFAULT gen_random_uuid()` in migrations - let PostgreSQL generate them

#### 4. ON CONFLICT Optimization
- **Use `DO NOTHING`** when only checking existence, not `DO UPDATE SET field = EXCLUDED.field` 
- **Use CTEs with COALESCE** to ensure IDs are always returned from upserts:
```sql
WITH ins AS (
  INSERT INTO table (field) VALUES ($1)
  ON CONFLICT (field) DO NOTHING
  RETURNING id
)
SELECT COALESCE(
  (SELECT id FROM ins),
  (SELECT id FROM table WHERE field = $1)
)
```

#### 5. Query Strategies
- **Single aggregated query > Multiple queries:** Use JSON aggregation to fetch all relations in one query
- **Use JOINs efficiently:** For fetching related data that will always be needed
- **Use subqueries with EXISTS/IN:** When filtering by related data existence
- **Avoid `tokio::try_join!` when possible:** Prefer single comprehensive queries over parallel execution

#### 6. Best Practices
- **Return domain types** from repositories, not raw database rows
- **Keep row structs in models**, not scattered in repository methods  
- **Use `Into::into()` for conversions** instead of manual field mapping
- **Order matters in aggregations:** Use `ORDER BY` in `jsonb_agg` for consistent results
- **Always use COALESCE** with aggregations to return empty arrays instead of NULL

### Authentication Flow
1. User initiates OAuth via `/auth/login` endpoint
2. Redirects to OpenID provider (configured via `AUTH_ISSUER_URL`)
3. Callback to `AUTH_OAUTH_CALLBACK_URL` (e.g., `/auth/callback`)
4. JWT issued with configurable expiry (`AUTH_JWT_EXPIRY_HOURS`)
5. JWT stored client-side for GraphQL requests

### GraphQL API
- Playground available at `/graphql` (dev mode)
- Modular schema organization in `apps/api/src/graphql/schema/`
- Guards for auth (`#[graphql(guard = "AuthGuard")]`) and admin (`#[graphql(guard = "AdminGuard")]`)
- Context injection for database, auth service, and source registry

### Environment Configuration
Critical environment variables:
- `DATABASE_URL`: PostgreSQL connection string
- `SQLX_OFFLINE`: Set to `true` for offline compilation checks
- `AUTH_JWT_SECRET`: JWT signing secret
- `AUTH_ISSUER_URL`, `AUTH_CLIENT_ID`, `AUTH_CLIENT_SECRET`: OAuth config
- `SOURCES_ENABLED_LANGUAGES`: Comma-separated language codes (e.g., `EN,FR`)
- `SOURCES_FLARESOLVERR_URL`: Optional Cloudflare bypass service

### Testing Approach
- Unit tests alongside source files
- Integration tests for source scrapers
- Use `RUST_LOG=debug` for verbose test output
- Mock sources available via `SOURCES_ENABLE_MOCK=true`