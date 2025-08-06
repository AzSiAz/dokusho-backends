use crate::DatabaseError;
use sqlx::{migrate::Migrator, PgPool};

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub async fn run(pool: &PgPool) -> Result<(), DatabaseError> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub async fn info(_pool: &PgPool) -> Result<Vec<String>, DatabaseError> {
    let migrations = MIGRATOR
        .iter()
        .map(|m| format!("{}: {}", m.version, m.description))
        .collect();
    Ok(migrations)
}
