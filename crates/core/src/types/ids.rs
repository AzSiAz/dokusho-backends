use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SourceId(String);

impl SourceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SerieId(String);

impl SerieId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VolumeId(String);

impl VolumeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChapterId(String);

impl ChapterId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Display, From, Into, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GenreId(String);

impl GenreId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_id_creation() {
        let id = SourceId::new("mangadex");
        assert_eq!(id.as_str(), "mangadex");
        assert_eq!(id.to_string(), "mangadex");
    }

    #[test]
    fn test_serie_id_serialization() {
        let id = SerieId::new("abc123");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, r#""abc123""#);
        
        let deserialized: SerieId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, id);
    }

    #[test]
    fn test_different_id_types_are_distinct() {
        let serie_id = SerieId::new("123");
        let volume_id = VolumeId::new("123");
        
        // This would not compile, proving type safety:
        // let _: SerieId = volume_id;
        
        assert_eq!(serie_id.as_str(), volume_id.as_str());
    }
}