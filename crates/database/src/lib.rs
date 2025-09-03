pub mod error;
pub mod models;
pub mod repositories;

pub use error::DatabaseError;

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

use repositories::user::UserRepository;

use crate::repositories::SerieRepository;
use chrono::Utc;
use dokusho_core::sources::{
    SourceInformation, SourceSerieGenre, SourceSerieStatus, SourceSerieType,
};
use strum::IntoEnumIterator;
use uuid::Uuid;

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
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        tracing::info!("Database migrations completed");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        Ok(())
    }

    pub fn users(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn series(&self) -> SerieRepository {
        SerieRepository::new(self.pool.clone())
    }

    pub async fn cleanup(&self) -> Result<(), DatabaseError> {
        // No-op: sessions and auth states removed in pure resource server mode
        Ok(())
    }

    /// Upsert static lookup data on startup
    /// - serie_types from `SourceSerieType`
    /// - statuses from `SourceSerieStatus`
    /// - genres from `SourceSerieGenre`
    /// - sources from provided `SourceInformation` list
    pub async fn upsert_static_data(
        &self,
        source_infos: Vec<SourceInformation>,
    ) -> Result<(), DatabaseError> {
        let mut txn = self
            .pool
            .begin()
            .await
            .map_err(|e| DatabaseError::Transaction(e.to_string()))?;

        tracing::info!("Upserting serie types");
        for serie_type in SourceSerieType::iter() {
            sqlx::query!(
                r#"
                INSERT INTO serie_types (id, serie_type) 
                VALUES ($1, $2)
                ON CONFLICT (serie_type) DO NOTHING
                "#,
                Uuid::new_v4(),
                serie_type.to_string()
            )
            .execute(&mut *txn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        tracing::info!("Upserting statuses");
        for status in SourceSerieStatus::iter() {
            sqlx::query!(
                r#"
                INSERT INTO statuses (id, status) 
                VALUES ($1, $2)
                ON CONFLICT (status) DO NOTHING
                "#,
                Uuid::new_v4(),
                status.to_string()
            )
            .execute(&mut *txn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        tracing::info!("Upserting genres");
        for genre in SourceSerieGenre::iter() {
            sqlx::query!(
                r#"
                INSERT INTO genres (id, genre) 
                VALUES ($1, $2)
                ON CONFLICT (genre) DO NOTHING
                "#,
                Uuid::new_v4(),
                genre.to_string()
            )
            .execute(&mut *txn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        tracing::info!("Upserting sources");
        for s in source_infos {
            sqlx::query!(
                r#"
                INSERT INTO sources (id, name, url, created_at, updated_at) 
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (id) 
                DO UPDATE SET 
                    name = EXCLUDED.name,
                    url = EXCLUDED.url,
                    updated_at = EXCLUDED.updated_at
                "#,
                s.id,
                s.name,
                s.url.to_string(),
                Utc::now(),
                Utc::now()
            )
            .execute(&mut *txn)
            .await
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        }

        txn.commit()
            .await
            .map_err(|e| DatabaseError::Transaction(e.to_string()))?;
        Ok(())
    }
}
