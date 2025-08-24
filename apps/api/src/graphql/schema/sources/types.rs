use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use dokusho_core::{
    FetchSearchSerieFilter, FetchSearchSerieFilterGenres, FetchSearchSerieFilterOrder,
    FetchSearchSerieFilterSort, MultiLanguageString, SourceChapters, SourceId, SourceInformation,
    SourceLanguage, SourcePaginatedSmallSerie, SourceSerie, SourceSerieChapter,
    SourceSerieChapterId, SourceSerieChapterData, SourceSerieChapterImage, SourceSerieChapterText,
    SourceSerieGenre, SourceSerieId, SourceSerieStatus, SourceSerieType,
    SourceSmallSerie, SupportedFilters,
};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSource {
    id: SourceId,
    pub name: String,
    pub url: String,
    pub icon: String,
    pub languages: Vec<SourceLanguage>,
    pub enabled_languages: Vec<SourceLanguage>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub include_nsfw: bool,
    pub filters: SupportedFilters,
}

impl From<SourceInformation> for GraphQLSource {
    fn from(value: SourceInformation) -> Self {
        Self {
            id: value.id,
            name: value.name,
            icon: value.icon.to_string(),
            enabled_languages: value.enabled_languages,
            languages: value.languages,
            updated_at: value.updated_at,
            version: value.version,
            include_nsfw: value.include_nsfw,
            url: value.url.to_string(),
            filters: value.search_filters,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSmallSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub cover: String,
}

impl From<SourceSmallSerie> for GraphQLSmallSerie {
    fn from(value: SourceSmallSerie) -> Self {
        Self {
            id: value.id,
            title: value.title,
            cover: value.cover.to_string(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLPaginatedSmallSerie {
    pub has_next_page: bool,
    pub series: Vec<GraphQLSmallSerie>,
}

impl From<SourcePaginatedSmallSerie> for GraphQLPaginatedSmallSerie {
    fn from(value: SourcePaginatedSmallSerie) -> Self {
        Self {
            has_next_page: value.has_next_page,
            series: value.series.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub alternates_titles: MultiLanguageString,
    pub cover: String,
    pub synopsis: MultiLanguageString,
    pub status: Vec<SourceSerieStatus>,
    pub serie_type: SourceSerieType,
    pub genres: Vec<SourceSerieGenre>,
    pub authors: Vec<String>,
    pub artists: Vec<String>,
}

impl From<SourceSerie> for GraphQLSerie {
    fn from(value: SourceSerie) -> Self {
        Self {
            id: value.id,
            title: value.title,
            alternates_titles: value.alternates_titles,
            cover: value.cover.to_string(),
            synopsis: value.synopsis,
            status: value.status,
            serie_type: value.serie_type,
            genres: value.genres,
            authors: value.authors,
            artists: value.artists,
        }
    }
}

#[derive(Debug, Clone, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLFetchSearchSerieFilterGenres {
    pub includes: Option<Vec<SourceSerieGenre>>,
    pub excludes: Option<Vec<SourceSerieGenre>>,
}

impl From<GraphQLFetchSearchSerieFilterGenres> for FetchSearchSerieFilterGenres {
    fn from(value: GraphQLFetchSearchSerieFilterGenres) -> Self {
        Self {
            includes: value.includes,
            excludes: value.excludes,
        }
    }
}

#[derive(Debug, Clone, InputObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLFetchSearchSerieFilter {
    pub query: Option<String>,
    pub order: Option<FetchSearchSerieFilterOrder>,
    pub sort: Option<FetchSearchSerieFilterSort>,
    pub artists: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub types: Option<Vec<SourceSerieType>>,
    pub genres: Option<GraphQLFetchSearchSerieFilterGenres>,
    pub status: Option<Vec<SourceSerieStatus>>,
}

impl From<GraphQLFetchSearchSerieFilter> for FetchSearchSerieFilter {
    fn from(value: GraphQLFetchSearchSerieFilter) -> Self {
        Self {
            query: value.query,
            order: value.order,
            sort: value.sort,
            artists: value.artists,
            authors: value.authors,
            types: value.types,
            genres: value.genres.map(Into::into),
            status: value.status,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerieChapter {
    pub id: SourceSerieChapterId,
    pub title: String,
    pub chapter_number: f64,
    pub volume_number: Option<f64>,
    pub volume_name: Option<String>,
    pub language: SourceLanguage,
    pub date_upload: DateTime<Utc>,
    pub external_url: Option<String>,
}

impl From<SourceSerieChapter> for GraphQLSerieChapter {
    fn from(value: SourceSerieChapter) -> Self {
        Self {
            id: value.id,
            title: value.title,
            chapter_number: value.chapter_number,
            volume_number: value.volume_number,
            volume_name: value.volume_name,
            language: value.language,
            date_upload: value.date_upload,
            external_url: value.external_url.map(|url| url.to_string()),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLChapters {
    pub missing_chapters: Vec<f64>,
    pub chapters: Vec<GraphQLSerieChapter>,
}

impl From<SourceChapters> for GraphQLChapters {
    fn from(value: SourceChapters) -> Self {
        Self {
            missing_chapters: value.missing_chapters,
            chapters: value.chapters.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerieChapterImage {
    pub index: i16,
    pub url: String,
}

impl From<SourceSerieChapterImage> for GraphQLSerieChapterImage {
    fn from(value: SourceSerieChapterImage) -> Self {
        Self {
            index: value.index,
            url: value.url.to_string(),
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerieChapterText {
    pub index: i16,
    pub text: String,
}

impl From<SourceSerieChapterText> for GraphQLSerieChapterText {
    fn from(value: SourceSerieChapterText) -> Self {
        Self {
            index: value.index,
            text: value.text,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerieChapterTextData {
    pub texts: Vec<GraphQLSerieChapterText>,
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct GraphQLSerieChapterImageData {
    pub images: Vec<GraphQLSerieChapterImage>,
}

#[derive(Debug, Clone, async_graphql::Union)]
pub enum GraphQLSerieChapterData {
    Text(GraphQLSerieChapterTextData),
    Image(GraphQLSerieChapterImageData),
}

impl From<SourceSerieChapterData> for GraphQLSerieChapterData {
    fn from(value: SourceSerieChapterData) -> Self {
        match value {
            SourceSerieChapterData::Text(texts) => {
                GraphQLSerieChapterData::Text(GraphQLSerieChapterTextData {
                    texts: texts.into_iter().map(Into::into).collect(),
                })
            }
            SourceSerieChapterData::Image(images) => {
                GraphQLSerieChapterData::Image(GraphQLSerieChapterImageData {
                    images: images.into_iter().map(Into::into).collect(),
                })
            }
        }
    }
}