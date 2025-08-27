pub mod entities;
pub mod error;
pub mod repositories;

pub use error::DatabaseError;

use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database as SeaDatabase, DatabaseConnection};
use std::time::Duration;

use repositories::{auth_state::AuthStateRepository, user::UserRepository};

use crate::repositories::SerieRepository;
use chrono::Utc;
use dokusho_core::sources::{
    SourceInformation, SourceSerieGenre, SourceSerieStatus, SourceSerieType,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use strum::IntoEnumIterator;
use uuid::Uuid;

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

    pub async fn cleanup(&self) -> Result<(), DatabaseError> {
        let mut total_cleaned = 0;

        match self.users().delete_expired_sessions().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Cleaned up {} expired user sessions", count);
                    total_cleaned += count;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to clean up expired sessions: {}", e);
            }
        }

        match self.auth_states().delete_expired().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Cleaned up {} expired auth states", count);
                    total_cleaned += count;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to clean up expired auth states: {}", e);
            }
        }

        if total_cleaned > 0 {
            tracing::info!("Total cleanup: {} records removed", total_cleaned);
        }

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
        use crate::entities::{genres, prelude::*, serie_types, sources, statuses};

        let txn = self.conn.begin().await?;

        tracing::info!("Upserting serie types");
        for serie_type in SourceSerieType::iter() {
            let existing = SerieTypes::find()
                .filter(serie_types::Column::SerieType.eq(serie_type.to_string()))
                .one(&txn)
                .await?;

            if existing.is_none() {
                let model = serie_types::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    serie_type: Set(serie_type.to_string()),
                };
                let _ = model.insert(&txn).await?;
            }
        }

        tracing::info!("Upserting statuses");
        for status in SourceSerieStatus::iter() {
            let existing = Statuses::find()
                .filter(statuses::Column::Status.eq(status.to_string()))
                .one(&txn)
                .await?;

            if existing.is_none() {
                let model = statuses::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    status: Set(status.to_string()),
                };
                let _ = model.insert(&txn).await?;
            }
        }

        tracing::info!("Upserting genres");
        for genre in SourceSerieGenre::iter() {
            let existing = Genres::find()
                .filter(genres::Column::Genre.eq(genre.to_string()))
                .one(&txn)
                .await?;

            if existing.is_none() {
                let model = genres::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    genre: Set(genre.to_string()),
                };
                let _ = model.insert(&txn).await?;
            }
        }

        tracing::info!("Upserting sources");
        for s in source_infos {
            let existing = Sources::find_by_id(&s.id).one(&txn).await?;
            if let Some(existing) = existing {
                let mut am: sources::ActiveModel = existing.into();
                am.name = Set(s.name.clone());
                am.url = Set(Some(s.url.to_string()));
                am.updated_at = Set(Utc::now().into());
                let _ = am.update(&txn).await?;
            } else {
                let am = sources::ActiveModel {
                    id: Set(s.id.clone()),
                    name: Set(s.name.clone()),
                    url: Set(Some(s.url.to_string())),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                };
                let _ = am.insert(&txn).await?;
            }
        }

        txn.commit().await?;
        Ok(())
    }
}
