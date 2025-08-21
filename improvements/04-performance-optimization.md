# Performance Optimization Guide

## Current Performance Issues

1. Selector parsing happens on every scraping operation
2. Frequent string allocations and cloning
3. No caching layer for scraped data
4. Missing concurrent request handling
5. No connection pooling for HTTP clients

## Selector Caching

### Problem
Selectors are parsed repeatedly in hot paths, causing unnecessary CPU overhead.

### Solution: Static Selector Initialization

```rust
// crates/sources/src/scrapers/weebcentral/selectors.rs
use once_cell::sync::Lazy;
use scraper::Selector;
use std::collections::HashMap;

pub struct WeebCentralSelectors {
    pub article: Selector,
    pub cover: Selector,
    pub title: Selector,
    pub link: Selector,
    pub chapter: Selector,
    pub chapter_name: Selector,
    pub chapter_time: Selector,
    pub image: Selector,
}

impl WeebCentralSelectors {
    fn new() -> Result<Self, String> {
        Ok(Self {
            article: Selector::parse("body > article")
                .map_err(|e| format!("Failed to parse article selector: {:?}", e))?,
            cover: Selector::parse("section:first-child a > article > picture > source")
                .map_err(|e| format!("Failed to parse cover selector: {:?}", e))?,
            title: Selector::parse("section:last-child div:first-child a")
                .map_err(|e| format!("Failed to parse title selector: {:?}", e))?,
            link: Selector::parse("section:first-child a")
                .map_err(|e| format!("Failed to parse link selector: {:?}", e))?,
            chapter: Selector::parse("body > * > a.flex")
                .map_err(|e| format!("Failed to parse chapter selector: {:?}", e))?,
            chapter_name: Selector::parse("span.flex > span")
                .map_err(|e| format!("Failed to parse chapter name selector: {:?}", e))?,
            chapter_time: Selector::parse("time")
                .map_err(|e| format!("Failed to parse time selector: {:?}", e))?,
            image: Selector::parse("img")
                .map_err(|e| format!("Failed to parse image selector: {:?}", e))?,
        })
    }
}

pub static SELECTORS: Lazy<WeebCentralSelectors> = Lazy::new(|| {
    WeebCentralSelectors::new().expect("Failed to initialize selectors")
});

// Usage in scraper
impl WeebCentralScraper {
    fn parse_series(&self, html: &str) -> Vec<SourceSerie> {
        let document = Html::parse_document(html);
        let mut series = Vec::new();
        
        for article in document.select(&SELECTORS.article) {
            // Use pre-parsed selectors
            let cover = article.select(&SELECTORS.cover).next()...
        }
        
        series
    }
}
```

## String Optimization

### Use Arc<str> for Immutable Strings

```rust
// crates/core/src/sources/source_types.rs
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SourceSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub description: Option<Arc<str>>, // Changed from String
    pub cover: Url,
    pub series_type: SourceSerieType,
    pub status: SourceSerieStatus,
    pub genres: Vec<Arc<str>>, // Changed from Vec<String>
    pub authors: Vec<Arc<str>>,
    pub artists: Vec<Arc<str>>,
}

// String interning for common strings
pub struct StringInterner {
    cache: DashMap<String, Arc<str>>,
}

impl StringInterner {
    pub fn intern(&self, s: String) -> Arc<str> {
        self.cache
            .entry(s.clone())
            .or_insert_with(|| Arc::from(s.as_str()))
            .clone()
    }
}

// Global interner for genres/tags
static GENRE_INTERNER: Lazy<StringInterner> = Lazy::new(|| StringInterner::default());
```

## HTTP Client Optimization

### Connection Pooling and Reuse

```rust
// crates/clients/src/http/client_pool.rs
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub struct OptimizedHttpClient {
    client: Client,
    semaphore: Arc<Semaphore>, // Limit concurrent requests
}

impl OptimizedHttpClient {
    pub fn new(max_concurrent: usize) -> Result<Self, ClientError> {
        let client = ClientBuilder::new()
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(32)
            .timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .gzip(true)
            .brotli(true)
            .build()?;
        
        Ok(Self {
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        })
    }
    
    pub async fn get_with_limit(&self, url: &Url) -> Result<Response, ClientError> {
        let _permit = self.semaphore.acquire().await?;
        self.client.get(url.as_str()).send().await
            .map_err(ClientError::from)
    }
}
```

## Caching Layer

### 1. In-Memory Cache for Hot Data

```rust
// crates/sources/src/cache/memory.rs
use moka::future::Cache;
use std::time::Duration;

pub struct SeriesCache {
    series: Cache<SourceSerieId, Arc<SourceSerie>>,
    chapters: Cache<(SourceSerieId, u32), Arc<Vec<SourceChapter>>>, // (serie_id, page)
}

impl SeriesCache {
    pub fn new() -> Self {
        Self {
            series: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(300)) // 5 minutes
                .build(),
            chapters: Cache::builder()
                .max_capacity(500)
                .time_to_live(Duration::from_secs(600)) // 10 minutes
                .build(),
        }
    }
    
    pub async fn get_serie(
        &self,
        id: &SourceSerieId,
        fetch: impl Future<Output = Result<SourceSerie, Error>>,
    ) -> Result<Arc<SourceSerie>, Error> {
        self.series
            .try_get_with(id.clone(), async {
                fetch.await.map(Arc::new)
            })
            .await
            .map_err(|e| e.as_ref().clone())
    }
}
```

### 2. Redis Cache for Distributed Systems

```rust
// crates/sources/src/cache/redis.rs
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{Deserialize, Serialize};

pub struct RedisCache {
    conn: ConnectionManager,
    ttl: u64,
}

impl RedisCache {
    pub async fn get_or_fetch<T, F, Fut>(
        &mut self,
        key: &str,
        fetch: F,
    ) -> Result<T, CacheError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        // Try to get from cache
        if let Ok(cached) = self.conn.get::<_, Vec<u8>>(key).await {
            if let Ok(value) = bincode::deserialize(&cached) {
                return Ok(value);
            }
        }
        
        // Fetch and cache
        let value = fetch().await?;
        let serialized = bincode::serialize(&value)?;
        self.conn.set_ex(key, serialized, self.ttl).await?;
        
        Ok(value)
    }
}
```

## Concurrent Processing

### Batch Processing with Parallelism

```rust
// crates/sources/src/scrapers/batch.rs
use futures::stream::{self, StreamExt};

pub async fn fetch_series_batch(
    client: &OptimizedHttpClient,
    serie_ids: Vec<SourceSerieId>,
) -> Vec<Result<SourceSerie, Error>> {
    const BATCH_SIZE: usize = 10;
    
    stream::iter(serie_ids)
        .chunks(BATCH_SIZE)
        .then(|batch| async move {
            // Process batch concurrently
            let futures = batch.into_iter().map(|id| {
                fetch_single_serie(client, id)
            });
            
            futures::future::join_all(futures).await
        })
        .flat_map(stream::iter)
        .collect()
        .await
}

// GraphQL DataLoader integration
use async_graphql::dataloader::{DataLoader, Loader};

pub struct SerieLoader {
    client: Arc<OptimizedHttpClient>,
}

#[async_trait::async_trait]
impl Loader<SourceSerieId> for SerieLoader {
    type Value = SourceSerie;
    type Error = Error;
    
    async fn load(&self, keys: &[SourceSerieId]) -> Result<HashMap<SourceSerieId, Self::Value>, Self::Error> {
        let results = fetch_series_batch(&self.client, keys.to_vec()).await;
        
        let mut map = HashMap::new();
        for (id, result) in keys.iter().zip(results) {
            if let Ok(serie) = result {
                map.insert(id.clone(), serie);
            }
        }
        
        Ok(map)
    }
}
```

## Database Query Optimization

### 1. Prepared Statements Cache

```rust
// crates/database/src/prepared.rs
use sqlx::{PgPool, postgres::PgStatement};
use dashmap::DashMap;

pub struct PreparedStatements {
    statements: DashMap<String, PgStatement<'static>>,
    pool: PgPool,
}

impl PreparedStatements {
    pub async fn get_or_prepare(&self, sql: &str) -> Result<PgStatement<'static>, DatabaseError> {
        if let Some(stmt) = self.statements.get(sql) {
            return Ok(stmt.clone());
        }
        
        let stmt = self.pool.prepare(sql).await?;
        self.statements.insert(sql.to_string(), stmt.clone());
        Ok(stmt)
    }
}
```

### 2. Batch Insert Optimization

```rust
// crates/database/src/repositories/batch.rs
impl UserRepository {
    pub async fn insert_batch(&self, users: &[NewUser]) -> Result<Vec<User>, DatabaseError> {
        if users.is_empty() {
            return Ok(vec![]);
        }
        
        // Build batch insert query
        let mut query_builder = QueryBuilder::new(
            "INSERT INTO users (email, name, provider_id, role) "
        );
        
        query_builder.push_values(users, |mut b, user| {
            b.push_bind(&user.email)
             .push_bind(&user.name)
             .push_bind(&user.provider_id)
             .push_bind(&user.role);
        });
        
        query_builder.push(" RETURNING *");
        
        let users = query_builder
            .build_query_as::<User>()
            .fetch_all(&self.pool)
            .await?;
        
        Ok(users)
    }
}
```

## Memory Management

### Arena Allocation for Parsing

```rust
// crates/sources/src/scrapers/arena.rs
use bumpalo::Bump;

pub struct ParsingArena {
    arena: Bump,
}

impl ParsingArena {
    pub fn parse_with_arena<'a>(&'a self, html: &str) -> ParsedDocument<'a> {
        // Allocate strings in arena to avoid individual allocations
        let title = self.arena.alloc_str(extracted_title);
        let description = self.arena.alloc_str(extracted_desc);
        
        ParsedDocument {
            title,
            description,
            // ... other fields
        }
    }
}
```

## Monitoring and Profiling

### Performance Metrics

```rust
// crates/core/src/metrics.rs
use prometheus::{Histogram, HistogramOpts, IntCounter, register_histogram, register_int_counter};

lazy_static! {
    pub static ref SCRAPE_DURATION: Histogram = register_histogram!(
        HistogramOpts::new("scrape_duration_seconds", "Time taken to scrape a page")
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
    ).unwrap();
    
    pub static ref CACHE_HITS: IntCounter = register_int_counter!(
        "cache_hits_total", "Number of cache hits"
    ).unwrap();
    
    pub static ref CACHE_MISSES: IntCounter = register_int_counter!(
        "cache_misses_total", "Number of cache misses"
    ).unwrap();
}

// Usage
pub async fn scrape_with_metrics(url: &Url) -> Result<Document, Error> {
    let timer = SCRAPE_DURATION.start_timer();
    let result = actual_scrape(url).await;
    timer.observe_duration();
    result
}
```

## Testing Performance

```rust
#[cfg(test)]
mod bench {
    use criterion::{black_box, criterion_group, criterion_main, Criterion};
    
    fn bench_selector_parsing(c: &mut Criterion) {
        c.bench_function("parse selectors", |b| {
            b.iter(|| {
                let _ = Selector::parse(black_box("body > article")).unwrap();
            });
        });
    }
    
    fn bench_with_cache(c: &mut Criterion) {
        c.bench_function("cached selectors", |b| {
            b.iter(|| {
                let _ = &SELECTORS.article;
            });
        });
    }
    
    criterion_group!(benches, bench_selector_parsing, bench_with_cache);
    criterion_main!(benches);
}