use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Not found")]
    NotFound,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DatabaseError::NotFound,
            sqlx::Error::Database(db_err) => {
                if let Some(constraint) = db_err.constraint() {
                    DatabaseError::ConstraintViolation(constraint.to_string())
                } else {
                    DatabaseError::Query(db_err.to_string())
                }
            }
            sqlx::Error::PoolTimedOut => {
                DatabaseError::Connection("Connection pool timeout".to_string())
            }
            sqlx::Error::PoolClosed => {
                DatabaseError::Connection("Connection pool closed".to_string())
            }
            sqlx::Error::Migrate(source) => DatabaseError::Migration(source.to_string()),
            _ => DatabaseError::Query(err.to_string()),
        }
    }
}

impl From<sqlx::migrate::MigrateError> for DatabaseError {
    fn from(err: sqlx::migrate::MigrateError) -> Self {
        DatabaseError::Migration(err.to_string())
    }
}
