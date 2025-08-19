pub mod scrapers;
pub mod utils;

use dokusho_config::SourcesConfig;
use dokusho_core::{SourceApi, SourceError};
use scrapers::MockSource;
use std::{collections::HashMap, sync::Arc};

pub fn build_sources(config: &SourcesConfig) -> Result<Vec<Box<dyn SourceApi>>, SourceError> {
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

    if config.enable_mock.unwrap_or(false) {
        sources.push(Box::new(
            MockSource::new().expect("Couldn't build mock source"),
        ));
    }

    Ok(sources)
}

pub struct SourceRegistry {
    sources: HashMap<String, Arc<dyn SourceApi>>,
}

impl SourceRegistry {
    pub fn new(config: SourcesConfig) -> Self {
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
