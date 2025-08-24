pub mod entities;
pub mod error;
pub mod repositories;

pub use error::DatabaseError;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database as SeaDatabase, DatabaseConnection};
use std::time::Duration;

use repositories::{auth_state::AuthStateRepository, user::UserRepository};

use crate::repositories::SerieRepository;

#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(10)
            .min_connections(1)
            .connect_timeout(Duration::from_secs(30))
            .sqlx_logging(false);

        let conn = SeaDatabase::connect(opt).await?;

        Ok(Self { conn })
    }

    pub fn from_connection(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    pub async fn new_with_config(
        database_url: &str,
        max_connections: u32,
        min_connections: u32,
    ) -> Result<Self, DatabaseError> {
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(max_connections)
            .min_connections(min_connections)
            .connect_timeout(Duration::from_secs(30))
            .sqlx_logging(false);

        let conn = SeaDatabase::connect(opt).await?;

        Ok(Self { conn })
    }

    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        Migrator::up(&self.conn, None).await?;
        tracing::info!("Database migrations completed");
        Ok(())
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        use sea_orm::{ConnectionTrait, Statement};
        let _ = self
            .conn
            .query_one(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT 1",
            ))
            .await?;
        Ok(())
    }

    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.conn.clone())
    }

    pub fn series(&self) -> SerieRepository {
        SerieRepository::new(self.conn.clone())
    }

    pub fn auth_states(&self) -> AuthStateRepository {
        AuthStateRepository::new(self.conn.clone())
    }
}
