use chrono::Utc;
use dokusho_core::sources::{Source, SourceSerie, SourceSerieGenre, SourceSerieStatus};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    DatabaseError,
    models::{
        ids::*,
        serie::{
            Artist, Author, Genre, Serie, SerieSource, SerieSynopsis, SerieTitle,
            SerieWithRelations, SerieWithRelationsRow, SerieWithTitles, SerieWithTitlesRow, Status,
        },
    },
};

pub struct SerieRepository {
    pool: PgPool,
}

impl SerieRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create or update a serie from source data
    pub async fn upsert(
        &self,
        source_serie: &SourceSerie,
        source: &Source,
    ) -> Result<Serie, DatabaseError> {
        let mut txn = self.pool.begin().await?;

        // First, ensure the source exists in the database
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
            source.source_information.id,
            source.source_information.name,
            source.source_information.url.to_string(),
            Utc::now(),
            Utc::now()
        )
        .execute(&mut *txn)
        .await?;

        // Check if serie exists by looking for matching source and external_id
        let existing_serie_id = sqlx::query_scalar!(
            r#"
            SELECT serie_id
            FROM serie_sources
            WHERE source_id = $1 AND external_id = $2
            "#,
            source.source_information.id,
            source_serie.id
        )
        .fetch_optional(&mut *txn)
        .await?;

        // Get or create serie type
        let serie_type_id = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO serie_types (serie_type)
                VALUES ($1)
                ON CONFLICT (serie_type) DO NOTHING
                RETURNING id
            )
            SELECT COALESCE(
                (SELECT id FROM ins),
                (SELECT id FROM serie_types WHERE serie_type = $1)
            )
            "#,
            source_serie.serie_type.to_string()
        )
        .fetch_one(&mut *txn)
        .await?;

        let serie = if let Some(serie_id) = existing_serie_id {
            // Update existing serie
            sqlx::query_as!(
                Serie,
                r#"
                UPDATE series
                SET cover_url = $2, serie_type_id = $3, updated_at = $4
                WHERE id = $1
                RETURNING id, cover_url, serie_type_id, created_at, updated_at
                "#,
                serie_id,
                source_serie.cover.to_string(),
                serie_type_id,
                Utc::now()
            )
            .fetch_one(&mut *txn)
            .await?
        } else {
            // Create new serie
            let new_serie = sqlx::query_as!(
                Serie,
                r#"
                INSERT INTO series (cover_url, serie_type_id, created_at, updated_at)
                VALUES ($1, $2, $3, $4)
                RETURNING id, cover_url, serie_type_id, created_at, updated_at
                "#,
                source_serie.cover.to_string(),
                serie_type_id,
                Utc::now(),
                Utc::now()
            )
            .fetch_one(&mut *txn)
            .await?;

            // Create serie_sources entry
            sqlx::query!(
                r#"
                INSERT INTO serie_sources (serie_id, source_id, external_id, url, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                *new_serie.id,
                source.source_information.id,
                source_serie.id,
                None::<String>, // SourceSerie doesn't have a url field
                Utc::now()
            )
            .execute(&mut *txn)
            .await?;

            new_serie
        };

        // Update serie titles
        let titles: Vec<(String, String, bool)> = source_serie
            .title
            .iter()
            .flat_map(|(lang, texts)| {
                texts
                    .iter()
                    .map(|text| (lang.to_string(), text.clone(), false))
                    .collect::<Vec<_>>()
            })
            .chain(
                source_serie
                    .alternates_titles
                    .iter()
                    .flat_map(|(lang, texts)| {
                        texts
                            .iter()
                            .map(|text| (lang.to_string(), text.clone(), true))
                            .collect::<Vec<_>>()
                    }),
            )
            .collect();

        if !titles.is_empty() {
            self.upsert_titles(&mut txn, serie.id, &titles).await?;
        }

        // Update serie synopsis
        let synopsis: Vec<(String, Vec<String>)> = source_serie
            .synopsis
            .iter()
            .map(|(lang, texts)| (lang.to_string(), texts.clone()))
            .collect();

        if !synopsis.is_empty() {
            self.upsert_synopsis(&mut txn, serie.id, &synopsis).await?;
        }

        // Update serie status
        if !source_serie.status.is_empty() {
            for status in &source_serie.status {
                self.upsert_status(&mut txn, serie.id, *status).await?;
            }
        }

        // Update serie genres
        if !source_serie.genres.is_empty() {
            self.upsert_genres(&mut txn, serie.id, &source_serie.genres)
                .await?;
        }

        // Update serie authors
        if !source_serie.authors.is_empty() {
            self.upsert_authors(&mut txn, serie.id, &source_serie.authors)
                .await?;
        }

        // Update serie artists
        if !source_serie.artists.is_empty() {
            self.upsert_artists(&mut txn, serie.id, &source_serie.artists)
                .await?;
        }

        txn.commit().await?;
        Ok(serie)
    }

    async fn upsert_titles(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        titles: &[(String, String, bool)], // (language, title, is_alternate)
    ) -> Result<(), DatabaseError> {
        // Clear existing titles for this serie
        sqlx::query!(
            r#"DELETE FROM serie_titles WHERE serie_id = $1"#,
            serie_id.0
        )
        .execute(&mut **txn)
        .await?;

        // Insert new titles
        for (lang, title, is_alt) in titles {
            sqlx::query!(
                r#"
                INSERT INTO serie_titles (serie_id, language, title, is_alternate)
                VALUES ($1, $2, $3, $4)
                "#,
                serie_id.0,
                lang,
                title,
                is_alt
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(())
    }

    async fn upsert_synopsis(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        synopsis_list: &[(String, Vec<String>)], // (language, synopsis paragraphs)
    ) -> Result<(), DatabaseError> {
        for (lang, synopsis) in synopsis_list {
            sqlx::query!(
                r#"
                INSERT INTO serie_synopsis (serie_id, language, synopsis)
                VALUES ($1, $2, $3)
                ON CONFLICT (serie_id, language)
                DO UPDATE SET synopsis = EXCLUDED.synopsis
                "#,
                serie_id.0,
                lang,
                synopsis as &[String]
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(())
    }

    async fn upsert_status(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        status: SourceSerieStatus,
    ) -> Result<(), DatabaseError> {
        // Get or create status
        let status_id = sqlx::query_scalar!(
            r#"
            WITH ins AS (
                INSERT INTO statuses (status)
                VALUES ($1)
                ON CONFLICT (status) DO NOTHING
                RETURNING id
            )
            SELECT COALESCE(
                (SELECT id FROM ins),
                (SELECT id FROM statuses WHERE status = $1)
            )
            "#,
            status.to_string()
        )
        .fetch_one(&mut **txn)
        .await?;
        let status_id = StatusId::from(status_id.expect("Status ID should always be returned"));

        // Clear existing status for this serie
        sqlx::query!(
            r#"DELETE FROM serie_status WHERE serie_id = $1"#,
            serie_id.0
        )
        .execute(&mut **txn)
        .await?;

        // Insert new status
        sqlx::query!(
            r#"
            INSERT INTO serie_status (serie_id, status_id)
            VALUES ($1, $2)
            "#,
            serie_id.0,
            status_id.0
        )
        .execute(&mut **txn)
        .await?;

        Ok(())
    }

    async fn upsert_genres(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        genres: &[SourceSerieGenre],
    ) -> Result<(), DatabaseError> {
        // Clear existing genres for this serie
        sqlx::query!(
            r#"DELETE FROM serie_genres WHERE serie_id = $1"#,
            serie_id.0
        )
        .execute(&mut **txn)
        .await?;

        for genre in genres {
            // Get or create genre
            let genre_id = sqlx::query_scalar!(
                r#"
                WITH ins AS (
                    INSERT INTO genres (genre)
                    VALUES ($1)
                    ON CONFLICT (genre) DO NOTHING
                    RETURNING id
                )
                SELECT COALESCE(
                    (SELECT id FROM ins),
                    (SELECT id FROM genres WHERE genre = $1)
                )
                "#,
                genre.to_string()
            )
            .fetch_one(&mut **txn)
            .await?;
            let genre_id = GenreId::from(genre_id.expect("Genre ID should always be returned"));

            // Insert serie_genre relation
            sqlx::query!(
                r#"
                INSERT INTO serie_genres (serie_id, genre_id)
                VALUES ($1, $2)
                "#,
                serie_id.0,
                genre_id.0
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(())
    }

    async fn upsert_authors(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        authors: &[String],
    ) -> Result<(), DatabaseError> {
        // Clear existing authors for this serie
        sqlx::query!(
            r#"DELETE FROM serie_authors WHERE serie_id = $1"#,
            serie_id.0
        )
        .execute(&mut **txn)
        .await?;

        for author in authors {
            // Get or create author
            let author_id = sqlx::query_scalar!(
                r#"
                WITH ins AS (
                    INSERT INTO authors (name)
                    VALUES ($1)
                    ON CONFLICT (name) DO NOTHING
                    RETURNING id
                )
                SELECT COALESCE(
                    (SELECT id FROM ins),
                    (SELECT id FROM authors WHERE name = $1)
                )
                "#,
                author
            )
            .fetch_one(&mut **txn)
            .await?;
            let author_id = AuthorId::from(author_id.expect("Author ID should always be returned"));

            // Insert serie_author relation
            sqlx::query!(
                r#"
                INSERT INTO serie_authors (serie_id, author_id)
                VALUES ($1, $2)
                "#,
                serie_id.0,
                author_id.0
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(())
    }

    async fn upsert_artists(
        &self,
        txn: &mut Transaction<'_, Postgres>,
        serie_id: SerieId,
        artists: &[String],
    ) -> Result<(), DatabaseError> {
        // Clear existing artists for this serie
        sqlx::query!(
            r#"DELETE FROM serie_artists WHERE serie_id = $1"#,
            serie_id.0
        )
        .execute(&mut **txn)
        .await?;

        for artist in artists {
            // Get or create artist
            let artist_id = sqlx::query_scalar!(
                r#"
                WITH ins AS (
                    INSERT INTO artists (name)
                    VALUES ($1)
                    ON CONFLICT (name) DO NOTHING
                    RETURNING id
                )
                SELECT COALESCE(
                    (SELECT id FROM ins),
                    (SELECT id FROM artists WHERE name = $1)
                )
                "#,
                artist
            )
            .fetch_one(&mut **txn)
            .await?;
            let artist_id = ArtistId::from(artist_id.expect("Artist ID should always be returned"));

            // Insert serie_artist relation
            sqlx::query!(
                r#"
                INSERT INTO serie_artists (serie_id, artist_id)
                VALUES ($1, $2)
                "#,
                serie_id.0,
                artist_id.0
            )
            .execute(&mut **txn)
            .await?;
        }

        Ok(())
    }

    /// Find a serie by its ID
    pub async fn find_by_id(&self, id: SerieId) -> Result<Option<Serie>, DatabaseError> {
        let serie = sqlx::query_as!(
            Serie,
            r#"
            SELECT id, cover_url, serie_type_id, created_at, updated_at
            FROM series
            WHERE id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(serie)
    }

    /// Find existing series by source and external IDs
    /// Returns a vector of tuples (external_id, serie_id)
    pub async fn find_existing_by_source_and_external_ids(
        &self,
        source_id: &str,
        external_ids: &[String],
    ) -> Result<Vec<(String, Uuid)>, DatabaseError> {
        if external_ids.is_empty() {
            return Ok(vec![]);
        }

        // Use ANY for efficient batch lookup
        let results = sqlx::query!(
            r#"
            SELECT external_id, serie_id
            FROM serie_sources
            WHERE source_id = $1 AND external_id = ANY($2)
            "#,
            source_id,
            external_ids
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .filter_map(|r| r.external_id.map(|ext_id| (ext_id, r.serie_id)))
            .collect())
    }

    /// List series with their titles for pagination using an efficient single query with JSON aggregation
    pub async fn list_with_titles(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<SerieWithTitles>, DatabaseError> {
        let rows = sqlx::query_as!(
            SerieWithTitlesRow,
            r#"
            SELECT
                s.id,
                s.cover_url,
                s.serie_type_id,
                s.created_at,
                s.updated_at,
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', t.id,
                            'serie_id', t.serie_id,
                            'language', t.language,
                            'title', t.title,
                            'is_alternate', t.is_alternate
                        ) ORDER BY t.is_alternate, t.language
                    )
                    FROM serie_titles t
                    WHERE t.serie_id = s.id),
                    '[]'::jsonb
                ) as "titles!: sqlx::types::Json<Vec<SerieTitle>>"
            FROM series s
            ORDER BY s.updated_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Find a serie with all its relations using a single optimized query with JSON aggregation
    pub async fn find_with_relations(
        &self,
        id: SerieId,
    ) -> Result<Option<SerieWithRelations>, DatabaseError> {
        let row = sqlx::query_as!(
            SerieWithRelationsRow,
            r#"
            SELECT
                s.id,
                s.cover_url,
                s.serie_type_id,
                s.created_at,
                s.updated_at,
                t.id as type_id,
                t.serie_type,
                -- Titles
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', st.id,
                            'serie_id', st.serie_id,
                            'language', st.language,
                            'title', st.title,
                            'is_alternate', st.is_alternate
                        ) ORDER BY st.is_alternate, st.language
                    )
                    FROM serie_titles st
                    WHERE st.serie_id = s.id),
                    '[]'::jsonb
                ) as "titles!: sqlx::types::Json<Vec<SerieTitle>>",
                -- Synopsis
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', syn.id,
                            'serie_id', syn.serie_id,
                            'language', syn.language,
                            'synopsis', syn.synopsis
                        ) ORDER BY syn.language
                    )
                    FROM serie_synopsis syn
                    WHERE syn.serie_id = s.id),
                    '[]'::jsonb
                ) as "synopsis!: sqlx::types::Json<Vec<SerieSynopsis>>",
                -- Statuses
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', st.id,
                            'status', st.status
                        )
                    )
                    FROM statuses st
                    INNER JOIN serie_status ss ON st.id = ss.status_id
                    WHERE ss.serie_id = s.id),
                    '[]'::jsonb
                ) as "statuses!: sqlx::types::Json<Vec<Status>>",
                -- Genres
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', g.id,
                            'genre', g.genre
                        ) ORDER BY g.genre
                    )
                    FROM genres g
                    INNER JOIN serie_genres sg ON g.id = sg.genre_id
                    WHERE sg.serie_id = s.id),
                    '[]'::jsonb
                ) as "genres!: sqlx::types::Json<Vec<Genre>>",
                -- Authors
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', a.id,
                            'name', a.name
                        ) ORDER BY a.name
                    )
                    FROM authors a
                    INNER JOIN serie_authors sa ON a.id = sa.author_id
                    WHERE sa.serie_id = s.id),
                    '[]'::jsonb
                ) as "authors!: sqlx::types::Json<Vec<Author>>",
                -- Artists
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'id', ar.id,
                            'name', ar.name
                        ) ORDER BY ar.name
                    )
                    FROM artists ar
                    INNER JOIN serie_artists sa ON ar.id = sa.artist_id
                    WHERE sa.serie_id = s.id),
                    '[]'::jsonb
                ) as "artists!: sqlx::types::Json<Vec<Artist>>",
                -- Sources
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            'serie_id', ss.serie_id,
                            'source_id', ss.source_id,
                            'external_id', ss.external_id,
                            'url', ss.url,
                            'created_at', ss.created_at
                        )
                    )
                    FROM serie_sources ss
                    WHERE ss.serie_id = s.id),
                    '[]'::jsonb
                ) as "sources!: sqlx::types::Json<Vec<SerieSource>>"
            FROM series s
            JOIN serie_types t ON s.serie_type_id = t.id
            WHERE s.id = $1
            "#,
            id.0
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }
}
