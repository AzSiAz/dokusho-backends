use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::enums::{Language, SerieStatus, SerieType};
use super::ids::{ChapterId, GenreId, SerieId, SourceId, VolumeId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MultiLanguageString(pub HashMap<Language, String>);

impl MultiLanguageString {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_language(mut self, lang: Language, text: impl Into<String>) -> Self {
        self.0.insert(lang, text.into());
        self
    }

    pub fn get(&self, lang: Language) -> Option<&str> {
        self.0.get(&lang).map(|s| s.as_str())
    }

    pub fn get_or_first(&self, lang: Language) -> Option<&str> {
        self.get(lang).or_else(|| self.0.values().next().map(|s| s.as_str()))
    }
}

impl Default for MultiLanguageString {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInformation {
    pub id: SourceId,
    pub name: String,
    pub version: String,
    pub icon: String,
    pub has_cloudflare: bool,
    pub base_url: String,
    pub supported_languages: Vec<Language>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: GenreId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallSerie {
    pub id: SerieId,
    pub title: MultiLanguageString,
    pub cover: String,
    pub serie_type: SerieType,
    pub status: Vec<SerieStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedSmallSeries {
    pub series: Vec<SmallSerie>,
    pub has_next_page: bool,
    pub total_pages: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Serie {
    pub id: SerieId,
    pub title: MultiLanguageString,
    pub cover: String,
    pub synopsis: MultiLanguageString,
    pub status: Vec<SerieStatus>,
    pub serie_type: SerieType,
    pub genres: Vec<Genre>,
    pub authors: Vec<String>,
    pub artists: Vec<String>,
    pub volumes: Vec<Volume>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: VolumeId,
    pub name: MultiLanguageString,
    pub number: Option<f32>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub title: MultiLanguageString,
    pub number: Option<f32>,
    pub language: Language,
    pub pages: u32,
    pub published_at: Option<DateTime<Utc>>,
    pub scanlation_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ChapterData {
    Image { images: Vec<ChapterImage> },
    Text { texts: Vec<ChapterText> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterImage {
    pub url: String,
    pub page: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterText {
    pub content: String,
    pub page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub query: Option<String>,
    pub genres: Vec<GenreId>,
    pub excluded_genres: Vec<GenreId>,
    pub status: Vec<SerieStatus>,
    pub serie_types: Vec<SerieType>,
    pub sort: Option<FilterSort>,
    pub order: Option<FilterOrder>,
}

impl Default for SearchFilters {
    fn default() -> Self {
        Self {
            query: None,
            genres: Vec::new(),
            excluded_genres: Vec::new(),
            status: Vec::new(),
            serie_types: Vec::new(),
            sort: None,
            order: None,
        }
    }
}

use super::enums::{FilterOrder, FilterSort};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_language_string() {
        let mls = MultiLanguageString::new()
            .with_language(Language::English, "Hello")
            .with_language(Language::French, "Bonjour");

        assert_eq!(mls.get(Language::English), Some("Hello"));
        assert_eq!(mls.get(Language::French), Some("Bonjour"));
        assert_eq!(mls.get(Language::Japanese), None);
        assert_eq!(mls.get_or_first(Language::Japanese), Some("Hello"));
    }

    #[test]
    fn test_chapter_data_serialization() {
        let image_data = ChapterData::Image {
            images: vec![ChapterImage {
                url: "https://example.com/image.jpg".to_string(),
                page: 1,
                width: Some(800),
                height: Some(1200),
            }],
        };

        let json = serde_json::to_string(&image_data).unwrap();
        assert!(json.contains(r#""type":"image""#));

        let deserialized: ChapterData = serde_json::from_str(&json).unwrap();
        match deserialized {
            ChapterData::Image { images } => {
                assert_eq!(images.len(), 1);
                assert_eq!(images[0].url, "https://example.com/image.jpg");
            }
            _ => panic!("Expected Image variant"),
        }
    }

    #[test]
    fn test_search_filters_default() {
        let filters = SearchFilters::default();
        assert!(filters.query.is_none());
        assert!(filters.genres.is_empty());
        assert!(filters.sort.is_none());
    }
}