pub mod types;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dokusho_clients::http::CloudflareAwareHttpClient;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use tokio::time::{Duration, sleep};
use tracing::debug;
use url::Url;

use dokusho_core::{
    FetchSearchSerieFilter, FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort, Source,
    SourceApi, SourceApiInformation, SourceChapters, SourceError, SourceInformation,
    SourceLanguage, SourcePaginatedSmallSerie, SourceSerie, SourceSerieChapter,
    SourceSerieChapterData, SourceSerieChapterId, SourceSerieChapterImage, SourceSerieId,
    SourceSerieType, SupportedFilters, SupportedFiltersGenres,
};

use self::types::{
    MangaDexAtHomeResponse, MangaDexChapter, MangaDexListResponse, MangaDexManga,
    MangaDexSingleResponse, MangadexGenre, MangadexLanguage, MangadexOrder, MangadexSort,
    MangadexStatus,
};
use crate::utils::calculate_missing_chapters;

#[derive(Clone)]
pub struct Mangadex {
    http: CloudflareAwareHttpClient,
    source: Source,
}

impl Mangadex {
    pub fn new(
        enabled_languages: Vec<SourceLanguage>,
        http: CloudflareAwareHttpClient,
    ) -> Result<Self, SourceError> {
        let updated_at =
            DateTime::parse_from_str("2025-08-14T17:10:00+02:00", "%Y-%m-%dT%H:%M:%S%z")
                .map_err(|e| SourceError::BuildingURL(e.to_string()))?
                .with_timezone(&Utc);

        let languages: Vec<SourceLanguage> = MangadexLanguage::iter().map(|v| v.into()).collect();
        let only_enable_supported: Vec<SourceLanguage> = enabled_languages
            .into_iter()
            .filter(|lang| languages.contains(lang))
            .collect();

        let source = Source {
            source_information: SourceInformation {
                id: "Mangadex".into(),
                name: "Mangadex".into(),
                url: Url::parse("https://mangadex.org")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                icon: Url::parse("https://mangadex.org/favicon.ico")
                    .map_err(|e| SourceError::BuildingURL(e.to_string()))?,
                version: "1.0.0".to_string(),
                include_nsfw: true,
                updated_at,
                languages,
                enabled_languages: only_enable_supported,
                search_filters: SupportedFilters {
                    artists: false,
                    authors: false,
                    genres: SupportedFiltersGenres {
                        include: true,
                        exclude: true,
                        accepted_values: MangadexGenre::iter().map(|v| v.into()).collect(),
                    },
                    sort: MangadexSort::iter().map(|v| v.into()).collect(),
                    order: MangadexOrder::iter().map(|v| v.into()).collect(),
                    query: true,
                    status: MangadexStatus::iter().map(|v| v.into()).collect(),
                    types: vec![
                        SourceSerieType::Manga,
                        SourceSerieType::Manhwa,
                        SourceSerieType::Manhua,
                        SourceSerieType::Doujinshi,
                    ],
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Url::parse("https://api.mangadex.org")
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
                minimum_update_interval: 300 * 60,
                timeout: Duration::from_secs(30),
            },
        };

        Ok(Self { http, source })
    }
}

#[async_trait]
impl SourceApi for Mangadex {
    fn get_information(&self) -> SourceInformation {
        self.source.source_information.clone()
    }

    fn get_api_information(&self) -> SourceApiInformation {
        self.source.source_api_information.clone()
    }

    fn serie_url(&self, serie_id: SourceSerieId) -> Result<Url, SourceError> {
        let mut url = self.source.source_information.url.clone();
        url.set_path(&format!("title/{}", serie_id));

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
        filter: FetchSearchSerieFilter,
    ) -> Result<SourcePaginatedSmallSerie, SourceError> {
        let limit = 20;
        let offset = (page - 1) * limit;

        let mut url = self.source.source_api_information.api_url.clone();
        url.set_path("/manga");
        url.query_pairs_mut()
            .append_pair("limit", &limit.to_string())
            .append_pair("offset", &offset.to_string())
            .append_pair("includedTagsMode", "AND")
            .append_pair("excludedTagsMode", "OR")
            .append_pair("contentRating[]", "safe")
            .append_pair("contentRating[]", "suggestive")
            .append_pair("contentRating[]", "erotica")
            .append_pair("includes[]", "cover_art");

        for language in &self.source.source_information.enabled_languages {
            let lang: MangadexLanguage = language.clone().into();
            url.query_pairs_mut()
                .append_pair("availableTranslatedLanguage[]", lang.to_string().as_str());
        }

        if let Some(query) = filter.query {
            url.query_pairs_mut().append_pair("title", &query);
        }

        if let Some(order) = filter.order
            && let Some(sort) = filter.sort
        {
            let mangadex_sort: MangadexSort = sort.into();
            let mangadex_order: MangadexOrder = order.into();
            let key = format!("order[{}]", mangadex_sort);

            url.query_pairs_mut()
                .append_pair(&key, &mangadex_order.to_string());
        }

        if let Some(genre_filter) = filter.genres {
            if self.source.source_information.search_filters.genres.include
                && let Some(includes) = genre_filter.includes
            {
                for include in includes {
                    let mangadex_genre: MangadexGenre = include.into();

                    url.query_pairs_mut()
                        .append_pair("includedTags[]", &mangadex_genre.to_string());
                }
            }
            if self.source.source_information.search_filters.genres.exclude
                && let Some(excludes) = genre_filter.excludes
            {
                for excludes in excludes {
                    let mangadex_genre: MangadexGenre = excludes.into();

                    url.query_pairs_mut()
                        .append_pair("excludedTags[]", &mangadex_genre.to_string());
                }
            }
        }

        if let Some(statuses) = filter.status {
            for status in statuses {
                let mangadex_status: MangadexStatus = status.into();

                url.query_pairs_mut()
                    .append_pair("status[]", &mangadex_status.to_string());
            }
        }

        // TODO(stef): Implement Autors and Artists filtering, this will need to fetch artist and authors from mangadex

        debug!("Fetching MangaDex manga list from URL: {}", url);

        let response: MangaDexListResponse<MangaDexManga> =
            self.http.get_json(&url).await.map_err(|e| {
                tracing::error!("Failed to fetch from MangaDex API: {}", e);
                SourceError::HTTPRequestFailed(format!("MangaDex API request failed: {}", e))
            })?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        Ok(SourcePaginatedSmallSerie {
            has_next_page: response.offset + response.limit < response.total,
            series: response
                .data
                .into_iter()
                .map(|d| d.try_into().expect("Failed conversion"))
                .collect(),
        })
    }

    async fn fetch_serie_detail(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceSerie, SourceError> {
        let mut url = self.source.source_api_information.api_url.clone();
        url.set_path(&format!("/manga/{}", serie_id));
        url.query_pairs_mut()
            .append_pair("includes[]", "author")
            .append_pair("includes[]", "artist")
            .append_pair("includes[]", "cover_art");

        let response: MangaDexSingleResponse<MangaDexManga> = self
            .http
            .get_json(&url)
            .await
            .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        response.data.try_into()
    }

    async fn fetch_serie_chapters(
        &self,
        serie_id: SourceSerieId,
    ) -> Result<SourceChapters, SourceError> {
        let mut all_chapters = Vec::new();
        let mut offset = 0;
        let limit = 500;

        let mut url = self.source.source_api_information.api_url.clone();
        url.set_path(&format!("/manga/{}/feed", serie_id));
        url.query_pairs_mut()
            .append_pair("order[volume]", "desc")
            .append_pair("order[chapter]", "desc")
            .append_pair("limit", &limit.to_string());

        let langs: Vec<MangadexLanguage> = self
            .source
            .source_information
            .enabled_languages
            .iter()
            .map(|lang| (*lang).into())
            .collect();
        for lang in langs {
            url.query_pairs_mut()
                .append_pair("translatedLanguage[]", &lang.to_string());
        }

        loop {
            let mut url = url.clone();

            url.query_pairs_mut()
                .append_pair("offset", &offset.to_string());

            let response: MangaDexListResponse<MangaDexChapter> = self
                .http
                .get_json(&url)
                .await
                .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

            if response.result != "ok" {
                return Err(SourceError::Other(anyhow::anyhow!(
                    "MangaDex API error: {}",
                    response.result
                )));
            }

            all_chapters.extend(response.data);

            if offset + limit >= response.total {
                break;
            }

            offset += limit;

            sleep(Duration::from_millis(500)).await;
        }

        let chapters: Vec<SourceSerieChapter> =
            all_chapters.iter().map(|c| c.clone().into()).collect();
        let chapter_numbers: Vec<f64> = chapters.iter().map(|c| c.chapter_number).collect();

        Ok(SourceChapters {
            missing_chapters: calculate_missing_chapters(&chapter_numbers),
            chapters,
        })
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: SourceSerieId,
        chapter_id: SourceSerieChapterId,
    ) -> Result<SourceSerieChapterData, SourceError> {
        let mut url = self.source.source_api_information.api_url.clone();
        url.set_path(&format!("at-home/server/{chapter_id}"));
        url.query_pairs_mut().append_pair("forcePort443", "false");

        let response: MangaDexAtHomeResponse = self
            .http
            .get_json(&url)
            .await
            .map_err(|e| SourceError::HTTPRequestFailed(e.to_string()))?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        let images: Vec<SourceSerieChapterImage> = response
            .chapter
            .data
            .into_iter()
            .enumerate()
            .map(|(idx, filename)| SourceSerieChapterImage {
                index: (idx + 1) as i16,
                url: Url::parse(&format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, filename
                ))
                .expect("Failed to parse URL"),
            })
            .collect();

        Ok(SourceSerieChapterData::Image(images))
    }
}
