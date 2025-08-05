pub mod mangadex;
pub mod mock;
pub mod weebcentral;

use dokusho_core::{SourceApi, SourceError};

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
        match WeebCentral::new(flaresolver_url) {
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
            .any(|s| s.information().id.as_str() == "mangadex");
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
            .any(|s| s.information().id.as_str() == "mock");
        assert!(has_mock);
    }
}
