use thiserror::Error;

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
