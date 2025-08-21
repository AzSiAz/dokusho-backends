# Error Handling Improvements

## Current Issues

The codebase contains multiple instances of `unwrap()` and `expect()` that could cause panics in production. This is particularly problematic in web scrapers where external content can be unpredictable.

## Problematic Patterns Found

### 1. Hardcoded URL Parsing
```rust
// crates/sources/src/scrapers/weebcentral/mod.rs:263
let no_image_url: Url = Url::parse("https://i.imgur.com/6TrIues.jpeg").unwrap();
```

### 2. Selector Parsing in Loops
```rust
// crates/sources/src/scrapers/weebcentral/mod.rs:267
let cover_selector = Selector::parse("section:first-child a > article > picture > source").unwrap();
```

### 3. Unwrap with Defaults Available
```rust
// Multiple locations
.unwrap_or_else(|| no_image_url.clone());
```

## Recommended Solutions

### 1. Use Lazy Static for Constants
```rust
use once_cell::sync::Lazy;

static DEFAULT_COVER_URL: Lazy<Url> = Lazy::new(|| {
    Url::parse("https://i.imgur.com/6TrIues.jpeg")
        .expect("Default cover URL should be valid")
});

static SELECTORS: Lazy<Selectors> = Lazy::new(|| Selectors {
    article: Selector::parse("body > article")
        .expect("Article selector should be valid"),
    cover: Selector::parse("section:first-child a > article > picture > source")
        .expect("Cover selector should be valid"),
    // ... other selectors
});
```

### 2. Create Fallible Initialization
```rust
impl WeebCentralScraper {
    pub fn new() -> Result<Self, ScraperError> {
        // Validate all selectors and constants at initialization
        let selectors = Selectors::new()?;
        Ok(Self { selectors })
    }
}
```

### 3. Use Result Types in Parsing Functions
```rust
fn parse_cover_url(&self, element: &ElementRef) -> Result<Url, ScraperError> {
    element
        .select(&self.selectors.cover)
        .next()
        .and_then(|el| el.value().attr("srcset"))
        .ok_or_else(|| ScraperError::MissingElement("cover image"))?
        .parse()
        .map_err(|e| ScraperError::InvalidUrl(e))
}
```

### 4. Implement Graceful Degradation
```rust
fn extract_serie_info(&self, article: &ElementRef) -> SourceSerie {
    let cover = self.parse_cover_url(article)
        .unwrap_or_else(|_| DEFAULT_COVER_URL.clone());
    
    let title = self.parse_title(article)
        .unwrap_or_else(|_| MultiLanguageString::new()
            .insert(SourceLanguage::En, "Unknown Title".to_string()));
    
    // Continue with other fields...
}
```

## Implementation Priority

1. **High Priority**: Fix all `unwrap()` in HTTP request handlers and API endpoints
2. **Medium Priority**: Replace `unwrap()` in scrapers with proper error handling
3. **Low Priority**: Review and update test code (unwraps are more acceptable here)

## Testing Strategy

1. Add unit tests for error conditions:
```rust
#[test]
fn test_invalid_html_parsing() {
    let scraper = WeebCentralScraper::new().unwrap();
    let invalid_html = "<div>broken html";
    let result = scraper.parse_series_list(invalid_html);
    assert!(matches!(result, Err(ScraperError::ParseError(_))));
}
```

2. Add integration tests with malformed responses
3. Use property-based testing for URL parsing

## Monitoring

After implementing these changes:
1. Add error metrics for each error type
2. Set up alerts for error rate thresholds
3. Log errors with appropriate context for debugging