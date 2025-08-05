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

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Permission denied")]
    PermissionDenied,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<reqwest::Error> for SourceError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SourceError::Network("Request timed out".to_string())
        } else if err.is_connect() {
            SourceError::Network("Connection failed".to_string())
        } else if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
            SourceError::NotFound("Resource not found".to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            SourceError::RateLimited
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            SourceError::AuthenticationRequired
        } else if err.status() == Some(reqwest::StatusCode::FORBIDDEN) {
            SourceError::PermissionDenied
        } else {
            SourceError::Network(err.to_string())
        }
    }
}

impl From<serde_json::Error> for SourceError {
    fn from(err: serde_json::Error) -> Self {
        SourceError::Parse(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for SourceError {
    fn from(err: std::io::Error) -> Self {
        SourceError::Other(err.into())
    }
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Not found")]
    NotFound,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            sqlx::Error::Database(e) => DatabaseError::Query(e.to_string()),
            sqlx::Error::PoolTimedOut => DatabaseError::Connection("Pool timed out".to_string()),
            sqlx::Error::PoolClosed => DatabaseError::Connection("Pool closed".to_string()),
            _ => DatabaseError::Other(err.into()),
        }
    }
}

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid state")]
    InvalidState,

    #[error("Provider error: {0}")]
    Provider(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
pub enum JobError {
    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Processing error: {0}")]
    Processing(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Max retries exceeded")]
    MaxRetriesExceeded,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<serde_json::Error> for JobError {
    fn from(err: serde_json::Error) -> Self {
        JobError::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_error_display() {
        let err = SourceError::Network("Connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: Connection refused");

        let err = SourceError::NotFound("Serie not found".to_string());
        assert_eq!(err.to_string(), "Not found: Serie not found");

        let err = SourceError::RateLimited;
        assert_eq!(err.to_string(), "Rate limited");
    }

    #[test]
    fn test_error_conversion() {
        let json_err = serde_json::from_str::<String>("invalid json").unwrap_err();
        let source_err: SourceError = json_err.into();
        assert!(matches!(source_err, SourceError::Parse(_)));
    }
}