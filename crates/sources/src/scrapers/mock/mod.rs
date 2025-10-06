use async_trait::async_trait;
use chrono::Utc;
use url::Url;

use dokusho_core::{
    FetchSearchSerieFilter, FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort,
    MultiLanguageString, Source, SourceApi, SourceApiInformation, SourceChapters, SourceError,
    SourceInformation, SourceLanguage, SourcePaginatedSmallSerie, SourceSerie, SourceSerieChapter,
    SourceSerieChapterData, SourceSerieChapterId, SourceSerieChapterImage, SourceSerieGenre,
    SourceSerieId, SourceSerieStatus, SourceSerieType, SourceSmallSerie, SupportedFilters,
    SupportedFiltersGenres,
};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MockSource {
    source: Source,
}

impl MockSource {
    pub fn new() -> Result<Self, SourceError> {
        let source = Source {
            source_information: SourceInformation {
                id: "mock".to_string(),
                name: "Mock Source".to_string(),
                url: Url::parse("https://mock.example.com")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                icon: Url::parse("https://example.com/icon.png")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                languages: vec![SourceLanguage::En],
                enabled_languages: vec![SourceLanguage::En],
                updated_at: Utc::now().into(),
                version: "1.0.0".to_string(),
                include_nsfw: true,
                search_filters: SupportedFilters {
                    query: true,
                    order: vec![
                        FetchSearchSerieFilterOrder::ASC,
                        FetchSearchSerieFilterOrder::DESC,
                    ],
                    sort: vec![
                        FetchSearchSerieFilterSort::Alphabetic,
                        FetchSearchSerieFilterSort::Latest,
                    ],
                    artists: false,
                    authors: false,
                    types: vec![SourceSerieType::Manga],
                    genres: SupportedFiltersGenres {
                        include: true,
                        exclude: true,
                        accepted_values: vec![
                            SourceSerieGenre::Action,
                            SourceSerieGenre::Adventure,
                            SourceSerieGenre::Comedy,
                            SourceSerieGenre::Drama,
                        ],
                    },
                    status: vec![SourceSerieStatus::Ongoing, SourceSerieStatus::Completed],
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Url::parse("https://api.mock.example.com")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                headers: HashMap::new(),
                minimum_update_interval: 300,
                timeout: tokio::time::Duration::from_secs(30),
                can_block_scraping: false,
            },
        };

        Ok(Self { source })
    }

    fn create_mock_small_serie(&self, id: &str) -> Result<SourceSmallSerie, SourceError> {
        Ok(SourceSmallSerie {
            id: id.to_string(),
            title: MultiLanguageString::new()
                .insert(SourceLanguage::En, format!("Mock Serie {}", id)),
            cover: Url::parse(&format!("https://example.com/cover/{}.jpg", id))
                .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
        })
    }

    fn create_mock_serie_detail(&self, id: &str) -> Result<SourceSerie, SourceError> {
        Ok(SourceSerie {
            id: id.to_string(),
            title: MultiLanguageString::new()
                .insert(SourceLanguage::En, format!("Mock Serie {}", id)),
            alternates_titles: MultiLanguageString::new()
                .insert(SourceLanguage::En, format!("Alternative Title {}", id)),
            cover: Url::parse(&format!("https://example.com/cover/{}.jpg", id))
                .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
            synopsis: MultiLanguageString::new().insert(
                SourceLanguage::En,
                "This is a mock serie for testing purposes.".to_string(),
            ),
            serie_type: SourceSerieType::Manga,
            genres: vec![SourceSerieGenre::Action, SourceSerieGenre::Adventure],
            status: vec![SourceSerieStatus::Ongoing],
            authors: vec!["Mock Author".to_string()],
            artists: vec!["Mock Artist".to_string()],
        })
    }

    fn create_mock_chapters(&self, serie_id: &str) -> Result<SourceChapters, SourceError> {
        let chapters = vec![
            SourceSerieChapter {
                id: format!("{}-ch1", serie_id),
                title: "Chapter 1".to_string(),
                chapter_number: 1.0,
                volume_number: Some(1.0),
                volume_name: Some("Volume 1".to_string()),
                language: SourceLanguage::En,
                date_upload: Utc::now().into(),
                external_url: Some(
                    Url::parse(&format!("https://example.com/serie/{}/chapter/1", serie_id))
                        .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                ),
            },
            SourceSerieChapter {
                id: format!("{}-ch2", serie_id),
                title: "Chapter 2".to_string(),
                chapter_number: 2.0,
                volume_number: Some(1.0),
                volume_name: Some("Volume 1".to_string()),
                language: SourceLanguage::En,
                date_upload: Utc::now().into(),
                external_url: Some(
                    Url::parse(&format!("https://example.com/serie/{}/chapter/2", serie_id))
                        .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                ),
            },
        ];

        Ok(SourceChapters {
            missing_chapters: vec![],
            chapters,
        })
    }
}

impl Default for MockSource {
    fn default() -> Self {
        Self::new().expect("Failed to create MockSource")
    }
}

#[async_trait]
impl SourceApi for MockSource {
    fn get_information(&self) -> SourceInformation {
        self.source.source_information.clone()
    }

    fn get_api_information(&self) -> SourceApiInformation {
        self.source.source_api_information.clone()
    }

    fn serie_url(&self, serie_id: SourceSerieId) -> Result<Url, SourceError> {
        Url::parse(&format!("https://mock.example.com/serie/{}", serie_id))
            .map_err(|e| SourceError::BuildingURL(e.to_string()))
    }

    async fn fetch_popular_serie(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        let series: Result<Vec<_>, _> = (1..=10)
            .map(|i| self.create_mock_small_serie(&format!("popular-{}", i)))
            .collect();

        Ok(SourcePaginatedSmallSerie {
            has_next_page: page < 3,
            series: series?,
        })
    }

    async fn fetch_latest_updates(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        let series: Result<Vec<_>, _> = (1..=10)
            .map(|i| self.create_mock_small_serie(&format!("latest-{}", i)))
            .collect();

        Ok(SourcePaginatedSmallSerie {
            has_next_page: page < 3,
            series: series?,
        })
    }

    async fn fetch_search_serie(
        &self,
        page: i16,
        filters: FetchSearchSerieFilter,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        let query = filters.query.as_deref().unwrap_or("test");

        let series: Result<Vec<SourceSmallSerie>, SourceError> = (1..=5)
            .map(|i| {
                let mut serie = self.create_mock_small_serie(&format!("search-{}", i))?;
                serie.title = MultiLanguageString::new()
                    .insert(SourceLanguage::En, format!("{} Result {}", query, i));
                Ok(serie)
            })
            .collect();

        Ok(SourcePaginatedSmallSerie {
            has_next_page: page < 2,
            series: series?,
        })
    }

    async fn fetch_serie_detail(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceSerie, SourceError> {
        if serie_id == "not-found" {
            return Err(SourceError::InvalidSerieID(format!(
                "Serie {} not found",
                serie_id
            )));
        }

        self.create_mock_serie_detail(&serie_id)
    }

    async fn fetch_serie_chapters(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceChapters, SourceError> {
        self.create_mock_chapters(&serie_id)
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: SourceSerieId,
        chapter_id: SourceSerieChapterId,
    ) -> Result<SourceSerieChapterData, SourceError> {
        let images: Vec<SourceSerieChapterImage> = (1..=10)
            .map(|i| {
                let url = format!("https://example.com/chapters/{}/page-{}.jpg", chapter_id, i);
                let parsed_url = Url::parse(&url).unwrap();
                SourceSerieChapterImage {
                    index: i,
                    url: parsed_url,
                }
            })
            .collect();

        Ok(SourceSerieChapterData::Image(images))
    }
}
