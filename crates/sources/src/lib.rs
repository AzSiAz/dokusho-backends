pub mod scrapers;
pub mod utils;

use dokusho_core::{SourceApi, SourceError};
use scrapers::{Mangadex, MockSource, WeebCentral};
use std::{collections::HashMap, sync::Arc};

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
    // sources.push(Box::new(MangaDex::new()?));

    // Add WeebCentral with optional FlareSolver
    // match create_weebcentral_with_cloudflare(config.flaresolver_url.clone()) {
    //     Ok(source) => sources.push(Box::new(source)),
    //     Err(e) => {
    //         tracing::warn!("Failed to initialize WeebCentral: {}", e);
    //     }
    // }

    // Add mock source if enabled
    if config.enable_mock {
        sources.push(Box::new(MockSource::new().unwrap()));
    }

    Ok(sources)
}

pub struct SourceRegistry {
    sources: HashMap<String, Arc<dyn SourceApi>>,
}

impl SourceRegistry {
    pub fn new(use_flaresolver: bool, flaresolver_url: Option<String>) -> Self {
        let config = SourceConfig {
            flaresolver_url: if use_flaresolver {
                flaresolver_url
            } else {
                None
            },
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
}
