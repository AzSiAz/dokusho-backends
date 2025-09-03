use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};
use url::Url;

use dokusho_core::{
    FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort, MultiLanguageString, SourceError,
    SourceLanguage, SourceSerie, SourceSerieChapter, SourceSerieGenre, SourceSerieStatus,
    SourceSerieType, SourceSmallSerie,
};

const NO_IMAGE_URL: &str = "https://i.imgur.com/6TrIues.jpeg";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum MangadexGenre {
    #[serde(rename = "fad12b5e-68ba-460e-b933-9ae8318f5b65")]
    #[strum(serialize = "fad12b5e-68ba-460e-b933-9ae8318f5b65")]
    Gyaru,
    #[serde(rename = "f8f62932-27da-4fe4-8ee1-6779a8c5edba")]
    #[strum(serialize = "f8f62932-27da-4fe4-8ee1-6779a8c5edba")]
    Tragedy,
    #[serde(rename = "f5ba408b-0e7a-484d-8d49-4e9125ac96de")]
    #[strum(serialize = "f5ba408b-0e7a-484d-8d49-4e9125ac96de")]
    FullColor,
    #[serde(rename = "f42fbf9e-188a-447b-9fdc-f19dc1e4d685")]
    #[strum(serialize = "f42fbf9e-188a-447b-9fdc-f19dc1e4d685")]
    Music,
    #[serde(rename = "f4122d1c-3b44-44d0-9936-ff7502c39ad3")]
    #[strum(serialize = "f4122d1c-3b44-44d0-9936-ff7502c39ad3")]
    Adaptation,
    #[serde(rename = "ee968100-4191-4968-93d3-f82d72be7e46")]
    #[strum(serialize = "ee968100-4191-4968-93d3-f82d72be7e46")]
    Mystery,
    #[serde(rename = "eabc5b4c-6aff-42f3-b657-3e90cbd00b75")]
    #[strum(serialize = "eabc5b4c-6aff-42f3-b657-3e90cbd00b75")]
    Supernatural,
    #[serde(rename = "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869")]
    #[strum(serialize = "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869")]
    Cooking,
    #[serde(rename = "e64f6742-c834-471d-8d72-dd51fc02b835")]
    #[strum(serialize = "e64f6742-c834-471d-8d72-dd51fc02b835")]
    Aliens,
    #[serde(rename = "e5301a23-ebd9-49dd-a0cb-2add944c7fe9")]
    #[strum(serialize = "e5301a23-ebd9-49dd-a0cb-2add944c7fe9")]
    SliceOfLife,
    #[serde(rename = "e197df38-d0e7-43b5-9b09-2842d0c326dd")]
    #[strum(serialize = "e197df38-d0e7-43b5-9b09-2842d0c326dd")]
    WebComic,
    #[serde(rename = "df33b754-73a3-4c54-80e6-1a74a8058539")]
    #[strum(serialize = "df33b754-73a3-4c54-80e6-1a74a8058539")]
    Police,
    #[serde(rename = "0a39b5a1-b235-4886-a747-1d05d216532d")]
    #[strum(serialize = "0a39b5a1-b235-4886-a747-1d05d216532d")]
    AwardWinning,
    #[serde(rename = "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0")]
    #[strum(serialize = "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0")]
    Reincarnation,
    #[serde(rename = "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a")]
    #[strum(serialize = "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a")]
    GenderSwap,
    #[serde(rename = "2d1f5d56-a1e5-4d0d-a961-2193588b08ec")]
    #[strum(serialize = "2d1f5d56-a1e5-4d0d-a961-2193588b08ec")]
    Lolicon,
    #[serde(rename = "3b60b75c-a2d7-4860-ab56-05f391bb889c")]
    #[strum(serialize = "3b60b75c-a2d7-4860-ab56-05f391bb889c")]
    Psychological,
    #[serde(rename = "3bb26d85-09d5-4d2e-880c-c34b974339e9")]
    #[strum(serialize = "3bb26d85-09d5-4d2e-880c-c34b974339e9")]
    Ghost,
    #[serde(rename = "3de8c75d-8ee3-48ff-98ee-e20a65c86451")]
    #[strum(serialize = "3de8c75d-8ee3-48ff-98ee-e20a65c86451")]
    Animals,
    #[serde(rename = "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d")]
    #[strum(serialize = "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d")]
    LongStrip,
    #[serde(rename = "4d32cc48-9f00-4cca-9b5a-a839f0764984")]
    #[strum(serialize = "4d32cc48-9f00-4cca-9b5a-a839f0764984")]
    Comedy,
    #[serde(rename = "5bd0e105-4481-44ca-b6e7-7544da56b1a3")]
    #[strum(serialize = "5bd0e105-4481-44ca-b6e7-7544da56b1a3")]
    Incest,
    #[serde(rename = "5ca48985-9a9d-4bd8-be29-80dc0303db72")]
    #[strum(serialize = "5ca48985-9a9d-4bd8-be29-80dc0303db72")]
    Crime,
    #[serde(rename = "5fff9cde-849c-4d78-aab0-0d52b2ee1d25")]
    #[strum(serialize = "5fff9cde-849c-4d78-aab0-0d52b2ee1d25")]
    Survival,
    #[serde(rename = "7b2ce280-79ef-4c09-9b58-12b7c23a9b78")]
    #[strum(serialize = "7b2ce280-79ef-4c09-9b58-12b7c23a9b78")]
    FanColored,
    #[serde(rename = "8c86611e-fab7-4986-9dec-d1a2f44acdd5")]
    #[strum(serialize = "8c86611e-fab7-4986-9dec-d1a2f44acdd5")]
    VirtualReality,
    #[serde(rename = "9ab53f92-3eed-4e9b-903a-917c86035ee3")]
    #[strum(serialize = "9ab53f92-3eed-4e9b-903a-917c86035ee3")]
    Crossdressing,
    #[serde(rename = "36fd93ea-e8b8-445e-b836-358f02b3d33d")]
    #[strum(serialize = "36fd93ea-e8b8-445e-b836-358f02b3d33d")]
    Monsters,
    #[serde(rename = "51d83883-4103-437c-b4b1-731cb73d786c")]
    #[strum(serialize = "51d83883-4103-437c-b4b1-731cb73d786c")]
    Anthology,
    #[serde(rename = "81c836c9-914a-4eca-981a-560dad663e73")]
    #[strum(serialize = "81c836c9-914a-4eca-981a-560dad663e73")]
    MagicalGirls,
    #[serde(rename = "85daba54-a71c-4554-8a28-9901a8b0afad")]
    #[strum(serialize = "85daba54-a71c-4554-8a28-9901a8b0afad")]
    Mafia,
    #[serde(rename = "87cc87cd-a395-47af-b27a-93258283bbc6")]
    #[strum(serialize = "87cc87cd-a395-47af-b27a-93258283bbc6")]
    Adventure,
    #[serde(rename = "92d6d951-ca5e-429c-ac78-451071cbf064")]
    #[strum(serialize = "92d6d951-ca5e-429c-ac78-451071cbf064")]
    OfficeWorkers,
    #[serde(rename = "0234a31e-a729-4e28-9d6a-3f87c4966b9e")]
    #[strum(serialize = "0234a31e-a729-4e28-9d6a-3f87c4966b9e")]
    OneShot,
    #[serde(rename = "256c8bd9-4904-4360-bf4f-508a76d67183")]
    #[strum(serialize = "256c8bd9-4904-4360-bf4f-508a76d67183")]
    SciFi,
    #[serde(rename = "292e862b-2d17-4062-90a2-0356caa4ae27")]
    #[strum(serialize = "292e862b-2d17-4062-90a2-0356caa4ae27")]
    TimeTravel,
    #[serde(rename = "391b0423-d847-456f-aff0-8b0cfc03066b")]
    #[strum(serialize = "391b0423-d847-456f-aff0-8b0cfc03066b")]
    Action,
    #[serde(rename = "423e2eae-a7a2-4a8b-ac03-a8351462d71d")]
    #[strum(serialize = "423e2eae-a7a2-4a8b-ac03-a8351462d71d")]
    Romance,
    #[serde(rename = "489dd859-9b61-4c37-af75-5b18e88daafc")]
    #[strum(serialize = "489dd859-9b61-4c37-af75-5b18e88daafc")]
    Ninja,
    #[serde(rename = "631ef465-9aba-4afb-b0fc-ea10efe274a8")]
    #[strum(serialize = "631ef465-9aba-4afb-b0fc-ea10efe274a8")]
    Zombies,
    #[serde(rename = "799c202e-7daa-44eb-9cf7-8a3c0441531e")]
    #[strum(serialize = "799c202e-7daa-44eb-9cf7-8a3c0441531e")]
    MartialArts,
    #[serde(rename = "891cf039-b895-47f0-9229-bef4c96eccd4")]
    #[strum(serialize = "891cf039-b895-47f0-9229-bef4c96eccd4")]
    SelfPublished,
    #[serde(rename = "5920b825-4181-4a17-beeb-9918b0ff7a30")]
    #[strum(serialize = "5920b825-4181-4a17-beeb-9918b0ff7a30")]
    BoysLove,
    #[serde(rename = "7064a261-a137-4d3a-8848-2d385de3a99c")]
    #[strum(serialize = "7064a261-a137-4d3a-8848-2d385de3a99c")]
    SuperHero,
    #[serde(rename = "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8")]
    #[strum(serialize = "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8")]
    VideoGames,
    #[serde(rename = "31932a7e-5b8e-49a6-9f12-2afa39dc544c")]
    #[strum(serialize = "31932a7e-5b8e-49a6-9f12-2afa39dc544c")]
    TraditionalGames,
    #[serde(rename = "50880a9d-5440-4732-9afb-8f457127e836")]
    #[strum(serialize = "50880a9d-5440-4732-9afb-8f457127e836")]
    Mecha,
    #[serde(rename = "65761a2a-415e-47f3-bef2-a9dababba7a6")]
    #[strum(serialize = "65761a2a-415e-47f3-bef2-a9dababba7a6")]
    ReverseHarem,
    #[serde(rename = "69964a64-2f90-4d33-beeb-f3ed2875eb4c")]
    #[strum(serialize = "69964a64-2f90-4d33-beeb-f3ed2875eb4c")]
    Sports,
    #[serde(rename = "97893a4c-12af-4dac-b6be-0dffb353568e")]
    #[strum(serialize = "97893a4c-12af-4dac-b6be-0dffb353568e")]
    SexualViolence,
    #[serde(rename = "320831a8-4026-470b-94f6-8353740e6f04")]
    #[strum(serialize = "320831a8-4026-470b-94f6-8353740e6f04")]
    OfficialColored,
    #[serde(rename = "07251805-a27e-4d59-b488-f0bfbec15168")]
    #[strum(serialize = "07251805-a27e-4d59-b488-f0bfbec15168")]
    Thriller,
    #[serde(rename = "9467335a-1b83-4497-9231-765337a00b96")]
    #[strum(serialize = "9467335a-1b83-4497-9231-765337a00b96")]
    PostApocalyptic,
    #[serde(rename = "33771934-028e-4cb3-8744-691e866a923e")]
    #[strum(serialize = "33771934-028e-4cb3-8744-691e866a923e")]
    Historical,
    #[serde(rename = "39730448-9a5f-48a2-85b0-a70db87b1233")]
    #[strum(serialize = "39730448-9a5f-48a2-85b0-a70db87b1233")]
    Demons,
    #[serde(rename = "81183756-1453-4c81-aa9e-f6e1b63be016")]
    #[strum(serialize = "81183756-1453-4c81-aa9e-f6e1b63be016")]
    Samurai,
    #[serde(rename = "a1f53773-c69a-4ce5-8cab-fffcd90b1565")]
    #[strum(serialize = "a1f53773-c69a-4ce5-8cab-fffcd90b1565")]
    Magic,
    #[serde(rename = "a3c67850-4684-404e-9b7f-c69850ee5da6")]
    #[strum(serialize = "a3c67850-4684-404e-9b7f-c69850ee5da6")]
    GirlsLove,
    #[serde(rename = "aafb99c1-7f60-43fa-b75f-fc9502ce29c7")]
    #[strum(serialize = "aafb99c1-7f60-43fa-b75f-fc9502ce29c7")]
    Harem,
    #[serde(rename = "ac72833b-c4e9-4878-b9db-6c8a4a99444a")]
    #[strum(serialize = "ac72833b-c4e9-4878-b9db-6c8a4a99444a")]
    Military,
    #[serde(rename = "acc803a4-c95a-4c22-86fc-eb6b582d82a2")]
    #[strum(serialize = "acc803a4-c95a-4c22-86fc-eb6b582d82a2")]
    Wuxia,
    #[serde(rename = "ace04997-f6bd-436e-b261-779182193d3d")]
    #[strum(serialize = "ace04997-f6bd-436e-b261-779182193d3d")]
    Isekai,
    #[serde(rename = "b1e97889-25b4-4258-b28b-cd7f4d28ea9b")]
    #[strum(serialize = "b1e97889-25b4-4258-b28b-cd7f4d28ea9b")]
    Philosophical,
    #[serde(rename = "b9af3a63-f058-46de-a9a0-e0c13906197a")]
    #[strum(serialize = "b9af3a63-f058-46de-a9a0-e0c13906197a")]
    Drama,
    #[serde(rename = "b11fda93-8f1d-4bef-b2ed-8803d3733170")]
    #[strum(serialize = "b11fda93-8f1d-4bef-b2ed-8803d3733170")]
    FourKoma,
    #[serde(rename = "b13b2a48-c720-44a9-9c77-39c9979373fb")]
    #[strum(serialize = "b13b2a48-c720-44a9-9c77-39c9979373fb")]
    Doujinshi,
    #[serde(rename = "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d")]
    #[strum(serialize = "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d")]
    Gore,
    #[serde(rename = "c8cbe35b-1b2b-4a3f-9c37-db84c4514856")]
    #[strum(serialize = "c8cbe35b-1b2b-4a3f-9c37-db84c4514856")]
    Medical,
    #[serde(rename = "caaa44eb-cd40-4177-b930-79d3ef2afe87")]
    #[strum(serialize = "caaa44eb-cd40-4177-b930-79d3ef2afe87")]
    SchoolLife,
    #[serde(rename = "cdad7e68-1419-41dd-bdce-27753074a640")]
    #[strum(serialize = "cdad7e68-1419-41dd-bdce-27753074a640")]
    Horror,
    #[serde(rename = "cdc58593-87dd-415e-bbc0-2ec27bf404cc")]
    #[strum(serialize = "cdc58593-87dd-415e-bbc0-2ec27bf404cc")]
    Fantasy,
    #[serde(rename = "d7d1730f-6eb0-4ba6-9437-602cac38664c")]
    #[strum(serialize = "d7d1730f-6eb0-4ba6-9437-602cac38664c")]
    Vampires,
    #[serde(rename = "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41")]
    #[strum(serialize = "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41")]
    Villainess,
    #[serde(rename = "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea")]
    #[strum(serialize = "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea")]
    Delinquents,
    #[serde(rename = "dd1f77c5-dea9-4e2b-97ae-224af09caf99")]
    #[strum(serialize = "dd1f77c5-dea9-4e2b-97ae-224af09caf99")]
    MonsterGirls,
    #[serde(rename = "ddefd648-5140-4e5f-ba18-4eca4071d19b")]
    #[strum(serialize = "ddefd648-5140-4e5f-ba18-4eca4071d19b")]
    Shotacon,

    #[serde(rename = "Unknown")]
    Unknown,
}

impl TryFrom<String> for MangadexGenre {
    type Error = SourceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "fad12b5e-68ba-460e-b933-9ae8318f5b65" => Ok(Self::Gyaru),
            "f8f62932-27da-4fe4-8ee1-6779a8c5edba" => Ok(Self::Tragedy),
            "f5ba408b-0e7a-484d-8d49-4e9125ac96de" => Ok(Self::FullColor),
            "f42fbf9e-188a-447b-9fdc-f19dc1e4d685" => Ok(Self::Music),
            "f4122d1c-3b44-44d0-9936-ff7502c39ad3" => Ok(Self::Adaptation),
            "ee968100-4191-4968-93d3-f82d72be7e46" => Ok(Self::Mystery),
            "eabc5b4c-6aff-42f3-b657-3e90cbd00b75" => Ok(Self::Supernatural),
            "ea2bc92d-1c26-4930-9b7c-d5c0dc1b6869" => Ok(Self::Cooking),
            "e64f6742-c834-471d-8d72-dd51fc02b835" => Ok(Self::Aliens),
            "e5301a23-ebd9-49dd-a0cb-2add944c7fe9" => Ok(Self::SliceOfLife),
            "e197df38-d0e7-43b5-9b09-2842d0c326dd" => Ok(Self::WebComic),
            "df33b754-73a3-4c54-80e6-1a74a8058539" => Ok(Self::Police),
            "0a39b5a1-b235-4886-a747-1d05d216532d" => Ok(Self::AwardWinning),
            "0bc90acb-ccc1-44ca-a34a-b9f3a73259d0" => Ok(Self::Reincarnation),
            "2bd2e8d0-f146-434a-9b51-fc9ff2c5fe6a" => Ok(Self::GenderSwap),
            "2d1f5d56-a1e5-4d0d-a961-2193588b08ec" => Ok(Self::Lolicon),
            "3b60b75c-a2d7-4860-ab56-05f391bb889c" => Ok(Self::Psychological),
            "3bb26d85-09d5-4d2e-880c-c34b974339e9" => Ok(Self::Ghost),
            "3de8c75d-8ee3-48ff-98ee-e20a65c86451" => Ok(Self::Animals),
            "3e2b8dae-350e-4ab8-a8ce-016e844b9f0d" => Ok(Self::LongStrip),
            "4d32cc48-9f00-4cca-9b5a-a839f0764984" => Ok(Self::Comedy),
            "5bd0e105-4481-44ca-b6e7-7544da56b1a3" => Ok(Self::Incest),
            "5ca48985-9a9d-4bd8-be29-80dc0303db72" => Ok(Self::Crime),
            "5fff9cde-849c-4d78-aab0-0d52b2ee1d25" => Ok(Self::Survival),
            "7b2ce280-79ef-4c09-9b58-12b7c23a9b78" => Ok(Self::FanColored),
            "8c86611e-fab7-4986-9dec-d1a2f44acdd5" => Ok(Self::VirtualReality),
            "9ab53f92-3eed-4e9b-903a-917c86035ee3" => Ok(Self::Crossdressing),
            "36fd93ea-e8b8-445e-b836-358f02b3d33d" => Ok(Self::Monsters),
            "51d83883-4103-437c-b4b1-731cb73d786c" => Ok(Self::Anthology),
            "81c836c9-914a-4eca-981a-560dad663e73" => Ok(Self::MagicalGirls),
            "85daba54-a71c-4554-8a28-9901a8b0afad" => Ok(Self::Mafia),
            "87cc87cd-a395-47af-b27a-93258283bbc6" => Ok(Self::Adventure),
            "92d6d951-ca5e-429c-ac78-451071cbf064" => Ok(Self::OfficeWorkers),
            "0234a31e-a729-4e28-9d6a-3f87c4966b9e" => Ok(Self::OneShot),
            "256c8bd9-4904-4360-bf4f-508a76d67183" => Ok(Self::SciFi),
            "292e862b-2d17-4062-90a2-0356caa4ae27" => Ok(Self::TimeTravel),
            "391b0423-d847-456f-aff0-8b0cfc03066b" => Ok(Self::Action),
            "423e2eae-a7a2-4a8b-ac03-a8351462d71d" => Ok(Self::Romance),
            "489dd859-9b61-4c37-af75-5b18e88daafc" => Ok(Self::Ninja),
            "631ef465-9aba-4afb-b0fc-ea10efe274a8" => Ok(Self::Zombies),
            "799c202e-7daa-44eb-9cf7-8a3c0441531e" => Ok(Self::MartialArts),
            "891cf039-b895-47f0-9229-bef4c96eccd4" => Ok(Self::SelfPublished),
            "5920b825-4181-4a17-beeb-9918b0ff7a30" => Ok(Self::BoysLove),
            "7064a261-a137-4d3a-8848-2d385de3a99c" => Ok(Self::SuperHero),
            "9438db5a-7e2a-4ac0-b39e-e0d95a34b8a8" => Ok(Self::VideoGames),
            "31932a7e-5b8e-49a6-9f12-2afa39dc544c" => Ok(Self::TraditionalGames),
            "50880a9d-5440-4732-9afb-8f457127e836" => Ok(Self::Mecha),
            "65761a2a-415e-47f3-bef2-a9dababba7a6" => Ok(Self::ReverseHarem),
            "69964a64-2f90-4d33-beeb-f3ed2875eb4c" => Ok(Self::Sports),
            "97893a4c-12af-4dac-b6be-0dffb353568e" => Ok(Self::SexualViolence),
            "320831a8-4026-470b-94f6-8353740e6f04" => Ok(Self::OfficialColored),
            "07251805-a27e-4d59-b488-f0bfbec15168" => Ok(Self::Thriller),
            "9467335a-1b83-4497-9231-765337a00b96" => Ok(Self::PostApocalyptic),
            "33771934-028e-4cb3-8744-691e866a923e" => Ok(Self::Historical),
            "39730448-9a5f-48a2-85b0-a70db87b1233" => Ok(Self::Demons),
            "81183756-1453-4c81-aa9e-f6e1b63be016" => Ok(Self::Samurai),
            "a1f53773-c69a-4ce5-8cab-fffcd90b1565" => Ok(Self::Magic),
            "a3c67850-4684-404e-9b7f-c69850ee5da6" => Ok(Self::GirlsLove),
            "aafb99c1-7f60-43fa-b75f-fc9502ce29c7" => Ok(Self::Harem),
            "ac72833b-c4e9-4878-b9db-6c8a4a99444a" => Ok(Self::Military),
            "acc803a4-c95a-4c22-86fc-eb6b582d82a2" => Ok(Self::Wuxia),
            "ace04997-f6bd-436e-b261-779182193d3d" => Ok(Self::Isekai),
            "b1e97889-25b4-4258-b28b-cd7f4d28ea9b" => Ok(Self::Philosophical),
            "b9af3a63-f058-46de-a9a0-e0c13906197a" => Ok(Self::Drama),
            "b11fda93-8f1d-4bef-b2ed-8803d3733170" => Ok(Self::FourKoma),
            "b13b2a48-c720-44a9-9c77-39c9979373fb" => Ok(Self::Doujinshi),
            "b29d6a3d-1569-4e7a-8caf-7557bc92cd5d" => Ok(Self::Gore),
            "c8cbe35b-1b2b-4a3f-9c37-db84c4514856" => Ok(Self::Medical),
            "caaa44eb-cd40-4177-b930-79d3ef2afe87" => Ok(Self::SchoolLife),
            "cdad7e68-1419-41dd-bdce-27753074a640" => Ok(Self::Horror),
            "cdc58593-87dd-415e-bbc0-2ec27bf404cc" => Ok(Self::Fantasy),
            "d7d1730f-6eb0-4ba6-9437-602cac38664c" => Ok(Self::Vampires),
            "d14322ac-4d6f-4e9b-afd9-629d5f4d8a41" => Ok(Self::Villainess),
            "da2d50ca-3018-4cc0-ac7a-6b7d472a29ea" => Ok(Self::Delinquents),
            "dd1f77c5-dea9-4e2b-97ae-224af09caf99" => Ok(Self::MonsterGirls),
            "ddefd648-5140-4e5f-ba18-4eca4071d19b" => Ok(Self::Shotacon),
            _ => Err(SourceError::InvalidGenre(value)),
        }
    }
}

impl From<MangadexGenre> for SourceSerieGenre {
    fn from(value: MangadexGenre) -> Self {
        match value {
            MangadexGenre::Action => Self::Action,
            MangadexGenre::Adaptation => Self::Adaptation,
            MangadexGenre::Adventure => Self::Adventure,
            MangadexGenre::Aliens => Self::Aliens,
            MangadexGenre::Animals => Self::Animals,
            MangadexGenre::Anthology => Self::Anthology,
            MangadexGenre::AwardWinning => Self::AwardWinning,
            MangadexGenre::BoysLove => Self::BoysLove,
            MangadexGenre::Comedy => Self::Comedy,
            MangadexGenre::Cooking => Self::Cooking,
            MangadexGenre::Crime => Self::Crime,
            MangadexGenre::Crossdressing => Self::Crossdressing,
            MangadexGenre::Delinquents => Self::Delinquents,
            MangadexGenre::Demons => Self::Demons,
            MangadexGenre::Doujinshi => Self::Doujinshi,
            MangadexGenre::Drama => Self::Drama,
            MangadexGenre::FanColored => Self::FanColored,
            MangadexGenre::Fantasy => Self::Fantasy,
            MangadexGenre::FourKoma => Self::FourKoma,
            MangadexGenre::FullColor => Self::FullColor,
            MangadexGenre::GenderSwap => Self::GenderSwap,
            MangadexGenre::Ghost => Self::Ghost,
            MangadexGenre::GirlsLove => Self::GirlsLove,
            MangadexGenre::Gore => Self::Gore,
            MangadexGenre::Gyaru => Self::Gyaru,
            MangadexGenre::Harem => Self::Harem,
            MangadexGenre::Historical => Self::Historical,
            MangadexGenre::Horror => Self::Horror,
            MangadexGenre::Incest => Self::Incest,
            MangadexGenre::Isekai => Self::Isekai,
            MangadexGenre::Lolicon => Self::Lolicon,
            MangadexGenre::LongStrip => Self::LongStrip,
            MangadexGenre::Mafia => Self::Mafia,
            MangadexGenre::Magic => Self::Magic,
            MangadexGenre::MagicalGirls => Self::MagicalGirls,
            MangadexGenre::MartialArts => Self::MartialArts,
            MangadexGenre::Mecha => Self::Mecha,
            MangadexGenre::Medical => Self::Medical,
            MangadexGenre::Military => Self::Military,
            MangadexGenre::MonsterGirls => Self::MonsterGirls,
            MangadexGenre::Monsters => Self::Monsters,
            MangadexGenre::Music => Self::Music,
            MangadexGenre::Mystery => Self::Mystery,
            MangadexGenre::Ninja => Self::Ninja,
            MangadexGenre::OfficeWorkers => Self::OfficeWorkers,
            MangadexGenre::OfficialColored => Self::OfficialColored,
            MangadexGenre::OneShot => Self::OneShot,
            MangadexGenre::Philosophical => Self::Philosophical,
            MangadexGenre::Police => Self::Police,
            MangadexGenre::PostApocalyptic => Self::PostApocalyptic,
            MangadexGenre::Psychological => Self::Psychological,
            MangadexGenre::Reincarnation => Self::Reincarnation,
            MangadexGenre::ReverseHarem => Self::ReverseHarem,
            MangadexGenre::Romance => Self::Romance,
            MangadexGenre::Samurai => Self::Samurai,
            MangadexGenre::SchoolLife => Self::SchoolLife,
            MangadexGenre::SciFi => Self::SciFi,
            MangadexGenre::SelfPublished => Self::SelfPublished,
            MangadexGenre::SexualViolence => Self::SexualViolence,
            MangadexGenre::Shotacon => Self::Shotacon,
            MangadexGenre::SliceOfLife => Self::SliceOfLife,
            MangadexGenre::Sports => Self::Sports,
            MangadexGenre::SuperHero => Self::SuperHero,
            MangadexGenre::Supernatural => Self::Supernatural,
            MangadexGenre::Survival => Self::Survival,
            MangadexGenre::Thriller => Self::Thriller,
            MangadexGenre::TimeTravel => Self::TimeTravel,
            MangadexGenre::TraditionalGames => Self::TraditionalGames,
            MangadexGenre::Tragedy => Self::Tragedy,
            MangadexGenre::Vampires => Self::Vampires,
            MangadexGenre::VideoGames => Self::VideoGames,
            MangadexGenre::Villainess => Self::Villainess,
            MangadexGenre::VirtualReality => Self::VirtualReality,
            MangadexGenre::WebComic => Self::WebComic,
            MangadexGenre::Wuxia => Self::Wuxia,
            MangadexGenre::Zombies => Self::Zombies,
            MangadexGenre::Unknown => Self::Unknown,
        }
    }
}

impl From<SourceSerieGenre> for MangadexGenre {
    fn from(value: SourceSerieGenre) -> Self {
        match value {
            SourceSerieGenre::Other => MangadexGenre::Unknown,
            SourceSerieGenre::GenderBender => MangadexGenre::Unknown,
            SourceSerieGenre::Hentai => MangadexGenre::Unknown,
            SourceSerieGenre::Josei => MangadexGenre::Unknown,
            SourceSerieGenre::Kids => MangadexGenre::Unknown,
            SourceSerieGenre::Mature => MangadexGenre::Unknown,
            SourceSerieGenre::PsychologicalRomance => MangadexGenre::Unknown,
            SourceSerieGenre::Seinen => MangadexGenre::Unknown,
            SourceSerieGenre::Shoujo => MangadexGenre::Unknown,
            SourceSerieGenre::ShoujoAi => MangadexGenre::Unknown,
            SourceSerieGenre::Shounen => MangadexGenre::Unknown,
            SourceSerieGenre::ShounenAi => MangadexGenre::Unknown,
            SourceSerieGenre::Smut => MangadexGenre::Unknown,
            SourceSerieGenre::Space => MangadexGenre::Unknown,
            SourceSerieGenre::Suspense => MangadexGenre::Unknown,
            SourceSerieGenre::Toomics => MangadexGenre::Unknown,
            SourceSerieGenre::Yaoi => MangadexGenre::Unknown,
            SourceSerieGenre::Yuri => MangadexGenre::Unknown,
            SourceSerieGenre::Adult => MangadexGenre::Unknown,
            SourceSerieGenre::Action => MangadexGenre::Action,
            SourceSerieGenre::Adaptation => MangadexGenre::Adaptation,
            SourceSerieGenre::Adventure => MangadexGenre::Adventure,
            SourceSerieGenre::Aliens => MangadexGenre::Aliens,
            SourceSerieGenre::Animals => MangadexGenre::Animals,
            SourceSerieGenre::Anthology => MangadexGenre::Anthology,
            SourceSerieGenre::AwardWinning => MangadexGenre::AwardWinning,
            SourceSerieGenre::BoysLove => MangadexGenre::BoysLove,
            SourceSerieGenre::Comedy => MangadexGenre::Comedy,
            SourceSerieGenre::Cooking => MangadexGenre::Cooking,
            SourceSerieGenre::Crime => MangadexGenre::Crime,
            SourceSerieGenre::Crossdressing => MangadexGenre::Crossdressing,
            SourceSerieGenre::Delinquents => MangadexGenre::Delinquents,
            SourceSerieGenre::Demons => MangadexGenre::Demons,
            SourceSerieGenre::Doujinshi => MangadexGenre::Doujinshi,
            SourceSerieGenre::Drama => MangadexGenre::Drama,
            SourceSerieGenre::Ecchi => MangadexGenre::Unknown,
            SourceSerieGenre::FanColored => MangadexGenre::FanColored,
            SourceSerieGenre::Fantasy => MangadexGenre::Fantasy,
            SourceSerieGenre::FourKoma => MangadexGenre::FourKoma,
            SourceSerieGenre::FullColor => MangadexGenre::FullColor,
            SourceSerieGenre::GenderSwap => MangadexGenre::GenderSwap,
            SourceSerieGenre::Ghost => MangadexGenre::Ghost,
            SourceSerieGenre::GirlsLove => MangadexGenre::GirlsLove,
            SourceSerieGenre::Gore => MangadexGenre::Gore,
            SourceSerieGenre::Gyaru => MangadexGenre::Gyaru,
            SourceSerieGenre::Harem => MangadexGenre::Harem,
            SourceSerieGenre::Historical => MangadexGenre::Historical,
            SourceSerieGenre::Horror => MangadexGenre::Horror,
            SourceSerieGenre::Incest => MangadexGenre::Incest,
            SourceSerieGenre::Isekai => MangadexGenre::Isekai,
            SourceSerieGenre::Lolicon => MangadexGenre::Lolicon,
            SourceSerieGenre::LongStrip => MangadexGenre::LongStrip,
            SourceSerieGenre::Mafia => MangadexGenre::Mafia,
            SourceSerieGenre::Magic => MangadexGenre::Magic,
            SourceSerieGenre::MagicalGirls => MangadexGenre::MagicalGirls,
            SourceSerieGenre::MartialArts => MangadexGenre::MartialArts,
            SourceSerieGenre::Mecha => MangadexGenre::Mecha,
            SourceSerieGenre::Medical => MangadexGenre::Medical,
            SourceSerieGenre::Military => MangadexGenre::Military,
            SourceSerieGenre::MonsterGirls => MangadexGenre::MonsterGirls,
            SourceSerieGenre::Monsters => MangadexGenre::Monsters,
            SourceSerieGenre::Music => MangadexGenre::Music,
            SourceSerieGenre::Mystery => MangadexGenre::Mystery,
            SourceSerieGenre::Ninja => MangadexGenre::Ninja,
            SourceSerieGenre::OfficeWorkers => MangadexGenre::OfficeWorkers,
            SourceSerieGenre::OfficialColored => MangadexGenre::OfficialColored,
            SourceSerieGenre::OneShot => MangadexGenre::OneShot,
            SourceSerieGenre::Philosophical => MangadexGenre::Philosophical,
            SourceSerieGenre::Police => MangadexGenre::Police,
            SourceSerieGenre::PostApocalyptic => MangadexGenre::PostApocalyptic,
            SourceSerieGenre::Psychological => MangadexGenre::Psychological,
            SourceSerieGenre::Reincarnation => MangadexGenre::Reincarnation,
            SourceSerieGenre::ReverseHarem => MangadexGenre::ReverseHarem,
            SourceSerieGenre::Romance => MangadexGenre::Romance,
            SourceSerieGenre::Samurai => MangadexGenre::Samurai,
            SourceSerieGenre::SchoolLife => MangadexGenre::SchoolLife,
            SourceSerieGenre::SciFi => MangadexGenre::SciFi,
            SourceSerieGenre::SelfPublished => MangadexGenre::SelfPublished,
            SourceSerieGenre::SexualViolence => MangadexGenre::SexualViolence,
            SourceSerieGenre::Shotacon => MangadexGenre::Shotacon,
            SourceSerieGenre::SliceOfLife => MangadexGenre::SliceOfLife,
            SourceSerieGenre::Sports => MangadexGenre::Sports,
            SourceSerieGenre::SuperHero => MangadexGenre::SuperHero,
            SourceSerieGenre::Supernatural => MangadexGenre::Supernatural,
            SourceSerieGenre::Survival => MangadexGenre::Survival,
            SourceSerieGenre::Thriller => MangadexGenre::Thriller,
            SourceSerieGenre::TimeTravel => MangadexGenre::TimeTravel,
            SourceSerieGenre::TraditionalGames => MangadexGenre::TraditionalGames,
            SourceSerieGenre::Tragedy => MangadexGenre::Tragedy,
            SourceSerieGenre::Vampires => MangadexGenre::Vampires,
            SourceSerieGenre::VideoGames => MangadexGenre::VideoGames,
            SourceSerieGenre::Villainess => MangadexGenre::Villainess,
            SourceSerieGenre::VirtualReality => MangadexGenre::VirtualReality,
            SourceSerieGenre::WebComic => MangadexGenre::WebComic,
            SourceSerieGenre::Wuxia => MangadexGenre::Wuxia,
            SourceSerieGenre::Zombies => MangadexGenre::Zombies,
            SourceSerieGenre::Unknown => MangadexGenre::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum MangadexStatus {
    #[serde(rename = "ongoing")]
    #[strum(serialize = "ongoing")]
    Ongoing,
    #[serde(rename = "completed")]
    #[strum(serialize = "completed")]
    Completed,
    #[serde(rename = "hiatus")]
    #[strum(serialize = "hiatus")]
    Hiatus,
    #[serde(rename = "canceled")]
    #[strum(serialize = "canceled")]
    Canceled,
    #[serde(rename = "published")]
    #[strum(serialize = "published")]
    Published,
    #[serde(rename = "unknown")]
    #[strum(serialize = "unknown")]
    Unknown,
}

impl TryFrom<String> for MangadexStatus {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "ongoing" => Ok(Self::Ongoing),
            "completed" => Ok(Self::Completed),
            "hiatus" => Ok(Self::Hiatus),
            "canceled" => Ok(Self::Canceled),
            "published" => Ok(Self::Published),
            "unknown" => Ok(Self::Unknown),
            _ => Err(format!("Invalid status: {}", value)),
        }
    }
}

impl From<MangadexStatus> for SourceSerieStatus {
    fn from(value: MangadexStatus) -> Self {
        match value {
            MangadexStatus::Ongoing => Self::Ongoing,
            MangadexStatus::Completed => Self::Completed,
            MangadexStatus::Hiatus => Self::Hiatus,
            MangadexStatus::Canceled => Self::Canceled,
            MangadexStatus::Published => Self::Published,
            MangadexStatus::Unknown => Self::Unknown,
        }
    }
}

impl From<SourceSerieStatus> for MangadexStatus {
    fn from(value: SourceSerieStatus) -> Self {
        match value {
            SourceSerieStatus::Ongoing => MangadexStatus::Ongoing,
            SourceSerieStatus::Completed => MangadexStatus::Completed,
            SourceSerieStatus::Hiatus => MangadexStatus::Hiatus,
            SourceSerieStatus::Canceled => MangadexStatus::Canceled,
            SourceSerieStatus::Published => MangadexStatus::Published,
            SourceSerieStatus::Unknown => MangadexStatus::Unknown,
            SourceSerieStatus::Publishing => MangadexStatus::Unknown,
            SourceSerieStatus::Scanlating => MangadexStatus::Unknown,
            SourceSerieStatus::Scanlated => MangadexStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum MangadexSort {
    #[serde(rename = "followedCount")]
    #[strum(serialize = "followedCount")]
    FollowerCount,
    #[serde(rename = "latestUploadedChapter")]
    #[strum(serialize = "latestUploadedChapter")]
    LatestUploadedChapter,
    #[serde(rename = "relevance")]
    #[strum(serialize = "relevance")]
    Relevance,
    #[serde(rename = "title")]
    #[strum(serialize = "title")]
    Title,
}

impl From<MangadexSort> for FetchSearchSerieFilterSort {
    fn from(value: MangadexSort) -> Self {
        match value {
            MangadexSort::FollowerCount => Self::Popularity,
            MangadexSort::LatestUploadedChapter => Self::Latest,
            MangadexSort::Relevance => Self::Relevance,
            MangadexSort::Title => Self::Alphabetic,
        }
    }
}

impl From<FetchSearchSerieFilterSort> for MangadexSort {
    fn from(value: FetchSearchSerieFilterSort) -> Self {
        match value {
            FetchSearchSerieFilterSort::Popularity => MangadexSort::FollowerCount,
            FetchSearchSerieFilterSort::Latest => MangadexSort::LatestUploadedChapter,
            FetchSearchSerieFilterSort::Relevance => MangadexSort::Relevance,
            FetchSearchSerieFilterSort::Alphabetic => MangadexSort::Title,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum MangadexOrder {
    #[serde(rename = "asc")]
    #[strum(serialize = "asc")]
    ASC,
    #[serde(rename = "desc")]
    #[strum(serialize = "desc")]
    DESC,
}

impl From<MangadexOrder> for FetchSearchSerieFilterOrder {
    fn from(value: MangadexOrder) -> Self {
        match value {
            MangadexOrder::ASC => Self::ASC,
            MangadexOrder::DESC => Self::DESC,
        }
    }
}

impl From<FetchSearchSerieFilterOrder> for MangadexOrder {
    fn from(value: FetchSearchSerieFilterOrder) -> Self {
        match value {
            FetchSearchSerieFilterOrder::ASC => MangadexOrder::ASC,
            FetchSearchSerieFilterOrder::DESC => MangadexOrder::DESC,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display)]
pub enum MangadexLanguage {
    #[serde(rename = "en")]
    #[strum(serialize = "en")]
    En,
    #[serde(rename = "fr")]
    #[strum(serialize = "fr")]
    Fr,
    #[serde(rename = "ko")]
    #[strum(serialize = "ko")]
    Ko,
    #[serde(rename = "ja")]
    #[strum(serialize = "ja")]
    Ja,
    #[serde(rename = "ja-ro")]
    #[strum(serialize = "ja-ro")]
    JaRo,
    #[serde(rename = "zh-hk")]
    #[strum(serialize = "zh-hk")]
    ZhHk,
    #[serde(rename = "zh")]
    #[strum(serialize = "zh")]
    Zh,
}

impl TryFrom<String> for MangadexLanguage {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "en" => Ok(Self::En),
            "fr" => Ok(Self::Fr),
            "ko" => Ok(Self::Ko),
            "ja" => Ok(Self::Ja),
            "ja-ro" => Ok(Self::JaRo),
            "zh-hk" => Ok(Self::ZhHk),
            "zh" | "zh-cn" => Ok(Self::Zh),
            _ => Err(format!("{} no match", value)),
        }
    }
}

impl From<MangadexLanguage> for SourceLanguage {
    fn from(value: MangadexLanguage) -> Self {
        match value {
            MangadexLanguage::En => Self::En,
            MangadexLanguage::Fr => Self::Fr,
            MangadexLanguage::Ko => Self::Ko,
            MangadexLanguage::Ja => Self::Jp,
            MangadexLanguage::JaRo => Self::JpRo,
            MangadexLanguage::ZhHk => Self::ZhHk,
            MangadexLanguage::Zh => Self::Zh,
        }
    }
}

impl From<SourceLanguage> for MangadexLanguage {
    fn from(value: SourceLanguage) -> Self {
        match value {
            SourceLanguage::En => Self::En,
            SourceLanguage::Fr => Self::Fr,
            SourceLanguage::Ko => Self::Ko,
            SourceLanguage::Jp => Self::Ja,
            SourceLanguage::JpRo => Self::JaRo,
            SourceLanguage::ZhHk => Self::ZhHk,
            SourceLanguage::Zh => Self::Zh,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexSingleResponse<T> {
    pub result: String,
    pub response: String,
    pub data: T,
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
    pub state: Option<String>,
    pub chapter_numbers_reset_on_new_volume: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
    // MangaDex occasionally returns null entries inside this array (e.g., ["en", null]).
    // Accept Option<String> items to avoid deserialization failures and ignore nulls upstream.
    pub available_translated_languages: Vec<Option<String>>,
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
    pub description: Option<serde_json::Value>,
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
pub struct MangaDexManga {
    pub id: String,
    #[serde(rename = "type")]
    pub manga_type: String,
    pub attributes: MangaDexMangaAttributes,
    pub relationships: Option<Vec<MangaDexRelationship>>,
}

impl MangaDexManga {
    pub fn cover_url(&self) -> Result<Url, SourceError> {
        // Try to extract the cover file name from relationships; fall back to a placeholder
        let filename_opt = self
            .relationships
            .as_ref()
            .and_then(|rels| rels.iter().find(|r| r.rel_type == "cover_art"))
            .and_then(|cover| {
                cover
                    .attributes
                    .as_ref()
                    .and_then(|a| a.get("fileName").and_then(|f| f.as_str()))
            });

        let url_str = if let Some(filename) = filename_opt {
            format!(
                "https://uploads.mangadex.org/covers/{}/{}.512.jpg",
                self.id, filename
            )
        } else {
            NO_IMAGE_URL.to_string()
        };

        Url::parse(&url_str).map_err(|e| SourceError::BuildingURL(e.to_string()))
    }

    fn get_type(
        &self,
        original_lang: MangadexLanguage,
        genres: Vec<SourceSerieGenre>,
    ) -> SourceSerieType {
        let is_long_strip = genres.contains(&SourceSerieGenre::LongStrip);
        let is_doujinshi = genres.contains(&SourceSerieGenre::Doujinshi);
        let is_web_comic = genres.contains(&SourceSerieGenre::WebComic);

        if is_doujinshi {
            return SourceSerieType::Doujinshi;
        }

        match original_lang {
            MangadexLanguage::Ja => SourceSerieType::Manga,
            MangadexLanguage::Ko => {
                if is_long_strip || is_web_comic {
                    SourceSerieType::Webtoon
                } else {
                    SourceSerieType::Manhwa
                }
            }
            MangadexLanguage::Zh | MangadexLanguage::ZhHk => SourceSerieType::Manhua,
            _ => SourceSerieType::Comic,
        }
    }
}

impl TryInto<SourceSmallSerie> for MangaDexManga {
    type Error = SourceError;

    fn try_into(self) -> Result<SourceSmallSerie, Self::Error> {
        let mut titles = MultiLanguageString::new();

        // Add main title
        for (lang_code, text) in &self.attributes.title {
            if let Ok(lang) = MangadexLanguage::try_from(lang_code.clone()) {
                titles = titles.insert(lang.into(), text.clone());
            }
        }

        Ok(SourceSmallSerie {
            id: self.id.clone(),
            title: titles,
            cover: self.cover_url()?,
        })
    }
}

impl TryInto<SourceSerie> for MangaDexManga {
    type Error = SourceError;

    fn try_into(self) -> Result<SourceSerie, Self::Error> {
        // Add main title
        let mut titles = MultiLanguageString::new();
        for (lang_code, text) in &self.attributes.title {
            if let Ok(lang) = MangadexLanguage::try_from(lang_code.clone()) {
                titles = titles.insert(lang.into(), text.clone());
            }
        }

        // Process descriptions
        let mut descriptions = MultiLanguageString::new();
        for (lang_code, text) in &self.attributes.description {
            if let Ok(lang) = MangadexLanguage::try_from(lang_code.clone()) {
                descriptions = descriptions.insert(lang.into(), text.clone());
            }
        }

        // Process alternative titles
        let mut alternative_titles = MultiLanguageString::new();
        for alt_title_map in &self.attributes.alt_titles {
            for (lang_code, text) in alt_title_map {
                if let Ok(lang) = MangadexLanguage::try_from(lang_code.clone()) {
                    alternative_titles = alternative_titles.insert(lang.into(), text.clone());
                }
            }
        }

        // Extract genres from tags
        let genres: Vec<SourceSerieGenre> = self
            .attributes
            .tags
            .iter()
            .filter(|tag| tag.attributes.group == "genre")
            .filter_map(|tag| MangadexGenre::try_from(tag.id.clone()).ok())
            .map(|genre| genre.into())
            .collect();

        // Extract authors and artists from relationships
        let mut authors = Vec::new();
        let mut artists = Vec::new();

        if let Some(relationships) = &self.relationships {
            for rel in relationships {
                match rel.rel_type.as_str() {
                    "author" => {
                        if let Some(attrs) = &rel.attributes
                            && let Some(name) = attrs.get("name").and_then(|v| v.as_str())
                        {
                            authors.push(name.to_string());
                        }
                    }
                    "artist" => {
                        if let Some(attrs) = &rel.attributes
                            && let Some(name) = attrs.get("name").and_then(|v| v.as_str())
                        {
                            artists.push(name.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Get status - handle both status and state
        let mut statuses = Vec::new();

        // Add status
        if let Ok(status) = MangadexStatus::try_from(self.attributes.status.clone()) {
            statuses.push(status.into());
        }

        // Add state if present
        if let Some(state_val) = &self.attributes.state
            && let Ok(state) = MangadexStatus::try_from(state_val.clone())
        {
            statuses.push(state.into());
        }

        // Get type based on original language and genres
        let original_lang = MangadexLanguage::try_from(self.attributes.original_language.clone())
            .expect("Error parsing mangadex original language");
        let serie_type = self.get_type(original_lang, genres.clone());

        Ok(SourceSerie {
            id: self.id.clone(),
            title: titles,
            alternates_titles: alternative_titles,
            cover: self.cover_url()?,
            synopsis: descriptions,
            serie_type,
            genres,
            status: statuses,
            authors,
            artists,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MangaDexChapter {
    pub id: String,
    #[serde(rename = "type")]
    pub chapter_type: String,
    pub attributes: MangaDexChapterAttributes,
    pub relationships: Vec<MangaDexRelationship>,
}

impl From<MangaDexChapter> for SourceSerieChapter {
    fn from(chapter: MangaDexChapter) -> Self {
        let (volume_number, volume_name) = if let Some(volume) = &chapter.attributes.volume {
            (volume.parse::<f64>().ok(), Some(volume.clone()))
        } else {
            (None, None)
        };

        let chapter_number = chapter
            .attributes
            .chapter
            .as_ref()
            .and_then(|c| c.parse::<f64>().ok())
            .unwrap_or(0.0);

        let title = chapter
            .attributes
            .title
            .clone()
            .unwrap_or_else(|| format!("Chapter {}", chapter_number));

        let language = MangadexLanguage::try_from(chapter.attributes.translated_language.clone())
            .expect("Failed to parse language")
            .into();

        let date_upload = chapter.attributes.publish_at;
        let chapter_id = chapter.id.clone();

        let external_url = Url::parse(&format!("https://mangadex.org/chapter/{}", chapter_id)).ok();

        SourceSerieChapter {
            id: chapter_id,
            title,
            chapter_number,
            language,
            date_upload: date_upload.into(),
            external_url,
            volume_number,
            volume_name,
        }
    }
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
