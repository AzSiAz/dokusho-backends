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

impl From<sea_orm::DbErr> for DatabaseError {
    fn from(err: sea_orm::DbErr) -> Self {
        use sea_orm::DbErr;
        match err {
            DbErr::RecordNotFound(_) => DatabaseError::NotFound,
            DbErr::Conn(msg) => DatabaseError::Connection(msg.to_string()),
            DbErr::Exec(msg) => DatabaseError::Query(msg.to_string()),
            DbErr::Query(msg) => DatabaseError::Query(msg.to_string()),
            DbErr::Migration(msg) => DatabaseError::Migration(msg),
            _ => DatabaseError::Query(err.to_string()),
        }
    }
}
