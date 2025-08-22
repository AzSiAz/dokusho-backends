use std::collections::HashMap;

use async_graphql::SimpleObject;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::time::Duration;
use url::Url;

use crate::{
    FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort, MultiLanguageString, SourceError,
    SourceId, SourceLanguage, SourceSerieChapterId, SourceSerieGenre, SourceSerieId,
    SourceSerieStatus, SourceSerieType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSerieChapterImage {
    pub index: i16,
    pub url: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSerieChapterText {
    pub index: i16,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceSerieChapterData {
    #[serde(rename = "texts")]
    Text(Vec<SourceSerieChapterText>),
    #[serde(rename = "images")]
    Image(Vec<SourceSerieChapterImage>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSerieChapter {
    pub id: SourceSerieChapterId,
    pub title: String,
    pub chapter_number: f64,
    pub volume_number: Option<f64>,
    pub volume_name: Option<String>,
    pub language: SourceLanguage,
    pub date_upload: DateTime<Utc>,
    pub external_url: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub alternates_titles: MultiLanguageString,
    pub cover: Url,
    pub synopsis: MultiLanguageString,
    pub status: Vec<SourceSerieStatus>,
    pub serie_type: SourceSerieType,
    pub genres: Vec<SourceSerieGenre>,
    pub authors: Vec<String>,
    pub artists: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSmallSerie {
    pub id: SourceSerieId,
    pub title: MultiLanguageString,
    pub cover: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePaginatedSmallSerie {
    pub has_next_page: bool,
    pub series: Vec<SourceSmallSerie>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceChapters {
    pub missing_chapters: Vec<f64>,
    pub chapters: Vec<SourceSerieChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSearchSerieFilterGenres {
    pub includes: Option<Vec<SourceSerieGenre>>,
    pub excludes: Option<Vec<SourceSerieGenre>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FetchSearchSerieFilter {
    pub query: Option<String>,
    pub order: Option<FetchSearchSerieFilterOrder>,
    pub sort: Option<FetchSearchSerieFilterSort>,
    pub artists: Option<Vec<String>>,
    pub authors: Option<Vec<String>>,
    pub types: Option<Vec<SourceSerieType>>,
    pub genres: Option<FetchSearchSerieFilterGenres>,
    pub status: Option<Vec<SourceSerieStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SupportedFiltersGenres {
    pub include: bool,
    pub exclude: bool,
    pub accepted_values: Vec<SourceSerieGenre>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SupportedFilters {
    pub query: bool,
    pub order: Vec<FetchSearchSerieFilterOrder>,
    pub sort: Vec<FetchSearchSerieFilterSort>,
    pub artists: bool,
    pub authors: bool,
    pub types: Vec<SourceSerieType>,
    pub genres: SupportedFiltersGenres,
    pub status: Vec<SourceSerieStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInformation {
    pub id: SourceId,
    pub name: String,
    pub url: Url,
    pub icon: Url,
    pub languages: Vec<SourceLanguage>,
    pub enabled_languages: Vec<SourceLanguage>,
    pub updated_at: DateTime<Utc>,
    pub version: String,
    pub include_nsfw: bool,
    pub search_filters: SupportedFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceApiInformation {
    pub api_url: Url,
    pub headers: HashMap<String, String>,
    pub minimum_update_interval: i16,
    pub timeout: Duration,
    pub can_block_scraping: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(flatten)]
    pub source_information: SourceInformation,
    #[serde(flatten)]
    pub source_api_information: SourceApiInformation,
}

#[async_trait]
pub trait SourceApi: Send + Sync {
    fn get_information(&self) -> SourceInformation;
    fn get_api_information(&self) -> SourceApiInformation;
    fn serie_url(&self, serie_id: SourceSerieId) -> Result<Url, SourceError>;

    async fn fetch_popular_serie(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError>;
    async fn fetch_latest_updates(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError>;
    async fn fetch_search_serie(
        &self,
        page: i16,
        filters: FetchSearchSerieFilter,
    ) -> Result<SourcePaginatedSmallSerie, SourceError>;
    async fn fetch_serie_detail(&self, serie_id: SourceSerieId)
    -> Result<SourceSerie, SourceError>;
    async fn fetch_serie_chapters(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceChapters, SourceError>;
    async fn fetch_chapter_data(
        &self,
        serie_id: SourceSerieId,
        chapter_id: SourceSerieChapterId,
    ) -> Result<SourceSerieChapterData, SourceError>;
}
