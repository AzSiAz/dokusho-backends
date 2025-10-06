use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<DatabaseError> for String {
    fn from(err: DatabaseError) -> Self {
        err.to_string()
    }
}
