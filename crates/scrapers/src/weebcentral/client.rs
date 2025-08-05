use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use scraper::{Html, Selector};

use dokusho_clients::FlareSolverClient;
use dokusho_core::{
    Chapter, ChapterData, ChapterId, ChapterImage, Genre, GenreId, Language, MultiLanguageString,
    PaginatedSmallSeries, SearchFilters, Serie, SerieId, SerieStatus, SerieType, SmallSerie,
    SourceApi, SourceError, SourceInformation, SourceId, Volume, VolumeId,
};

lazy_static! {
    static ref SERIE_ID_REGEX: Regex = Regex::new(r"/series/([^/]+)/").unwrap();
    static ref CHAPTER_ID_REGEX: Regex = Regex::new(r"/chapter/([^/]+)").unwrap();
}

pub struct WeebCentral {
    client: FlareSolverClient,
    source_info: SourceInformation,
}

impl WeebCentral {
    pub fn new(flaresolver_url: &str) -> Result<Self, SourceError> {
        let client = FlareSolverClient::new(flaresolver_url)
            .map_err(|e| SourceError::Other(e.into()))?;

        let source_info = SourceInformation {
            id: SourceId::new("weebcentral"),
            name: "WeebCentral".to_string(),
            version: "1.0.0".to_string(),
            icon: "https://weebcentral.com/favicon.ico".to_string(),
            has_cloudflare: true,
            base_url: "https://weebcentral.com".to_string(),
            supported_languages: vec![Language::English],
        };

        Ok(Self {
            client,
            source_info,
        })
    }

    async fn parse_serie_list(&self, html: &str) -> Result<Vec<SmallSerie>, SourceError> {
        let document = Html::parse_document(html);
        
        // Try grid layout first (browse pages)
        let serie_selector = Selector::parse("div.grid > a").unwrap();
        let mut series = Vec::new();

        if document.select(&serie_selector).count() > 0 {
            let title_selector = Selector::parse("h3").unwrap();
            let img_selector = Selector::parse("img").unwrap();

            for element in document.select(&serie_selector) {
                let href = element
                    .value()
                    .attr("href")
                    .ok_or_else(|| SourceError::Parse("Missing href".to_string()))?;

                let serie_id = SERIE_ID_REGEX
                    .captures(href)
                    .and_then(|cap| cap.get(1))
                    .map(|m| m.as_str())
                    .ok_or_else(|| SourceError::Parse("Failed to extract serie ID".to_string()))?;

                let title_text = element
                    .select(&title_selector)
                    .next()
                    .map(|el| el.text().collect::<String>())
                    .unwrap_or_default();

                let cover_url = element
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("src"))
                    .unwrap_or_default()
                    .to_string();

                let title = MultiLanguageString::new().with_language(Language::English, title_text);

                series.push(SmallSerie {
                    id: SerieId::new(serie_id),
                    title,
                    cover: cover_url,
                    serie_type: SerieType::Manga,
                    status: vec![],
                });
            }
        } else {
            // Try search results layout
            let article_selector = Selector::parse("article.bg-base-300").unwrap();
            let link_selector = Selector::parse("a.link.link-hover").unwrap();
            let img_selector = Selector::parse("img").unwrap();

            for article in document.select(&article_selector) {
                if let Some(link) = article.select(&link_selector).next() {
                    let href = link
                        .value()
                        .attr("href")
                        .ok_or_else(|| SourceError::Parse("Missing href".to_string()))?;

                    let serie_id = SERIE_ID_REGEX
                        .captures(href)
                        .and_then(|cap| cap.get(1))
                        .map(|m| m.as_str())
                        .ok_or_else(|| SourceError::Parse("Failed to extract serie ID".to_string()))?;

                    let title_text = link.text().collect::<String>();

                    let cover_url = article
                        .select(&img_selector)
                        .next()
                        .and_then(|img| img.value().attr("src"))
                        .unwrap_or_default()
                        .to_string();

                    let title = MultiLanguageString::new().with_language(Language::English, title_text);

                    series.push(SmallSerie {
                        id: SerieId::new(serie_id),
                        title,
                        cover: cover_url,
                        serie_type: SerieType::Manga,
                        status: vec![],
                    });
                }
            }
        }

        Ok(series)
    }

    async fn parse_serie_detail(&self, html: &str) -> Result<Serie, SourceError> {
        let document = Html::parse_document(html);
        
        // Extract title
        let title_selector = Selector::parse("h1").unwrap();
        let title_text = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .ok_or_else(|| SourceError::Parse("Missing title".to_string()))?;

        // Extract cover image
        let cover_selector = Selector::parse("img.rounded-lg").unwrap();
        let cover_url = document
            .select(&cover_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .unwrap_or_default()
            .to_string();

        // Extract synopsis
        let synopsis_selector = Selector::parse("div.prose").unwrap();
        let synopsis_text = document
            .select(&synopsis_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        // Extract metadata
        let meta_selector = Selector::parse("div.flex.flex-col.gap-2 > div").unwrap();
        let mut status = vec![];
        let mut serie_type = SerieType::Manga;
        let mut authors = Vec::new();
        let mut artists = Vec::new();
        let mut genres = Vec::new();

        for element in document.select(&meta_selector) {
            let text = element.text().collect::<String>();
            
            if text.contains("Status:") {
                if text.contains("Ongoing") {
                    status.push(SerieStatus::Ongoing);
                } else if text.contains("Completed") {
                    status.push(SerieStatus::Completed);
                }
            } else if text.contains("Type:") {
                if text.contains("Manhwa") {
                    serie_type = SerieType::Manhwa;
                } else if text.contains("Manhua") {
                    serie_type = SerieType::Manhua;
                }
            } else if text.contains("Author:") {
                if let Some(author) = text.split(':').nth(1) {
                    authors.push(author.trim().to_string());
                }
            } else if text.contains("Artist:") {
                if let Some(artist) = text.split(':').nth(1) {
                    artists.push(artist.trim().to_string());
                }
            }
        }

        // Extract genres
        let genre_selector = Selector::parse("a[href*='/browse?genre=']").unwrap();
        for element in document.select(&genre_selector) {
            let genre_name = element.text().collect::<String>();
            genres.push(Genre {
                id: GenreId::new(genre_name.to_lowercase().replace(' ', "-")),
                name: genre_name,
            });
        }

        // Extract serie ID from canonical URL
        let canonical_selector = Selector::parse("link[rel='canonical']").unwrap();
        let serie_id = document
            .select(&canonical_selector)
            .next()
            .and_then(|link| link.value().attr("href"))
            .and_then(|href| SERIE_ID_REGEX.captures(href))
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| SourceError::Parse("Failed to extract serie ID".to_string()))?;

        let title = MultiLanguageString::new().with_language(Language::English, title_text);
        let synopsis = MultiLanguageString::new().with_language(Language::English, synopsis_text);

        Ok(Serie {
            id: SerieId::new(serie_id),
            title,
            cover: cover_url,
            synopsis,
            status,
            serie_type,
            genres,
            authors,
            artists,
            volumes: vec![], // Will be populated with chapters
            updated_at: None,
            created_at: None,
        })
    }

    async fn parse_chapters(&self, html: &str) -> Result<Vec<Chapter>, SourceError> {
        let document = Html::parse_document(html);
        let chapter_selector = Selector::parse("a[href*='/chapter/']").unwrap();
        let mut chapters = Vec::new();

        for element in document.select(&chapter_selector) {
            let href = element
                .value()
                .attr("href")
                .ok_or_else(|| SourceError::Parse("Missing chapter href".to_string()))?;

            let chapter_id = CHAPTER_ID_REGEX
                .captures(href)
                .and_then(|cap| cap.get(1))
                .map(|m| m.as_str())
                .ok_or_else(|| SourceError::Parse("Failed to extract chapter ID".to_string()))?;

            let text = element.text().collect::<String>();
            
            // Extract chapter number from text like "Chapter 123"
            let chapter_number = text
                .split_whitespace()
                .find_map(|word| word.parse::<f32>().ok());

            let title = MultiLanguageString::new().with_language(Language::English, text);

            chapters.push(Chapter {
                id: ChapterId::new(chapter_id),
                title,
                number: chapter_number,
                language: Language::English,
                pages: 0, // Will be determined when fetching chapter data
                published_at: None,
                scanlation_group: Some("WeebCentral".to_string()),
            });
        }

        Ok(chapters)
    }

    async fn parse_chapter_images(&self, html: &str) -> Result<Vec<ChapterImage>, SourceError> {
        let document = Html::parse_document(html);
        let mut images = Vec::new();

        // First try data-src attribute
        let img_selector = Selector::parse("img[data-src]").unwrap();
        for (idx, element) in document.select(&img_selector).enumerate() {
            if let Some(url) = element.value().attr("data-src") {
                images.push(ChapterImage {
                    url: url.to_string(),
                    page: (idx + 1) as u32,
                    width: None,
                    height: None,
                });
            }
        }

        // If no images found, try regular src attribute
        if images.is_empty() {
            let img_selector = Selector::parse("section img[src]").unwrap();
            for (idx, element) in document.select(&img_selector).enumerate() {
                if let Some(url) = element.value().attr("src") {
                    // Skip broken image placeholders
                    if !url.contains("broken_image") {
                        images.push(ChapterImage {
                            url: url.to_string(),
                            page: (idx + 1) as u32,
                            width: None,
                            height: None,
                        });
                    }
                }
            }
        }

        Ok(images)
    }
}

#[async_trait]
impl SourceApi for WeebCentral {
    fn information(&self) -> &SourceInformation {
        &self.source_info
    }

    async fn fetch_popular_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let url = format!("{}/browse?sort=views&page={}", self.source_info.base_url, page);
        let html = self.client.get_html(&url).await?;
        let series = self.parse_serie_list(&html).await?;

        Ok(PaginatedSmallSeries {
            has_next_page: !series.is_empty(), // Simple heuristic
            total_pages: None,
            series,
        })
    }

    async fn fetch_latest_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let url = format!("{}/browse?sort=latest&page={}", self.source_info.base_url, page);
        let html = self.client.get_html(&url).await?;
        let series = self.parse_serie_list(&html).await?;

        Ok(PaginatedSmallSeries {
            has_next_page: !series.is_empty(),
            total_pages: None,
            series,
        })
    }

    async fn search_series(
        &self,
        filters: &SearchFilters,
        page: u32,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let mut url = format!("{}/browse?page={}", self.source_info.base_url, page);

        if let Some(query) = &filters.query {
            url.push_str(&format!("&q={}", urlencoding::encode(query)));
        }

        let html = self.client.get_html(&url).await?;
        let series = self.parse_serie_list(&html).await?;

        Ok(PaginatedSmallSeries {
            has_next_page: !series.is_empty(),
            total_pages: None,
            series,
        })
    }

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let url = format!("{}/series/{}", self.source_info.base_url, serie_id.as_str());
        let html = self.client.get_html(&url).await?;
        
        let mut serie = self.parse_serie_detail(&html).await?;
        let chapters = self.parse_chapters(&html).await?;

        // Group all chapters into a single volume
        serie.volumes = vec![Volume {
            id: VolumeId::new("all"),
            name: MultiLanguageString::new().with_language(Language::English, "All Chapters"),
            number: None,
            chapters,
        }];

        Ok(serie)
    }

    async fn fetch_chapter_data(
        &self,
        serie_id: &SerieId,
        _volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError> {
        let url = format!(
            "{}/series/{}/chapter/{}",
            self.source_info.base_url,
            serie_id.as_str(),
            chapter_id.as_str()
        );
        
        let html = self.client.get_html(&url).await?;
        let images = self.parse_chapter_images(&html).await?;

        Ok(ChapterData::Image { images })
    }
}