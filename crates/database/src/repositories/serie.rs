use chrono::Utc;
use dokusho_core::sources::{Source, SourceSerie};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    DatabaseError,
    entities::{
        artists, authors, genres, prelude::*, serie_artists, serie_authors, serie_genres,
        serie_sources, serie_status, serie_synopsis, serie_titles, serie_types, series, sources,
        statuses,
    },
};

pub struct SerieRepository {
    conn: DatabaseConnection,
}

impl SerieRepository {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// Create or update a serie from source data
    pub async fn upsert(
        &self,
        source_serie: &SourceSerie,
        source: &Source,
    ) -> Result<series::Model, DatabaseError> {
        let txn = self.conn.begin().await?;

        // First, ensure the source exists in the database
        let source_entity = Sources::find_by_id(&source.source_information.id)
            .one(&txn)
            .await?;

        if let Some(existing_source) = source_entity {
            // Update source info if it exists
            let mut active_model: sources::ActiveModel = existing_source.into();
            active_model.name = Set(source.source_information.name.clone());
            active_model.url = Set(Some(source.source_information.url.to_string()));
            active_model.updated_at = Set(Utc::now().into());
            active_model.update(&txn).await?
        } else {
            // Create new source
            let new_source = sources::ActiveModel {
                id: Set(source.source_information.id.clone()),
                name: Set(source.source_information.name.clone()),
                url: Set(Some(source.source_information.url.to_string())),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            new_source.insert(&txn).await?
        };

        // Check if serie exists by looking for matching source and external_id
        let existing_serie = SerieSources::find()
            .filter(serie_sources::Column::SourceId.eq(&source.source_information.id))
            .filter(serie_sources::Column::ExternalId.eq(&source_serie.id))
            .one(&txn)
            .await?
            .map(|ss| ss.serie_id);

        // Get or create serie type
        let serie_type = SerieTypes::find()
            .filter(serie_types::Column::SerieType.eq(source_serie.serie_type.to_string()))
            .one(&txn)
            .await?;

        let serie_type_id = if let Some(st) = serie_type {
            st.id
        } else {
            let new_type = serie_types::ActiveModel {
                id: Set(Uuid::new_v4()),
                serie_type: Set(source_serie.serie_type.to_string()),
            };
            new_type.insert(&txn).await?.id
        };

        let serie_entity = if let Some(serie_id) = existing_serie {
            // Update existing serie
            let existing = Series::find_by_id(serie_id)
                .one(&txn)
                .await?
                .ok_or(DatabaseError::NotFound)?;

            let mut active_model: series::ActiveModel = existing.into();
            active_model.cover_url = Set(source_serie.cover.to_string());
            active_model.serie_type_id = Set(serie_type_id);
            active_model.updated_at = Set(Utc::now().into());
            active_model.update(&txn).await?
        } else {
            // Create new serie
            let serie = series::ActiveModel {
                id: Set(Uuid::new_v4()),
                cover_url: Set(source_serie.cover.to_string()),
                serie_type_id: Set(serie_type_id),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
            };
            serie.insert(&txn).await?
        };

        let serie_id = serie_entity.id;

        // Clear existing related data if updating
        if existing_serie.is_some() {
            // Delete existing titles
            SerieTitles::delete_many()
                .filter(serie_titles::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing synopsis
            SerieSynopsis::delete_many()
                .filter(serie_synopsis::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing status
            SerieStatus::delete_many()
                .filter(serie_status::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing sources
            SerieSources::delete_many()
                .filter(serie_sources::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing authors
            SerieAuthors::delete_many()
                .filter(serie_authors::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing artists
            SerieArtists::delete_many()
                .filter(serie_artists::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;

            // Delete existing genres
            SerieGenres::delete_many()
                .filter(serie_genres::Column::SerieId.eq(serie_id))
                .exec(&txn)
                .await?;
        }

        // Insert main titles
        for (lang, titles) in source_serie.title.iter() {
            for title in titles {
                let title_model = serie_titles::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    serie_id: Set(serie_id),
                    title: Set(title.clone()),
                    language: Set(format!("{:?}", lang)),
                    is_alternate: Set(false),
                };
                title_model.insert(&txn).await?;
            }
        }

        // Insert alternate titles
        for (lang, titles) in source_serie.alternates_titles.iter() {
            for title in titles {
                let title_model = serie_titles::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    serie_id: Set(serie_id),
                    title: Set(title.clone()),
                    language: Set(format!("{:?}", lang)),
                    is_alternate: Set(true),
                };
                title_model.insert(&txn).await?;
            }
        }

        // Insert synopsis
        for (lang, synopsis_texts) in source_serie.synopsis.iter() {
            let synopsis_model = serie_synopsis::ActiveModel {
                id: Set(Uuid::new_v4()),
                serie_id: Set(serie_id),
                synopsis: Set(synopsis_texts.clone()),
                language: Set(format!("{:?}", lang)),
            };
            synopsis_model.insert(&txn).await?;
        }

        // Insert statuses
        for status in &source_serie.status {
            let status_entity = Statuses::find()
                .filter(statuses::Column::Status.eq(status.to_string()))
                .one(&txn)
                .await?;

            let status_id = if let Some(s) = status_entity {
                s.id
            } else {
                let new_status = statuses::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    status: Set(status.to_string()),
                };
                new_status.insert(&txn).await?.id
            };

            let status_model = serie_status::ActiveModel {
                serie_id: Set(serie_id),
                status_id: Set(status_id),
            };
            status_model.insert(&txn).await?;
        }

        // Insert source relationship
        let source_model = serie_sources::ActiveModel {
            serie_id: Set(serie_id),
            source_id: Set(source.source_information.id.clone()),
            external_id: Set(Some(source_serie.id.clone())),
            url: Set(None), // URL can be generated when needed
            created_at: Set(Utc::now().into()),
        };
        source_model.insert(&txn).await?;

        // Insert authors
        for author_name in &source_serie.authors {
            let author_entity = Authors::find()
                .filter(authors::Column::Name.eq(author_name))
                .one(&txn)
                .await?;

            let author_id = if let Some(a) = author_entity {
                a.id
            } else {
                let new_author = authors::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    name: Set(author_name.clone()),
                };
                new_author.insert(&txn).await?.id
            };

            let author_model = serie_authors::ActiveModel {
                serie_id: Set(serie_id),
                author_id: Set(author_id),
            };
            author_model.insert(&txn).await?;
        }

        // Insert artists
        for artist_name in &source_serie.artists {
            let artist_entity = Artists::find()
                .filter(artists::Column::Name.eq(artist_name))
                .one(&txn)
                .await?;

            let artist_id = if let Some(a) = artist_entity {
                a.id
            } else {
                let new_artist = artists::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    name: Set(artist_name.clone()),
                };
                new_artist.insert(&txn).await?.id
            };

            let artist_model = serie_artists::ActiveModel {
                serie_id: Set(serie_id),
                artist_id: Set(artist_id),
            };
            artist_model.insert(&txn).await?;
        }

        // Insert genres
        for genre in &source_serie.genres {
            let genre_entity = Genres::find()
                .filter(genres::Column::Genre.eq(genre.to_string()))
                .one(&txn)
                .await?;

            let genre_id = if let Some(g) = genre_entity {
                g.id
            } else {
                let new_genre = genres::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    genre: Set(genre.to_string()),
                };
                new_genre.insert(&txn).await?.id
            };

            let genre_model = serie_genres::ActiveModel {
                serie_id: Set(serie_id),
                genre_id: Set(genre_id),
            };
            genre_model.insert(&txn).await?;
        }

        txn.commit().await?;

        Ok(serie_entity)
    }

    /// Find a serie by ID with all its related data
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SerieWithRelations>, DatabaseError> {
        let serie = Series::find_by_id(id).one(&self.conn).await?;

        if let Some(serie) = serie {
            let relations = self.load_serie_relations(&serie).await?;
            Ok(Some(SerieWithRelations {
                serie,
                titles: relations.titles,
                synopsis: relations.synopsis,
                status: relations.status,
                sources: relations.sources,
                authors: relations.authors,
                artists: relations.artists,
                genres: relations.genres,
                serie_type: relations.serie_type,
            }))
        } else {
            Ok(None)
        }
    }

    /// Find series by source and external ID
    pub async fn find_by_source_and_external_id(
        &self,
        source_id: &str,
        external_id: &str,
    ) -> Result<Option<series::Model>, DatabaseError> {
        let serie_source = SerieSources::find()
            .filter(serie_sources::Column::SourceId.eq(source_id))
            .filter(serie_sources::Column::ExternalId.eq(external_id))
            .one(&self.conn)
            .await?;

        if let Some(serie_source) = serie_source {
            let serie = Series::find_by_id(serie_source.serie_id)
                .one(&self.conn)
                .await?;
            Ok(serie)
        } else {
            Ok(None)
        }
    }

    /// Find many existing series for a source by a list of external ids (batch)
    pub async fn find_existing_by_source_and_external_ids(
        &self,
        source_id: &str,
        external_ids: &[String],
    ) -> Result<Vec<(String, Uuid)>, DatabaseError> {
        use crate::entities::prelude::*;
        use crate::entities::serie_sources;

        if external_ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = SerieSources::find()
            .filter(serie_sources::Column::SourceId.eq(source_id))
            .filter(serie_sources::Column::ExternalId.is_in(external_ids.iter().cloned().collect::<Vec<_>>()))
            .all(&self.conn)
            .await?;

        Ok(rows
            .into_iter()
            .filter_map(|r| r.external_id.map(|ext| (ext, r.serie_id)))
            .collect())
    }

    /// Search series by title
    pub async fn search_by_title(
        &self,
        query: &str,
        language: Option<&str>,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<SerieWithRelations>, DatabaseError> {
        let mut q = SerieTitles::find().filter(serie_titles::Column::Title.contains(query));

        if let Some(lang) = language {
            q = q.filter(serie_titles::Column::Language.eq(lang));
        }

        let serie_titles = q.limit(limit).offset(offset).all(&self.conn).await?;

        let serie_ids: Vec<Uuid> = serie_titles.iter().map(|st| st.serie_id).collect();

        let mut result = Vec::new();
        for serie_id in serie_ids {
            if let Some(serie_with_relations) = self.find_by_id(serie_id).await? {
                result.push(serie_with_relations);
            }
        }

        Ok(result)
    }

    /// Update serie cover URL
    pub async fn update_cover_url(
        &self,
        id: Uuid,
        cover_url: String,
    ) -> Result<series::Model, DatabaseError> {
        let serie = Series::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or(DatabaseError::NotFound)?;

        let mut active_model: series::ActiveModel = serie.into();
        active_model.cover_url = Set(cover_url);
        active_model.updated_at = Set(Utc::now().into());

        let updated_serie = active_model.update(&self.conn).await?;
        Ok(updated_serie)
    }

    /// Add or update a title for a serie
    pub async fn upsert_title(
        &self,
        serie_id: Uuid,
        title: String,
        language: String,
        is_alternate: bool,
    ) -> Result<serie_titles::Model, DatabaseError> {
        let existing = SerieTitles::find()
            .filter(serie_titles::Column::SerieId.eq(serie_id))
            .filter(serie_titles::Column::Language.eq(&language))
            .filter(serie_titles::Column::IsAlternate.eq(is_alternate))
            .one(&self.conn)
            .await?;

        let title_entity = if let Some(existing_title) = existing {
            let mut active_model: serie_titles::ActiveModel = existing_title.into();
            active_model.title = Set(title);
            active_model.update(&self.conn).await?
        } else {
            let new_title = serie_titles::ActiveModel {
                id: Set(Uuid::new_v4()),
                serie_id: Set(serie_id),
                title: Set(title),
                language: Set(language),
                is_alternate: Set(is_alternate),
            };
            new_title.insert(&self.conn).await?
        };

        Ok(title_entity)
    }

    /// Update serie status
    pub async fn update_status(
        &self,
        serie_id: Uuid,
        status_id: Uuid,
    ) -> Result<serie_status::Model, DatabaseError> {
        let existing = SerieStatus::find()
            .filter(serie_status::Column::SerieId.eq(serie_id))
            .one(&self.conn)
            .await?;

        let status_entity = if let Some(existing_status) = existing {
            let mut active_model: serie_status::ActiveModel = existing_status.into();
            active_model.status_id = Set(status_id);
            active_model.update(&self.conn).await?
        } else {
            let new_status = serie_status::ActiveModel {
                serie_id: Set(serie_id),
                status_id: Set(status_id),
            };
            new_status.insert(&self.conn).await?
        };

        Ok(status_entity)
    }

    /// Add a genre to a serie
    pub async fn add_genre(
        &self,
        serie_id: Uuid,
        genre_id: Uuid,
    ) -> Result<serie_genres::Model, DatabaseError> {
        // Check if already exists
        let existing = SerieGenres::find()
            .filter(serie_genres::Column::SerieId.eq(serie_id))
            .filter(serie_genres::Column::GenreId.eq(genre_id))
            .one(&self.conn)
            .await?;

        if let Some(existing) = existing {
            return Ok(existing);
        }

        let genre_model = serie_genres::ActiveModel {
            serie_id: Set(serie_id),
            genre_id: Set(genre_id),
        };

        let genre_entity = genre_model.insert(&self.conn).await?;
        Ok(genre_entity)
    }

    /// Remove a genre from a serie
    pub async fn remove_genre(
        &self,
        serie_id: Uuid,
        genre_id: Uuid,
    ) -> Result<bool, DatabaseError> {
        let result = SerieGenres::delete_many()
            .filter(serie_genres::Column::SerieId.eq(serie_id))
            .filter(serie_genres::Column::GenreId.eq(genre_id))
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected > 0)
    }

    /// Add a source to a serie
    pub async fn add_source(
        &self,
        serie_id: Uuid,
        source_id: String,
        external_id: Option<String>,
        url: Option<String>,
    ) -> Result<serie_sources::Model, DatabaseError> {
        // Check if already exists
        let existing = SerieSources::find()
            .filter(serie_sources::Column::SerieId.eq(serie_id))
            .filter(serie_sources::Column::SourceId.eq(&source_id))
            .one(&self.conn)
            .await?;

        if let Some(existing) = existing {
            // Update if exists
            let mut active_model: serie_sources::ActiveModel = existing.into();
            if external_id.is_some() {
                active_model.external_id = Set(external_id);
            }
            if url.is_some() {
                active_model.url = Set(url);
            }
            return Ok(active_model.update(&self.conn).await?);
        }

        let source_model = serie_sources::ActiveModel {
            serie_id: Set(serie_id),
            source_id: Set(source_id),
            external_id: Set(external_id),
            url: Set(url),
            created_at: Set(Utc::now().into()),
        };

        let source_entity = source_model.insert(&self.conn).await?;
        Ok(source_entity)
    }

    /// Delete a serie and all its related data
    pub async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let txn = self.conn.begin().await?;

        // Delete all related data
        SerieTitles::delete_many()
            .filter(serie_titles::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieSynopsis::delete_many()
            .filter(serie_synopsis::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieStatus::delete_many()
            .filter(serie_status::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieSources::delete_many()
            .filter(serie_sources::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieAuthors::delete_many()
            .filter(serie_authors::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieArtists::delete_many()
            .filter(serie_artists::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        SerieGenres::delete_many()
            .filter(serie_genres::Column::SerieId.eq(id))
            .exec(&txn)
            .await?;

        // Delete the serie itself
        let result = Series::delete_by_id(id).exec(&txn).await?;

        txn.commit().await?;

        Ok(result.rows_affected > 0)
    }

    /// List all series with pagination
    pub async fn list(&self, limit: u64, offset: u64) -> Result<Vec<series::Model>, DatabaseError> {
        // Stable ordering for pagination: updated_at DESC, then id DESC as a tiebreaker
        let series = Series::find()
            .order_by_desc(series::Column::UpdatedAt)
            .order_by_desc(series::Column::Id)
            .limit(limit)
            .offset(offset)
            .all(&self.conn)
            .await?;

        Ok(series)
    }

    /// List series with their titles preloaded in one go (avoids N+1)
    pub async fn list_with_titles(
        &self,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<(series::Model, Vec<serie_titles::Model>)>, DatabaseError> {
        use crate::entities::prelude::SerieTitles;

        let rows = Series::find()
            .order_by_desc(series::Column::UpdatedAt)
            .order_by_desc(series::Column::Id)
            .limit(limit)
            .offset(offset)
            .find_with_related(SerieTitles)
            .all(&self.conn)
            .await?;

        Ok(rows)
    }

    /// Get series by genre
    pub async fn find_by_genre(
        &self,
        genre_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<series::Model>, DatabaseError> {
        let serie_genres = SerieGenres::find()
            .filter(serie_genres::Column::GenreId.eq(genre_id))
            .limit(limit)
            .offset(offset)
            .all(&self.conn)
            .await?;

        let serie_ids: Vec<Uuid> = serie_genres.iter().map(|sg| sg.serie_id).collect();

        let series = Series::find()
            .filter(series::Column::Id.is_in(serie_ids))
            .all(&self.conn)
            .await?;

        Ok(series)
    }

    /// Get series by author
    pub async fn find_by_author(
        &self,
        author_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<series::Model>, DatabaseError> {
        let serie_authors = SerieAuthors::find()
            .filter(serie_authors::Column::AuthorId.eq(author_id))
            .limit(limit)
            .offset(offset)
            .all(&self.conn)
            .await?;

        let serie_ids: Vec<Uuid> = serie_authors.iter().map(|sa| sa.serie_id).collect();

        let series = Series::find()
            .filter(series::Column::Id.is_in(serie_ids))
            .all(&self.conn)
            .await?;

        Ok(series)
    }

    /// Get series by type
    pub async fn find_by_type(
        &self,
        serie_type_id: Uuid,
        limit: u64,
        offset: u64,
    ) -> Result<Vec<series::Model>, DatabaseError> {
        let series = Series::find()
            .filter(series::Column::SerieTypeId.eq(serie_type_id))
            .order_by_desc(series::Column::UpdatedAt)
            .limit(limit)
            .offset(offset)
            .all(&self.conn)
            .await?;

        Ok(series)
    }

    /// Helper to load all relations for a serie
    async fn load_serie_relations(
        &self,
        serie: &series::Model,
    ) -> Result<SerieRelations, DatabaseError> {
        let titles = serie.find_related(SerieTitles).all(&self.conn).await?;
        let synopsis = serie.find_related(SerieSynopsis).all(&self.conn).await?;

        let status =
            if let Some(serie_status) = serie.find_related(SerieStatus).one(&self.conn).await? {
                Statuses::find_by_id(serie_status.status_id)
                    .one(&self.conn)
                    .await?
            } else {
                None
            };

        let serie_sources = serie.find_related(SerieSources).all(&self.conn).await?;
        let source_ids: Vec<String> = serie_sources.iter().map(|s| s.source_id.clone()).collect();
        let sources = Sources::find()
            .filter(sources::Column::Id.is_in(source_ids))
            .all(&self.conn)
            .await?;

        let serie_authors = serie.find_related(SerieAuthors).all(&self.conn).await?;
        let author_ids: Vec<Uuid> = serie_authors.iter().map(|a| a.author_id).collect();
        let authors = Authors::find()
            .filter(authors::Column::Id.is_in(author_ids))
            .all(&self.conn)
            .await?;

        let serie_artists = serie.find_related(SerieArtists).all(&self.conn).await?;
        let artist_ids: Vec<Uuid> = serie_artists.iter().map(|a| a.artist_id).collect();
        let artists = Artists::find()
            .filter(artists::Column::Id.is_in(artist_ids))
            .all(&self.conn)
            .await?;

        let serie_genres = serie.find_related(SerieGenres).all(&self.conn).await?;
        let genre_ids: Vec<Uuid> = serie_genres.iter().map(|g| g.genre_id).collect();
        let genres = Genres::find()
            .filter(genres::Column::Id.is_in(genre_ids))
            .all(&self.conn)
            .await?;

        let serie_type = SerieTypes::find_by_id(serie.serie_type_id)
            .one(&self.conn)
            .await?;

        Ok(SerieRelations {
            titles,
            synopsis,
            status,
            sources,
            authors,
            artists,
            genres,
            serie_type,
        })
    }
}

/// Structure to hold a serie with all its relations
#[derive(Debug, Clone)]
pub struct SerieWithRelations {
    pub serie: series::Model,
    pub titles: Vec<serie_titles::Model>,
    pub synopsis: Vec<serie_synopsis::Model>,
    pub status: Option<statuses::Model>,
    pub sources: Vec<sources::Model>,
    pub authors: Vec<authors::Model>,
    pub artists: Vec<artists::Model>,
    pub genres: Vec<genres::Model>,
    pub serie_type: Option<serie_types::Model>,
}

/// Internal structure for loading relations
struct SerieRelations {
    titles: Vec<serie_titles::Model>,
    synopsis: Vec<serie_synopsis::Model>,
    status: Option<statuses::Model>,
    sources: Vec<sources::Model>,
    authors: Vec<authors::Model>,
    artists: Vec<artists::Model>,
    genres: Vec<genres::Model>,
    serie_type: Option<serie_types::Model>,
}
