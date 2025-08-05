use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Language {
    #[serde(rename = "en")]
    #[strum(serialize = "en")]
    English,
    
    #[serde(rename = "fr")]
    #[strum(serialize = "fr")]
    French,
    
    #[serde(rename = "jp")]
    #[strum(serialize = "jp")]
    Japanese,
    
    #[serde(rename = "es")]
    #[strum(serialize = "es")]
    Spanish,
    
    #[serde(rename = "de")]
    #[strum(serialize = "de")]
    German,
    
    #[serde(rename = "it")]
    #[strum(serialize = "it")]
    Italian,
    
    #[serde(rename = "pt")]
    #[strum(serialize = "pt")]
    Portuguese,
    
    #[serde(rename = "ko")]
    #[strum(serialize = "ko")]
    Korean,
    
    #[serde(rename = "zh")]
    #[strum(serialize = "zh")]
    Chinese,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SerieStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SerieType {
    Manga,
    Manhwa,
    Manhua,
    Comic,
    Novel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FilterOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FilterSort {
    Title,
    UpdatedAt,
    CreatedAt,
    ChapterCount,
    Rating,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_language_serialization() {
        let lang = Language::English;
        let json = serde_json::to_string(&lang).unwrap();
        assert_eq!(json, r#""en""#);
        
        let deserialized: Language = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, lang);
    }

    #[test]
    fn test_language_from_string() {
        assert_eq!(Language::from_str("en").unwrap(), Language::English);
        assert_eq!(Language::from_str("fr").unwrap(), Language::French);
        assert!(Language::from_str("invalid").is_err());
    }

    #[test]
    fn test_serie_status_serialization() {
        let status = SerieStatus::Ongoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""ONGOING""#);
        
        let deserialized: SerieStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_serie_type_display() {
        assert_eq!(SerieType::Manga.to_string(), "manga");
        assert_eq!(SerieType::Manhwa.to_string(), "manhwa");
    }
}