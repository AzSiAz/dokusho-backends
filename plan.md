# 🦀 Dokusho-Backends Rust Migration Plan

## Overview

This document outlines the plan to migrate Dokusho-Backends from Go to Rust, leveraging Rust's type system, traits, and modern async ecosystem.

## 1. Project Structure

```
dokusho-backend/
├── Cargo.toml (workspace)
├── apps/
│   ├── api/                    # GraphQL API server
│   └── worker/                 # Background worker with PGMQ
├── crates/
│   ├── core/                   # Core types and traits
│   ├── scrapers/               # Source implementations
│   ├── clients/                # HTTP clients (FlareSolver)
│   ├── database/               # Database layer (SQLx + migrations)
│   ├── auth/                   # Authentication (OpenID Connect)
│   └── jobs/                   # Job queue abstractions (PGMQ)
└── docker/
```

## 2. Core Types & Traits

### Branded Types (NewType Pattern)

```rust
// core/src/types/ids.rs
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerieId(String);

// Similar for VolumeId, ChapterId
```

### Core Enums

```rust
// core/src/types/enums.rs
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "jp")]
    Japanese,
    // ...
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SerieStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
}
```

### Domain Models

```rust
// core/src/types/models.rs
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiLanguageString(pub HashMap<Language, String>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Serie {
    pub id: SerieId,
    pub title: MultiLanguageString,
    pub cover: String,
    pub synopsis: MultiLanguageString,
    pub status: Vec<SerieStatus>,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChapterData {
    Image { images: Vec<ChapterImage> },
    Text { texts: Vec<ChapterText> },
}
```

### Core Trait

```rust
// core/src/traits/source_api.rs
use async_trait::async_trait;

#[async_trait]
pub trait SourceApi: Send + Sync {
    fn information(&self) -> &SourceInformation;
    
    async fn fetch_popular_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError>;
    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError>;
    async fn fetch_chapter_data(
        &self, 
        serie_id: &SerieId, 
        volume_id: &VolumeId, 
        chapter_id: &ChapterId
    ) -> Result<ChapterData, SourceError>;
}
```

## 3. Error Handling Strategy

```rust
// core/src/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Cloudflare protection detected")]
    CloudflareProtection,

    #[error("FlareSolver error: {0}")]
    FlareSolver(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Conversion from external errors
impl From<reqwest::Error> for SourceError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SourceError::Network("Request timed out".to_string())
        } else if err.is_connect() {
            SourceError::Network("Connection failed".to_string())
        } else {
            SourceError::Network(err.to_string())
        }
    }
}
```

## 4. GraphQL API Server

```rust
// apps/api/src/main.rs
use axum::{Router, routing::{get, post}};
use async_graphql::{Schema, EmptySubscription};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_env()?;
    
    // Initialize services
    let pool = PgPool::connect(&config.database_url).await?;
    let sources = build_sources(&config);
    let openid_client = OpenIDClient::new(&config.auth).await?;
    
    // Build GraphQL schema
    let schema = Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(sources)
        .data(pool)
        .data(openid_client)
        .finish();
    
    // Build router
    let app = Router::new()
        .route("/graphql", get(graphql_playground).post(graphql_handler))
        .route("/auth/callback", get(oauth_callback)) // Only REST endpoint
        .route("/health", get(health_check))
        .with_state(Arc::new(AppState { schema, config }));
    
    // axum 0.8 server start
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### GraphQL Schema

```rust
// apps/api/src/graphql/query.rs
#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn sources(&self, ctx: &Context<'_>) -> Result<Vec<SourceInformation>> {
        let sources = ctx.data::<Vec<Box<dyn SourceApi>>>()?;
        Ok(sources.iter().map(|s| s.information().clone()).collect())
    }
    
    async fn serie(
        &self,
        ctx: &Context<'_>,
        source_id: ID,
        serie_id: ID,
    ) -> Result<Option<Serie>> {
        let sources = ctx.data::<Vec<Box<dyn SourceApi>>>()?;
        let source = sources.iter()
            .find(|s| s.information().id == SourceId::from(source_id.as_str()))
            .ok_or_else(|| async_graphql::Error::new("Source not found"))?;
        
        match source.fetch_serie_detail(&SerieId::from(serie_id.as_str())).await {
            Ok(serie) => Ok(Some(serie)),
            Err(SourceError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
```

## 5. Authentication Flow

### OpenID Connect via GraphQL + Single REST Endpoint

```graphql
# Step 1: Initiate auth
mutation InitiateAuth {
  initiateAuthentication(redirectUri: "http://localhost:3000/auth/complete") {
    authorizationUrl
    state
  }
}

# Step 2: User authenticates with provider
# Step 3: Provider redirects to /auth/callback
# Step 4: Callback generates JWT and redirects to client
# Step 5: Client uses JWT for authenticated requests
```

```rust
// apps/api/src/auth/callback.rs
async fn oauth_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Redirect, AuthError> {
    // Validate state
    let auth_state = state.db.get_auth_state(&params.state).await?
        .ok_or(AuthError::InvalidState)?;
    
    // Exchange code for token
    let token_response = state.openid_client
        .exchange_code(params.code)
        .await?;
    
    // Get user info and create/update user
    let user = create_or_update_user(&state.db, &token_response).await?;
    
    // Generate JWT
    let jwt = generate_jwt(&user, &state.config.jwt_secret)?;
    
    // Redirect with token
    Ok(Redirect::to(&format!("{}#token={}", auth_state.redirect_uri, jwt)))
}
```

## 6. Database Migrations

### Embedded Migrations with SQLx

```rust
// crates/database/src/migrations.rs
use sqlx::migrate::Migrator;

// This embeds all migrations from the migrations/ directory at compile time
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

// crates/database/src/lib.rs
use sqlx::{PgPool, postgres::PgPoolOptions};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        Ok(Self { pool })
    }
    
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        MIGRATOR.run(&self.pool).await?;
        tracing::info!("Database migrations completed");
        Ok(())
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
```

### Migration Files Structure

```
crates/database/migrations/
├── 001_initial_schema.sql
├── 002_add_pgmq.sql
└── 003_add_indexes.sql
```

```sql
-- crates/database/migrations/001_initial_schema.sql
-- Auth state for OAuth flow
CREATE TABLE IF NOT EXISTS auth_states (
    state VARCHAR(255) PRIMARY KEY,
    redirect_uri TEXT NOT NULL,
    nonce VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ DEFAULT NOW() + INTERVAL '10 minutes'
);

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sub VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255),
    name VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Popular series cache
CREATE TABLE IF NOT EXISTS popular_series_cache (
    source_id VARCHAR(255) PRIMARY KEY,
    data JSONB NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Workflows for job system
CREATE TABLE IF NOT EXISTS workflows (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    definition JSONB NOT NULL,
    state JSONB NOT NULL,
    status VARCHAR(50) NOT NULL,
    current_step INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

```sql
-- crates/database/migrations/002_add_pgmq.sql
-- Install PGMQ extension
CREATE EXTENSION IF NOT EXISTS pgmq CASCADE;

-- Create job queue
SELECT pgmq.create('jobs');
```

```sql
-- crates/database/migrations/003_add_indexes.sql
-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_auth_states_expires ON auth_states(expires_at);
CREATE INDEX IF NOT EXISTS idx_users_sub ON users(sub);
CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
CREATE INDEX IF NOT EXISTS idx_popular_series_cache_updated ON popular_series_cache(updated_at);
```

### Usage in Applications

```rust
// apps/api/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    
    // Create database and run migrations
    let database = Database::new(&config.database_url).await?;
    database.migrate().await?;
    
    // Rest of initialization...
}

// apps/worker/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    
    // Create database and run migrations
    let database = Database::new(&config.database_url).await?;
    database.migrate().await?;
    
    // Worker will ensure PGMQ tables exist
    let pool = database.pool().clone();
    // ...
}
```

### Development Commands

```bash
# Create a new migration
sqlx migrate add add_user_preferences

# Run migrations (for development)
sqlx migrate run --database-url $DATABASE_URL

# Revert last migration
sqlx migrate revert --database-url $DATABASE_URL

# Check migration status
sqlx migrate info --database-url $DATABASE_URL
```

### Build Requirements

```toml
# crates/database/Cargo.toml
[package]
name = "database"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlx = { workspace = true }
tracing = { workspace = true }

[build-dependencies]
sqlx = { workspace = true, features = ["migrate"] }
```

### Important Notes

1. **Compile-time verification**: SQLx validates migrations at compile time
2. **Automatic execution**: Migrations run on startup, ensuring schema is always up-to-date
3. **Idempotent**: Use `IF NOT EXISTS` to make migrations safe to run multiple times
4. **Version control**: Migration files are tracked in git
5. **No external tools**: Everything is embedded in the binary

## 7. Background Worker with PGMQ

Note: PGMQ relies on a PostgreSQL extension. Only enable the migration and the `pgmq` dependency when your environment provides the extension (e.g., Tembo stack or a PG instance with pgmq installed). Otherwise, keep the migration commented and guard queue code behind a feature flag.

### Job Types

```rust
// crates/jobs/src/types.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum JobType {
    UpdatePopularSeries { source_id: Option<SourceId> },
    CheckNewChapters { serie_id: SerieId, source_id: SourceId },
    ProcessWorkflow { workflow_id: Uuid, step: WorkflowStep },
    CleanupOldData { older_than: DateTime<Utc> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: JobType,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: serde_json::Value,
}
```

### PGMQ Integration

```rust
// crates/jobs/src/queue.rs
use pgmq::{PGMQueue, PGMQueueExt, Message};

// Option 1: Standalone queue
pub struct JobQueue {
    pgmq: PGMQueue,
    queue_name: String,
}

impl JobQueue {
    pub async fn new(database_url: &str, queue_name: &str) -> Result<Self, JobError> {
        let pgmq = PGMQueue::new(database_url.to_owned()).await?;
        pgmq.create(queue_name).await?;
        Ok(Self { pgmq, queue_name: queue_name.to_string() })
    }
}

// Option 2: With existing pool (supports transactions)
pub struct JobQueueExt {
    pgmq: PGMQueueExt,
    queue_name: String,
}

impl JobQueueExt {
    pub async fn new(pool: PgPool, queue_name: &str) -> Result<Self, JobError> {
        let pgmq = PGMQueueExt::new_with_pool(pool).await;
        pgmq.init().await?;
        pgmq.create(queue_name).await?;
        Ok(Self { pgmq, queue_name: queue_name.to_string() })
    }
    
    // Transaction support
    pub async fn send_in_transaction<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
        job: &Job,
    ) -> Result<i64, JobError> {
        let message_id = sqlx::query_scalar!(
            "SELECT pgmq.send($1, $2::jsonb)",
            &self.queue_name,
            serde_json::to_value(job)?
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(message_id)
    }
}
```

### Worker Implementation

```rust
// apps/worker/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;
    let pool = PgPool::connect(&config.database_url).await?;
    
    // Install PGMQ extension
    sqlx::query("CREATE EXTENSION IF NOT EXISTS pgmq CASCADE")
        .execute(&pool)
        .await?;
    
    // Create queue and scheduler
    let queue = JobQueueExt::new(pool.clone(), "jobs").await?;
    let scheduler = JobScheduler::new(queue.clone());
    
    // Worker loop
    const VISIBILITY_TIMEOUT: i32 = 300;
    const BATCH_SIZE: i32 = 10;
    
    loop {
        match queue.read_batch(VISIBILITY_TIMEOUT, BATCH_SIZE).await {
            Ok(messages) => {
                for msg in messages {
                    process_job(msg, &handlers, &ctx).await;
                }
            }
            Err(JobError::Queue(PgmqError::DatabaseError(sqlx::Error::PoolTimedOut))) => {
                tracing::error!("Database pool timed out, exiting");
                break;
            }
            Err(e) => {
                tracing::error!("Failed to read jobs: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
        
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    
    Ok(())
}
```

## 8. Scraper Implementations

### MangaDex (API-based)

```rust
// scrapers/src/mangadex/mod.rs
pub struct MangaDex {
    client: Client,
    source_info: SourceInformation,
}

#[async_trait]
impl SourceApi for MangaDex {
    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let response = self.client
            .get(format!("{}/manga/{}", self.api_url, serie_id))
            .send()
            .await
            .map_err(|e| SourceError::Network(e.to_string()))?;
        
        if response.status() == 404 {
            return Err(SourceError::NotFound(format!("Serie {} not found", serie_id)));
        }
        
        let manga: MangaDexResponse = response.json().await
            .map_err(|e| SourceError::Parse(e.to_string()))?;
        
        Ok(manga.into())
    }
}
```

### WeebCentral (HTML Scraping)

```rust
// scrapers/src/weebcentral/mod.rs
pub struct WeebCentral {
    client: FlareSolverClient,
    source_info: SourceInformation,
}

#[async_trait]
impl SourceApi for WeebCentral {
    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let html = self.client.get_html(&format!("{}/serie/{}", self.url, serie_id))
            .await
            .map_err(|e| SourceError::FlareSolver(e.to_string()))?;
        
        self.parse_serie_detail(&html)
    }
}
```

## 9. Testing Strategy

```rust
// Example unit test with fixtures
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    #[tokio::test]
    async fn test_mangadex_serie_detail() {
        let mock_server = MockServer::start().await;
        let fixture = include_str!("fixtures/serie_detail.json");
        
        Mock::given(method("GET"))
            .and(path("/manga/test-id"))
            .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
            .mount(&mock_server)
            .await;
        
        let mangadex = MangaDex::new_with_url(&mock_server.uri());
        let result = mangadex.fetch_serie_detail(&SerieId::from("test-id")).await;
        
        assert!(result.is_ok());
    }
}
```

## 10. Configuration Management

```rust
// apps/api/src/config.rs
use std::env;
use serde::Deserialize;
use config::{Config, ConfigError, Environment, File};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub sources: SourcesConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub base_url: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfig {
    pub enabled: bool,
    pub issuer_url: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub redirect_url: Option<String>,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SourcesConfig {
    pub flaresolver_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config").required(false))
            .add_source(Environment::default().separator("__"))
            // sensible defaults
            .set_default("server.port", 8080)?
            .set_default("auth.enabled", false)?
            .set_default("auth.jwt_expiry_hours", 24)?
            .set_default("logging.level", "info")?
            .build()?;

        let mut config: AppConfig = s.try_deserialize()?;

        // Allow JWT secret from env when empty
        if config.auth.jwt_secret.is_empty() {
            config.auth.jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        }

        Ok(config)
    }
}
```

## 11. Key Dependencies

```toml
[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
async-trait = "0.1"

# Web framework
axum = { version = "0.8", features = ["macros"] }
async-graphql = { version = "7.0", features = ["chrono", "uuid"] }
async-graphql-axum = "7.0"

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate", "json", "macros"
] }
# pgmq = { version = "0.28" } # optional; enable only if the extension is available

# Authentication
openidconnect = "3.0"
jsonwebtoken = "9.0"

# HTTP client
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
derive_more = "0.99"
strum = { version = "0.25", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
config = "0.13"
```

Note: PGMQ is optional. The SQL migration to install it and queue creation should be gated behind an environment check or feature flag, and only enabled when your PostgreSQL instance provides the pgmq extension.

## 12. Migration Phases

### Phase 1: Core Types & Traits (Week 1)
- Set up Rust workspace structure
- Implement core types with NewType pattern
- Define SourceApi trait
- Add comprehensive tests

### Phase 2: HTTP Clients (Week 2)
- Implement FlareSolver client
- Add retry logic and error handling
- Integration tests with mock servers

### Phase 3: Scrapers (Weeks 3-4)
- Implement MangaDex (API)
- Implement WeebCentral (HTML)
- Port test fixtures from Go

### Phase 4: GraphQL API (Week 5)
- Implement GraphQL schema
- Add authentication flow
- Integration tests

### Phase 5: Database & Auth (Week 6)
- Set up SQLx with migrations
- Implement OpenID Connect
- JWT validation

### Phase 6: Background Worker (Week 7)
- Install PGMQ extension
- Implement job queue with retry logic
- Build scheduler and handlers
- Add monitoring endpoints

### Phase 7: Deployment (Week 8)
- Docker configuration
- CI/CD pipelines
- Documentation
- Performance benchmarking

## 13. PGMQ Best Practices

### Configuration
```rust
const DEFAULT_VISIBILITY_TIMEOUT: i32 = 300;  // 5 minutes
const DEFAULT_BATCH_SIZE: i32 = 10;
const DEFAULT_POLL_INTERVAL_MS: i32 = 250;
const MAX_RETRIES: u32 = 3;
```

### Error Handling
- Implement exponential backoff
- Use dead letter queue for failed jobs
- Handle PoolTimedOut errors gracefully
- Log all failures with context

### Performance
- Batch read operations
- Use appropriate poll intervals
- Consider NOTIFY/LISTEN for real-time updates
- Monitor queue depth and processing rates

## 14. Key Improvements Over Go Version

1. **Type Safety**: NewType pattern prevents ID mixing
2. **Error Handling**: Result<T, E> with proper error chaining
3. **Performance**: Zero-copy deserialization, efficient pooling
4. **Memory Safety**: No null pointers or data races
5. **Developer Experience**: Better IDE support, comprehensive docs

## 15. Development Setup for Maximum Velocity

### Project Structure for Tool Optimization

```bash
# Initialize workspace
cargo new dokusho-backend --name dokusho
cd dokusho-backend

# Create workspace structure
mkdir -p apps/{api,worker}/src
mkdir -p crates/{core,scrapers,clients,database,auth,jobs}/src
mkdir -p migrations docker

# Create Cargo.toml files
cat > Cargo.toml << 'EOF'
[workspace]
members = ["apps/*", "crates/*"]
resolver = "2"

[workspace.dependencies]
# Add dependencies from section 11
EOF

# Initialize crates
for crate in core scrapers clients database auth jobs; do
    cat > crates/$crate/Cargo.toml << EOF
[package]
name = "$crate"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF
done

# Initialize apps
for app in api worker; do
    cat > apps/$app/Cargo.toml << EOF
[package]
name = "$app"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "$app"
path = "src/main.rs"

[dependencies]
EOF
done
```

### Development Workflow

1. **Use Glob/Grep for Navigation**
   ```bash
   # Find all source files
   glob "**/*.rs"
   
   # Find specific implementations
   grep "impl SourceApi" --include "*.rs"
   grep "async fn fetch" --include "*.rs"
   ```

2. **Batch File Operations**
   ```bash
   # Read multiple related files at once
   read crates/core/src/lib.rs crates/core/src/types/mod.rs crates/core/src/traits/mod.rs
   ```

3. **Incremental Development**
   - Start with core types, test them
   - Add one scraper at a time
   - Use mock implementations for testing

### Recommended File Organization

```
crates/core/src/
├── lib.rs          # Re-exports
├── types/
│   ├── mod.rs      # Type definitions
│   ├── ids.rs      # NewType IDs
│   └── enums.rs    # Enums
├── traits/
│   ├── mod.rs      # Trait definitions
│   └── source_api.rs
└── errors.rs       # Error types

apps/api/src/
├── main.rs         # Entry point
├── config.rs       # Configuration
├── graphql/
│   ├── mod.rs      # Schema builder
│   ├── query.rs    # Query resolvers
│   └── mutation.rs # Mutation resolvers
└── auth/
    └── callback.rs # OAuth callback
```

### Testing Strategy for Fast Iteration

```bash
# Run tests in watch mode
cargo watch -x "test --package core"

# Test specific module
cargo test --package scrapers mangadex::

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Docker Compose for Dependencies

```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_DB: dokusho
      POSTGRES_USER: dokusho
      POSTGRES_PASSWORD: dokusho
    ports:
      - "5432:5432"
    command: >
      postgres
      -c shared_preload_libraries=pg_stat_statements
      -c pg_stat_statements.track=all

  flaresolver:
    image: flaresolverr/flaresolverr:latest
    ports:
      - "8191:8191"
```

### Environment Setup

```bash
# .env.development
DATABASE_URL=postgres://dokusho:dokusho@localhost/dokusho
FLARESOLVER_URL=http://localhost:8191
JWT_SECRET=development-secret-change-in-production
BASE_URL=http://localhost:8080
RUST_LOG=debug,sqlx=warn

# Auth config
AUTH__ISSUER_URL=https://accounts.google.com
AUTH__CLIENT_ID=your-client-id
AUTH__CLIENT_SECRET=your-client-secret
AUTH__REDIRECT_URL=http://localhost:8080/auth/callback
```

### VS Code Configuration

```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.procMacro.enable": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.diagnostics.experimental.enable": true
}
```

### Makefile for Common Tasks

```makefile
# Makefile
.PHONY: dev test migrate setup

setup:
	docker-compose -f docker-compose.dev.yml up -d
	cargo install sqlx-cli cargo-watch
	sqlx database create
	sqlx migrate run

dev:
	cargo watch -x "run --bin api"

test:
	cargo test --workspace

migrate:
	sqlx migrate run

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings
```

## 16. Available Development Tools

I can use these CLIs directly through the bash tool:

### PostgreSQL (`psql`)
```bash
# Connect to database
psql $DATABASE_URL -c "SELECT * FROM pgmq.list_queues()"

# Run migrations
psql $DATABASE_URL -f migrations/001_initial.sql

# Check PGMQ messages
psql $DATABASE_URL -c "SELECT * FROM pgmq.read('jobs', 30, 5)"

# Debug queries
psql $DATABASE_URL -x -c "SELECT * FROM users WHERE email = 'test@example.com'"
```

### Docker CLI
```bash
# Start services
docker-compose -f docker-compose.dev.yml up -d

# Check logs
docker-compose logs -f api
docker logs dokusho-backend-worker-1 --tail 50

# Execute commands in containers
docker exec -it dokusho-backend-postgres-1 psql -U dokusho

# Restart services
docker-compose restart api worker
```

### Git CLI
```bash
# Create feature branch
git checkout -b feat/add-mangadex-scraper

# Stage and commit with detailed messages
git add crates/scrapers/src/mangadex/
git commit -m "feat: add MangaDex scraper implementation

- Implement SourceApi trait for MangaDex
- Add response type mappings
- Include test fixtures"

# Check status and diffs
git status
git diff --staged
git log --oneline -10
```

### Cargo & Rust Tools
```bash
# Run with specific features
cargo run --bin api --features "debug-logging"

# Test with output
RUST_LOG=debug cargo test mangadex -- --nocapture

# Check for issues
cargo clippy --all-targets --all-features
cargo fmt --all --check

# Add dependencies
cargo add -p scrapers scraper --features "derive"
```

With these tools, I can provide a complete development workflow:
- Create and modify code files
- Run and debug tests
- Manage database schema and data
- Control Docker services
- Handle version control
- Debug issues in real-time

## 17. Future Enhancements

- WebSocket support for real-time updates
- Redis caching layer
- Prometheus metrics
- OpenTelemetry tracing
- gRPC for internal services
- Advanced PGMQ features (priorities, dependencies, cancellation)
