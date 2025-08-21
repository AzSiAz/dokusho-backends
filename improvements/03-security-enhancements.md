# Security Enhancements

## Current Security Gaps

1. JWT secret has no validation for strength
2. CORS allows "*" by default (too permissive)
3. No rate limiting on API endpoints
4. Missing request size limits
5. No SQL injection protection beyond SQLX's query macros
6. Sensitive data might be logged

## JWT Security Improvements

### 1. Secret Validation
```rust
// crates/auth/src/models.rs
use zeroize::Zeroize;

#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct JwtSecret(String);

impl JwtSecret {
    pub fn new(secret: String) -> Result<Self, AuthError> {
        // Validate minimum entropy
        if secret.len() < 32 {
            return Err(AuthError::Configuration(
                "JWT secret must be at least 32 characters".into()
            ));
        }
        
        // Check for common weak patterns
        if Self::is_weak_secret(&secret) {
            return Err(AuthError::Configuration(
                "JWT secret appears to be weak".into()
            ));
        }
        
        Ok(Self(secret))
    }
    
    fn is_weak_secret(secret: &str) -> bool {
        // Check for repeated characters
        let unique_chars: std::collections::HashSet<_> = secret.chars().collect();
        if unique_chars.len() < 10 {
            return true;
        }
        
        // Check for common patterns
        let weak_patterns = ["123456", "password", "secret", "admin", "default"];
        weak_patterns.iter().any(|pattern| secret.contains(pattern))
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
```

### 2. Token Rotation Strategy
```rust
// crates/auth/src/token.rs
pub struct TokenRotation {
    current_secret: JwtSecret,
    previous_secret: Option<JwtSecret>,
    rotation_time: chrono::DateTime<chrono::Utc>,
}

impl TokenRotation {
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        // Try current secret first
        if let Ok(claims) = validate_jwt(token, &self.current_secret) {
            return Ok(claims);
        }
        
        // Fall back to previous secret if within grace period
        if let Some(ref prev_secret) = self.previous_secret {
            if chrono::Utc::now() < self.rotation_time + chrono::Duration::hours(1) {
                return validate_jwt(token, prev_secret);
            }
        }
        
        Err(AuthError::InvalidToken)
    }
}
```

## CORS Configuration

### 1. Strict CORS Policy
```rust
// apps/api/src/middleware/cors.rs
use tower_http::cors::{CorsLayer, AllowOrigin};
use http::header::{AUTHORIZATION, CONTENT_TYPE};

pub fn create_cors_layer(allowed_origins: &[String]) -> CorsLayer {
    if allowed_origins.is_empty() {
        panic!("CORS origins must be explicitly configured");
    }
    
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| {
            // Validate origin format
            if !origin.starts_with("http://") && !origin.starts_with("https://") {
                tracing::warn!("Invalid CORS origin: {}", origin);
                return None;
            }
            origin.parse().ok()
        })
        .collect();
    
    if origins.is_empty() {
        panic!("No valid CORS origins configured");
    }
    
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}
```

## Rate Limiting

### 1. Token Bucket Implementation
```rust
// apps/api/src/middleware/rate_limit.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::net::IpAddr;

pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<IpAddr, TokenBucket>>>,
    max_tokens: u32,
    refill_rate: u32, // tokens per second
}

struct TokenBucket {
    tokens: u32,
    last_refill: std::time::Instant,
}

impl RateLimiter {
    pub fn new(max_tokens: u32, refill_rate: u32) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            max_tokens,
            refill_rate,
        }
    }
    
    pub async fn check_rate_limit(&self, ip: IpAddr) -> Result<(), RateLimitError> {
        let mut buckets = self.buckets.write().await;
        let now = std::time::Instant::now();
        
        let bucket = buckets.entry(ip).or_insert_with(|| TokenBucket {
            tokens: self.max_tokens,
            last_refill: now,
        });
        
        // Refill tokens
        let elapsed = now.duration_since(bucket.last_refill);
        let tokens_to_add = (elapsed.as_secs() as u32 * self.refill_rate)
            .min(self.max_tokens - bucket.tokens);
        bucket.tokens += tokens_to_add;
        bucket.last_refill = now;
        
        // Check if request is allowed
        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            Ok(())
        } else {
            Err(RateLimitError::TooManyRequests)
        }
    }
}

// Middleware implementation
pub async fn rate_limit_middleware<B>(
    State(limiter): State<Arc<RateLimiter>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    if let Err(_) = limiter.check_rate_limit(addr.ip()).await {
        return Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Retry-After", "60")
            .body("Too many requests".into())
            .unwrap();
    }
    
    next.run(request).await
}
```

### 2. GraphQL Query Complexity Limiting
```rust
// apps/api/src/graphql/middleware.rs
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextParseQuery};

pub struct ComplexityLimit {
    max_complexity: usize,
}

impl ExtensionFactory for ComplexityLimit {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(ComplexityLimitExtension {
            max_complexity: self.max_complexity,
        })
    }
}

struct ComplexityLimitExtension {
    max_complexity: usize,
}

#[async_trait::async_trait]
impl Extension for ComplexityLimitExtension {
    async fn parse_query(
        &self,
        ctx: &ExtensionContext<'_>,
        query: &str,
        variables: &Variables,
        next: NextParseQuery<'_>,
    ) -> ServerResult<ExecutableDocument> {
        let doc = next.run(ctx, query, variables).await?;
        
        // Calculate query complexity
        let complexity = calculate_complexity(&doc);
        
        if complexity > self.max_complexity {
            return Err(ServerError::new(
                format!("Query too complex: {} > {}", complexity, self.max_complexity),
                None,
            ));
        }
        
        Ok(doc)
    }
}
```

## Request Security

### 1. Request Size Limits
```rust
// apps/api/src/middleware/security.rs
use tower_http::limit::RequestBodyLimitLayer;

pub fn security_layers() -> impl Layer<Routes> {
    ServiceBuilder::new()
        // Limit request body size to 10MB
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
        // Add timeout
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        // Compression
        .layer(CompressionLayer::new())
}
```

### 2. Input Sanitization
```rust
// crates/core/src/validation.rs
use regex::Regex;
use once_cell::sync::Lazy;

static SQL_INJECTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\b(SELECT|INSERT|UPDATE|DELETE|DROP|UNION|ALTER|CREATE)\b)|(--|;|/\*|\*/)")
        .unwrap()
});

pub fn sanitize_user_input(input: &str) -> Result<String, ValidationError> {
    // Check length
    if input.len() > 1000 {
        return Err(ValidationError::InputTooLong);
    }
    
    // Check for SQL injection patterns
    if SQL_INJECTION_PATTERN.is_match(&input.to_uppercase()) {
        return Err(ValidationError::SuspiciousInput);
    }
    
    // Remove null bytes
    let cleaned = input.replace('\0', "");
    
    // HTML escape
    let escaped = html_escape::encode_text(&cleaned);
    
    Ok(escaped.to_string())
}
```

## Logging Security

### 1. Sensitive Data Redaction
```rust
// crates/core/src/logging.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct SanitizedRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
}

impl From<&Request<Body>> for SanitizedRequest {
    fn from(req: &Request<Body>) -> Self {
        let mut headers = HashMap::new();
        
        for (key, value) in req.headers() {
            let key_str = key.as_str();
            
            // Redact sensitive headers
            let value_str = if key_str.eq_ignore_ascii_case("authorization") {
                "***REDACTED***".to_string()
            } else if key_str.eq_ignore_ascii_case("cookie") {
                "***REDACTED***".to_string()
            } else {
                value.to_str().unwrap_or("").to_string()
            };
            
            headers.insert(key_str.to_string(), value_str);
        }
        
        Self {
            method: req.method().to_string(),
            path: req.uri().path().to_string(),
            headers,
        }
    }
}
```

## Security Headers

### Add Security Headers Middleware
```rust
// apps/api/src/middleware/headers.rs
use tower_http::set_header::SetResponseHeaderLayer;

pub fn security_headers() -> impl Layer<Routes> {
    ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'"),
        ))
}
```

## Testing Security

### Security Test Suite
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sql_injection_prevention() {
        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "1 OR 1=1",
            "admin'--",
            "1; SELECT * FROM users",
        ];
        
        for input in malicious_inputs {
            let result = sanitize_user_input(input);
            assert!(result.is_err() || !result.unwrap().contains("DROP"));
        }
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(10, 1);
        let ip = "127.0.0.1".parse().unwrap();
        
        // Should allow first 10 requests
        for _ in 0..10 {
            assert!(limiter.check_rate_limit(ip).await.is_ok());
        }
        
        // Should block 11th request
        assert!(limiter.check_rate_limit(ip).await.is_err());
    }
}
```