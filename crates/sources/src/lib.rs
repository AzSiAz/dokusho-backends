pub mod scrapers;
pub mod utils;

use async_trait::async_trait;
use dokusho_clients::{http::CloudflareAwareHttpClient, retry::RetryConfig};
use dokusho_config::SourcesConfig;
use dokusho_core::{
    FetchSearchSerieFilter, SourceApi, SourceApiInformation, SourceChapters, SourceError, SourceId,
    SourceInformation, SourcePaginatedSmallSerie, SourceSerie, SourceSerieChapterData,
    SourceSerieChapterId, SourceSerieId,
};
use scrapers::{Mangadex, MockSource, WeebCentral};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::info;
use url::Url;

/// Enum wrapper for all source implementations
#[derive(Clone)]
pub enum Source {
    Mangadex(Mangadex),
    WeebCentral(WeebCentral),
    Mock(MockSource),
}

#[async_trait]
impl SourceApi for Source {
    fn get_information(&self) -> SourceInformation {
        match self {
            Source::Mangadex(s) => s.get_information(),
            Source::WeebCentral(s) => s.get_information(),
            Source::Mock(s) => s.get_information(),
        }
    }

    fn get_api_information(&self) -> SourceApiInformation {
        match self {
            Source::Mangadex(s) => s.get_api_information(),
            Source::WeebCentral(s) => s.get_api_information(),
            Source::Mock(s) => s.get_api_information(),
        }
    }

    fn serie_url(&self, serie_id: SourceSerieId) -> Result<Url, SourceError> {
        match self {
            Source::Mangadex(s) => s.serie_url(serie_id),
            Source::WeebCentral(s) => s.serie_url(serie_id),
            Source::Mock(s) => s.serie_url(serie_id),
        }
    }

    async fn fetch_popular_serie(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_popular_serie(page).await,
            Source::WeebCentral(s) => s.fetch_popular_serie(page).await,
            Source::Mock(s) => s.fetch_popular_serie(page).await,
        }
    }

    async fn fetch_latest_updates(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_latest_updates(page).await,
            Source::WeebCentral(s) => s.fetch_latest_updates(page).await,
            Source::Mock(s) => s.fetch_latest_updates(page).await,
        }
    }

    async fn fetch_search_serie(
        &self,
        page: i16,
        filters: FetchSearchSerieFilter,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_search_serie(page, filters).await,
            Source::WeebCentral(s) => s.fetch_search_serie(page, filters).await,
            Source::Mock(s) => s.fetch_search_serie(page, filters).await,
        }
    }

    async fn fetch_serie_detail(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceSerie, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_serie_detail(serie_id).await,
            Source::WeebCentral(s) => s.fetch_serie_detail(serie_id).await,
            Source::Mock(s) => s.fetch_serie_detail(serie_id).await,
        }
    }

    async fn fetch_serie_chapters(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceChapters, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_serie_chapters(serie_id).await,
            Source::WeebCentral(s) => s.fetch_serie_chapters(serie_id).await,
            Source::Mock(s) => s.fetch_serie_chapters(serie_id).await,
        }
    }

    async fn fetch_chapter_data(
        &self,
        serie_id: SourceSerieId,
        chapter_id: SourceSerieChapterId,
    ) -> Result<SourceSerieChapterData, SourceError> {
        match self {
            Source::Mangadex(s) => s.fetch_chapter_data(serie_id, chapter_id).await,
            Source::WeebCentral(s) => s.fetch_chapter_data(serie_id, chapter_id).await,
            Source::Mock(s) => s.fetch_chapter_data(serie_id, chapter_id).await,
        }
    }
}

pub fn build_sources(config: &SourcesConfig) -> Result<Vec<Source>, SourceError> {
    let mut sources: Vec<Source> = Vec::new();
    let mut http = CloudflareAwareHttpClient::new(Duration::from_secs(30), RetryConfig::default())
        .map_err(|e| SourceError::Other(e.into()))?;

    if let Some(flaresolver) = config.flaresolverr.clone() {
        http = http
            .with_flaresolver(flaresolver.url)
            .map_err(|e| SourceError::Other(e.into()))?;
    }

    sources.push(Source::Mangadex(Mangadex::new(
        config.enabled_languages.clone(),
        http.clone(),
    )?));

    sources.push(Source::WeebCentral(WeebCentral::new(
        config.enabled_languages.clone(),
        http.clone(),
    )?));

    if config.enable_mock.unwrap_or(false) {
        sources.push(Source::Mock(
            MockSource::new().expect("Couldn't build mock source"),
        ));
    }

    Ok(sources)
}

pub struct SourceRegistry {
    sources: HashMap<SourceId, Arc<Source>>,
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
                sources.insert(info.id.to_string(), Arc::new(source));
            }
        }

        Self { sources }
    }

    pub fn get_source(&self, name: &str) -> Option<Arc<Source>> {
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

    pub fn get_sources(&self) -> Vec<Arc<Source>> {
        self.sources
            .values()
            .filter(|source| !source.get_information().enabled_languages.is_empty())
            .cloned()
            .collect()
    }
}
