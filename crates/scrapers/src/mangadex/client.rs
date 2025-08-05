use std::collections::HashMap;

use async_trait::async_trait;
use tracing::debug;

use dokusho_clients::HttpClient;
use dokusho_core::{
    Chapter, ChapterData, ChapterId, ChapterImage, FilterOrder, FilterSort, MultiLanguageString,
    PaginatedSmallSeries, SearchFilters, Serie, SerieId, SmallSerie, Source, SourceApi,
    SourceApiInformation, SourceError, SourceInformation, SourceLanguage, SourceSerieGenre,
    SourceSerieType, SupportedFilters, SupportedFiltersGenres, Volume, VolumeId,
};

use super::types::*;
use crate::chapter_utils::calculate_missing_chapters;
use std::convert::TryInto;

const NO_IMAGE_URL: &str = "https://i.imgur.com/6TrIues.jpeg";

pub struct MangaDex {
    client: HttpClient,
    source: Source,
    api_url: String,
}

impl MangaDex {
    pub fn new() -> Result<Self, SourceError> {
        let client = HttpClient::new()?;

        let source = Source {
            source_information: SourceInformation {
                id: "mangadex".into(),
                name: "MangaDex".to_string(),
                url: "https://mangadex.org".to_string(),
                icon: "https://mangadex.org/favicon.ico".to_string(),
                languages: vec![
                    SourceLanguage::En,
                    SourceLanguage::Fr,
                    SourceLanguage::Jp,
                    SourceLanguage::Ko,
                    SourceLanguage::Zh,
                    SourceLanguage::ZhHk,
                ],
                updated_at: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                nsfw: true,
                search_filters: SupportedFilters {
                    query: true,
                    orders: vec![FilterOrder::Ascending, FilterOrder::Descending],
                    sorts: vec![
                        FilterSort::Title,
                        FilterSort::UpdatedAt,
                        FilterSort::CreatedAt,
                        FilterSort::Rating,
                        FilterSort::Popularity,
                    ],
                    artists: true,
                    authors: true,
                    types: vec![
                        SourceSerieType::Manga,
                        SourceSerieType::Manhwa,
                        SourceSerieType::Manhua,
                        SourceSerieType::Doujinshi,
                        SourceSerieType::Comic,
                        SourceSerieType::Novel,
                    ],
                    genres: SupportedFiltersGenres {
                        included: true,
                        excluded: true,
                        possible_values: get_searchable_genres(),
                    },
                    status: get_searchable_status(),
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Some("https://api.mangadex.org".to_string()),
                headers: None,
                minimum_update_interval: std::time::Duration::from_secs(300), // 5 minutes
                timeout: std::time::Duration::from_secs(30),
                can_block_scraping: true,
            },
        };

        Ok(Self {
            client,
            source,
            api_url: "https://api.mangadex.org".to_string(),
        })
    }

    #[cfg(test)]
    pub fn new_with_url(api_url: &str) -> Self {
        let client = HttpClient::new().unwrap();
        let source = Source {
            source_information: SourceInformation {
                id: "mangadex".into(),
                name: "MangaDex".to_string(),
                url: api_url.to_string(),
                icon: "https://mangadex.org/favicon.ico".to_string(),
                languages: vec![SourceLanguage::En],
                updated_at: chrono::Utc::now(),
                version: "1.0.0".to_string(),
                nsfw: true,
                search_filters: SupportedFilters {
                    query: true,
                    orders: vec![FilterOrder::Descending],
                    sorts: vec![FilterSort::UpdatedAt],
                    artists: false,
                    authors: false,
                    types: vec![],
                    genres: SupportedFiltersGenres {
                        included: false,
                        excluded: false,
                        possible_values: vec![],
                    },
                    status: vec![],
                },
            },
            source_api_information: SourceApiInformation {
                api_url: Some(api_url.to_string()),
                headers: None,
                minimum_update_interval: std::time::Duration::from_secs(300),
                timeout: std::time::Duration::from_secs(30),
                can_block_scraping: true,
            },
        };

        Self {
            client,
            source,
            api_url: api_url.to_string(),
        }
    }

    fn convert_to_small_serie(&self, manga: MangaDexManga) -> SmallSerie {
        let mut title = MultiLanguageString::new();

        // Add main title
        for (lang_code, text) in &manga.attributes.title {
            if let Some(lang) = self.parse_language(lang_code) {
                title = title.with_language(lang, text);
            }
        }

        // Add alternative titles if main language is missing
        for alt_title in &manga.attributes.alt_titles {
            for (lang_code, text) in alt_title {
                if let Some(lang) = self.parse_language(lang_code) {
                    if title.get(lang).is_none() {
                        title = title.with_language(lang, text);
                    }
                }
            }
        }

        SmallSerie {
            id: SerieId::new(manga.id.clone()),
            title,
            cover: manga.get_cover_url().unwrap_or_else(|| NO_IMAGE_URL.to_string()),
        }
    }

    fn convert_to_serie(&self, manga: MangaDexManga) -> Serie {
        let mut title = MultiLanguageString::new();
        let mut synopsis = MultiLanguageString::new();

        // Process titles - handle all language fields properly
        for (lang_code, text) in &manga.attributes.title {
            if let Some(lang) = self.parse_language(lang_code) {
                title = title.with_language(lang, text);
            }
        }

        // Process descriptions
        for (lang_code, text) in &manga.attributes.description {
            if let Some(lang) = self.parse_language(lang_code) {
                synopsis = synopsis.with_language(lang, text);
            }
        }

        // Process alternative titles
        let mut alternative_titles = Vec::new();
        for alt_title_map in &manga.attributes.alt_titles {
            let mut alt_title = MultiLanguageString::new();
            for (lang_code, text) in alt_title_map {
                if let Some(lang) = self.parse_language(lang_code) {
                    alt_title = alt_title.with_language(lang, text);
                }
            }
            // Only add non-empty alternative titles
            if alt_title.en.is_some() || alt_title.jp.is_some() || alt_title.jp_ro.is_some() 
                || alt_title.ko.is_some() || alt_title.zh.is_some() || alt_title.zh_hk.is_some() 
                || alt_title.fr.is_some() {
                alternative_titles.push(alt_title);
            }
        }

        // Extract genres from tags
        let genres: Vec<SourceSerieGenre> = manga
            .attributes
            .tags
            .iter()
            .filter(|tag| tag.attributes.group == "genre")
            .filter_map(|tag| self.parse_genre_from_tag_id(&tag.id))
            .collect();

        // Extract authors and artists from relationships
        let mut authors = Vec::new();
        let mut artists = Vec::new();

        if let Some(relationships) = &manga.relationships {
            for rel in relationships {
                match rel.rel_type.as_str() {
                    "author" => {
                        if let Some(attrs) = &rel.attributes {
                            if let Some(name) = attrs.get("name").and_then(|v| v.as_str()) {
                                authors.push(name.to_string());
                            }
                        }
                    }
                    "artist" => {
                        if let Some(attrs) = &rel.attributes {
                            if let Some(name) = attrs.get("name").and_then(|v| v.as_str()) {
                                artists.push(name.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Get status - handle both status and state
        let mut statuses = Vec::new();
        
        // Add status
        if let Ok(status) = MangadexStatus::try_from(manga.attributes.status.as_str()) {
            if let Ok(source_status) = status.try_into() {
                statuses.push(source_status);
            }
        }
        
        // Add state if present
        if let Ok(state) = MangadexStatus::try_from(manga.attributes.state.as_str()) {
            if let Ok(source_state) = state.try_into() {
                statuses.push(source_state);
            }
        }

        // Get type based on original language and genres
        let serie_type = self.get_type(&manga.attributes.original_language, &genres);

        Serie {
            id: SerieId::new(manga.id.clone()),
            title,
            alternative_titles: if alternative_titles.is_empty() { None } else { Some(alternative_titles) },
            cover: manga.get_cover_url().unwrap_or_else(|| NO_IMAGE_URL.to_string()),
            synopsis,
            serie_type,
            genres,
            status: statuses,
            authors,
            artists,
            volumes: Vec::new(), // Will be populated separately
        }
    }



    fn parse_language(&self, code: &str) -> Option<SourceLanguage> {
        use MangadexLanguage::*;
        let mangadex_lang = match code {
            "en" => En,
            "fr" => Fr,
            "ko" => Ko,
            "ja" => Ja,
            "ja-ro" => JaRo,
            "zh-hk" => ZhHk,
            "zh" | "zh-cn" => Zh,
            _ => return None,
        };
        
        mangadex_lang.try_into().ok()
    }

    fn parse_language_to_mangadex(&self, code: &str) -> Option<MangadexLanguage> {
        use MangadexLanguage::*;
        match code {
            "en" => Some(En),
            "fr" => Some(Fr),
            "ko" => Some(Ko),
            "ja" => Some(Ja),
            "ja-ro" => Some(JaRo),
            "zh-hk" => Some(ZhHk),
            "zh" | "zh-cn" => Some(Zh),
            _ => None,
        }
    }

    async fn fetch_all_chapters_with_languages(
        &self,
        manga_id: &str,
        languages: &[MangadexLanguage],
    ) -> Result<Vec<MangaDexChapter>, SourceError> {
        let mut all_chapters = Vec::new();
        let mut offset = 0;
        let limit = 500;

        loop {
            let mut url = format!(
                "{}/manga/{}/feed?limit={}&offset={}&order[volume]=desc&order[chapter]=desc",
                self.api_url, manga_id, limit, offset
            );

            // Add language filters
            for lang in languages {
                url.push_str(&format!("&translatedLanguage[]={}", lang.as_str()));
            }

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

            all_chapters.extend(response.data);

            if offset + limit >= response.total {
                break;
            }

            offset += limit;
            
            // Rate limit
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        Ok(all_chapters)
    }

    fn convert_mangadex_chapters(&self, chapters: Vec<MangaDexChapter>) -> Vec<Volume> {
        let mut volume_map: HashMap<String, Vec<Chapter>> = HashMap::new();

        for chapter in chapters {
            let volume_num = chapter.attributes.volume
                .clone()
                .unwrap_or_else(|| "0".to_string());

            let chapter_number = chapter.attributes.chapter
                .as_ref()
                .and_then(|c| c.parse::<f64>().ok())
                .unwrap_or(0.0);

            let title = chapter.attributes.title
                .clone()
                .unwrap_or_else(|| format!("Chapter {}", chapter_number));

            let language = self.parse_language(&chapter.attributes.translated_language)
                .unwrap_or(SourceLanguage::En);

            let chapter_id = chapter.id.clone();
            let converted_chapter = Chapter {
                id: ChapterId::new(chapter_id.clone()),
                name: title,
                chapter_number,
                language,
                date_upload: chapter.attributes.publish_at,
                external_url: Some(format!("https://mangadex.org/chapter/{}", chapter_id)),
            };

            volume_map
                .entry(volume_num)
                .or_insert_with(Vec::new)
                .push(converted_chapter);
        }

        // Convert to sorted volumes
        let mut volumes: Vec<Volume> = volume_map
            .into_iter()
            .map(|(vol_num, mut chapters)| {
                // Sort chapters by number descending
                chapters.sort_by(|a, b| {
                    b.chapter_number
                        .partial_cmp(&a.chapter_number)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let volume_number = vol_num.parse::<f64>().unwrap_or(0.0);
                
                // Calculate missing chapters for this volume
                let chapter_numbers: Vec<f64> = chapters.iter().map(|c| c.chapter_number).collect();
                let missing_chapters = calculate_missing_chapters(&chapter_numbers);
                
                Volume {
                    id: VolumeId::new(format!("volume-{}", vol_num)),
                    name: format!("Volume {}", vol_num),
                    volume_number,
                    missing_chapters,
                    chapters,
                }
            })
            .collect();

        // Sort volumes by number descending
        volumes.sort_by(|a, b| {
            b.volume_number
                .partial_cmp(&a.volume_number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        volumes
    }

    fn parse_genre_from_tag_id(&self, tag_id: &str) -> Option<SourceSerieGenre> {
        // Find the MangadexGenre that matches this tag ID
        use MangadexGenre::*;
        let mangadex_genre = match tag_id {
            "fad12b5e-68ba-460e-b933-9ae8318f5b65" => Gyaru,
            "f8f62932-27da-4fe4-8ee1-6779a8c5edba" => Tragedy,
            "f5ba408b-0e7a-484d-8d49-4e9125ac96de" => FullColor,
            "f42fbf9e-188a-447b-9fdc-f19dc1e4d685" => Music,
            "f4122d1c-3b44-44d0-9936-ff7502c39ad3" => Adaptation,
            "ee968100-4191-4968-93d3-f82d72be7e46" => Mystery,
            "eabc5b4c-6aff-42f3-b657-3e90cbd00b75" => Supernatural,
            "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869" => Cooking,
            "e64f6742-c834-471d-8d72-dd51fc02b835" => Aliens,
            "e5301a23-ebd9-49dd-a0cb-2add944c7fe9" => SliceOfLife,
            "e197df38-d0e7-43b5-9b09-2842d0c326dd" => WebComic,
            "df33b754-73a3-4c54-80e6-1a74a8058539" => Police,
            "0a39b5a1-b235-4886-a747-1d05d216532d" => AwardWinning,
            "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0" => Reincarnation,
            "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a" => Genderswap,
            "2d1f5d56-a1e5-4d0d-a961-2193588b08ec" => Lolicon,
            "3b60b75c-a2d7-4860-ab56-05f391bb889c" => Psychological,
            "3bb26d85-09d5-4d2e-880c-c34b974339e9" => Ghost,
            "3de8c75d-8ee3-48ff-98ee-e20a65c86451" => Animals,
            "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d" => LongStrip,
            "4d32cc48-9f00-4cca-9b5a-a839f0764984" => Comedy,
            "5bd0e105-4481-44ca-b6e7-7544da56b1a3" => Incest,
            "5ca48985-9a9d-4bd8-be29-80dc0303db72" => Crime,
            "5fff9cde-849c-4d78-aab0-0d52b2ee1d25" => Survival,
            "7b2ce280-79ef-4c09-9b58-12b7c23a9b78" => FanColored,
            "8c86611e-fab7-4986-9dec-d1a2f44acdd5" => VirtualReality,
            "9ab53f92-3eed-4e9b-903a-917c86035ee3" => Crossdressing,
            "36fd93ea-e8b8-445e-b836-358f02b3d33d" => Monsters,
            "51d83883-4103-437c-b4b1-731cb73d786c" => Anthology,
            "81c836c9-914a-4eca-981a-560dad663e73" => MagicalGirls,
            "85daba54-a71c-4554-8a28-9901a8b0afad" => Mafia,
            "87cc87cd-a395-47af-b27a-93258283bbc6" => Adventure,
            "92d6d951-ca5e-429c-ac78-451071cbf064" => OfficeWorkers,
            "0234a31e-a729-4e28-9d6a-3f87c4966b9e" => OneShot,
            "256c8bd9-4904-4360-bf4f-508a76d67183" => SciFi,
            "292e862b-2d17-4062-90a2-0356caa4ae27" => TimeTravel,
            "391b0423-d847-456f-aff0-8b0cfc03066b" => Action,
            "423e2eae-a7a2-4a8b-ac03-a8351462d71d" => Romance,
            "489dd859-9b61-4c37-af75-5b18e88daafc" => Ninja,
            "631ef465-9aba-4afb-b0fc-ea10efe274a8" => Zombies,
            "799c202e-7daa-44eb-9cf7-8a3c0441531e" => MartialArts,
            "891cf039-b895-47f0-9229-bef4c96eccd4" => SelfPublished,
            "5920b825-4181-4a17-beeb-9918b0ff7a30" => BoysLove,
            "7064a261-a137-4d3a-8848-2d385de3a99c" => Superhero,
            "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8" => VideoGames,
            "31932a7e-5b8e-49a6-9f12-2afa39dc544c" => TraditionalGames,
            "50880a9d-5440-4732-9afb-8f457127e836" => Mecha,
            "65761a2a-415e-47f3-bef2-a9dababba7a6" => ReverseHarem,
            "69964a64-2f90-4d33-beeb-f3ed2875eb4c" => Sports,
            "97893a4c-12af-4dac-b6be-0dffb353568e" => SexualViolence,
            "320831a8-4026-470b-94f6-8353740e6f04" => OfficialColored,
            "07251805-a27e-4d59-b488-f0bfbec15168" => Thriller,
            "9467335a-1b83-4497-9231-765337a00b96" => PostApocalyptic,
            "33771934-028e-4cb3-8744-691e866a923e" => Historical,
            "39730448-9a5f-48a2-85b0-a70db87b1233" => Demons,
            "81183756-1453-4c81-aa9e-f6e1b63be016" => Samurai,
            "a1f53773-c69a-4ce5-8cab-fffcd90b1565" => Magic,
            "a3c67850-4684-404e-9b7f-c69850ee5da6" => GirlsLove,
            "aafb99c1-7f60-43fa-b75f-fc9502ce29c7" => Harem,
            "ac72833b-c4e9-4878-b9db-6c8a4a99444a" => Military,
            "acc803a4-c95a-4c22-86fc-eb6b582d82a2" => Wuxia,
            "ace04997-f6bd-436e-b261-779182193d3d" => Isekai,
            "b1e97889-25b4-4258-b28b-cd7f4d28ea9b" => Philosophical,
            "b9af3a63-f058-46de-a9a0-e0c13906197a" => Drama,
            "b11fda93-8f1d-4bef-b2ed-8803d3733170" => FourKoma,
            "b13b2a48-c720-44a9-9c77-39c9979373fb" => Doujinshi,
            "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d" => Gore,
            "c8cbe35b-1b2b-4a3f-9c37-db84c4514856" => Medical,
            "caaa44eb-cd40-4177-b930-79d3ef2afe87" => SchoolLife,
            "cdad7e68-1419-41dd-bdce-27753074a640" => Horror,
            "cdc58593-87dd-415e-bbc0-2ec27bf404cc" => Fantasy,
            "d7d1730f-6eb0-4ba6-9437-602cac38664c" => Vampires,
            "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41" => Villainess,
            "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea" => Delinquents,
            "dd1f77c5-dea9-4e2b-97ae-224af09caf99" => MonsterGirls,
            "ddefd648-5140-4e5f-ba18-4eca4071d19b" => Shotacon,
            _ => return None,
        };
        
        mangadex_genre.try_into().ok()
    }
}

#[async_trait]
impl SourceApi for MangaDex {
    fn get_information(&self) -> SourceInformation {
        self.source.source_information.clone()
    }

    fn get_api_information(&self) -> SourceApiInformation {
        self.source.source_api_information.clone()
    }

    async fn fetch_popular_series(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        self.search_series(
            page,
            SearchFilters {
                sort: FilterSort::Popularity,
                order: FilterOrder::Descending,
                ..Default::default()
            },
        )
        .await
    }

    async fn fetch_latest_updates(&self, page: i32) -> Result<PaginatedSmallSeries, SourceError> {
        self.search_series(
            page,
            SearchFilters {
                sort: FilterSort::UpdatedAt,
                order: FilterOrder::Descending,
                ..Default::default()
            },
        )
        .await
    }

    async fn search_series(
        &self,
        page: i32,
        filters: SearchFilters,
    ) -> Result<PaginatedSmallSeries, SourceError> {
        let limit = 20;
        let offset = (page - 1) * limit;

        let mut url = format!(
            "{}/manga?limit={}&offset={}&includes[]=cover_art",
            self.api_url, limit, offset
        );

        // Add query
        if !filters.query.is_empty() {
            url.push_str(&format!("&title={}", urlencoding::encode(&filters.query)));
        }

        // Add order and sort using proper conversions
        if let Ok(mangadex_sort) = filters.sort.try_into() {
            let sort_param: MangadexSort = mangadex_sort;
            let order_param: MangadexOrder = filters.order.into();
            
            url.push_str(&format!("&order[{}]={}", sort_param.as_str(), order_param.as_str()));
        } else {
            // Default to latest updated
            url.push_str("&order[latestUploadedChapter]=desc");
        }

        // Add genres
        for genre in &filters.genres.include {
            if let Some(tag_id) = self.get_genre_tag_id(genre) {
                url.push_str(&format!("&includedTags[]={}", tag_id));
            }
        }

        for genre in &filters.genres.exclude {
            if let Some(tag_id) = self.get_genre_tag_id(genre) {
                url.push_str(&format!("&excludedTags[]={}", tag_id));
            }
        }

        // Add status using proper conversions
        for status in &filters.status {
            if let Ok(mangadex_status) = (*status).try_into() {
                let status_enum: MangadexStatus = mangadex_status;
                url.push_str(&format!("&status[]={}", status_enum.as_str()));
            }
        }

        // Add types - MangaDex doesn't filter by type in search, it's determined by originalLanguage
        // We can add content rating filters if needed
        if filters.types.iter().any(|t| matches!(t, SourceSerieType::Doujinshi)) {
            url.push_str("&contentRating[]=suggestive&contentRating[]=erotica&contentRating[]=pornographic");
        } else {
            url.push_str("&contentRating[]=safe&contentRating[]=suggestive");
        }

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

        let series: Vec<SmallSerie> = response
            .data
            .into_iter()
            .map(|manga| self.convert_to_small_serie(manga))
            .collect();

        Ok(PaginatedSmallSeries {
            has_next_page: response.offset + response.limit < response.total,
            series,
        })
    }

    async fn fetch_serie_detail(&self, serie_id: &SerieId) -> Result<Serie, SourceError> {
        let url = format!(
            "{}/manga/{}?includes[]=cover_art&includes[]=author&includes[]=artist",
            self.api_url,
            serie_id.as_str()
        );

        let response: MangaDexSingleResponse<MangaDexManga> = self
            .client
            .get_json(&url)
            .await
            .map_err(|e| match e.to_string() {
                s if s.contains("404") => {
                    SourceError::NotFound(format!("Serie {} not found", serie_id))
                }
                _ => SourceError::Network(e.to_string()),
            })?;

        if response.result != "ok" {
            return Err(SourceError::Other(anyhow::anyhow!(
                "MangaDex API error: {}",
                response.result
            )));
        }

        let manga = response.data;
        let mut serie = self.convert_to_serie(manga.clone());

        // Get available languages for fetching chapters
        let available_languages: Vec<MangadexLanguage> = manga.attributes
            .available_translated_languages
            .iter()
            .filter_map(|lang| self.parse_language_to_mangadex(lang))
            .collect();

        // Fetch all chapters for available languages
        let chapters = self.fetch_all_chapters_with_languages(serie_id.as_str(), &available_languages).await?;

        // Convert chapters to volumes
        serie.volumes = self.convert_mangadex_chapters(chapters);

        Ok(serie)
    }

    async fn fetch_chapter_data(
        &self,
        _serie_id: &SerieId,
        _volume_id: &VolumeId,
        chapter_id: &ChapterId,
    ) -> Result<ChapterData, SourceError> {
        let url = format!("{}/at-home/server/{}", self.api_url, chapter_id.as_str());

        let response: MangaDexAtHomeResponse = self
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
                index: (idx + 1) as i32,
                url: format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, filename
                ),
            })
            .collect();

        Ok(ChapterData::from_images(images))
    }

    async fn get_serie_url(&self, serie_id: &SerieId) -> Result<String, SourceError> {
        Ok(format!("https://mangadex.org/title/{}", serie_id.as_str()))
    }
}

impl MangaDex {
    fn get_genre_tag_id(&self, genre: &SourceSerieGenre) -> Option<&'static str> {
        // Convert SourceSerieGenre to MangadexGenre, then get its string ID
        let mangadex_genre: MangadexGenre = (*genre).try_into().ok()?;
        Some(mangadex_genre.as_str())
    }

    fn get_type(&self, original_lang: &str, genres: &[SourceSerieGenre]) -> SourceSerieType {
        let is_long_strip = genres.contains(&SourceSerieGenre::LongStrip);
        let is_doujinshi = genres.contains(&SourceSerieGenre::Doujinshi);
        let is_web_comic = genres.contains(&SourceSerieGenre::WebComic);

        if is_doujinshi {
            return SourceSerieType::Doujinshi;
        }

        match original_lang {
            "ja" => SourceSerieType::Manga,
            "ko" => {
                if is_long_strip || is_web_comic {
                    SourceSerieType::Webtoon
                } else {
                    SourceSerieType::Manhwa
                }
            }
            "zh" | "zh-hk" => SourceSerieType::Manhua,
            _ => SourceSerieType::Comic,
        }
    }
}
