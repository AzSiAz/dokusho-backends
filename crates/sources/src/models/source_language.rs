use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::SourceError;

fn source_serie_language_parse_not_found(s: &str) -> SourceError {
    SourceError::InvalidGenre(s.to_string())
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    EnumCount,
    Display,
    Hash,
    PartialEq,
    Eq,
)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_language_parse_not_found,
)]
pub enum SourceLanguage {
    #[strum(serialize = "EN")]
    #[serde(rename = "EN")]
    En,
    #[strum(serialize = "JP")]
    #[serde(rename = "JP")]
    Jp,
    #[strum(serialize = "JP-RO")]
    #[serde(rename = "JP-RO")]
    JpRo,
    #[strum(serialize = "FR")]
    #[serde(rename = "FR")]
    Fr,
    #[strum(serialize = "KO")]
    #[serde(rename = "KO")]
    Ko,
    #[strum(serialize = "ZH-HK")]
    #[serde(rename = "ZH-HK")]
    ZhHk,
    #[strum(serialize = "ZH")]
    #[serde(rename = "ZH")]
    Zh,
}

fn serialize_skip_none<S>(
    map: &HashMap<SourceLanguage, Option<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let filtered: HashMap<&SourceLanguage, &String> = map
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
        .collect();
    filtered.serialize(serializer)
}

fn deserialize_with_none<'de, D>(
    deserializer: D,
) -> Result<HashMap<SourceLanguage, Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let map: HashMap<SourceLanguage, String> = HashMap::deserialize(deserializer)?;
    Ok(map.into_iter().map(|(k, v)| (k, Some(v))).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MultiLanguageString {
    #[serde(
        serialize_with = "serialize_skip_none",
        deserialize_with = "deserialize_with_none"
    )]
    value: HashMap<SourceLanguage, Option<String>>,
}

impl MultiLanguageString {
    pub fn new() -> Self {
        Self {
            value: HashMap::new(),
        }
    }

    pub fn insert(mut self, lang: SourceLanguage, text: String) -> Self {
        self.value.insert(lang, Some(text));
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multilanguagestring_serialize_skip_none() {
        let mut map = HashMap::new();
        map.insert(SourceLanguage::En, Some("Hello".to_string()));
        map.insert(SourceLanguage::Jp, Some("こんにちは".to_string()));
        map.insert(SourceLanguage::Fr, None);

        let multi_lang = MultiLanguageString { value: map };
        let json = serde_json::to_string(&multi_lang).unwrap();

        // Should only contain EN and JP, not FR
        assert!(json.contains("\"EN\":\"Hello\""));
        assert!(json.contains("\"JP\":\"こんにちは\""));
        assert!(!json.contains("FR"));
        assert!(!json.contains("null"));
    }

    #[test]
    fn test_multilanguagestring_deserialize() {
        let json = r#"{"EN":"Hello","JP":"こんにちは"}"#;
        let multi_lang: MultiLanguageString = serde_json::from_str(json).unwrap();

        assert_eq!(
            multi_lang.value.get(&SourceLanguage::En),
            Some(&Some("Hello".to_string()))
        );
        assert_eq!(
            multi_lang.value.get(&SourceLanguage::Jp),
            Some(&Some("こんにちは".to_string()))
        );
        assert_eq!(multi_lang.value.get(&SourceLanguage::Fr), None);
    }
}
