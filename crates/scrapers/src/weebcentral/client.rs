use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::HashMap;

use dokusho_clients::FlareSolverClient;
use dokusho_core::{
    Chapter, ChapterData, ChapterImage, ChapterId, FilterOrder, FilterSort,
    MultiLanguageString, PaginatedSmallSeries, SearchFilters, Serie, SerieId, SmallSerie,
    Source, SourceApi, SourceApiInformation, SourceError, SourceInformation, SourceLanguage,
    SourceSerieGenre, SourceSerieStatus, SourceSerieType, SupportedFilters,
    SupportedFiltersGenres, Volume, VolumeId,
};

use crate::chapter_utils::calculate_missing_chapters;
use super::types::*;

pub struct WeebCentral {
    client: FlareSolverClient,
    source: Source,
    base_url: String,
    chapter_number_regex: Regex,
}

impl WeebCentral {
    pub fn new(flaresolver_url: String) -> Result<Self, SourceError> {
        let client = FlareSolverClient::new(flaresolver_url)
            .map_err(|e| SourceError::Other(e.into()))?;
        
        let chapter_number_regex = Regex::new(r"\d+(\.\d+)?")
            .map_err(|e| SourceError::Other(anyhow::anyhow!("Failed to compile regex: {}", e)))?;
        
        let source = Source {
            source_information: SourceInformation {
                id: "weebcentral".into(),
                name: "WeebCentral".to_string(),
                url: "https://weebcentral.com".to_string(),
                icon: "https://weebcentral.com/favicon.ico".to_string(),
                languages: vec![SourceLanguage::En],
                updated_at: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                nsfw: false,
                search_filters: SupportedFilters {
                    query: true,
                    orders: vec![FilterOrder::Ascending, FilterOrder::Descending],
                    sorts: vec![
                        FilterSort::Title,
                        FilterSort::UpdatedAt,
                        FilterSort::Popularity,
                        FilterSort::UpdatedAt,
                    ],
                    artists: true,
                    authors: true,
                    types: get_searchable_types(),
                    genres: SupportedFiltersGenres {
                        included: true,
                        excluded: true,
                        possible_values: get_searchable_genres(),
                    },
                    status: get_searchable_status(),
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Some("https://weebcentral.com".to_string()),
                headers: Some({
                    let mut headers = HashMap::new();
                    headers.insert(
                        "User-Agent".to_string(),
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:71.0) Gecko/20100101 Firefox/77.0".to_string(),
                    );
                    headers
                }),
                minimum_update_interval: std::time::Duration::from_secs(300),
                timeout: std::time::Duration::from_secs(30),
                can_block_scraping: true,
            },
        };

        Ok(Self { 
            client, 
            source,
            base_url: "https://weebcentral.com".to_string(),
            chapter_number_regex,
        })
    }

    #[cfg(test)]
    pub fn new_with_url(flaresolver_url: String, base_url: String) -> Result<Self, SourceError> {
        let mut wc = Self::new(flaresolver_url)?;
        wc.base_url = base_url.clone();
        wc.source.source_information.url = base_url.clone();
        wc.source.source_api_information.api_url = Some(base_url);
        Ok(wc)
    }

    async fn fetch_with_search_data_api(
        &self,
        page: i32,
        filters: &SearchFilters,
    ) -> Result<String, SourceError> {
        let limit = 24;
        let offset = (page - 1) * limit;
        
        let mut url = format!("{}/search/data?limit={}", self.base_url, limit);
        
        if offset > 0 {
            url.push_str(&format!("&offset={}", offset));
        }
        
        url.push_str("&official=Any&display_mode=Full Display");
        
        // Add query
        if !filters.query.is_empty() {
            url.push_str(&format!("&text={}", urlencoding::encode(&filters.query)));
        }
        
        // Add sort
        if let Ok(wc_sort) = WeebCentralSort::try_from(filters.sort) {
            url.push_str(&format!("&sort={}", wc_sort.as_str()));
        }
        
        // Add order
        let wc_order = WeebCentralOrder::from(filters.order);
        url.push_str(&format!("&order={}", wc_order.as_str()));
        
        // Add types
        for serie_type in &filters.types {
            if let Ok(wc_type) = WeebCentralType::try_from(match serie_type {
                SourceSerieType::Manga => "manga",
                SourceSerieType::Manhwa => "manhwa",
                SourceSerieType::Manhua => "manhua",
                SourceSerieType::Comic => "comic",
                SourceSerieType::Webtoon => "webtoon",
                SourceSerieType::Novel => "novel",
                _ => continue,
            }) {
                url.push_str(&format!("&included_type={}", wc_type.as_str()));
            }
        }
        
        // Add status
        for status in &filters.status {
            if let Ok(wc_status) = WeebCentralStatus::try_from(match status {
                SourceSerieStatus::Ongoing => "ongoing",
                SourceSerieStatus::Completed => "completed",
                SourceSerieStatus::Hiatus => "hiatus",
                SourceSerieStatus::Canceled => "cancelled",
                _ => continue,
            }) {
                url.push_str(&format!("&included_status={}", wc_status.as_str()));
            }
        }
        
        // Add genres
        for genre in &filters.genres.include {
            let wc_genre = convert_genre_to_weebcentral(genre);
            url.push_str(&format!("&included_tag={}", wc_genre));
        }
        
        for genre in &filters.genres.exclude {
            let wc_genre = convert_genre_to_weebcentral(genre);
            url.push_str(&format!("&excluded_tag={}", wc_genre));
        }
        
        // Add authors/artists
        for author in &filters.authors {
            url.push_str(&format!("&author={}", urlencoding::encode(author)));
        }
        
        for artist in &filters.artists {
            url.push_str(&format!("&author={}", urlencoding::encode(artist)));
        }
        
        self.client.get_html(&url).await
            .map_err(|e| SourceError::Other(e.into()))
    }

    async fn parse_search_data_response(&self, html: &str) -> Result<Vec<SmallSerie>, SourceError> {
        let document = Html::parse_document(html);
        let mut series = Vec::new();

        // Parse the search data response format
        let article_selector = Selector::parse("body > article").unwrap();
        
        for article in document.select(&article_selector) {
            // Extract cover
            let cover_selector = Selector::parse("section:first-child a > article > picture > source").unwrap();
            let cover_url = article
                .select(&cover_selector)
                .next()
                .and_then(|el| el.value().attr("srcset"))
                .unwrap_or_default()
                .to_string();
            
            // Extract title
            let title_selector = Selector::parse("section:last-child div:first-child a").unwrap();
            let title = article
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_else(|| "Unknown Title".to_string());
            
            // Extract ID from link
            let link_selector = Selector::parse("section:first-child a").unwrap();
            let href = article
                .select(&link_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .ok_or_else(|| SourceError::Parse("Missing href attribute".to_string()))?;
            
            let parts: Vec<&str> = href.split('/').collect();
            if parts.len() < 2 {
                continue;
            }
            let serie_id = parts[parts.len() - 2];
            
            series.push(SmallSerie {
                id: SerieId::new(serie_id),
                title: MultiLanguageString::new().with_language(SourceLanguage::En, title),
                cover: cover_url,
            });
        }
        
        Ok(series)
    }

    async fn fetch_serie_with_chapters(
        &self,
        serie_id: &str,
    ) -> Result<(Serie, Vec<Chapter>), SourceError> {
        let serie_url = format!("{}/series/{}", self.base_url, serie_id);
        let chapters_url = format!("{}/series/{}/full-chapter-list", self.base_url, serie_id);
        
        // Fetch both pages in parallel
        let (serie_html, chapters_html) = tokio::try_join!(
            self.client.get_html(&serie_url),
            self.client.get_html(&chapters_url)
        )?;
        
        let mut serie = self.parse_serie_detail(&serie_html, serie_id)?;
        let chapters = self.parse_chapters_list(&chapters_html)?;
        
        // Calculate missing chapters and create volume
        let chapter_numbers: Vec<f64> = chapters.iter().map(|c| c.chapter_number).collect();
        let missing_chapters = calculate_missing_chapters(&chapter_numbers);
        
        serie.volumes = vec![Volume {
            id: VolumeId::new("volume-1"),
            name: "Volume 1".to_string(),
            volume_number: 1.0,
            missing_chapters,
            chapters,
        }];
        
        Ok((serie, vec![]))
    }

    fn parse_serie_detail(&self, html: &str, serie_id: &str) -> Result<Serie, SourceError> {
        let document = Html::parse_document(html);
        let main_selector = Selector::parse("body > main").unwrap();
        let main = document
            .select(&main_selector)
            .next()
            .ok_or_else(|| SourceError::Parse("Main content not found".to_string()))?;
        
        // Extract title
        let title_selector = Selector::parse("h1").unwrap();
        let title = main
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();
        
        // Extract cover
        let cover_selector = Selector::parse("img").unwrap();
        let cover_url = main
            .select(&cover_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .unwrap_or_default()
            .to_string();
        
        // Extract synopsis
        // Look for li elements and check if they contain Description
        let li_selector = Selector::parse("li").unwrap();
        let mut synopsis = String::new();
        for li in document.select(&li_selector) {
            let text = li.text().collect::<String>();
            if text.starts_with("Description") {
                let p_selector = Selector::parse("p").unwrap();
                if let Some(p) = li.select(&p_selector).next() {
                    synopsis = p.text().collect::<String>();
                    break;
                }
            }
        }
        
        // Extract type
        let mut type_text = "Manga".to_string();
        for li in document.select(&li_selector) {
            let text = li.text().collect::<String>();
            if text.starts_with("Type") {
                let a_selector = Selector::parse("a").unwrap();
                if let Some(a) = li.select(&a_selector).next() {
                    type_text = a.text().collect::<String>();
                    break;
                }
            }
        }
        
        let serie_type = WeebCentralType::try_from(type_text.as_str())
            .and_then(|wc_type| wc_type.try_into())
            .unwrap_or(SourceSerieType::Manga);
        
        // Extract status
        let mut status_text = "Unknown".to_string();
        for li in document.select(&li_selector) {
            let text = li.text().collect::<String>();
            if text.starts_with("Status") {
                let a_selector = Selector::parse("a").unwrap();
                if let Some(a) = li.select(&a_selector).next() {
                    status_text = a.text().collect::<String>();
                    break;
                }
            }
        }
        
        let status = WeebCentralStatus::try_from(status_text.as_str())
            .and_then(|wc_status| wc_status.try_into())
            .ok()
            .map(|s| vec![s])
            .unwrap_or_default();
        
        // Extract genres
        let mut genres = Vec::new();
        for li in document.select(&li_selector) {
            let text = li.text().collect::<String>();
            if text.starts_with("Tags") {
                let a_selector = Selector::parse("span > a").unwrap();
                for a in li.select(&a_selector) {
                    let genre_text = a.text().collect::<String>();
                    if let Some(genre) = parse_weebcentral_genre(&genre_text) {
                        genres.push(genre);
                    }
                }
                break;
            }
        }
        
        // Extract authors
        let mut authors = Vec::new();
        for li in document.select(&li_selector) {
            let text = li.text().collect::<String>();
            if text.starts_with("Author") {
                let a_selector = Selector::parse("span > a").unwrap();
                for a in li.select(&a_selector) {
                    authors.push(a.text().collect::<String>());
                }
                break;
            }
        }
        
        Ok(Serie {
            id: SerieId::new(serie_id),
            title: MultiLanguageString::new().with_language(SourceLanguage::En, title),
            alternative_titles: None,
            cover: cover_url,
            synopsis: MultiLanguageString::new().with_language(SourceLanguage::En, synopsis),
            serie_type,
            genres,
            status,
            authors,
            artists: vec![],
            volumes: vec![], // Will be populated with chapters
        })
    }

    fn parse_chapters_list(&self, html: &str) -> Result<Vec<Chapter>, SourceError> {
        let document = Html::parse_document(html);
        let mut chapters = Vec::new();
        
        // Select all chapter links
        let chapter_selector = Selector::parse("body > * > a.flex").unwrap();
        
        for element in document.select(&chapter_selector) {
            let href = element
                .value()
                .attr("href")
                .ok_or_else(|| SourceError::Parse("Missing chapter href".to_string()))?;
            
            let chapter_id = href
                .split('/')
                .filter(|s| !s.is_empty())
                .last()
                .ok_or_else(|| SourceError::Parse("Failed to extract chapter ID".to_string()))?;
            
            // Extract chapter name
            let name_selector = Selector::parse("span.flex > span").unwrap();
            let name = element
                .select(&name_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_else(|| format!("Chapter {}", chapter_id));
            
            // Extract chapter number from name using regex
            let chapter_number = self.chapter_number_regex
                .find(&name)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(0.0);
            
            // Round to 3 decimal places to avoid floating point issues
            let chapter_number = (chapter_number * 1000.0).round() / 1000.0;
            
            // Extract upload date
            let time_selector = Selector::parse("time").unwrap();
            let date_upload = element
                .select(&time_selector)
                .next()
                .and_then(|el| el.value().attr("datetime"))
                .and_then(|dt| chrono::DateTime::parse_from_rfc3339(dt).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);
            
            chapters.push(Chapter {
                id: ChapterId::new(chapter_id),
                name,
                chapter_number,
                language: SourceLanguage::En,
                date_upload,
                external_url: None,
            });
        }
        
        // Sort chapters by number descending
        chapters.sort_by(|a, b| {
            b.chapter_number
                .partial_cmp(&a.chapter_number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        Ok(chapters)
    }

    async fn parse_chapter_images(&self, html: &str) -> Result<Vec<String>, SourceError> {
        let document = Html::parse_document(html);
        let mut images = Vec::new();
        
        // Select all images in the chapter
        let img_selector = Selector::parse("img").unwrap();
        
        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src") {
                // Skip broken image placeholders
                if !src.is_empty() && !src.contains("broken_image") {
                    images.push(src.to_string());
                }
            }
        }
        
        Ok(images)
    }
}

#[async_trait]
impl SourceApi for WeebCentral {
    fn get_information(&self) -> SourceInformation {
        self.source.source_information.clone()
    }

    fn get_api_information(&self) -> SourceApiInformation {
        self.source.source_api_information.clone()
    }

    async fn fetch_popular_series(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        let filters = SearchFilters {
            sort: FilterSort::Popularity,
            order: FilterOrder::Descending,
            ..Default::default()
        };
        
        let html = self.fetch_with_search_data_api(page, &filters).await?;
        let series = self.parse_search_data_response(&html).await?;
        
        // Check if there's a "View More Results..." button to determine if there's a next page
        let has_next_page = html.contains("View More Results...");
        
        Ok(PaginatedSmallSeries {
            has_next_page,
            series,
        })
    }

    async fn fetch_latest_updates(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        let filters = SearchFilters {
            sort: FilterSort::UpdatedAt,
            order: FilterOrder::Descending,
            ..Default::default()
        };
        
        let html = self.fetch_with_search_data_api(page, &filters).await?;
        let series = self.parse_search_data_response(&html).await?;
        
        let has_next_page = html.contains("View More Results...");
        
        Ok(PaginatedSmallSeries {
            has_next_page,
            series,
        })
    }

    async fn search_series(
        &self,
        page: i32,
        filters: SearchFilters,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let html = self.fetch_with_search_data_api(page, &filters).await?;
        let series = self.parse_search_data_response(&html).await?;
        
        let has_next_page = html.contains("View More Results...");
        
        Ok(PaginatedSmallSeries {
            has_next_page,
            series,
        })
    }

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let (serie, _) = self.fetch_serie_with_chapters(serie_id.as_str()).await?;
        Ok(serie)
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: &SerieId,
        _volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError> {
        let url = format!("{}/chapters/{}/images?reading_style=long_strip", self.base_url, chapter_id.as_str());
        let html = self.client.get_html(&url).await?;
        let image_urls = self.parse_chapter_images(&html).await?;
        
        let images: Vec<ChapterImage> = image_urls
            .into_iter()
            .enumerate()
            .map(|(idx, url)| ChapterImage {
                index: (idx + 1) as i32,
                url,
            })
            .collect();
        
        Ok(ChapterData::from_images(images))
    }

    async fn get_serie_url(&self, serie_id: &SerieId) -> Result<String, SourceError> {
        Ok(format!("{}/series/{}", self.base_url, serie_id.as_str()))
    }
}