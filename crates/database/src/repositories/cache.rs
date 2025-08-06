use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::PgPool;

use crate::{
    models::{LatestSeriesCache, PopularSeriesCache, SeriesDetailCache},
    DatabaseError,
};

pub struct CacheRepository {
    pool: PgPool,
    cache_duration_hours: i64,
}

impl CacheRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cache_duration_hours: 1, // Default 1 hour cache
        }
    }

    pub fn with_duration(pool: PgPool, cache_duration_hours: i64) -> Self {
        Self {
            pool,
            cache_duration_hours,
        }
    }

    pub async fn get_popular_series(
        &self,
        source_id: &str,
        page: i32,
    ) -> Result<Option<PopularSeriesCache>, DatabaseError> {
        let cutoff = Utc::now() - Duration::hours(self.cache_duration_hours);

        let cache = sqlx::query_as!(
            PopularSeriesCache,
            r#"
            SELECT source_id, page, data, updated_at
            FROM popular_series_cache
            WHERE source_id = $1 AND page = $2 AND updated_at > $3
            "#,
            source_id,
            page,
            cutoff
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(cache)
    }

    pub async fn set_popular_series(
        &self,
        source_id: String,
        page: i32,
        data: Value,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO popular_series_cache (source_id, page, data, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (source_id, page) DO UPDATE
            SET data = $3, updated_at = NOW()
            "#,
            source_id,
            page,
            data
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_series(
        &self,
        source_id: &str,
        page: i32,
    ) -> Result<Option<LatestSeriesCache>, DatabaseError> {
        let cutoff = Utc::now() - Duration::hours(self.cache_duration_hours);

        let cache = sqlx::query_as!(
            LatestSeriesCache,
            r#"
            SELECT source_id, page, data, updated_at
            FROM latest_series_cache
            WHERE source_id = $1 AND page = $2 AND updated_at > $3
            "#,
            source_id,
            page,
            cutoff
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(cache)
    }

    pub async fn set_latest_series(
        &self,
        source_id: String,
        page: i32,
        data: Value,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO latest_series_cache (source_id, page, data, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (source_id, page) DO UPDATE
            SET data = $3, updated_at = NOW()
            "#,
            source_id,
            page,
            data
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_series_detail(
        &self,
        source_id: &str,
        series_id: &str,
    ) -> Result<Option<SeriesDetailCache>, DatabaseError> {
        let cutoff = Utc::now() - Duration::hours(self.cache_duration_hours * 24); // Longer cache for details

        let cache = sqlx::query_as!(
            SeriesDetailCache,
            r#"
            SELECT source_id, series_id, data, updated_at
            FROM series_detail_cache
            WHERE source_id = $1 AND series_id = $2 AND updated_at > $3
            "#,
            source_id,
            series_id,
            cutoff
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(cache)
    }

    pub async fn set_series_detail(
        &self,
        source_id: String,
        series_id: String,
        data: Value,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO series_detail_cache (source_id, series_id, data, updated_at)
            VALUES ($1, $2, $3, NOW())
            ON CONFLICT (source_id, series_id) DO UPDATE
            SET data = $3, updated_at = NOW()
            "#,
            source_id,
            series_id,
            data
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn clear_cache(&self, source_id: Option<&str>) -> Result<u64, DatabaseError> {
        let mut total_deleted = 0u64;

        if let Some(source_id) = source_id {
            let result = sqlx::query!(
                r#"DELETE FROM popular_series_cache WHERE source_id = $1"#,
                source_id
            )
            .execute(&self.pool)
            .await?;
            total_deleted += result.rows_affected();

            let result = sqlx::query!(
                r#"DELETE FROM latest_series_cache WHERE source_id = $1"#,
                source_id
            )
            .execute(&self.pool)
            .await?;
            total_deleted += result.rows_affected();

            let result = sqlx::query!(
                r#"DELETE FROM series_detail_cache WHERE source_id = $1"#,
                source_id
            )
            .execute(&self.pool)
            .await?;
            total_deleted += result.rows_affected();
        } else {
            let result = sqlx::query!(r#"DELETE FROM popular_series_cache"#)
                .execute(&self.pool)
                .await?;
            total_deleted += result.rows_affected();

            let result = sqlx::query!(r#"DELETE FROM latest_series_cache"#)
                .execute(&self.pool)
                .await?;
            total_deleted += result.rows_affected();

            let result = sqlx::query!(r#"DELETE FROM series_detail_cache"#)
                .execute(&self.pool)
                .await?;
            total_deleted += result.rows_affected();
        }

        Ok(total_deleted)
    }

    pub async fn clear_expired_cache(&self) -> Result<u64, DatabaseError> {
        let cutoff = Utc::now() - Duration::hours(self.cache_duration_hours * 24 * 7); // 7 days
        let mut total_deleted = 0u64;

        let result = sqlx::query!(
            r#"DELETE FROM popular_series_cache WHERE updated_at < $1"#,
            cutoff
        )
        .execute(&self.pool)
        .await?;
        total_deleted += result.rows_affected();

        let result = sqlx::query!(
            r#"DELETE FROM latest_series_cache WHERE updated_at < $1"#,
            cutoff
        )
        .execute(&self.pool)
        .await?;
        total_deleted += result.rows_affected();

        let result = sqlx::query!(
            r#"DELETE FROM series_detail_cache WHERE updated_at < $1"#,
            cutoff
        )
        .execute(&self.pool)
        .await?;
        total_deleted += result.rows_affected();

        Ok(total_deleted)
    }
}
