pub mod types;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dokusho_clients::http::CloudflareAwareHttpClient;
use regex::Regex;
use scraper::{Html, Selector};
use std::{collections::HashMap, vec};
use strum::IntoEnumIterator;
use tracing::{debug, warn};
use url::Url;

use dokusho_core::{
    FetchSearchSerieFilter, FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort,
    MultiLanguageString, Source, SourceApi, SourceApiInformation, SourceChapters, SourceError,
    SourceInformation, SourceLanguage, SourcePaginatedSmallSerie, SourceSerie, SourceSerieChapter,
    SourceSerieChapterData, SourceSerieChapterId, SourceSerieChapterImage, SourceSerieGenre,
    SourceSerieId, SourceSerieStatus, SourceSerieType, SourceSmallSerie, SupportedFilters,
    SupportedFiltersGenres,
};

use self::types::{
    WeebCentralGenre, WeebCentralOrder, WeebCentralSort, WeebCentralStatus, WeebCentralType,
};
use crate::utils::calculate_missing_chapters;

pub struct WeebCentral {
    client: CloudflareAwareHttpClient,
    source: Source,
    chapter_number_regex: Regex,
}

impl WeebCentral {
    pub fn new(
        enabled_languages: Vec<SourceLanguage>,
        flaresolver_url: Option<Url>,
    ) -> Result<Self, SourceError> {
        let mut client =
            CloudflareAwareHttpClient::new().map_err(|e| SourceError::Other(e.into()))?;

        if let Some(flaresolver_url) = flaresolver_url {
            client = client
                .with_flaresolver(flaresolver_url)
                .map_err(|e| SourceError::Other(e.into()))?;
        }

        let chapter_number_regex = Regex::new(r"\d+(\.\d+)?")
            .map_err(|e| SourceError::Other(anyhow::anyhow!("Failed to compile regex: {}", e)))?;

        let updated_at =
            DateTime::parse_from_str("2025-08-14T17:10:00+02:00", "%Y-%m-%dT%H:%M:%S%z")
                .map_err(|e| SourceError::BuildingURL(e.to_string()))?
                .with_timezone(&Utc);

        // WeebCentral only supports English
        let languages = vec![SourceLanguage::En];
        let only_enable_supported: Vec<SourceLanguage> = enabled_languages
            .into_iter()
            .filter(|lang| languages.contains(lang))
            .collect();

        let source = Source {
            source_information: SourceInformation {
                id: "weebcentral".into(),
                name: "WeebCentral".to_string(),
                url: Url::parse("https://weebcentral.com")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                icon: Url::parse("https://weebcentral.com/favicon.ico")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                version: "1.0.0".to_string(),
                include_nsfw: true,
                updated_at,
                languages,
                enabled_languages: only_enable_supported,
                search_filters: SupportedFilters {
                    query: true,
                    artists: true,
                    authors: true,
                    genres: SupportedFiltersGenres {
                        include: true,
                        exclude: true,
                        accepted_values: WeebCentralGenre::iter().map(|v| v.into()).collect(),
                    },
                    sort: WeebCentralSort::iter().map(|v| v.into()).collect(),
                    order: WeebCentralOrder::iter().map(|v| v.into()).collect(),
                    status: WeebCentralStatus::iter().map(|v| v.into()).collect(),
                    types: WeebCentralType::iter().map(|v| v.into()).collect(),
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Url::parse("https://weebcentral.com")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                headers: {
                    let mut headers = HashMap::new();
                    headers.insert(
                        "User-Agent".to_string(),
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:71.0) Gecko/20100101 Firefox/77.0".to_string(),
                    );
                    headers
                },
                can_block_scraping: true,
                minimum_update_interval: 300,
                timeout: tokio::time::Duration::from_secs(30),
            },
        };

        Ok(Self {
            client,
            source,
            chapter_number_regex,
        })
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

    fn serie_url(&self, serie_id: SourceSerieId) -> Result<Url, SourceError> {
        let mut url = self.source.source_information.url.clone();
        url.set_path(&format!("series/{}", serie_id));
        Ok(url)
    }

    async fn fetch_popular_serie(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        self.fetch_search_serie(
            page,
            FetchSearchSerieFilter {
                sort: Some(FetchSearchSerieFilterSort::Popularity),
                order: Some(FetchSearchSerieFilterOrder::DESC),
                ..Default::default()
            },
        )
        .await
    }

    async fn fetch_latest_updates(
        &self,
        page: i16,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        self.fetch_search_serie(
            page,
            FetchSearchSerieFilter {
                sort: Some(FetchSearchSerieFilterSort::Latest),
                order: Some(FetchSearchSerieFilterOrder::DESC),
                ..Default::default()
            },
        )
        .await
    }

    async fn fetch_search_serie(
        &self,
        page: i16,
        filters: FetchSearchSerieFilter,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        let limit = 24;
        let offset = (page - 1) * limit;
        let mut url = self.source.source_api_information.api_url.clone();
        url.set_path("/search/data");
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string())
            .append_pair("offset", &offset.to_string())
            .append_pair("official", "Any")
            .append_pair("display_mode", "Full Display");

        if let Some(query) = filters.query {
            url.query_pairs_mut().append_pair("text", &query);
        }

        if let Some(source_sort) = filters.sort {
            let sort: WeebCentralSort = source_sort.into();
            url.query_pairs_mut().append_pair("sort", &sort.to_string());
        }

        if let Some(source_order) = filters.order {
            let order: WeebCentralOrder = source_order.into();
            url.query_pairs_mut()
                .append_pair("order", &order.to_string());
        }

        if let Some(source_types) = filters.types {
            for source_type in source_types {
                let wc_type: Result<WeebCentralType, _> = source_type.try_into();
                if let Ok(wc_type) = wc_type {
                    url.query_pairs_mut()
                        .append_pair("included_type", &wc_type.to_string());
                }
            }
        }

        if let Some(source_statuses) = filters.status {
            for source_status in source_statuses {
                let wc_status: Result<WeebCentralStatus, _> = source_status.try_into();
                if let Ok(wc_status) = wc_status {
                    url.query_pairs_mut()
                        .append_pair("included_type", &wc_status.to_string());
                }
            }
        }

        if let Some(genres_filter) = filters.genres {
            if self.source.source_information.search_filters.genres.include
                && let Some(genres) = genres_filter.includes
            {
                for source_genre in genres {
                    let wc_genre: Result<WeebCentralGenre, _> = source_genre.try_into();
                    if let Ok(wc_genre) = wc_genre {
                        url.query_pairs_mut()
                            .append_pair("included_tag", &wc_genre.to_string());
                    }
                }
            }

            if self.source.source_information.search_filters.genres.exclude
                && let Some(genres) = genres_filter.excludes
            {
                for source_genre in genres {
                    let wc_genre: Result<WeebCentralGenre, _> = source_genre.try_into();
                    if let Ok(wc_genre) = wc_genre {
                        url.query_pairs_mut()
                            .append_pair("excluded_tag", &wc_genre.to_string());
                    }
                }
            }
        }

        if let Some(artists) = filters.artists {
            for artist in artists {
                url.query_pairs_mut().append_pair("author", &artist);
            }
        }

        if let Some(authors) = filters.authors {
            for author in authors {
                url.query_pairs_mut().append_pair("author", &author);
            }
        }

        debug!(url = url.as_str(), "weebcentral url");

        let html = self
            .client
            .get_html(&url)
            .await
            .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

        let document = Html::parse_document(html.as_str());
        let mut series: Vec<SourceSmallSerie> = Vec::new();

        let article_selector =
            Selector::parse("body > article").expect("Failed to parse article selector");

        let no_image_url: Url = Url::parse("https://i.imgur.com/6TrIues.jpeg").unwrap();

        for article in document.select(&article_selector) {
            let cover_selector =
                Selector::parse("section:first-child a > article > picture > source").unwrap();
            let cover = article
                .select(&cover_selector)
                .next()
                .and_then(|el| el.value().attr("srcset"))
                .and_then(|str| Url::parse(str).ok())
                .unwrap_or_else(|| no_image_url.clone());

            let title_selector = Selector::parse("section:last-child div:first-child a").unwrap();
            let title = article
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .map(|title| {
                    MultiLanguageString::new().insert(SourceLanguage::En, title.trim().to_string())
                })
                .unwrap_or_else(|| {
                    MultiLanguageString::new()
                        .insert(SourceLanguage::En, "Unknown Title".to_string())
                });

            let link_selector = Selector::parse("section:first-child a").unwrap();
            let href = article
                .select(&link_selector)
                .next()
                .and_then(|el| el.value().attr("href"))
                .ok_or_else(|| SourceError::ExtractingData("Missing href attribute".to_string()))?;

            let parts: Vec<&str> = href.split('/').collect();
            if parts.len() < 2 {
                continue;
            }
            let serie_id = parts[parts.len() - 2];

            series.push(SourceSmallSerie {
                id: serie_id.to_string(),
                title,
                cover,
            });
        }

        let has_next_page = html.contains("View More Results...");

        Ok(SourcePaginatedSmallSerie {
            has_next_page,
            series,
        })
    }

    async fn fetch_serie_detail(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceSerie, SourceError> {
        let url = self
            .serie_url(serie_id.clone())
            .expect("error build serie url");

        let html = self
            .client
            .get_html(&url)
            .await
            .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

        let document = Html::parse_document(html.as_str());
        let main_selector = Selector::parse("body > main > div#top").unwrap();
        let main = document
            .select(&main_selector)
            .next()
            .ok_or_else(|| SourceError::ExtractingData("Main content not found".to_string()))?;

        // Extract title
        let title_selector = Selector::parse("h1").unwrap();
        let title = main
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .map(|title| {
                MultiLanguageString::new().insert(SourceLanguage::En, title.trim().to_string())
            })
            .unwrap_or_default();

        // Extract cover
        let cover_selector = Selector::parse("img").unwrap();
        let cover_url = main
            .select(&cover_selector)
            .next()
            .and_then(|img| img.value().attr("src"))
            .unwrap_or_default()
            .to_string();

        let mut synopsis = MultiLanguageString::new();
        let mut type_text = String::new();
        let mut status_text = String::new();
        let mut genres: Vec<SourceSerieGenre> = Vec::new();
        let mut authors = Vec::new();
        let mut alternates_titles = MultiLanguageString::new();
        let li_selector = Selector::parse("li").expect("weebcentral type selector failed");
        for li in main.select(&li_selector) {
            let text = li.text().collect::<String>();
            match text.trim() {
                text if text.contains("Description") => {
                    let p_selector = Selector::parse("p").unwrap();
                    if let Some(p) = li.select(&p_selector).next() {
                        synopsis = synopsis.insert(
                            SourceLanguage::En,
                            p.text().collect::<String>().trim().to_string(),
                        );
                    }
                }
                text if text.starts_with("Type") => {
                    let a_selector = Selector::parse("a").unwrap();
                    if let Some(a) = li.select(&a_selector).next() {
                        type_text = a.text().collect::<String>();
                    }
                }
                text if text.starts_with("Status") => {
                    let a_selector = Selector::parse("a").unwrap();
                    if let Some(a) = li.select(&a_selector).next() {
                        status_text = a.text().collect::<String>();
                    }
                }
                text if text.starts_with("Tags") => {
                    let a_selector = Selector::parse("span > a").unwrap();
                    for a in li.select(&a_selector) {
                        let genre_text = a.text().collect::<String>();
                        if let Ok(wc_genre) = WeebCentralGenre::try_from(genre_text.trim()) {
                            genres.push(wc_genre.into());
                        } else {
                            warn!(genre = genre_text, "Unknown genre");
                        }
                    }
                }
                text if text.starts_with("Author") => {
                    let a_selector = Selector::parse("span > a").unwrap();
                    for a in li.select(&a_selector) {
                        authors.push(a.text().collect::<String>().trim().to_string());
                    }
                }
                text if text.starts_with("Associated Name") => {
                    let selector = Selector::parse("ul > li").unwrap();
                    for el in li.select(&selector) {
                        alternates_titles = alternates_titles.insert(
                            SourceLanguage::En,
                            el.text().collect::<String>().trim().to_string(),
                        );
                    }
                }
                _ => continue,
            }
        }

        let cover = Url::parse(&cover_url)
            .unwrap_or_else(|_| Url::parse("https://i.imgur.com/6TrIues.jpeg").unwrap());

        let serie_type = WeebCentralType::try_from(type_text.trim())
            .map(|wc_type| wc_type.into())
            .unwrap_or(SourceSerieType::Unknown);
        if serie_type == SourceSerieType::Unknown {
            warn!(type = type_text, "Unknown serie type");
        }

        let status = WeebCentralStatus::try_from(status_text.trim())
            .ok()
            .map(|wc_status| wc_status.into())
            .unwrap_or(SourceSerieStatus::Unknown);
        if status == SourceSerieStatus::Unknown {
            warn!(status = status_text, "Unknown serie status");
        }

        Ok(SourceSerie {
            id: serie_id,
            title,
            alternates_titles,
            cover,
            synopsis,
            serie_type,
            genres,
            status: vec![status],
            authors,
            artists: vec![],
        })
    }

    async fn fetch_serie_chapters(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceChapters, SourceError> {
        let mut chapters_url = self.serie_url(serie_id)?;
        chapters_url
            .path_segments_mut()
            .expect("cannot be base")
            .push("full-chapter-list");

        let html = self
            .client
            .get_html(&chapters_url)
            .await
            .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

        let document = Html::parse_document(&html);
        let mut chapters = Vec::new();

        // Select all chapter links
        let chapter_selector = Selector::parse("body > * > a.flex").unwrap();

        for element in document.select(&chapter_selector) {
            let href = element
                .value()
                .attr("href")
                .ok_or_else(|| SourceError::ExtractingData("Missing chapter href".to_string()))?;

            let chapter_id = href
                .split('/')
                .filter(|s| !s.is_empty())
                .next_back()
                .ok_or_else(|| {
                    SourceError::ExtractingData("Failed to extract chapter ID".to_string())
                })?;

            // Extract chapter name
            let name_selector = Selector::parse("span.flex > span").unwrap();
            let name = element
                .select(&name_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_else(|| format!("Chapter {}", chapter_id));

            // Extract chapter number from name using regex
            let chapter_number = self
                .chapter_number_regex
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

            chapters.push(SourceSerieChapter {
                id: chapter_id.to_string(),
                title: name,
                chapter_number,
                volume_number: None,
                volume_name: None,
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

        // Calculate missing chapters
        let chapter_numbers: Vec<f64> = chapters.iter().map(|c| c.chapter_number).collect();
        let missing_chapters = calculate_missing_chapters(&chapter_numbers);

        Ok(SourceChapters {
            missing_chapters,
            chapters,
        })
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: SourceSerieId,
        chapter_id: SourceSerieChapterId,
    ) -> Result<SourceSerieChapterData, SourceError> {
        let mut url = self.source.source_information.url.clone();
        url.path_segments_mut()
            .map_err(|()| SourceError::BuildingURL("".to_string()))?
            .push("chapters")
            .push(chapter_id.as_str())
            .push("images");
        url.query_pairs_mut()
            .append_pair("reading_style", "long_strip");

        let html = self
            .client
            .get_html(&url)
            .await
            .map_err(|e| SourceError::Other(anyhow::anyhow!("HTTP error: {}", e)))?;

        let document = Html::parse_document(&html);
        let mut images = Vec::new();

        let img_selector = Selector::parse("img").unwrap();

        for element in document.select(&img_selector) {
            if let Some(src) = element.value().attr("src")
                && !src.is_empty()
                && !src.contains("broken_image")
                && let Ok(url) = Url::parse(src)
            {
                images.push(url);
            }
        }

        let images: Vec<SourceSerieChapterImage> = images
            .into_iter()
            .enumerate()
            .map(|(idx, url)| SourceSerieChapterImage {
                index: (idx + 1) as i16,
                url,
            })
            .collect();

        Ok(SourceSerieChapterData::Image(images))
    }
}

#[cfg(test)]
mod tests {
    use dokusho_core::{SourceApi, SourceLanguage, SourceSerieChapterData};

    use crate::scrapers::WeebCentral;

    #[tokio::test]
    async fn test_fetch_serie_detail() {
        let client =
            WeebCentral::new(vec![SourceLanguage::En], None).expect("Failed to create client");
        let serie = client
            .fetch_serie_detail("01J76XYD9NRZYHRQPENCD0HPJG".to_string())
            .await
            .unwrap();
        assert_eq!(serie.id, "01J76XYD9NRZYHRQPENCD0HPJG");
        assert!(serie.title.get(SourceLanguage::En).is_some());
        assert!(serie.alternates_titles.get(SourceLanguage::En).is_some());
        assert!(serie.synopsis.get(SourceLanguage::En).is_some());
        assert_eq!(serie.status.len(), 1);
        assert!(!serie.genres.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_serie_chaptes() {
        let client =
            WeebCentral::new(vec![SourceLanguage::En], None).expect("Failed to create client");
        let data = client
            .fetch_serie_chapters("01J76XYD9NRZYHRQPENCD0HPJG".to_string())
            .await
            .unwrap();

        assert!(data.chapters.len() > 1);
    }

    #[tokio::test]
    async fn test_fetch_chapter_data() {
        let client =
            WeebCentral::new(vec![SourceLanguage::En], None).expect("Failed to create client");
        let data = client
            .fetch_chapter_data(
                "01J76XYD9NRZYHRQPENCD0HPJG".to_string(),
                "01K2QJDBZ7A9JJM0E7B1NNWJR4".to_string(),
            )
            .await
            .unwrap();

        if let SourceSerieChapterData::Image(images) = data {
            assert!(!images.is_empty());
        }
    }
}
