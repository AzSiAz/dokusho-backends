use std::collections::HashMap;
use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use dokusho_core::{FilterOrder, FilterSort, SourceLanguage, SourceSerieGenre, SourceSerieStatus, SourceSerieType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexResponse<T> {
    pub result: String,
    pub response: String,
    pub data: T,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexListResponse<T> {
    pub result: String,
    pub response: String,
    pub data: Vec<T>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexSingleResponse<T> {
    pub result: String,
    pub response: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexManga {
    pub id: String,
    #[serde(rename = "type")]
    pub manga_type: String,
    pub attributes: MangaDexMangaAttributes,
    pub relationships: Option<Vec<MangaDexRelationship>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexMangaAttributes {
    pub title: HashMap<String, String>,
    pub alt_titles: Vec<HashMap<String, String>>,
    pub description: HashMap<String, String>,
    pub is_locked: bool,
    pub links: Option<HashMap<String, String>>,
    pub original_language: String,
    pub last_volume: Option<String>,
    pub last_chapter: Option<String>,
    pub publication_demographic: Option<String>,
    pub status: String,
    pub year: Option<i32>,
    pub content_rating: String,
    pub tags: Vec<MangaDexTag>,
    pub state: String,
    pub chapter_numbers_reset_on_new_volume: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    pub available_translated_languages: Vec<String>,
    pub latest_uploaded_chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexTag {
    pub id: String,
    #[serde(rename = "type")]
    pub tag_type: String,
    pub attributes: MangaDexTagAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexTagAttributes {
    pub name: HashMap<String, String>,
    pub description: HashMap<String, String>,
    pub group: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexRelationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    pub related: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexChapter {
    pub id: String,
    #[serde(rename = "type")]
    pub chapter_type: String,
    pub attributes: MangaDexChapterAttributes,
    pub relationships: Vec<MangaDexRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterAttributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: Option<String>,
    pub translated_language: String,
    pub external_url: Option<String>,
    pub publish_at: DateTime<Utc>,
    pub readable_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub pages: i32,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterPages {
    pub result: String,
    pub base_url: String,
    pub chapter: MangaDexChapterPagesData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexChapterPagesData {
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

impl MangaDexManga {
    pub fn get_cover_url(&self) -> Option<String> {
        self.relationships.as_ref().and_then(|rels| {
            rels.iter()
                .find(|r| r.rel_type == "cover_art")
                .and_then(|cover| {
                    cover.attributes.as_ref().and_then(|attrs| {
                        attrs.get("fileName").and_then(|f| f.as_str())
                    })
                })
                .map(|filename| {
                    format!(
                        "https://uploads.mangadex.org/covers/{}/{}.512.jpg",
                        self.id, filename
                    )
                })
        })
    }

    pub fn get_status(&self) -> Vec<SourceSerieStatus> {
        match self.attributes.status.as_str() {
            "ongoing" => vec![SourceSerieStatus::Ongoing],
            "completed" => vec![SourceSerieStatus::Completed],
            "hiatus" => vec![SourceSerieStatus::Hiatus],
            "cancelled" => vec![SourceSerieStatus::Canceled],
            _ => vec![],
        }
    }

    pub fn get_serie_type(&self) -> SourceSerieType {
        match self.attributes.original_language.as_str() {
            "ja" => SourceSerieType::Manga,
            "ko" => SourceSerieType::Manhwa,
            "zh" | "zh-hk" => SourceSerieType::Manhua,
            _ => SourceSerieType::Comic,
        }
    }

    pub fn parse_language(lang_code: &str) -> Option<SourceLanguage> {
        match lang_code {
            "en" => Some(SourceLanguage::En),
            "fr" => Some(SourceLanguage::Fr),
            "ja" => Some(SourceLanguage::Jp),
            "ko" => Some(SourceLanguage::Ko),
            "zh" | "zh-cn" => Some(SourceLanguage::Zh),
            "zh-hk" => Some(SourceLanguage::ZhHk),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexAtHomeResponse {
    pub result: String,
    pub base_url: String,
    pub chapter: MangaDexAtHomeChapter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MangaDexAtHomeChapter {
    pub hash: String,
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

// MangaDex-specific genre enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MangadexGenre {
    #[serde(rename = "fad12b5e-68ba-460e-b933-9ae8318f5b65")]
    Gyaru,
    #[serde(rename = "f8f62932-27da-4fe4-8ee1-6779a8c5edba")]
    Tragedy,
    #[serde(rename = "f5ba408b-0e7a-484d-8d49-4e9125ac96de")]
    FullColor,
    #[serde(rename = "f42fbf9e-188a-447b-9fdc-f19dc1e4d685")]
    Music,
    #[serde(rename = "f4122d1c-3b44-44d0-9936-ff7502c39ad3")]
    Adaptation,
    #[serde(rename = "ee968100-4191-4968-93d3-f82d72be7e46")]
    Mystery,
    #[serde(rename = "eabc5b4c-6aff-42f3-b657-3e90cbd00b75")]
    Supernatural,
    #[serde(rename = "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869")]
    Cooking,
    #[serde(rename = "e64f6742-c834-471d-8d72-dd51fc02b835")]
    Aliens,
    #[serde(rename = "e5301a23-ebd9-49dd-a0cb-2add944c7fe9")]
    SliceOfLife,
    #[serde(rename = "e197df38-d0e7-43b5-9b09-2842d0c326dd")]
    WebComic,
    #[serde(rename = "df33b754-73a3-4c54-80e6-1a74a8058539")]
    Police,
    #[serde(rename = "0a39b5a1-b235-4886-a747-1d05d216532d")]
    AwardWinning,
    #[serde(rename = "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0")]
    Reincarnation,
    #[serde(rename = "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a")]
    Genderswap,
    #[serde(rename = "2d1f5d56-a1e5-4d0d-a961-2193588b08ec")]
    Lolicon,
    #[serde(rename = "3b60b75c-a2d7-4860-ab56-05f391bb889c")]
    Psychological,
    #[serde(rename = "3bb26d85-09d5-4d2e-880c-c34b974339e9")]
    Ghost,
    #[serde(rename = "3de8c75d-8ee3-48ff-98ee-e20a65c86451")]
    Animals,
    #[serde(rename = "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d")]
    LongStrip,
    #[serde(rename = "4d32cc48-9f00-4cca-9b5a-a839f0764984")]
    Comedy,
    #[serde(rename = "5bd0e105-4481-44ca-b6e7-7544da56b1a3")]
    Incest,
    #[serde(rename = "5ca48985-9a9d-4bd8-be29-80dc0303db72")]
    Crime,
    #[serde(rename = "5fff9cde-849c-4d78-aab0-0d52b2ee1d25")]
    Survival,
    #[serde(rename = "7b2ce280-79ef-4c09-9b58-12b7c23a9b78")]
    FanColored,
    #[serde(rename = "8c86611e-fab7-4986-9dec-d1a2f44acdd5")]
    VirtualReality,
    #[serde(rename = "9ab53f92-3eed-4e9b-903a-917c86035ee3")]
    Crossdressing,
    #[serde(rename = "36fd93ea-e8b8-445e-b836-358f02b3d33d")]
    Monsters,
    #[serde(rename = "51d83883-4103-437c-b4b1-731cb73d786c")]
    Anthology,
    #[serde(rename = "81c836c9-914a-4eca-981a-560dad663e73")]
    MagicalGirls,
    #[serde(rename = "85daba54-a71c-4554-8a28-9901a8b0afad")]
    Mafia,
    #[serde(rename = "87cc87cd-a395-47af-b27a-93258283bbc6")]
    Adventure,
    #[serde(rename = "92d6d951-ca5e-429c-ac78-451071cbf064")]
    OfficeWorkers,
    #[serde(rename = "0234a31e-a729-4e28-9d6a-3f87c4966b9e")]
    OneShot,
    #[serde(rename = "256c8bd9-4904-4360-bf4f-508a76d67183")]
    SciFi,
    #[serde(rename = "292e862b-2d17-4062-90a2-0356caa4ae27")]
    TimeTravel,
    #[serde(rename = "391b0423-d847-456f-aff0-8b0cfc03066b")]
    Action,
    #[serde(rename = "423e2eae-a7a2-4a8b-ac03-a8351462d71d")]
    Romance,
    #[serde(rename = "489dd859-9b61-4c37-af75-5b18e88daafc")]
    Ninja,
    #[serde(rename = "631ef465-9aba-4afb-b0fc-ea10efe274a8")]
    Zombies,
    #[serde(rename = "799c202e-7daa-44eb-9cf7-8a3c0441531e")]
    MartialArts,
    #[serde(rename = "891cf039-b895-47f0-9229-bef4c96eccd4")]
    SelfPublished,
    #[serde(rename = "5920b825-4181-4a17-beeb-9918b0ff7a30")]
    BoysLove,
    #[serde(rename = "7064a261-a137-4d3a-8848-2d385de3a99c")]
    Superhero,
    #[serde(rename = "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8")]
    VideoGames,
    #[serde(rename = "31932a7e-5b8e-49a6-9f12-2afa39dc544c")]
    TraditionalGames,
    #[serde(rename = "50880a9d-5440-4732-9afb-8f457127e836")]
    Mecha,
    #[serde(rename = "65761a2a-415e-47f3-bef2-a9dababba7a6")]
    ReverseHarem,
    #[serde(rename = "69964a64-2f90-4d33-beeb-f3ed2875eb4c")]
    Sports,
    #[serde(rename = "97893a4c-12af-4dac-b6be-0dffb353568e")]
    SexualViolence,
    #[serde(rename = "320831a8-4026-470b-94f6-8353740e6f04")]
    OfficialColored,
    #[serde(rename = "07251805-a27e-4d59-b488-f0bfbec15168")]
    Thriller,
    #[serde(rename = "9467335a-1b83-4497-9231-765337a00b96")]
    PostApocalyptic,
    #[serde(rename = "33771934-028e-4cb3-8744-691e866a923e")]
    Historical,
    #[serde(rename = "39730448-9a5f-48a2-85b0-a70db87b1233")]
    Demons,
    #[serde(rename = "81183756-1453-4c81-aa9e-f6e1b63be016")]
    Samurai,
    #[serde(rename = "a1f53773-c69a-4ce5-8cab-fffcd90b1565")]
    Magic,
    #[serde(rename = "a3c67850-4684-404e-9b7f-c69850ee5da6")]
    GirlsLove,
    #[serde(rename = "aafb99c1-7f60-43fa-b75f-fc9502ce29c7")]
    Harem,
    #[serde(rename = "ac72833b-c4e9-4878-b9db-6c8a4a99444a")]
    Military,
    #[serde(rename = "acc803a4-c95a-4c22-86fc-eb6b582d82a2")]
    Wuxia,
    #[serde(rename = "ace04997-f6bd-436e-b261-779182193d3d")]
    Isekai,
    #[serde(rename = "b1e97889-25b4-4258-b28b-cd7f4d28ea9b")]
    Philosophical,
    #[serde(rename = "b9af3a63-f058-46de-a9a0-e0c13906197a")]
    Drama,
    #[serde(rename = "b11fda93-8f1d-4bef-b2ed-8803d3733170")]
    FourKoma,
    #[serde(rename = "b13b2a48-c720-44a9-9c77-39c9979373fb")]
    Doujinshi,
    #[serde(rename = "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d")]
    Gore,
    #[serde(rename = "c8cbe35b-1b2b-4a3f-9c37-db84c4514856")]
    Medical,
    #[serde(rename = "caaa44eb-cd40-4177-b930-79d3ef2afe87")]
    SchoolLife,
    #[serde(rename = "cdad7e68-1419-41dd-bdce-27753074a640")]
    Horror,
    #[serde(rename = "cdc58593-87dd-415e-bbc0-2ec27bf404cc")]
    Fantasy,
    #[serde(rename = "d7d1730f-6eb0-4ba6-9437-602cac38664c")]
    Vampires,
    #[serde(rename = "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41")]
    Villainess,
    #[serde(rename = "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea")]
    Delinquents,
    #[serde(rename = "dd1f77c5-dea9-4e2b-97ae-224af09caf99")]
    MonsterGirls,
    #[serde(rename = "ddefd648-5140-4e5f-ba18-4eca4071d19b")]
    Shotacon,
}

impl MangadexGenre {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gyaru => "fad12b5e-68ba-460e-b933-9ae8318f5b65",
            Self::Tragedy => "f8f62932-27da-4fe4-8ee1-6779a8c5edba",
            Self::FullColor => "f5ba408b-0e7a-484d-8d49-4e9125ac96de",
            Self::Music => "f42fbf9e-188a-447b-9fdc-f19dc1e4d685",
            Self::Adaptation => "f4122d1c-3b44-44d0-9936-ff7502c39ad3",
            Self::Mystery => "ee968100-4191-4968-93d3-f82d72be7e46",
            Self::Supernatural => "eabc5b4c-6aff-42f3-b657-3e90cbd00b75",
            Self::Cooking => "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869",
            Self::Aliens => "e64f6742-c834-471d-8d72-dd51fc02b835",
            Self::SliceOfLife => "e5301a23-ebd9-49dd-a0cb-2add944c7fe9",
            Self::WebComic => "e197df38-d0e7-43b5-9b09-2842d0c326dd",
            Self::Police => "df33b754-73a3-4c54-80e6-1a74a8058539",
            Self::AwardWinning => "0a39b5a1-b235-4886-a747-1d05d216532d",
            Self::Reincarnation => "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0",
            Self::Genderswap => "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a",
            Self::Lolicon => "2d1f5d56-a1e5-4d0d-a961-2193588b08ec",
            Self::Psychological => "3b60b75c-a2d7-4860-ab56-05f391bb889c",
            Self::Ghost => "3bb26d85-09d5-4d2e-880c-c34b974339e9",
            Self::Animals => "3de8c75d-8ee3-48ff-98ee-e20a65c86451",
            Self::LongStrip => "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d",
            Self::Comedy => "4d32cc48-9f00-4cca-9b5a-a839f0764984",
            Self::Incest => "5bd0e105-4481-44ca-b6e7-7544da56b1a3",
            Self::Crime => "5ca48985-9a9d-4bd8-be29-80dc0303db72",
            Self::Survival => "5fff9cde-849c-4d78-aab0-0d52b2ee1d25",
            Self::FanColored => "7b2ce280-79ef-4c09-9b58-12b7c23a9b78",
            Self::VirtualReality => "8c86611e-fab7-4986-9dec-d1a2f44acdd5",
            Self::Crossdressing => "9ab53f92-3eed-4e9b-903a-917c86035ee3",
            Self::Monsters => "36fd93ea-e8b8-445e-b836-358f02b3d33d",
            Self::Anthology => "51d83883-4103-437c-b4b1-731cb73d786c",
            Self::MagicalGirls => "81c836c9-914a-4eca-981a-560dad663e73",
            Self::Mafia => "85daba54-a71c-4554-8a28-9901a8b0afad",
            Self::Adventure => "87cc87cd-a395-47af-b27a-93258283bbc6",
            Self::OfficeWorkers => "92d6d951-ca5e-429c-ac78-451071cbf064",
            Self::OneShot => "0234a31e-a729-4e28-9d6a-3f87c4966b9e",
            Self::SciFi => "256c8bd9-4904-4360-bf4f-508a76d67183",
            Self::TimeTravel => "292e862b-2d17-4062-90a2-0356caa4ae27",
            Self::Action => "391b0423-d847-456f-aff0-8b0cfc03066b",
            Self::Romance => "423e2eae-a7a2-4a8b-ac03-a8351462d71d",
            Self::Ninja => "489dd859-9b61-4c37-af75-5b18e88daafc",
            Self::Zombies => "631ef465-9aba-4afb-b0fc-ea10efe274a8",
            Self::MartialArts => "799c202e-7daa-44eb-9cf7-8a3c0441531e",
            Self::SelfPublished => "891cf039-b895-47f0-9229-bef4c96eccd4",
            Self::BoysLove => "5920b825-4181-4a17-beeb-9918b0ff7a30",
            Self::Superhero => "7064a261-a137-4d3a-8848-2d385de3a99c",
            Self::VideoGames => "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8",
            Self::TraditionalGames => "31932a7e-5b8e-49a6-9f12-2afa39dc544c",
            Self::Mecha => "50880a9d-5440-4732-9afb-8f457127e836",
            Self::ReverseHarem => "65761a2a-415e-47f3-bef2-a9dababba7a6",
            Self::Sports => "69964a64-2f90-4d33-beeb-f3ed2875eb4c",
            Self::SexualViolence => "97893a4c-12af-4dac-b6be-0dffb353568e",
            Self::OfficialColored => "320831a8-4026-470b-94f6-8353740e6f04",
            Self::Thriller => "07251805-a27e-4d59-b488-f0bfbec15168",
            Self::PostApocalyptic => "9467335a-1b83-4497-9231-765337a00b96",
            Self::Historical => "33771934-028e-4cb3-8744-691e866a923e",
            Self::Demons => "39730448-9a5f-48a2-85b0-a70db87b1233",
            Self::Samurai => "81183756-1453-4c81-aa9e-f6e1b63be016",
            Self::Magic => "a1f53773-c69a-4ce5-8cab-fffcd90b1565",
            Self::GirlsLove => "a3c67850-4684-404e-9b7f-c69850ee5da6",
            Self::Harem => "aafb99c1-7f60-43fa-b75f-fc9502ce29c7",
            Self::Military => "ac72833b-c4e9-4878-b9db-6c8a4a99444a",
            Self::Wuxia => "acc803a4-c95a-4c22-86fc-eb6b582d82a2",
            Self::Isekai => "ace04997-f6bd-436e-b261-779182193d3d",
            Self::Philosophical => "b1e97889-25b4-4258-b28b-cd7f4d28ea9b",
            Self::Drama => "b9af3a63-f058-46de-a9a0-e0c13906197a",
            Self::FourKoma => "b11fda93-8f1d-4bef-b2ed-8803d3733170",
            Self::Doujinshi => "b13b2a48-c720-44a9-9c77-39c9979373fb",
            Self::Gore => "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d",
            Self::Medical => "c8cbe35b-1b2b-4a3f-9c37-db84c4514856",
            Self::SchoolLife => "caaa44eb-cd40-4177-b930-79d3ef2afe87",
            Self::Horror => "cdad7e68-1419-41dd-bdce-27753074a640",
            Self::Fantasy => "cdc58593-87dd-415e-bbc0-2ec27bf404cc",
            Self::Vampires => "d7d1730f-6eb0-4ba6-9437-602cac38664c",
            Self::Villainess => "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41",
            Self::Delinquents => "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea",
            Self::MonsterGirls => "dd1f77c5-dea9-4e2b-97ae-224af09caf99",
            Self::Shotacon => "ddefd648-5140-4e5f-ba18-4eca4071d19b",
        }
    }
}

// Status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MangadexStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
    Published,
    Unknown,
}

impl MangadexStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ongoing => "ongoing",
            Self::Completed => "completed",
            Self::Hiatus => "hiatus",
            Self::Cancelled => "cancelled",
            Self::Published => "published",
            Self::Unknown => "unknown",
        }
    }
}

impl TryFrom<&str> for MangadexStatus {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Ok(match s {
            "ongoing" => Self::Ongoing,
            "completed" => Self::Completed,
            "hiatus" => Self::Hiatus,
            "cancelled" => Self::Cancelled,
            "published" => Self::Published,
            "unknown" => Self::Unknown,
            _ => return Err(anyhow::anyhow!("Unknown MangaDex status: {}", s)),
        })
    }
}

// Sort enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MangadexSort {
    FollowerCount,
    LatestUploadedChapter,
    Title,
    CreatedAt,
    Rating,
}

impl MangadexSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FollowerCount => "followedCount",
            Self::LatestUploadedChapter => "latestUploadedChapter",
            Self::Title => "title",
            Self::CreatedAt => "createdAt",
            Self::Rating => "rating",
        }
    }
}

// Order enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MangadexOrder {
    Asc,
    Desc,
}

impl MangadexOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

// Language enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MangadexLanguage {
    #[serde(rename = "en")]
    En,
    #[serde(rename = "fr")]
    Fr,
    #[serde(rename = "ko")]
    Ko,
    #[serde(rename = "ja")]
    Ja,
    #[serde(rename = "ja-ro")]
    JaRo,
    #[serde(rename = "zh-hk")]
    ZhHk,
    #[serde(rename = "zh")]
    Zh,
}

impl MangadexLanguage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Fr => "fr",
            Self::Ko => "ko",
            Self::Ja => "ja",
            Self::JaRo => "ja-ro",
            Self::ZhHk => "zh-hk",
            Self::Zh => "zh",
        }
    }
}

// From trait implementations for conversions
impl TryFrom<MangadexGenre> for SourceSerieGenre {
    type Error = anyhow::Error;

    fn try_from(genre: MangadexGenre) -> Result<Self, Self::Error> {
        use SourceSerieGenre::*;
        Ok(match genre {
            MangadexGenre::Gyaru => Gyaru,
            MangadexGenre::Tragedy => Tragedy,
            MangadexGenre::FullColor => FullColor,
            MangadexGenre::Music => Music,
            MangadexGenre::Adaptation => Adaptation,
            MangadexGenre::Mystery => Mystery,
            MangadexGenre::Supernatural => Supernatural,
            MangadexGenre::Cooking => Cooking,
            MangadexGenre::Aliens => Aliens,
            MangadexGenre::SliceOfLife => SliceOfLife,
            MangadexGenre::WebComic => WebComic,
            MangadexGenre::Police => Police,
            MangadexGenre::AwardWinning => AwardWinning,
            MangadexGenre::Reincarnation => Reincarnation,
            MangadexGenre::Genderswap => Genderswap,
            MangadexGenre::Lolicon => Lolicon,
            MangadexGenre::Psychological => Psychological,
            MangadexGenre::Ghost => Ghost,
            MangadexGenre::Animals => Animals,
            MangadexGenre::LongStrip => LongStrip,
            MangadexGenre::Comedy => Comedy,
            MangadexGenre::Incest => Incest,
            MangadexGenre::Crime => Crime,
            MangadexGenre::Survival => Survival,
            MangadexGenre::FanColored => FanColored,
            MangadexGenre::VirtualReality => VirtualReality,
            MangadexGenre::Crossdressing => Crossdressing,
            MangadexGenre::Monsters => Monsters,
            MangadexGenre::Anthology => Anthology,
            MangadexGenre::MagicalGirls => MagicalGirls,
            MangadexGenre::Mafia => Mafia,
            MangadexGenre::Adventure => Adventure,
            MangadexGenre::OfficeWorkers => OfficeWorkers,
            MangadexGenre::OneShot => OneShot,
            MangadexGenre::SciFi => SciFi,
            MangadexGenre::TimeTravel => TimeTravel,
            MangadexGenre::Action => Action,
            MangadexGenre::Romance => Romance,
            MangadexGenre::Ninja => Ninja,
            MangadexGenre::Zombies => Zombies,
            MangadexGenre::MartialArts => MartialArts,
            MangadexGenre::SelfPublished => SelfPublished,
            MangadexGenre::BoysLove => BoysLove,
            MangadexGenre::Superhero => Superhero,
            MangadexGenre::VideoGames => VideoGames,
            MangadexGenre::TraditionalGames => TraditionalGames,
            MangadexGenre::Mecha => Mecha,
            MangadexGenre::ReverseHarem => ReverseHarem,
            MangadexGenre::Sports => Sports,
            MangadexGenre::SexualViolence => SexualViolence,
            MangadexGenre::OfficialColored => OfficialColored,
            MangadexGenre::Thriller => Thriller,
            MangadexGenre::PostApocalyptic => PostApocalyptic,
            MangadexGenre::Historical => Historical,
            MangadexGenre::Demons => Demons,
            MangadexGenre::Samurai => Samurai,
            MangadexGenre::Magic => Magic,
            MangadexGenre::GirlsLove => GirlsLove,
            MangadexGenre::Harem => Harem,
            MangadexGenre::Military => Military,
            MangadexGenre::Wuxia => Wuxia,
            MangadexGenre::Isekai => Isekai,
            MangadexGenre::Philosophical => Philosophical,
            MangadexGenre::Drama => Drama,
            MangadexGenre::FourKoma => FourKoma,
            MangadexGenre::Doujinshi => Doujinshi,
            MangadexGenre::Gore => Gore,
            MangadexGenre::Medical => Medical,
            MangadexGenre::SchoolLife => SchoolLife,
            MangadexGenre::Horror => Horror,
            MangadexGenre::Fantasy => Fantasy,
            MangadexGenre::Vampires => Vampires,
            MangadexGenre::Villainess => Villainess,
            MangadexGenre::Delinquents => Delinquents,
            MangadexGenre::MonsterGirls => MonsterGirls,
            MangadexGenre::Shotacon => Shotacon,
        })
    }
}

impl TryFrom<SourceSerieGenre> for MangadexGenre {
    type Error = anyhow::Error;

    fn try_from(genre: SourceSerieGenre) -> Result<Self, Self::Error> {
        use SourceSerieGenre::*;
        Ok(match genre {
            Gyaru => MangadexGenre::Gyaru,
            Tragedy => MangadexGenre::Tragedy,
            FullColor => MangadexGenre::FullColor,
            Music => MangadexGenre::Music,
            Adaptation => MangadexGenre::Adaptation,
            Mystery => MangadexGenre::Mystery,
            Supernatural => MangadexGenre::Supernatural,
            Cooking => MangadexGenre::Cooking,
            Aliens => MangadexGenre::Aliens,
            SliceOfLife => MangadexGenre::SliceOfLife,
            WebComic => MangadexGenre::WebComic,
            Police => MangadexGenre::Police,
            AwardWinning => MangadexGenre::AwardWinning,
            Reincarnation => MangadexGenre::Reincarnation,
            Genderswap => MangadexGenre::Genderswap,
            Lolicon => MangadexGenre::Lolicon,
            Psychological => MangadexGenre::Psychological,
            Ghost => MangadexGenre::Ghost,
            Animals => MangadexGenre::Animals,
            LongStrip => MangadexGenre::LongStrip,
            Comedy => MangadexGenre::Comedy,
            Incest => MangadexGenre::Incest,
            Crime => MangadexGenre::Crime,
            Survival => MangadexGenre::Survival,
            FanColored => MangadexGenre::FanColored,
            VirtualReality => MangadexGenre::VirtualReality,
            Crossdressing => MangadexGenre::Crossdressing,
            Monsters => MangadexGenre::Monsters,
            Anthology => MangadexGenre::Anthology,
            MagicalGirls => MangadexGenre::MagicalGirls,
            Mafia => MangadexGenre::Mafia,
            Adventure => MangadexGenre::Adventure,
            OfficeWorkers => MangadexGenre::OfficeWorkers,
            OneShot => MangadexGenre::OneShot,
            SciFi => MangadexGenre::SciFi,
            TimeTravel => MangadexGenre::TimeTravel,
            Action => MangadexGenre::Action,
            Romance => MangadexGenre::Romance,
            Ninja => MangadexGenre::Ninja,
            Zombies => MangadexGenre::Zombies,
            MartialArts => MangadexGenre::MartialArts,
            SelfPublished => MangadexGenre::SelfPublished,
            BoysLove => MangadexGenre::BoysLove,
            Superhero => MangadexGenre::Superhero,
            VideoGames => MangadexGenre::VideoGames,
            TraditionalGames => MangadexGenre::TraditionalGames,
            Mecha => MangadexGenre::Mecha,
            ReverseHarem => MangadexGenre::ReverseHarem,
            Sports => MangadexGenre::Sports,
            SexualViolence => MangadexGenre::SexualViolence,
            OfficialColored => MangadexGenre::OfficialColored,
            Thriller => MangadexGenre::Thriller,
            PostApocalyptic => MangadexGenre::PostApocalyptic,
            Historical => MangadexGenre::Historical,
            Demons => MangadexGenre::Demons,
            Samurai => MangadexGenre::Samurai,
            Magic => MangadexGenre::Magic,
            GirlsLove => MangadexGenre::GirlsLove,
            Harem => MangadexGenre::Harem,
            Military => MangadexGenre::Military,
            Wuxia => MangadexGenre::Wuxia,
            Isekai => MangadexGenre::Isekai,
            Philosophical => MangadexGenre::Philosophical,
            Drama => MangadexGenre::Drama,
            FourKoma => MangadexGenre::FourKoma,
            Doujinshi => MangadexGenre::Doujinshi,
            Gore => MangadexGenre::Gore,
            Medical => MangadexGenre::Medical,
            SchoolLife => MangadexGenre::SchoolLife,
            Horror => MangadexGenre::Horror,
            Fantasy => MangadexGenre::Fantasy,
            Vampires => MangadexGenre::Vampires,
            Villainess => MangadexGenre::Villainess,
            Delinquents => MangadexGenre::Delinquents,
            MonsterGirls => MangadexGenre::MonsterGirls,
            Shotacon => MangadexGenre::Shotacon,
            _ => return Err(anyhow::anyhow!("Genre {:?} is not supported by MangaDex", genre)),
        })
    }
}

impl TryFrom<MangadexStatus> for SourceSerieStatus {
    type Error = anyhow::Error;

    fn try_from(status: MangadexStatus) -> Result<Self, Self::Error> {
        use SourceSerieStatus::*;
        Ok(match status {
            MangadexStatus::Ongoing => Ongoing,
            MangadexStatus::Completed => Completed,
            MangadexStatus::Hiatus => Hiatus,
            MangadexStatus::Cancelled => Canceled,
            MangadexStatus::Published => Published,
            MangadexStatus::Unknown => Unknown,
        })
    }
}

impl TryFrom<SourceSerieStatus> for MangadexStatus {
    type Error = anyhow::Error;

    fn try_from(status: SourceSerieStatus) -> Result<Self, Self::Error> {
        use SourceSerieStatus::*;
        Ok(match status {
            Ongoing => MangadexStatus::Ongoing,
            Completed => MangadexStatus::Completed,
            Hiatus => MangadexStatus::Hiatus,
            Canceled => MangadexStatus::Cancelled,
            Published => MangadexStatus::Published,
            Unknown => MangadexStatus::Unknown,
            _ => return Err(anyhow::anyhow!("Status {:?} is not supported by MangaDex", status)),
        })
    }
}

impl TryFrom<MangadexSort> for FilterSort {
    type Error = anyhow::Error;

    fn try_from(sort: MangadexSort) -> Result<Self, Self::Error> {
        Ok(match sort {
            MangadexSort::FollowerCount => FilterSort::Popularity,
            MangadexSort::LatestUploadedChapter => FilterSort::UpdatedAt,
            MangadexSort::Title => FilterSort::Title,
            MangadexSort::CreatedAt => FilterSort::CreatedAt,
            MangadexSort::Rating => FilterSort::Rating,
        })
    }
}

impl TryFrom<FilterSort> for MangadexSort {
    type Error = anyhow::Error;

    fn try_from(sort: FilterSort) -> Result<Self, Self::Error> {
        Ok(match sort {
            FilterSort::Popularity => MangadexSort::FollowerCount,
            FilterSort::UpdatedAt => MangadexSort::LatestUploadedChapter,
            FilterSort::Title => MangadexSort::Title,
            FilterSort::CreatedAt => MangadexSort::CreatedAt,
            FilterSort::Rating => MangadexSort::Rating,
            _ => return Err(anyhow::anyhow!("Sort {:?} is not supported by MangaDex", sort)),
        })
    }
}

impl From<MangadexOrder> for FilterOrder {
    fn from(order: MangadexOrder) -> Self {
        match order {
            MangadexOrder::Asc => FilterOrder::Ascending,
            MangadexOrder::Desc => FilterOrder::Descending,
        }
    }
}

impl From<FilterOrder> for MangadexOrder {
    fn from(order: FilterOrder) -> Self {
        match order {
            FilterOrder::Ascending => MangadexOrder::Asc,
            FilterOrder::Descending => MangadexOrder::Desc,
        }
    }
}

impl TryFrom<MangadexLanguage> for SourceLanguage {
    type Error = anyhow::Error;

    fn try_from(language: MangadexLanguage) -> Result<Self, Self::Error> {
        use SourceLanguage::*;
        Ok(match language {
            MangadexLanguage::En => En,
            MangadexLanguage::Fr => Fr,
            MangadexLanguage::Ko => Ko,
            MangadexLanguage::Ja => Jp,
            MangadexLanguage::JaRo => JpRo,
            MangadexLanguage::ZhHk => ZhHk,
            MangadexLanguage::Zh => Zh,
        })
    }
}

impl TryFrom<SourceLanguage> for MangadexLanguage {
    type Error = anyhow::Error;

    fn try_from(language: SourceLanguage) -> Result<Self, Self::Error> {
        use SourceLanguage::*;
        match language {
            En => Ok(MangadexLanguage::En),
            Fr => Ok(MangadexLanguage::Fr),
            Ko => Ok(MangadexLanguage::Ko),
            Jp => Ok(MangadexLanguage::Ja),
            JpRo => Ok(MangadexLanguage::JaRo),
            ZhHk => Ok(MangadexLanguage::ZhHk),
            Zh => Ok(MangadexLanguage::Zh),
        }
    }
}

// Helper functions for batch conversions
pub fn convert_mangadex_genres(genres: &[MangadexGenre]) -> Vec<SourceSerieGenre> {
    genres.iter()
        .filter_map(|&g| g.try_into().ok())
        .collect()
}

pub fn convert_source_genres(genres: &[SourceSerieGenre]) -> Vec<MangadexGenre> {
    genres.iter()
        .filter_map(|&g| g.try_into().ok())
        .collect()
}

// Get searchable functions
pub fn get_searchable_genres() -> Vec<SourceSerieGenre> {
    use MangadexGenre::*;
    let all_genres = vec![
        Gyaru, Tragedy, FullColor, Music, Adaptation, Mystery, Supernatural, Cooking,
        Aliens, SliceOfLife, WebComic, Police, AwardWinning, Reincarnation, Genderswap,
        Lolicon, Psychological, Ghost, Animals, LongStrip, Comedy, Incest, Crime,
        Survival, FanColored, VirtualReality, Crossdressing, Monsters, Anthology,
        MagicalGirls, Mafia, Adventure, OfficeWorkers, OneShot, SciFi, TimeTravel,
        Action, Romance, Ninja, Zombies, MartialArts, SelfPublished, BoysLove,
        Superhero, VideoGames, TraditionalGames, Mecha, ReverseHarem, Sports,
        SexualViolence, OfficialColored, Thriller, PostApocalyptic, Historical,
        Demons, Samurai, Magic, GirlsLove, Harem, Military, Wuxia, Isekai,
        Philosophical, Drama, FourKoma, Doujinshi, Gore, Medical, SchoolLife,
        Horror, Fantasy, Vampires, Villainess, Delinquents, MonsterGirls, Shotacon,
    ];
    
    let mut genres = convert_mangadex_genres(&all_genres);
    genres.dedup();
    genres
}

pub fn get_searchable_status() -> Vec<SourceSerieStatus> {
    use MangadexStatus::*;
    let all_status = vec![Ongoing, Completed, Hiatus, Cancelled, Published];
    
    let mut statuses: Vec<_> = all_status.into_iter()
        .filter_map(|s| s.try_into().ok())
        .collect();
    statuses.dedup();
    statuses
}

pub fn get_searchable_sorts() -> Vec<FilterSort> {
    use MangadexSort::*;
    let all_sorts = vec![FollowerCount, LatestUploadedChapter, Title, CreatedAt, Rating];
    
    let mut sorts: Vec<_> = all_sorts.into_iter()
        .filter_map(|s| s.try_into().ok())
        .collect();
    sorts.dedup();
    sorts
}

pub fn get_searchable_orders() -> Vec<FilterOrder> {
    vec![FilterOrder::Ascending, FilterOrder::Descending]
}