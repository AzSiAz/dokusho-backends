use chrono::{DateTime, FixedOffset};
use dokusho_core::{FetchSearchSerieFilter, FetchSearchSerieFilterGenres, SourceInformation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

// Re-export core types with ToSchema derive

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SourceResponse {
    pub id: String,
    pub name: String,
    pub url: String,
    pub icon: String,
    pub languages: Vec<String>,
    pub enabled_languages: Vec<String>,
    pub updated_at: DateTime<FixedOffset>,
    pub version: String,
    pub include_nsfw: bool,
}

impl From<SourceInformation> for SourceResponse {
    fn from(value: SourceInformation) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon: value.icon.to_string(),
            enabled_languages: value
                .enabled_languages
                .into_iter()
                .map(|l| format!("{:?}", l))
                .collect(),
            languages: value
                .languages
                .into_iter()
                .map(|l| format!("{:?}", l))
                .collect(),
            updated_at: value.updated_at,
            version: value.version,
            include_nsfw: value.include_nsfw,
            url: value.url.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SmallSerieResponse {
    pub id: String,
    pub title: HashMap<String, String>,
    pub cover: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PaginatedSmallSerieResponse {
    pub has_next_page: bool,
    pub series: Vec<SmallSerieResponse>,
}

impl From<dokusho_core::SourcePaginatedSmallSerie> for PaginatedSmallSerieResponse {
    fn from(value: dokusho_core::SourcePaginatedSmallSerie) -> Self {
        Self {
            has_next_page: value.has_next_page,
            series: value
                .series
                .into_iter()
                .map(|s| SmallSerieResponse {
                    id: s.id,
                    title: s.title.into_hashmap(),
                    cover: s.cover.to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SerieResponse {
    pub id: String,
    pub title: HashMap<String, String>,
    pub alternates_titles: HashMap<String, String>,
    pub cover: String,
    pub synopsis: HashMap<String, String>,
    pub status: Vec<String>,
    pub serie_type: String,
    pub genres: Vec<String>,
    pub authors: Vec<String>,
    pub artists: Vec<String>,
}

impl From<dokusho_core::SourceSerie> for SerieResponse {
    fn from(value: dokusho_core::SourceSerie) -> Self {
        Self {
            id: value.id,
            title: value.title.into_hashmap(),
            alternates_titles: value.alternates_titles.into_hashmap(),
            cover: value.cover.to_string(),
            synopsis: value.synopsis.into_hashmap(),
            status: value
                .status
                .into_iter()
                .map(|s| format!("{:?}", s))
                .collect(),
            serie_type: format!("{:?}", value.serie_type),
            genres: value
                .genres
                .into_iter()
                .map(|g| format!("{:?}", g))
                .collect(),
            authors: value.authors,
            artists: value.artists,
        }
    }
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SearchSerieRequest {
    pub query: Option<String>,
    pub order: Option<String>,
    pub sort: Option<String>,
    pub artists: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
    pub genres: Option<SearchSerieGenresFilter>,
    pub status: Option<Vec<String>>,
    #[serde(default = "default_page")]
    pub page: i16,
}

fn default_page() -> i16 {
    1
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SearchSerieGenresFilter {
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

impl From<SearchSerieRequest> for FetchSearchSerieFilter {
    fn from(req: SearchSerieRequest) -> Self {
        use dokusho_core::{
            FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort, SourceSerieStatus,
            SourceSerieType,
        };

        Self {
            query: req.query,
            order: req.order.and_then(|s| match s.as_str() {
                "ASC" => Some(FetchSearchSerieFilterOrder::ASC),
                "DESC" => Some(FetchSearchSerieFilterOrder::DESC),
                _ => None,
            }),
            sort: req.sort.and_then(|s| match s.as_str() {
                "Latest" => Some(FetchSearchSerieFilterSort::Latest),
                "Popularity" => Some(FetchSearchSerieFilterSort::Popularity),
                "Relevance" => Some(FetchSearchSerieFilterSort::Relevance),
                "Alphabetic" => Some(FetchSearchSerieFilterSort::Alphabetic),
                _ => None,
            }),
            artists: req.artists,
            authors: req.authors,
            types: req.types.map(|types| {
                types
                    .into_iter()
                    .filter_map(|t| match t.as_str() {
                        "Manga" => Some(SourceSerieType::Manga),
                        "Manhwa" => Some(SourceSerieType::Manhwa),
                        "Manhua" => Some(SourceSerieType::Manhua),
                        "Webtoon" => Some(SourceSerieType::Webtoon),
                        "Lightnovel" | "Light Novel" => Some(SourceSerieType::Lightnovel),
                        "Novel" => Some(SourceSerieType::Novel),
                        "Doujinshi" => Some(SourceSerieType::Doujinshi),
                        _ => None,
                    })
                    .collect()
            }),
            genres: req.genres.map(|_g| FetchSearchSerieFilterGenres {
                includes: None, // For simplicity, we'll skip genre parsing for now
                excludes: None,
            }),
            status: req.status.map(|statuses| {
                statuses
                    .into_iter()
                    .filter_map(|s| match s.as_str() {
                        "Ongoing" => Some(SourceSerieStatus::Ongoing),
                        "Completed" => Some(SourceSerieStatus::Completed),
                        "Hiatus" => Some(SourceSerieStatus::Hiatus),
                        "Canceled" => Some(SourceSerieStatus::Canceled),
                        "Publishing" => Some(SourceSerieStatus::Publishing),
                        _ => None,
                    })
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SerieChapterResponse {
    pub id: String,
    pub title: String,
    pub chapter_number: f64,
    pub volume_number: Option<f64>,
    pub volume_name: Option<String>,
    pub language: String,
    pub date_upload: DateTime<FixedOffset>,
    pub external_url: Option<String>,
}

impl From<dokusho_core::SourceSerieChapter> for SerieChapterResponse {
    fn from(value: dokusho_core::SourceSerieChapter) -> Self {
        Self {
            id: value.id,
            title: value.title,
            chapter_number: value.chapter_number,
            volume_number: value.volume_number,
            volume_name: value.volume_name,
            language: format!("{:?}", value.language),
            date_upload: value.date_upload,
            external_url: value.external_url.map(|u| u.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChaptersResponse {
    pub missing_chapters: Vec<f64>,
    pub chapters: Vec<SerieChapterResponse>,
}

impl From<dokusho_core::SourceChapters> for ChaptersResponse {
    fn from(value: dokusho_core::SourceChapters) -> Self {
        Self {
            missing_chapters: value.missing_chapters,
            chapters: value.chapters.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChapterImageResponse {
    pub index: i16,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ChapterTextResponse {
    pub index: i16,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChapterDataResponse {
    Text { texts: Vec<ChapterTextResponse> },
    Image { images: Vec<ChapterImageResponse> },
}

impl From<dokusho_core::SourceSerieChapterData> for ChapterDataResponse {
    fn from(value: dokusho_core::SourceSerieChapterData) -> Self {
        match value {
            dokusho_core::SourceSerieChapterData::Text(texts) => ChapterDataResponse::Text {
                texts: texts
                    .into_iter()
                    .map(|t| ChapterTextResponse {
                        index: t.index,
                        text: t.text,
                    })
                    .collect(),
            },
            dokusho_core::SourceSerieChapterData::Image(images) => ChapterDataResponse::Image {
                images: images
                    .into_iter()
                    .map(|i| ChapterImageResponse {
                        index: i.index,
                        url: i.url.to_string(),
                    })
                    .collect(),
            },
        }
    }
}
