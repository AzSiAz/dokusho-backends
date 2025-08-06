pub mod error;
pub mod migrations;
pub mod models;
pub mod repositories;

pub use error::DatabaseError;

use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

use repositories::{auth_state::AuthStateRepository, cache::CacheRepository, user::UserRepository};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn new_with_config(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> Result<Self, DatabaseError> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        migrations::run(&self.pool).await?;
        tracing::info!("Database migrations completed");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1").fetch_one(&self.pool).await?;
        Ok(())
    }

    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn auth_states(&self) -> AuthStateRepository {
        AuthStateRepository::new(self.pool.clone())
    }

    pub fn cache(&self) -> CacheRepository {
        CacheRepository::new(self.pool.clone())
    }
}
