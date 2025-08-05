use std::collections::HashMap;

use async_trait::async_trait;
use tracing::debug;

use dokusho_clients::HttpClient;
use dokusho_core::{
    Chapter, ChapterData, ChapterId, ChapterImage, Genre, GenreId, Language, MultiLanguageString,
    PaginatedSmallSeries, SearchFilters, Serie, SerieId, SmallSerie, SourceApi, SourceError,
    SourceInformation, SourceId, Volume, VolumeId,
};

use super::types::*;

pub struct MangaDex {
    client: HttpClient,
    source_info: SourceInformation,
    api_url: String,
}

impl MangaDex {
    pub fn new() -> Result<Self, SourceError> {
        let client = HttpClient::new()
            .map_err(|e| SourceError::Other(e.into()))?;

        let source_info = SourceInformation {
            id: SourceId::new("mangadex"),
            name: "MangaDex".to_string(),
            version: "1.0.0".to_string(),
            icon: "https://mangadex.org/favicon.ico".to_string(),
            has_cloudflare: false,
            base_url: "https://mangadex.org".to_string(),
            supported_languages: vec![
                Language::English,
                Language::French,
                Language::Spanish,
                Language::German,
                Language::Italian,
                Language::Portuguese,
                Language::Japanese,
                Language::Korean,
                Language::Chinese,
            ],
        };

        Ok(Self {
            client,
            source_info,
            api_url: "https://api.mangadex.org".to_string(),
        })
    }

    #[cfg(test)]
    pub fn new_with_url(api_url: &str) -> Self {
        let client = HttpClient::new().unwrap();
        let mut source_info = SourceInformation {
            id: SourceId::new("mangadex"),
            name: "MangaDex".to_string(),
            version: "1.0.0".to_string(),
            icon: "https://mangadex.org/favicon.ico".to_string(),
            has_cloudflare: false,
            base_url: "https://mangadex.org".to_string(),
            supported_languages: vec![Language::English],
        };
        source_info.base_url = api_url.to_string();

        Self {
            client,
            source_info,
            api_url: api_url.to_string(),
        }
    }

    async fn fetch_manga_list(&self, params: &str) -> Result<PaginatedSmallSeries, SourceError> {
        let url = format!("{}/manga?{}", self.api_url, params);
        debug!("Fetching manga list from: {}", url);

        let response: MangaDexListResponse<MangaDexManga> = self
            .client
            .get_json(&url)
            .await
            .map_err(|e| SourceError::Network(e.to_string()))?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        let series = response
            .data
            .into_iter()
            .map(|manga| self.convert_to_small_serie(manga))
            .collect();

        Ok(PaginatedSmallSeries {
            series,
            has_next_page: response.offset + response.limit < response.total,
            total_pages: Some((response.total + response.limit - 1) / response.limit),
        })
    }

    fn convert_to_small_serie(&self, manga: MangaDexManga) -> SmallSerie {
        let mut title = MultiLanguageString::new();
        
        // Add main title
        for (lang_code, text) in &manga.attributes.title {
            if let Some(lang) = MangaDexManga::parse_language(lang_code) {
                title = title.with_language(lang, text);
            }
        }

        // Add alternative titles
        for alt_title in &manga.attributes.alt_titles {
            for (lang_code, text) in alt_title {
                if let Some(lang) = MangaDexManga::parse_language(lang_code) {
                    if title.get(lang).is_none() {
                        title = title.with_language(lang, text);
                    }
                }
            }
        }

        SmallSerie {
            id: SerieId::new(manga.id.clone()),
            title,
            cover: manga.get_cover_url().unwrap_or_default(),
            serie_type: manga.get_serie_type(),
            status: manga.get_status(),
        }
    }

    fn convert_to_serie(&self, manga: MangaDexManga) -> Serie {
        let mut title = MultiLanguageString::new();
        let mut synopsis = MultiLanguageString::new();

        // Process titles
        for (lang_code, text) in &manga.attributes.title {
            if let Some(lang) = MangaDexManga::parse_language(lang_code) {
                title = title.with_language(lang, text);
            }
        }

        // Process descriptions
        for (lang_code, text) in &manga.attributes.description {
            if let Some(lang) = MangaDexManga::parse_language(lang_code) {
                synopsis = synopsis.with_language(lang, text);
            }
        }

        // Extract genres from tags
        let genres: Vec<Genre> = manga
            .attributes
            .tags
            .iter()
            .filter(|tag| tag.attributes.group == "genre")
            .map(|tag| Genre {
                id: GenreId::new(&tag.id),
                name: tag
                    .attributes
                    .name
                    .get("en")
                    .cloned()
                    .unwrap_or_else(|| tag.id.clone()),
            })
            .collect();

        // Extract authors and artists from relationships
        let mut authors = Vec::new();
        let mut artists = Vec::new();

        if let Some(relationships) = &manga.relationships {
            for rel in relationships {
                match rel.rel_type.as_str() {
                    "author" => {
                        if let Some(attrs) = &rel.attributes {
                            if let Some(name) = attrs.get("name").and_then(|n| n.as_str()) {
                                authors.push(name.to_string());
                            }
                        }
                    }
                    "artist" => {
                        if let Some(attrs) = &rel.attributes {
                            if let Some(name) = attrs.get("name").and_then(|n| n.as_str()) {
                                artists.push(name.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Serie {
            id: SerieId::new(manga.id.clone()),
            title,
            cover: manga.get_cover_url().unwrap_or_default(),
            synopsis,
            status: manga.get_status(),
            serie_type: manga.get_serie_type(),
            genres,
            authors,
            artists,
            volumes: Vec::new(), // Will be populated separately
            updated_at: Some(manga.attributes.updated_at),
            created_at: Some(manga.attributes.created_at),
        }
    }

    async fn fetch_chapters(&self, manga_id: &str) -> Result<Vec<Chapter>, SourceError> {
        let mut all_chapters = Vec::new();
        let mut offset = 0;
        let limit = 100;

        loop {
            let url = format!(
                "{}/manga/{}/feed?limit={}&offset={}&translatedLanguage[]=en&order[chapter]=desc",
                self.api_url, manga_id, limit, offset
            );

            let response: MangaDexListResponse<MangaDexChapter> = self
                .client
                .get_json(&url)
                .await
                .map_err(|e| SourceError::Network(e.to_string()))?;

            if response.result != "ok" {
                return Err(SourceError::Other(anyhow::anyhow!(
                    "MangaDex API error: {}",
                    response.result
                )));
            }

            for chapter in response.data {
                let chapter_number = chapter
                    .attributes
                    .chapter
                    .as_ref()
                    .and_then(|c| c.parse::<f32>().ok());

                let mut title = MultiLanguageString::new();
                if let Some(chapter_title) = &chapter.attributes.title {
                    title = title.with_language(Language::English, chapter_title);
                }

                // Get scanlation group name
                let scanlation_group = chapter
                    .relationships
                    .iter()
                    .find(|r| r.rel_type == "scanlation_group")
                    .and_then(|group| {
                        group.attributes.as_ref().and_then(|attrs| {
                            attrs.get("name").and_then(|n| n.as_str())
                        })
                    })
                    .map(|s| s.to_string());

                all_chapters.push(Chapter {
                    id: ChapterId::new(chapter.id),
                    title,
                    number: chapter_number,
                    language: Language::English,
                    pages: chapter.attributes.pages as u32,
                    published_at: Some(chapter.attributes.publish_at),
                    scanlation_group,
                });
            }

            if response.offset + response.limit >= response.total {
                break;
            }

            offset += limit;
        }

        Ok(all_chapters)
    }
}

#[async_trait]
impl SourceApi for MangaDex {
    fn information(&self) -> &SourceInformation {
        &self.source_info
    }

    async fn fetch_popular_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let offset = page.saturating_sub(1) * 20;
        let params = format!(
            "limit=20&offset={}&order[followedCount]=desc&includes[]=cover_art",
            offset
        );
        self.fetch_manga_list(&params).await
    }

    async fn fetch_latest_series(&self, page: u32) -> Result<PaginatedSmallSeries, SourceError> {
        let offset = page.saturating_sub(1) * 20;
        let params = format!(
            "limit=20&offset={}&order[latestUploadedChapter]=desc&includes[]=cover_art",
            offset
        );
        self.fetch_manga_list(&params).await
    }

    async fn search_series(
        &self,
        filters: &SearchFilters,
        page: u32,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let offset = page.saturating_sub(1) * 20;
        let mut params = format!("limit=20&offset={}&includes[]=cover_art", offset);

        if let Some(query) = &filters.query {
            params.push_str(&format!("&title={}", urlencoding::encode(query)));
        }

        // Add status filters
        for status in &filters.status {
            params.push_str(&format!("&status[]={}", status.to_string().to_lowercase()));
        }

        // Add genre filters
        for genre in &filters.genres {
            params.push_str(&format!("&includedTags[]={}", genre));
        }

        for genre in &filters.excluded_genres {
            params.push_str(&format!("&excludedTags[]={}", genre));
        }

        self.fetch_manga_list(&params).await
    }

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let url = format!(
            "{}/manga/{}?includes[]=cover_art&includes[]=author&includes[]=artist",
            self.api_url,
            serie_id.as_str()
        );

        let response: MangaDexResponse<MangaDexManga> = self
            .client
            .get_json(&url)
            .await
            .map_err(|e| {
                if e.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    SourceError::NotFound(format!("Serie {} not found", serie_id))
                } else {
                    SourceError::Network(e.to_string())
                }
            })?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        let mut serie = self.convert_to_serie(response.data);

        // Fetch chapters and organize into volumes
        let chapters = self.fetch_chapters(serie_id.as_str()).await?;
        
        // Group chapters by volume
        let mut volume_map: HashMap<String, Vec<Chapter>> = HashMap::new();
        
        for chapter in chapters {
            // For now, put all chapters in a single "default" volume
            // In a real implementation, you'd parse volume information from the API
            volume_map
                .entry("default".to_string())
                .or_insert_with(Vec::new)
                .push(chapter);
        }

        serie.volumes = volume_map
            .into_iter()
            .map(|(vol_id, mut chapters)| {
                chapters.sort_by(|a, b| {
                    b.number
                        .partial_cmp(&a.number)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                Volume {
                    id: VolumeId::new(vol_id),
                    name: MultiLanguageString::new().with_language(Language::English, "All Chapters"),
                    number: None,
                    chapters,
                }
            })
            .collect();

        Ok(serie)
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: &SerieId,
        _volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError> {
        let url = format!("{}/at-home/server/{}", self.api_url, chapter_id.as_str());

        let response: MangaDexChapterPages = self
            .client
            .get_json(&url)
            .await
            .map_err(|e| SourceError::Network(e.to_string()))?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        let images: Vec<ChapterImage> = response
            .chapter
            .data
            .into_iter()
            .enumerate()
            .map(|(idx, filename)| ChapterImage {
                url: format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, filename
                ),
                page: (idx + 1) as u32,
                width: None,
                height: None,
            })
            .collect();

        Ok(ChapterData::Image { images })
    }
}