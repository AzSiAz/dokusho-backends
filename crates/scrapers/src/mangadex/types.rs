use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use dokusho_core::{Language, SerieStatus, SerieType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexResponse<T> {
    pub result: String,
    pub response: String,
    pub data: T,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexListResponse<T> {
    pub result: String,
    pub response: String,
    pub data: Vec<T>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexManga {
    pub id: String,
    #[serde(rename = "type")]
    pub manga_type: String,
    pub attributes: MangaDexMangaAttributes,
    pub relationships: Option<Vec<MangaDexRelationship>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexMangaAttributes {
    pub title: HashMap<String, String>,
    pub alt_titles: Vec<HashMap<String, String>>,
    pub description: HashMap<String, String>,
    pub is_locked: bool,
    pub links: Option<HashMap<String, String>>,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<String>,
    pub status: String,
    pub year: Option<i32>,
    pub content_rating: String,
    pub tags: Vec<MangaDexTag>,
    pub state: String,
    pub chapter_numbers_reset_on_new_volume: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub available_translated_languages: Vec<String>,
    pub latest_uploaded_chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexTag {
    pub id: String,
    #[serde(rename = "type")]
    pub tag_type: String,
    pub attributes: MangaDexTagAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexTagAttributes {
    pub name: HashMap<String, String>,
    pub description: HashMap<String, String>,
    pub group: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexRelationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    pub related: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexChapter {
    pub id: String,
    #[serde(rename = "type")]
    pub chapter_type: String,
    pub attributes: MangaDexChapterAttributes,
    pub relationships: Vec<MangaDexRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterAttributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: Option<String>,
    pub translated_language: String,
    pub external_url: Option<String>,
    pub publish_at: DateTime<Utc>,
    pub readable_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pages: i32,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterPages {
    pub result: String,
    pub base_url: String,
    pub chapter: MangaDexChapterPagesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterPagesData {
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

impl MangaDexManga {
    pub fn get_cover_url(&self) -> Option<String> {
        self.relationships.as_ref().and_then(|rels| {
            rels.iter()
                .find(|r| r.rel_type == "cover_art")
                .and_then(|cover| {
                    cover.attributes.as_ref().and_then(|attrs| {
                        attrs.get("fileName").and_then(|f| f.as_str())
                    })
                })
                .map(|filename| {
                    format!(
                        "https://uploads.mangadex.org/covers/{}/{}.512.jpg",
                        self.id, filename
                    )
                })
        })
    }

    pub fn get_status(&self) -> Vec<SerieStatus> {
        match self.attributes.status.as_str() {
            "ongoing" => vec![SerieStatus::Ongoing],
            "completed" => vec![SerieStatus::Completed],
            "hiatus" => vec![SerieStatus::Hiatus],
            "cancelled" => vec![SerieStatus::Cancelled],
            _ => vec![],
        }
    }

    pub fn get_serie_type(&self) -> SerieType {
        match self.attributes.original_language.as_str() {
            "ja" => SerieType::Manga,
            "ko" => SerieType::Manhwa,
            "zh" | "zh-hk" => SerieType::Manhua,
            _ => SerieType::Comic,
        }
    }

    pub fn parse_language(lang_code: &str) -> Option<Language> {
        match lang_code {
            "en" => Some(Language::English),
            "fr" => Some(Language::French),
            "es" => Some(Language::Spanish),
            "de" => Some(Language::German),
            "it" => Some(Language::Italian),
            "pt" | "pt-br" => Some(Language::Portuguese),
            "ja" => Some(Language::Japanese),
            "ko" => Some(Language::Korean),
            "zh" | "zh-hk" => Some(Language::Chinese),
            _ => None,
        }
    }
}