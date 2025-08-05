use async_graphql::SimpleObject;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ids::{ChapterId, SerieId, SourceId, VolumeId};
use super::enums::*;

// ============= Language Types =============

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct MultiLanguageString {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub en: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jp_ro: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ko: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zh_hk: Option<String>,
}

impl MultiLanguageString {
    pub fn new() -> Self {
        Self {
            en: None,
            jp: None,
            jp_ro: None,
            fr: None,
            ko: None,
            zh: None,
            zh_hk: None,
        }
    }

    pub fn with_language(mut self, lang: SourceLanguage, text: impl Into<String>) -> Self {
        match lang {
            SourceLanguage::En => self.en = Some(text.into()),
            SourceLanguage::Jp => self.jp = Some(text.into()),
            SourceLanguage::JpRo => self.jp_ro = Some(text.into()),
            SourceLanguage::Fr => self.fr = Some(text.into()),
            SourceLanguage::Ko => self.ko = Some(text.into()),
            SourceLanguage::Zh => self.zh = Some(text.into()),
            SourceLanguage::ZhHk => self.zh_hk = Some(text.into()),
        }
        self
    }

    pub fn get(&self, lang: SourceLanguage) -> Option<&str> {
        match lang {
            SourceLanguage::En => self.en.as_deref(),
            SourceLanguage::Jp => self.jp.as_deref(),
            SourceLanguage::JpRo => self.jp_ro.as_deref(),
            SourceLanguage::Fr => self.fr.as_deref(),
            SourceLanguage::Ko => self.ko.as_deref(),
            SourceLanguage::Zh => self.zh.as_deref(),
            SourceLanguage::ZhHk => self.zh_hk.as_deref(),
        }
    }

    pub fn get_any(&self) -> Option<&str> {
        self.en.as_deref()
            .or(self.jp.as_deref())
            .or(self.fr.as_deref())
            .or(self.ko.as_deref())
            .or(self.zh.as_deref())
            .or(self.zh_hk.as_deref())
            .or(self.jp_ro.as_deref())
    }
}

impl Default for MultiLanguageString {
    fn default() -> Self {
        Self::new()
    }
}

// ============= Chapter Data Types =============

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ChapterImage {
    pub index: i32,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct ChapterText {
    pub index: i32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(async_graphql::Enum))]
#[serde(rename_all = "lowercase")]
pub enum ChapterDataType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterData {
    #[serde(rename = "type")]
    pub data_type: ChapterDataType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ChapterImage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texts: Option<Vec<ChapterText>>,
}

impl ChapterData {
    pub fn from_images(images: Vec<ChapterImage>) -> Self {
        Self {
            data_type: ChapterDataType::Image,
            images: Some(images),
            texts: None,
        }
    }

    pub fn from_texts(texts: Vec<ChapterText>) -> Self {
        Self {
            data_type: ChapterDataType::Text,
            images: None,
            texts: Some(texts),
        }
    }
}

// ============= Chapter & Volume Types =============

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Chapter {
    pub id: ChapterId,
    pub name: String,
    #[serde(rename = "chapterNumber")]
    pub chapter_number: f64,
    pub language: SourceLanguage,
    #[serde(rename = "dateUpload")]
    pub date_upload: DateTime<Utc>,
    #[serde(rename = "externalURL", skip_serializing_if = "Option::is_none")]
    pub external_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Volume {
    pub id: VolumeId,
    pub name: String,
    #[serde(rename = "volumeNumber")]
    pub volume_number: f64,
    #[serde(rename = "missingChapters")]
    pub missing_chapters: Vec<f64>,
    pub chapters: Vec<Chapter>,
}

// ============= Serie Types =============

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct SmallSerie {
    pub id: SerieId,
    pub title: MultiLanguageString,
    pub cover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct Serie {
    pub id: SerieId,
    pub title: MultiLanguageString,
    #[serde(rename = "alternativeTitles", skip_serializing_if = "Option::is_none")]
    pub alternative_titles: Option<Vec<MultiLanguageString>>,
    pub cover: String,
    pub synopsis: MultiLanguageString,
    #[serde(rename = "type")]
    pub serie_type: SourceSerieType,
    pub genres: Vec<SourceSerieGenre>,
    pub status: Vec<SourceSerieStatus>,
    pub authors: Vec<String>,
    pub artists: Vec<String>,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct PaginatedSmallSeries {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    pub series: Vec<SmallSerie>,
}

// ============= Filter Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilterGenres {
    pub include: Vec<SourceSerieGenre>,
    pub exclude: Vec<SourceSerieGenre>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub query: String,
    pub order: FilterOrder,
    pub sort: FilterSort,
    pub artists: Vec<String>,
    pub authors: Vec<String>,
    pub genres: SearchFilterGenres,
    pub types: Vec<SourceSerieType>,
    pub status: Vec<SourceSerieStatus>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            query: String::new(),
            order: FilterOrder::Descending,
            sort: FilterSort::UpdatedAt,
            artists: Vec::new(),
            authors: Vec::new(),
            genres: SearchFilterGenres {
                include: Vec::new(),
                exclude: Vec::new(),
            },
            types: Vec::new(),
            status: Vec::new(),
        }
    }
}

// ============= Source Information Types =============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFiltersGenres {
    pub included: bool,
    pub excluded: bool,
    #[serde(rename = "values")]
    pub possible_values: Vec<SourceSerieGenre>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportedFilters {
    pub query: bool,
    pub orders: Vec<FilterOrder>,
    pub sorts: Vec<FilterSort>,
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
    pub url: String,
    pub icon: String,
    pub languages: Vec<SourceLanguage>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub version: String,
    #[serde(rename = "nsfw")]
    pub nsfw: bool,
    #[serde(rename = "supportedFilters")]
    pub search_filters: SupportedFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceApiInformation {
    #[serde(rename = "apiURL", skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
    #[serde(rename = "minimumUpdateInterval")]
    pub minimum_update_interval: std::time::Duration,
    pub timeout: std::time::Duration,
    #[serde(rename = "canBlockScraping")]
    pub can_block_scraping: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    #[serde(flatten)]
    pub source_information: SourceInformation,
    #[serde(flatten)]
    pub source_api_information: SourceApiInformation,
}

// ============= GraphQL-only Types =============
// These are simplified types for GraphQL responses that don't need all the complexity

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
pub struct GraphQLSource {
    pub name: String,
    pub version: String,
    pub icon: String,
}