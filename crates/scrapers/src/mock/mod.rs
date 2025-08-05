use async_trait::async_trait;

use dokusho_core::{
    Chapter, ChapterData, ChapterId, ChapterImage, Genre, GenreId, Language, MultiLanguageString,
    PaginatedSmallSeries, SearchFilters, Serie, SerieId, SerieStatus, SerieType, SmallSerie,
    SourceApi, SourceError, SourceInformation, SourceId, Volume, VolumeId,
};

pub struct MockSource {
    source_info: SourceInformation,
}

impl MockSource {
    pub fn new() -> Self {
        let source_info = SourceInformation {
            id: SourceId::new("mock"),
            name: "Mock Source".to_string(),
            version: "1.0.0".to_string(),
            icon: "https://example.com/icon.png".to_string(),
            has_cloudflare: false,
            base_url: "https://mock.example.com".to_string(),
            supported_languages: vec![Language::English],
        };

        Self { source_info }
    }

    fn create_mock_serie(&self, id: &str) -> SmallSerie {
        SmallSerie {
            id: SerieId::new(id),
            title: MultiLanguageString::new()
                .with_language(Language::English, format!("Mock Serie {}", id)),
            cover: format!("https://example.com/cover/{}.jpg", id),
            serie_type: SerieType::Manga,
            status: vec![SerieStatus::Ongoing],
        }
    }

    fn create_mock_serie_detail(&self, id: &str) -> Serie {
        let chapters = vec![
            Chapter {
                id: ChapterId::new(format!("{}-ch1", id)),
                title: MultiLanguageString::new().with_language(Language::English, "Chapter 1"),
                number: Some(1.0),
                language: Language::English,
                pages: 20,
                published_at: None,
                scanlation_group: Some("Mock Scans".to_string()),
            },
            Chapter {
                id: ChapterId::new(format!("{}-ch2", id)),
                title: MultiLanguageString::new().with_language(Language::English, "Chapter 2"),
                number: Some(2.0),
                language: Language::English,
                pages: 25,
                published_at: None,
                scanlation_group: Some("Mock Scans".to_string()),
            },
        ];

        Serie {
            id: SerieId::new(id),
            title: MultiLanguageString::new()
                .with_language(Language::English, format!("Mock Serie {}", id)),
            cover: format!("https://example.com/cover/{}.jpg", id),
            synopsis: MultiLanguageString::new()
                .with_language(Language::English, "This is a mock serie for testing purposes."),
            status: vec![SerieStatus::Ongoing],
            serie_type: SerieType::Manga,
            genres: vec![
                Genre {
                    id: GenreId::new("action"),
                    name: "Action".to_string(),
                },
                Genre {
                    id: GenreId::new("adventure"),
                    name: "Adventure".to_string(),
                },
            ],
            authors: vec!["Mock Author".to_string()],
            artists: vec!["Mock Artist".to_string()],
            volumes: vec![Volume {
                id: VolumeId::new("vol1"),
                name: MultiLanguageString::new().with_language(Language::English, "Volume 1"),
                number: Some(1.0),
                chapters,
            }],
            updated_at: None,
            created_at: None,
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
    fn information(&self) -> &SourceInformation {
        &self.source_info
    }

    async fn fetch_popular_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let series: Vec<SmallSerie> = (1..=10)
            .map(|i| self.create_mock_serie(&format!("popular-{}", i)))
            .collect();

        Ok(PaginatedSmallSeries {
            series,
            has_next_page: page < 5,
            total_pages: Some(5),
        })
    }

    async fn fetch_latest_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let series: Vec<SmallSerie> = (1..=10)
            .map(|i| self.create_mock_serie(&format!("latest-{}", i)))
            .collect();

        Ok(PaginatedSmallSeries {
            series,
            has_next_page: page < 3,
            total_pages: Some(3),
        })
    }

    async fn search_series(
        &self,
        filters: &SearchFilters,
        page: u32,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let query = filters.query.as_deref().unwrap_or("test");
        let series: Vec<SmallSerie> = (1..=5)
            .map(|i| {
                let mut serie = self.create_mock_serie(&format!("search-{}", i));
                serie.title = MultiLanguageString::new()
                    .with_language(Language::English, format!("{} Result {}", query, i));
                serie
            })
            .collect();

        Ok(PaginatedSmallSeries {
            series,
            has_next_page: page < 2,
            total_pages: Some(2),
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
                url: format!(
                    "https://example.com/chapters/{}/page-{}.jpg",
                    chapter_id.as_str(),
                    i
                ),
                page: i,
                width: Some(800),
                height: Some(1200),
            })
            .collect();

        Ok(ChapterData::Image { images })
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
        assert_eq!(result.total_pages, Some(5));
    }

    #[tokio::test]
    async fn test_mock_source_search() {
        let source = MockSource::new();
        let filters = SearchFilters {
            query: Some("test manga".to_string()),
            ..Default::default()
        };
        
        let result = source.search_series(&filters, 1).await.unwrap();
        
        assert_eq!(result.series.len(), 5);
        assert_eq!(
            result.series[0].title.get(Language::English),
            Some("test manga Result 1")
        );
    }

    #[tokio::test]
    async fn test_mock_source_not_found() {
        let source = MockSource::new();
        let result = source.fetch_serie_detail(&SerieId::new("not-found")).await;
        
        assert!(matches!(result, Err(SourceError::NotFound(_))));
    }
}