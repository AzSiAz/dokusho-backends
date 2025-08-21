# Database Optimization and SQLX Configuration

## Current Issues

1. SQLX requires database connection at compile time for query verification
2. Hard-coded connection pool settings may not be optimal for all environments
3. Missing connection health checks and retry logic
4. No query performance monitoring

## SQLX Offline Mode Setup

### Problem
The project fails to build without a running PostgreSQL instance because SQLX macros need to verify queries at compile time.

### Solution

#### 1. Generate Offline Query Data
```bash
# With database running
export DATABASE_URL="postgresql://user:pass@localhost/dokusho"
cargo sqlx prepare --workspace

# This generates .sqlx/*.json files with query metadata
```

#### 2. Update .gitignore
```gitignore
# Remove .sqlx from gitignore to commit query metadata
# .sqlx/
```

#### 3. Add Build Documentation
Create `.env.example`:
```env
# For development with database
DATABASE_URL=postgresql://dokusho:dokusho@localhost/dokusho

# For building without database
SQLX_OFFLINE=true
```

#### 4. Update CI/CD Pipeline
```yaml
# .github/workflows/ci.yml
env:
  SQLX_OFFLINE: true
```

## Connection Pool Optimization

### Current Configuration
```rust
// Hard-coded defaults
.max_connections(10)
.min_connections(1)
.acquire_timeout(Duration::from_secs(30))
```

### Improved Configuration

#### 1. Environment-Based Sizing
```rust
// crates/config/src/lib.rs
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub connections_max: u32,
    pub connections_min: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: Option<u64>,
    pub max_lifetime_seconds: Option<u64>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get() as u32;
        Self {
            url: String::new(),
            // Formula: (cpu_cores * 2) + effective_spindle_count
            connections_max: (cpu_count * 2) + 1,
            connections_min: cpu_count,
            connection_timeout_seconds: 30,
            idle_timeout_seconds: Some(600), // 10 minutes
            max_lifetime_seconds: Some(1800), // 30 minutes
        }
    }
}
```

#### 2. Adaptive Pool Implementation
```rust
// crates/database/src/lib.rs
impl Database {
    pub async fn new_with_config(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(config.connections_max)
            .min_connections(config.connections_min)
            .acquire_timeout(Duration::from_secs(config.connection_timeout_seconds))
            .idle_timeout(config.idle_timeout_seconds.map(Duration::from_secs))
            .max_lifetime(config.max_lifetime_seconds.map(Duration::from_secs))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set connection parameters
                    conn.execute("SET statement_timeout = '30s'").await?;
                    conn.execute("SET lock_timeout = '10s'").await?;
                    Ok(())
                })
            })
            .connect_lazy(&config.url)?;

        Ok(Self { pool })
    }
}
```

## Query Performance Optimization

### 1. Add Query Instrumentation
```rust
// crates/database/src/repositories/user.rs
use tracing::instrument;

impl UserRepository {
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn find_by_id(&self, user_id: Uuid) -> Result<Option<User>, DatabaseError> {
        let start = std::time::Instant::now();
        
        let result = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users 
            WHERE id = $1
            "#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        let duration = start.elapsed();
        if duration > Duration::from_millis(100) {
            tracing::warn!("Slow query detected: find_by_id took {:?}", duration);
        }
        
        Ok(result)
    }
}
```

### 2. Add Connection Pool Metrics
```rust
// crates/database/src/metrics.rs
use sqlx::PgPool;

pub struct PoolMetrics {
    pub size: u32,
    pub num_idle: u32,
    pub num_acquires: u64,
    pub num_pending_acquires: u64,
}

impl Database {
    pub fn pool_metrics(&self) -> PoolMetrics {
        let options = self.pool.options();
        PoolMetrics {
            size: self.pool.size(),
            num_idle: self.pool.num_idle(),
            // These would need custom tracking
            num_acquires: 0,
            num_pending_acquires: 0,
        }
    }
}
```

## Migration Management

### 1. Add Migration Verification
```rust
// crates/database/src/migrations.rs
pub async fn verify_migrations(pool: &PgPool) -> Result<bool, DatabaseError> {
    let pending = sqlx::migrate!("./migrations")
        .validate(pool)
        .await?;
    
    Ok(pending.is_empty())
}
```

### 2. Add Rollback Support
```rust
pub async fn rollback_last_migration(pool: &PgPool) -> Result<(), DatabaseError> {
    // Implementation depends on tracking migration history
    todo!("Implement migration rollback")
}
```

## Health Checks

### Implement Comprehensive Health Check
```rust
impl Database {
    pub async fn detailed_health_check(&self) -> Result<HealthStatus, DatabaseError> {
        let start = std::time::Instant::now();
        
        // Basic connectivity
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        
        let latency = start.elapsed();
        
        // Check pool health
        let pool_metrics = self.pool_metrics();
        
        Ok(HealthStatus {
            connected: true,
            latency_ms: latency.as_millis() as u64,
            pool_size: pool_metrics.size,
            pool_available: pool_metrics.num_idle,
            migrations_current: self.verify_migrations().await?,
        })
    }
}
```

## Testing Strategy

### 1. Add Database Test Utilities
```rust
// crates/database/src/test_utils.rs
#[cfg(test)]
pub async fn setup_test_db() -> (Database, String) {
    use uuid::Uuid;
    
    let db_name = format!("test_db_{}", Uuid::new_v4());
    let admin_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost".to_string());
    
    // Create test database
    let admin_pool = PgPool::connect(&admin_url).await.unwrap();
    sqlx::query(&format!("CREATE DATABASE {}", db_name))
        .execute(&admin_pool)
        .await
        .unwrap();
    
    let test_url = format!("{}/{}", admin_url, db_name);
    let db = Database::new(&test_url).await.unwrap();
    db.migrate().await.unwrap();
    
    (db, db_name)
}
```

### 2. Add Integration Tests
```rust
#[tokio::test]
async fn test_connection_pool_recovery() {
    let (db, _) = setup_test_db().await;
    
    // Simulate connection drops
    for _ in 0..20 {
        let result = db.health_check().await;
        assert!(result.is_ok());
    }
    
    // Verify pool recovered
    let metrics = db.pool_metrics();
    assert!(metrics.num_idle > 0);
}