# Comprehensive Testing Strategy

## Current Testing Gaps

1. Limited unit test coverage
2. No integration tests for scrapers
3. Missing property-based testing
4. No performance benchmarks
5. Lack of test fixtures management

## Test Organization Structure

```
tests/
├── unit/           # Unit tests for individual functions
├── integration/    # Integration tests with real dependencies
├── e2e/           # End-to-end API tests
├── fixtures/      # Test data and mocks
├── common/        # Shared test utilities
└── benchmarks/    # Performance benchmarks
```

## Unit Testing

### 1. Core Business Logic Tests

```rust
// crates/core/src/sources/source_types.rs
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_source_serie_id_creation() {
        let id = SourceSerieId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
    }
    
    #[test]
    fn test_multi_language_string() {
        let mut mls = MultiLanguageString::new();
        mls.insert(SourceLanguage::En, "Hello".to_string());
        mls.insert(SourceLanguage::Ja, "こんにちは".to_string());
        
        assert_eq!(mls.get(&SourceLanguage::En), Some(&"Hello".to_string()));
        assert_eq!(mls.get(&SourceLanguage::Fr), None);
    }
    
    proptest! {
        #[test]
        fn test_source_serie_id_never_panics(s in "\\PC*") {
            let _ = SourceSerieId::new(&s);
        }
        
        #[test]
        fn test_chapter_number_parsing(n in 0.0f32..10000.0) {
            let chapter = SourceChapter {
                number: n,
                // ... other fields
            };
            assert!(chapter.number >= 0.0);
        }
    }
}
```

### 2. Error Handling Tests

```rust
// crates/clients/src/errors.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_display() {
        let err = ClientError::Timeout;
        assert_eq!(err.to_string(), "Request timeout");
        
        let err = ClientError::HttpError(404);
        assert_eq!(err.to_string(), "HTTP error: 404");
    }
    
    #[test]
    fn test_error_conversion() {
        let reqwest_err = reqwest::Error::from(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            "timeout"
        ));
        let client_err: ClientError = reqwest_err.into();
        assert!(matches!(client_err, ClientError::Timeout));
    }
}
```

## Integration Testing

### 1. Database Integration Tests

```rust
// crates/database/tests/integration.rs
use dokusho_database::{Database, models::User};
use sqlx::PgPool;
use uuid::Uuid;

async fn setup_test_db() -> (Database, TestCleanup) {
    let db_name = format!("test_{}", Uuid::new_v4());
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
    
    let cleanup = TestCleanup {
        db_name: db_name.clone(),
        admin_pool,
    };
    
    (db, cleanup)
}

struct TestCleanup {
    db_name: String,
    admin_pool: PgPool,
}

impl Drop for TestCleanup {
    fn drop(&mut self) {
        // Clean up test database
        let db_name = self.db_name.clone();
        let pool = self.admin_pool.clone();
        tokio::spawn(async move {
            let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS {}", db_name))
                .execute(&pool)
                .await;
        });
    }
}

#[tokio::test]
async fn test_user_crud_operations() {
    let (db, _cleanup) = setup_test_db().await;
    
    // Create user
    let user = db.users().create(NewUser {
        email: "test@example.com".to_string(),
        name: Some("Test User".to_string()),
        provider_id: "provider123".to_string(),
        role: UserRole::User,
    }).await.unwrap();
    
    assert_eq!(user.email, "test@example.com");
    
    // Find user
    let found = db.users().find_by_email("test@example.com")
        .await.unwrap()
        .expect("User should exist");
    
    assert_eq!(found.id, user.id);
    
    // Update user
    let updated = db.users().update(user.id, UpdateUser {
        name: Some("Updated Name".to_string()),
        ..Default::default()
    }).await.unwrap();
    
    assert_eq!(updated.name, Some("Updated Name".to_string()));
    
    // Delete user
    db.users().delete(user.id).await.unwrap();
    
    let deleted = db.users().find_by_id(user.id).await.unwrap();
    assert!(deleted.is_none());
}
```

### 2. Scraper Integration Tests

```rust
// crates/sources/tests/scrapers.rs
use sources::scrapers::{WeebCentralScraper, MangaDexScraper};
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_weebcentral_scraper_with_mock() {
    let mock_server = MockServer::start().await;
    
    // Setup mock response
    let mock_html = include_str!("fixtures/weebcentral_popular.html");
    Mock::given(method("GET"))
        .and(path("/browse"))
        .respond_with(ResponseTemplate::new(200).set_body_string(mock_html))
        .mount(&mock_server)
        .await;
    
    let scraper = WeebCentralScraper::new_with_base_url(
        mock_server.uri().parse().unwrap()
    );
    
    let series = scraper.get_popular(1).await.unwrap();
    
    assert!(!series.is_empty());
    assert_eq!(series[0].title.get(&SourceLanguage::En), Some(&"Test Manga".to_string()));
}

#[tokio::test]
async fn test_mangadex_api_client() {
    let mock_server = MockServer::start().await;
    
    let mock_response = include_str!("fixtures/mangadex_manga_response.json");
    Mock::given(method("GET"))
        .and(path("/manga"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_string(mock_response)
            .insert_header("content-type", "application/json"))
        .mount(&mock_server)
        .await;
    
    let client = MangaDexScraper::new_with_base_url(
        mock_server.uri().parse().unwrap()
    );
    
    let series = client.get_popular(1).await.unwrap();
    assert!(!series.is_empty());
}
```

## End-to-End Testing

### GraphQL API Tests

```rust
// tests/e2e/graphql.rs
use axum::test::{TestClient, TestServer};
use serde_json::json;

async fn setup_test_server() -> TestServer {
    let app = create_app_with_test_config().await;
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_graphql_query_sources() {
    let server = setup_test_server().await;
    let client = TestClient::new(server);
    
    let query = json!({
        "query": r#"
            query {
                sources {
                    name
                    languages
                }
            }
        "#
    });
    
    let response = client
        .post("/graphql")
        .json(&query)
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await;
    assert!(body["data"]["sources"].is_array());
}

#[tokio::test]
async fn test_graphql_authentication_required() {
    let server = setup_test_server().await;
    let client = TestClient::new(server);
    
    let query = json!({
        "query": r#"
            query {
                me {
                    id
                    email
                }
            }
        "#
    });
    
    let response = client
        .post("/graphql")
        .json(&query)
        .send()
        .await;
    
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await;
    assert!(body["errors"][0]["message"].as_str().unwrap().contains("Unauthorized"));
}
```

## Test Fixtures Management

### 1. Fixture Loading System

```rust
// tests/common/fixtures.rs
use std::path::PathBuf;
use once_cell::sync::Lazy;

pub struct Fixtures;

impl Fixtures {
    pub fn load(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name);
        
        std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
    }
    
    pub fn load_json<T: serde::de::DeserializeOwned>(name: &str) -> T {
        let content = Self::load(name);
        serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", name, e))
    }
}

// Cached fixtures for repeated use
pub static SAMPLE_MANGA_HTML: Lazy<String> = Lazy::new(|| {
    Fixtures::load("sample_manga.html")
});
```

### 2. Test Data Builders

```rust
// tests/common/builders.rs
use dokusho_core::sources::*;

pub struct SerieBuilder {
    serie: SourceSerie,
}

impl SerieBuilder {
    pub fn new() -> Self {
        Self {
            serie: SourceSerie {
                id: SourceSerieId::new("test-serie"),
                title: MultiLanguageString::new()
                    .insert(SourceLanguage::En, "Test Serie".to_string()),
                cover: Url::parse("https://example.com/cover.jpg").unwrap(),
                series_type: SourceSerieType::Manga,
                status: SourceSerieStatus::Ongoing,
                description: None,
                genres: vec![],
                authors: vec![],
                artists: vec![],
            }
        }
    }
    
    pub fn with_title(mut self, lang: SourceLanguage, title: &str) -> Self {
        self.serie.title.insert(lang, title.to_string());
        self
    }
    
    pub fn with_status(mut self, status: SourceSerieStatus) -> Self {
        self.serie.status = status;
        self
    }
    
    pub fn build(self) -> SourceSerie {
        self.serie
    }
}
```

## Performance Testing

### Benchmark Suite

```rust
// benches/scraping.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sources::scrapers::WeebCentralScraper;

fn bench_html_parsing(c: &mut Criterion) {
    let html = include_str!("../tests/fixtures/large_page.html");
    let scraper = WeebCentralScraper::new();
    
    c.bench_function("parse_series_list", |b| {
        b.iter(|| {
            scraper.parse_series_list(black_box(html))
        });
    });
}

fn bench_concurrent_requests(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("concurrent_requests");
    for size in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.to_async(&runtime).iter(|| async move {
                fetch_multiple_series(size).await
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_html_parsing, bench_concurrent_requests);
criterion_main!(benches);
```

## Continuous Testing

### GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_PASSWORD: postgres
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run migrations
      run: |
        cargo install sqlx-cli --no-default-features --features postgres
        DATABASE_URL=postgresql://postgres:postgres@localhost/test_db sqlx database create
        DATABASE_URL=postgresql://postgres:postgres@localhost/test_db sqlx migrate run
    
    - name: Run tests
      env:
        DATABASE_URL: postgresql://postgres:postgres@localhost/test_db
        TEST_DATABASE_URL: postgresql://postgres:postgres@localhost
      run: |
        cargo test --all-features --workspace
        cargo test --doc
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run benchmarks
      run: cargo bench --no-run
```

## Test Coverage

### Generate Coverage Reports

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir ./coverage

# With specific test filtering
cargo tarpaulin --ignore-tests --timeout 120 --exclude-files "*/tests/*" --out Lcov
```

### Coverage Goals

- Unit tests: >80% coverage
- Integration tests: All critical paths covered
- E2E tests: Happy path + major error cases
- Performance benchmarks: Key hot paths