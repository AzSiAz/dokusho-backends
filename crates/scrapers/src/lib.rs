pub mod chapter_utils;
pub mod mangadex;
pub mod mock;
pub mod weebcentral;

use dokusho_core::{GraphQLSource, SourceApi, SourceError};
use std::{collections::HashMap, sync::Arc};

pub use mangadex::MangaDex;
pub use mock::MockSource;
pub use weebcentral::WeebCentral;

pub struct SourceConfig {
    pub flaresolver_url: Option<String>,
    pub enable_mock: bool,
}

impl Default for SourceConfig {
    fn default() -> Self {
        Self {
            flaresolver_url: Some("http://localhost:8191".to_string()),
            enable_mock: false,
        }
    }
}

pub fn build_sources(config: &SourceConfig) -> Result<Vec<Box<dyn SourceApi>>, SourceError> {
    let mut sources: Vec<Box<dyn SourceApi>> = Vec::new();

    // Add MangaDex (doesn't require FlareSolver)
    sources.push(Box::new(MangaDex::new()?));

    // Add WeebCentral if FlareSolver is configured
    if let Some(flaresolver_url) = &config.flaresolver_url {
        match WeebCentral::new(flaresolver_url.clone()) {
            Ok(source) => sources.push(Box::new(source)),
            Err(e) => {
                tracing::warn!("Failed to initialize WeebCentral: {}", e);
            }
        }
    }

    // Add mock source if enabled
    if config.enable_mock {
        sources.push(Box::new(MockSource::new()));
    }

    Ok(sources)
}

pub struct SourceRegistry {
    sources: HashMap<String, Arc<dyn SourceApi>>,
}

impl SourceRegistry {
    pub fn new(use_flaresolver: bool, flaresolver_url: Option<String>) -> Self {
        let config = SourceConfig {
            flaresolver_url: if use_flaresolver { flaresolver_url } else { None },
            enable_mock: cfg!(debug_assertions),
        };

        let mut sources = HashMap::new();

        if let Ok(source_list) = build_sources(&config) {
            for source in source_list {
                let info = source.get_information();
                sources.insert(info.id.to_string(), Arc::from(source));
            }
        }

        Self { sources }
    }

    pub fn get_source(&self, name: &str) -> Option<Arc<dyn SourceApi>> {
        self.sources.get(name).cloned()
    }

    pub fn list_sources(&self) -> Vec<GraphQLSource> {
        self.sources
            .iter()
            .map(|(_, source)| {
                let info = source.get_information();
                GraphQLSource {
                    name: info.id.to_string(),
                    version: info.version,
                    icon: info.icon,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_sources_default() {
        let config = SourceConfig::default();
        let sources = build_sources(&config).unwrap();
        
        // Should have at least MangaDex
        assert!(!sources.is_empty());
        
        // Check that MangaDex is included
        let has_mangadex = sources
            .iter()
            .any(|s| s.get_information().id.as_str() == "mangadex");
        assert!(has_mangadex);
    }

    #[test]
    fn test_build_sources_with_mock() {
        let config = SourceConfig {
            enable_mock: true,
            ..Default::default()
        };
        
        let sources = build_sources(&config).unwrap();
        
        // Check that mock source is included
        let has_mock = sources
            .iter()
            .any(|s| s.get_information().id.as_str() == "mock");
        assert!(has_mock);
    }
}