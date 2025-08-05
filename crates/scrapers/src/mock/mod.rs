use async_trait::async_trait;

use dokusho_core::{
    Chapter, ChapterData, ChapterImage, ChapterId, FilterOrder, FilterSort,
    MultiLanguageString, PaginatedSmallSeries, SearchFilters, Serie, SerieId, SmallSerie,
    Source, SourceApi, SourceApiInformation, SourceError, SourceInformation, SourceLanguage,
    SourceSerieGenre, SourceSerieStatus, SourceSerieType, SupportedFilters,
    SupportedFiltersGenres, Volume, VolumeId,
};

pub struct MockSource {
    source: Source,
}

impl MockSource {
    pub fn new() -> Self {
        let source = Source {
            source_information: SourceInformation {
                id: "mock".into(),
                name: "Mock Source".to_string(),
                url: "https://mock.example.com".to_string(),
                icon: "https://example.com/icon.png".to_string(),
                languages: vec![SourceLanguage::En],
                updated_at: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                nsfw: false,
                search_filters: SupportedFilters {
                    query: true,
                    orders: vec![FilterOrder::Ascending, FilterOrder::Descending],
                    sorts: vec![FilterSort::Title, FilterSort::UpdatedAt],
                    artists: false,
                    authors: false,
                    types: vec![SourceSerieType::Manga],
                    genres: SupportedFiltersGenres {
                        included: true,
                        excluded: true,
                        possible_values: vec![
                            SourceSerieGenre::Action,
                            SourceSerieGenre::Adventure,
                            SourceSerieGenre::Comedy,
                            SourceSerieGenre::Drama,
                        ],
                    },
                    status: vec![
                        SourceSerieStatus::Ongoing,
                        SourceSerieStatus::Completed,
                    ],
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Some("https://api.mock.example.com".to_string()),
                headers: None,
                minimum_update_interval: std::time::Duration::from_secs(300),
                timeout: std::time::Duration::from_secs(30),
                can_block_scraping: false,
            },
        };

        Self { source }
    }

    fn create_mock_serie(&self, id: &str) -> SmallSerie {
        SmallSerie {
            id: SerieId::new(id),
            title: MultiLanguageString::new()
                .with_language(SourceLanguage::En, format!("Mock Serie {}", id)),
            cover: format!("https://example.com/cover/{}.jpg", id),
        }
    }

    fn create_mock_serie_detail(&self, id: &str) -> Serie {
        let chapters = vec![
            Chapter {
                id: ChapterId::new(format!("{}-ch1", id)),
                name: "Chapter 1".to_string(),
                chapter_number: 1.0,
                language: SourceLanguage::En,
                date_upload: chrono::Utc::now(),
                external_url: Some(format!("https://example.com/serie/{}/chapter/1", id)),
            },
            Chapter {
                id: ChapterId::new(format!("{}-ch2", id)),
                name: "Chapter 2".to_string(),
                chapter_number: 2.0,
                language: SourceLanguage::En,
                date_upload: chrono::Utc::now(),
                external_url: Some(format!("https://example.com/serie/{}/chapter/2", id)),
            },
        ];

        Serie {
            id: SerieId::new(id),
            title: MultiLanguageString::new()
                .with_language(SourceLanguage::En, format!("Mock Serie {}", id)),
            alternative_titles: Some(vec![
                MultiLanguageString::new()
                    .with_language(SourceLanguage::En, format!("Alternative Title {}", id)),
            ]),
            cover: format!("https://example.com/cover/{}.jpg", id),
            synopsis: MultiLanguageString::new()
                .with_language(SourceLanguage::En, "This is a mock serie for testing purposes."),
            serie_type: SourceSerieType::Manga,
            genres: vec![SourceSerieGenre::Action, SourceSerieGenre::Adventure],
            status: vec![SourceSerieStatus::Ongoing],
            authors: vec!["Mock Author".to_string()],
            artists: vec!["Mock Artist".to_string()],
            volumes: vec![Volume {
                id: VolumeId::new("vol1"),
                name: "Volume 1".to_string(),
                volume_number: 1.0,
                missing_chapters: vec![],
                chapters,
            }],
        }
    }
}

impl Default for MockSource {
    fn default() -> Self {
        Self::new()
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

    async fn fetch_popular_series(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        let series: Vec<SmallSerie> = (1..=10)
            .map(|i| self.create_mock_serie(&format!("popular-{}", i)))
            .collect();

        Ok(PaginatedSmallSeries {
            has_next_page: page < 3,
            series,
        })
    }

    async fn fetch_latest_updates(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        let series: Vec<SmallSerie> = (1..=10)
            .map(|i| self.create_mock_serie(&format!("latest-{}", i)))
            .collect();

        Ok(PaginatedSmallSeries {
            has_next_page: page < 3,
            series,
        })
    }

    async fn search_series(
        &self,
        page: i32,
        filters: SearchFilters,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let query = if filters.query.is_empty() {
            "test"
        } else {
            &filters.query
        };

        let series: Vec<SmallSerie> = (1..=5)
            .map(|i| {
                let mut serie = self.create_mock_serie(&format!("search-{}", i));
                serie.title = MultiLanguageString::new()
                    .with_language(SourceLanguage::En, format!("{} Result {}", query, i));
                serie
            })
            .collect();

        Ok(PaginatedSmallSeries {
            has_next_page: page < 2,
            series,
        })
    }

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        if serie_id.as_str() == "not-found" {
            return Err(SourceError::NotFound(format!(
                "Serie {} not found",
                serie_id
            )));
        }

        Ok(self.create_mock_serie_detail(serie_id.as_str()))
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: &SerieId,
        _volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError> {
        let images: Vec<ChapterImage> = (1..=10)
            .map(|i| ChapterImage {
                index: i,
                url: format!(
                    "https://example.com/chapters/{}/page-{}.jpg",
                    chapter_id.as_str(),
                    i
                ),
            })
            .collect();

        Ok(ChapterData::from_images(images))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_source_popular() {
        let source = MockSource::new();
        let result = source.fetch_popular_series(1).await.unwrap();
        
        assert_eq!(result.series.len(), 10);
        assert!(result.has_next_page);
        assert_eq!(result.series[0].id.as_str(), "popular-1");
    }

    #[tokio::test]
    async fn test_mock_source_search() {
        let source = MockSource::new();
        let filters = SearchFilters {
            query: "test manga".to_string(),
            ..Default::default()
        };
        let result = source.search_series(1, filters).await.unwrap();
        
        assert_eq!(result.series.len(), 5);
        assert!(result.series[0].title.en.as_ref().unwrap().contains("test manga"));
    }

    #[tokio::test]
    async fn test_mock_source_serie_detail() {
        let source = MockSource::new();
        let serie = source
            .fetch_serie_detail(&SerieId::new("test-123"))
            .await
            .unwrap();
        
        assert_eq!(serie.id.as_str(), "test-123");
        assert_eq!(serie.volumes.len(), 1);
        assert_eq!(serie.volumes[0].chapters.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_source_not_found() {
        let source = MockSource::new();
        let result = source.fetch_serie_detail(&SerieId::new("not-found")).await;
        
        assert!(matches!(result, Err(SourceError::NotFound(_))));
    }
}