use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use dokusho_core::{
    FetchSearchSerieFilter, FetchSearchSerieFilterGenres, FetchSearchSerieFilterOrder,
    FetchSearchSerieFilterSort, MultiLanguageString, SourceId, SourceInformation, SourceLanguage,
    SourcePaginatedSmallSerie, SourceSerieGenre, SourceSerieId, SourceSerieStatus,
    SourceSerieType, SourceSmallSerie, SupportedFilters,
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