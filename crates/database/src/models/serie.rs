use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ids::*;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Serie {
    #[sqlx(try_from = "Uuid")]
    pub id: SerieId,
    pub cover_url: String,
    #[sqlx(try_from = "Uuid")]
    pub serie_type_id: SerieTypeId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerieTitle {
    #[sqlx(try_from = "Uuid")]
    pub id: SerieTitleId,
    #[sqlx(try_from = "Uuid")]
    pub serie_id: SerieId,
    pub language: String,
    pub title: String,
    pub is_alternate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerieSynopsis {
    #[sqlx(try_from = "Uuid")]
    pub id: SerieSynopsisId,
    #[sqlx(try_from = "Uuid")]
    pub serie_id: SerieId,
    pub language: String,
    pub synopsis: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerieType {
    #[sqlx(try_from = "Uuid")]
    pub id: SerieTypeId,
    pub serie_type: String, // Store as String, convert to enum when needed
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Status {
    #[sqlx(try_from = "Uuid")]
    pub id: StatusId,
    pub status: String, // Store as String, convert to enum when needed
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Genre {
    #[sqlx(try_from = "Uuid")]
    pub id: GenreId,
    pub genre: String, // Store as String, convert to enum when needed
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Author {
    #[sqlx(try_from = "Uuid")]
    pub id: AuthorId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Artist {
    #[sqlx(try_from = "Uuid")]
    pub id: ArtistId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Source {
    #[sqlx(try_from = "String")]
    pub id: SourceId,
    pub name: String,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SerieSource {
    #[sqlx(try_from = "Uuid")]
    pub serie_id: SerieId,
    #[sqlx(try_from = "String")]
    pub source_id: SourceId,
    pub external_id: Option<String>,
    pub url: Option<String>,
    pub created_at: DateTime<Utc>,
}

// Helper struct for series with titles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerieWithTitles {
    #[serde(flatten)]
    pub serie: Serie,
    pub titles: Vec<SerieTitle>,
}

// Database row struct for series with titles query
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SerieWithTitlesRow {
    // Serie fields
    pub id: Uuid,
    pub cover_url: String,
    pub serie_type_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Titles as JSON array
    pub titles: sqlx::types::Json<Vec<SerieTitle>>,
}

impl From<SerieWithTitlesRow> for SerieWithTitles {
    fn from(row: SerieWithTitlesRow) -> Self {
        Self {
            serie: Serie {
                id: SerieId::from(row.id),
                cover_url: row.cover_url,
                serie_type_id: SerieTypeId::from(row.serie_type_id),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            titles: row.titles.0,
        }
    }
}

// Helper struct for complex queries with relations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerieWithRelations {
    pub serie: Serie,
    pub serie_type: SerieType,
    pub titles: Vec<SerieTitle>,
    pub synopsis: Vec<SerieSynopsis>,
    pub statuses: Vec<Status>,
    pub genres: Vec<Genre>,
    pub authors: Vec<Author>,
    pub artists: Vec<Artist>,
    pub sources: Vec<SerieSource>,
}

// Database row struct for series with all relations query
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SerieWithRelationsRow {
    // Serie fields
    pub id: Uuid,
    pub cover_url: String,
    pub serie_type_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Type fields
    pub type_id: Uuid,
    pub serie_type: String,
    // Aggregated relations as JSON arrays
    pub titles: sqlx::types::Json<Vec<SerieTitle>>,
    pub synopsis: sqlx::types::Json<Vec<SerieSynopsis>>,
    pub statuses: sqlx::types::Json<Vec<Status>>,
    pub genres: sqlx::types::Json<Vec<Genre>>,
    pub authors: sqlx::types::Json<Vec<Author>>,
    pub artists: sqlx::types::Json<Vec<Artist>>,
    pub sources: sqlx::types::Json<Vec<SerieSource>>,
}

impl From<SerieWithRelationsRow> for SerieWithRelations {
    fn from(row: SerieWithRelationsRow) -> Self {
        Self {
            serie: Serie {
                id: SerieId::from(row.id),
                cover_url: row.cover_url,
                serie_type_id: SerieTypeId::from(row.serie_type_id),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            serie_type: SerieType {
                id: SerieTypeId::from(row.type_id),
                serie_type: row.serie_type,
            },
            titles: row.titles.0,
            synopsis: row.synopsis.0,
            statuses: row.statuses.0,
            genres: row.genres.0,
            authors: row.authors.0,
            artists: row.artists.0,
            sources: row.sources.0,
        }
    }
}
