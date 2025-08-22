pub mod scrapers;
pub mod utils;

use dokusho_clients::http::CloudflareAwareHttpClient;
use dokusho_config::SourcesConfig;
use dokusho_core::{SourceApi, SourceError};
use scrapers::{Mangadex, MockSource};
use std::{collections::HashMap, sync::Arc};
use tracing::info;

use crate::scrapers::WeebCentral;

pub fn build_sources(config: &SourcesConfig) -> Result<Vec<Box<dyn SourceApi>>, SourceError> {
    let mut sources: Vec<Box<dyn SourceApi>> = Vec::new();
    let mut http = CloudflareAwareHttpClient::new().map_err(|e| SourceError::Other(e.into()))?;

    if let Some(flaresolver) = config.flaresolverr.clone() {
        http = http
            .with_flaresolver(flaresolver.url)
            .map_err(|e| SourceError::Other(e.into()))?;
    }

    sources.push(Box::new(Mangadex::new(
        config.enabled_languages.clone(),
        http.clone(),
    )?));

    sources.push(Box::new(WeebCentral::new(
        config.enabled_languages.clone(),
        http.clone(),
    )?));

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
        let langs = config
            .enabled_languages
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join(",");

        info!(enabled_lang = langs, "Enabling source langs");
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
        let source = self.sources.get(name);

        match source {
            None => None,
            Some(source) => {
                if !source.get_information().enabled_languages.is_empty() {
                    Some(source.clone())
                } else {
                    None
                }
            }
        }
    }

    pub fn get_sources(&self) -> Vec<Arc<dyn SourceApi>> {
        self.sources
            .values()
            .filter(|source| !source.get_information().enabled_languages.is_empty())
            .cloned()
            .collect()
    }
}
