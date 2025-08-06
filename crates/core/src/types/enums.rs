use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[cfg(feature = "graphql")]
use async_graphql::Enum;

// ============= Language =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SourceLanguage {
    #[serde(rename = "en")]
    #[strum(serialize = "en")]
    En,

    #[serde(rename = "jp")]
    #[strum(serialize = "jp")]
    Jp,

    #[serde(rename = "jp-ro")]
    #[strum(serialize = "jp-ro")]
    JpRo,

    #[serde(rename = "fr")]
    #[strum(serialize = "fr")]
    Fr,

    #[serde(rename = "ko")]
    #[strum(serialize = "ko")]
    Ko,

    #[serde(rename = "zh")]
    #[strum(serialize = "zh")]
    Zh,

    #[serde(rename = "zh-hk")]
    #[strum(serialize = "zh-hk")]
    ZhHk,
}

// Keep the old Language enum as an alias for compatibility
pub type Language = SourceLanguage;

// ============= Serie Status =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SourceSerieStatus {
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

    #[serde(rename = "publishing")]
    #[strum(serialize = "publishing")]
    Publishing,

    #[serde(rename = "published")]
    #[strum(serialize = "published")]
    Published,

    #[serde(rename = "scanlating")]
    #[strum(serialize = "scanlating")]
    Scanlating,

    #[serde(rename = "scanlated")]
    #[strum(serialize = "scanlated")]
    Scanlated,

    #[serde(rename = "unknown")]
    #[strum(serialize = "unknown")]
    Unknown,
}

// Keep the old SerieStatus enum as an alias
pub type SerieStatus = SourceSerieStatus;

// ============= Serie Type =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SourceSerieType {
    #[serde(rename = "manga")]
    #[strum(serialize = "manga")]
    Manga,

    #[serde(rename = "manhwa")]
    #[strum(serialize = "manhwa")]
    Manhwa,

    #[serde(rename = "manhua")]
    #[strum(serialize = "manhua")]
    Manhua,

    #[serde(rename = "webtoon")]
    #[strum(serialize = "webtoon")]
    Webtoon,

    #[serde(rename = "lightnovel")]
    #[strum(serialize = "lightnovel")]
    LightNovel,

    #[serde(rename = "novel")]
    #[strum(serialize = "novel")]
    Novel,

    #[serde(rename = "doujinshi")]
    #[strum(serialize = "doujinshi")]
    Doujinshi,

    #[serde(rename = "comic")]
    #[strum(serialize = "comic")]
    Comic,

    #[serde(rename = "oel")]
    #[strum(serialize = "oel")]
    Oel,

    #[serde(rename = "unknown")]
    #[strum(serialize = "unknown")]
    Unknown,
}

// Keep the old SerieType enum as an alias
pub type SerieType = SourceSerieType;

// ============= Filter Order =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum FilterOrder {
    #[serde(rename = "ASC")]
    #[strum(serialize = "ASC")]
    Ascending,

    #[serde(rename = "DESC")]
    #[strum(serialize = "DESC")]
    Descending,
}

// ============= Filter Sort =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum FilterSort {
    #[serde(rename = "TITLE")]
    #[strum(serialize = "TITLE")]
    Title,

    #[serde(rename = "LATEST")]
    #[strum(serialize = "LATEST")]
    UpdatedAt,

    #[serde(rename = "CREATED")]
    #[strum(serialize = "CREATED")]
    CreatedAt,

    #[serde(rename = "CHAPTERS")]
    #[strum(serialize = "CHAPTERS")]
    ChapterCount,

    #[serde(rename = "RATING")]
    #[strum(serialize = "RATING")]
    Rating,

    #[serde(rename = "POPULARITY")]
    #[strum(serialize = "POPULARITY")]
    Popularity,

    #[serde(rename = "TRENDING")]
    #[strum(serialize = "TRENDING")]
    Trending,
}

// Aliases for compatibility
pub type SortBy = FilterSort;
pub type SortOrder = FilterOrder;

// ============= Serie Genre =============

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumString, Serialize, Deserialize)]
#[cfg_attr(feature = "graphql", derive(Enum))]
pub enum SourceSerieGenre {
    #[serde(rename = "Unknown")]
    #[strum(serialize = "Unknown")]
    Unknown,

    #[serde(rename = "Other")]
    #[strum(serialize = "Other")]
    Other,

    #[serde(rename = "4-Koma")]
    #[strum(serialize = "4-Koma")]
    FourKoma,

    #[serde(rename = "Action")]
    #[strum(serialize = "Action")]
    Action,

    #[serde(rename = "Adaptation")]
    #[strum(serialize = "Adaptation")]
    Adaptation,

    #[serde(rename = "Adult")]
    #[strum(serialize = "Adult")]
    Adult,

    #[serde(rename = "Adventure")]
    #[strum(serialize = "Adventure")]
    Adventure,

    #[serde(rename = "Aliens")]
    #[strum(serialize = "Aliens")]
    Aliens,

    #[serde(rename = "Animals")]
    #[strum(serialize = "Animals")]
    Animals,

    #[serde(rename = "Anthology")]
    #[strum(serialize = "Anthology")]
    Anthology,

    #[serde(rename = "Award Winning")]
    #[strum(serialize = "Award Winning")]
    AwardWinning,

    #[serde(rename = "Boy's Love")]
    #[strum(serialize = "Boy's Love")]
    BoysLove,

    #[serde(rename = "Comedy")]
    #[strum(serialize = "Comedy")]
    Comedy,

    #[serde(rename = "Cooking")]
    #[strum(serialize = "Cooking")]
    Cooking,

    #[serde(rename = "Crime")]
    #[strum(serialize = "Crime")]
    Crime,

    #[serde(rename = "Crossdressing")]
    #[strum(serialize = "Crossdressing")]
    Crossdressing,

    #[serde(rename = "Delinquents")]
    #[strum(serialize = "Delinquents")]
    Delinquents,

    #[serde(rename = "Demons")]
    #[strum(serialize = "Demons")]
    Demons,

    #[serde(rename = "Doujinshi")]
    #[strum(serialize = "Doujinshi")]
    Doujinshi,

    #[serde(rename = "Drama")]
    #[strum(serialize = "Drama")]
    Drama,

    #[serde(rename = "Ecchi")]
    #[strum(serialize = "Ecchi")]
    Ecchi,

    #[serde(rename = "Fan Colored")]
    #[strum(serialize = "Fan Colored")]
    FanColored,

    #[serde(rename = "Fantasy")]
    #[strum(serialize = "Fantasy")]
    Fantasy,

    #[serde(rename = "Full Color")]
    #[strum(serialize = "Full Color")]
    FullColor,

    #[serde(rename = "Gender Bender")]
    #[strum(serialize = "Gender Bender")]
    GenderBender,

    #[serde(rename = "Genderswap")]
    #[strum(serialize = "Genderswap")]
    Genderswap,

    #[serde(rename = "Ghost")]
    #[strum(serialize = "Ghost")]
    Ghost,

    #[serde(rename = "Girl's Love")]
    #[strum(serialize = "Girl's Love")]
    GirlsLove,

    #[serde(rename = "Gore")]
    #[strum(serialize = "Gore")]
    Gore,

    #[serde(rename = "Gyaru")]
    #[strum(serialize = "Gyaru")]
    Gyaru,

    #[serde(rename = "Harem")]
    #[strum(serialize = "Harem")]
    Harem,

    #[serde(rename = "Hentai")]
    #[strum(serialize = "Hentai")]
    Hentai,

    #[serde(rename = "Historical")]
    #[strum(serialize = "Historical")]
    Historical,

    #[serde(rename = "Horror")]
    #[strum(serialize = "Horror")]
    Horror,

    #[serde(rename = "Incest")]
    #[strum(serialize = "Incest")]
    Incest,

    #[serde(rename = "Isekai")]
    #[strum(serialize = "Isekai")]
    Isekai,

    #[serde(rename = "Josei")]
    #[strum(serialize = "Josei")]
    Josei,

    #[serde(rename = "Kids")]
    #[strum(serialize = "Kids")]
    Kids,

    #[serde(rename = "Lolicon")]
    #[strum(serialize = "Lolicon")]
    Lolicon,

    #[serde(rename = "Long Strip")]
    #[strum(serialize = "Long Strip")]
    LongStrip,

    #[serde(rename = "Mafia")]
    #[strum(serialize = "Mafia")]
    Mafia,

    #[serde(rename = "Magic")]
    #[strum(serialize = "Magic")]
    Magic,

    #[serde(rename = "Magical Girls")]
    #[strum(serialize = "Magical Girls")]
    MagicalGirls,

    #[serde(rename = "Martial Arts")]
    #[strum(serialize = "Martial Arts")]
    MartialArts,

    #[serde(rename = "Mature")]
    #[strum(serialize = "Mature")]
    Mature,

    #[serde(rename = "Mecha")]
    #[strum(serialize = "Mecha")]
    Mecha,

    #[serde(rename = "Medical")]
    #[strum(serialize = "Medical")]
    Medical,

    #[serde(rename = "Military")]
    #[strum(serialize = "Military")]
    Military,

    #[serde(rename = "Monster Girls")]
    #[strum(serialize = "Monster Girls")]
    MonsterGirls,

    #[serde(rename = "Monsters")]
    #[strum(serialize = "Monsters")]
    Monsters,

    #[serde(rename = "Music")]
    #[strum(serialize = "Music")]
    Music,

    #[serde(rename = "Mystery")]
    #[strum(serialize = "Mystery")]
    Mystery,

    #[serde(rename = "Ninja")]
    #[strum(serialize = "Ninja")]
    Ninja,

    #[serde(rename = "Office Workers")]
    #[strum(serialize = "Office Workers")]
    OfficeWorkers,

    #[serde(rename = "Official Colored")]
    #[strum(serialize = "Official Colored")]
    OfficialColored,

    #[serde(rename = "One Shot")]
    #[strum(serialize = "One Shot")]
    OneShot,

    #[serde(rename = "Philosophical")]
    #[strum(serialize = "Philosophical")]
    Philosophical,

    #[serde(rename = "Police")]
    #[strum(serialize = "Police")]
    Police,

    #[serde(rename = "Post-Apocalyptic")]
    #[strum(serialize = "Post-Apocalyptic")]
    PostApocalyptic,

    #[serde(rename = "Psychological")]
    #[strum(serialize = "Psychological")]
    Psychological,

    #[serde(rename = "Psychological Romance")]
    #[strum(serialize = "Psychological Romance")]
    PsychologicalRomance,

    #[serde(rename = "Reincarnation")]
    #[strum(serialize = "Reincarnation")]
    Reincarnation,

    #[serde(rename = "Reverse Harem")]
    #[strum(serialize = "Reverse Harem")]
    ReverseHarem,

    #[serde(rename = "Romance")]
    #[strum(serialize = "Romance")]
    Romance,

    #[serde(rename = "Samurai")]
    #[strum(serialize = "Samurai")]
    Samurai,

    #[serde(rename = "School Life")]
    #[strum(serialize = "School Life")]
    SchoolLife,

    #[serde(rename = "Sci-Fi")]
    #[strum(serialize = "Sci-Fi")]
    SciFi,

    #[serde(rename = "Seinen")]
    #[strum(serialize = "Seinen")]
    Seinen,

    #[serde(rename = "Self Published")]
    #[strum(serialize = "Self Published")]
    SelfPublished,

    #[serde(rename = "Sexual Violence")]
    #[strum(serialize = "Sexual Violence")]
    SexualViolence,

    #[serde(rename = "Shotacon")]
    #[strum(serialize = "Shotacon")]
    Shotacon,

    #[serde(rename = "Shoujo")]
    #[strum(serialize = "Shoujo")]
    Shoujo,

    #[serde(rename = "Shoujo Ai")]
    #[strum(serialize = "Shoujo Ai")]
    ShoujoAi,

    #[serde(rename = "Shounen")]
    #[strum(serialize = "Shounen")]
    Shounen,

    #[serde(rename = "Shounen Ai")]
    #[strum(serialize = "Shounen Ai")]
    ShounenAi,

    #[serde(rename = "Slice of Life")]
    #[strum(serialize = "Slice of Life")]
    SliceOfLife,

    #[serde(rename = "Smut")]
    #[strum(serialize = "Smut")]
    Smut,

    #[serde(rename = "Space")]
    #[strum(serialize = "Space")]
    Space,

    #[serde(rename = "Sports")]
    #[strum(serialize = "Sports")]
    Sports,

    #[serde(rename = "Superhero")]
    #[strum(serialize = "Superhero")]
    Superhero,

    #[serde(rename = "Supernatural")]
    #[strum(serialize = "Supernatural")]
    Supernatural,

    #[serde(rename = "Survival")]
    #[strum(serialize = "Survival")]
    Survival,

    #[serde(rename = "Suspense")]
    #[strum(serialize = "Suspense")]
    Suspense,

    #[serde(rename = "Thriller")]
    #[strum(serialize = "Thriller")]
    Thriller,

    #[serde(rename = "Time Travel")]
    #[strum(serialize = "Time Travel")]
    TimeTravel,

    #[serde(rename = "Toomics")]
    #[strum(serialize = "Toomics")]
    Toomics,

    #[serde(rename = "Traditional Games")]
    #[strum(serialize = "Traditional Games")]
    TraditionalGames,

    #[serde(rename = "Tragedy")]
    #[strum(serialize = "Tragedy")]
    Tragedy,

    #[serde(rename = "Vampires")]
    #[strum(serialize = "Vampires")]
    Vampires,

    #[serde(rename = "Video Games")]
    #[strum(serialize = "Video Games")]
    VideoGames,

    #[serde(rename = "Villainess")]
    #[strum(serialize = "Villainess")]
    Villainess,

    #[serde(rename = "Virtual Reality")]
    #[strum(serialize = "Virtual Reality")]
    VirtualReality,

    #[serde(rename = "Web Comic")]
    #[strum(serialize = "Web Comic")]
    WebComic,

    #[serde(rename = "Wuxia")]
    #[strum(serialize = "Wuxia")]
    Wuxia,

    #[serde(rename = "Yaoi")]
    #[strum(serialize = "Yaoi")]
    Yaoi,

    #[serde(rename = "Yuri")]
    #[strum(serialize = "Yuri")]
    Yuri,

    #[serde(rename = "Zombies")]
    #[strum(serialize = "Zombies")]
    Zombies,
}

// Keep the old Genre type as an alias
pub type Genre = SourceSerieGenre;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_language_serialization() {
        let lang = SourceLanguage::En;
        let json = serde_json::to_string(&lang).unwrap();
        assert_eq!(json, r#""en""#);

        let deserialized: SourceLanguage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, lang);
    }

    #[test]
    fn test_language_from_string() {
        assert_eq!(SourceLanguage::from_str("en").unwrap(), SourceLanguage::En);
        assert_eq!(SourceLanguage::from_str("fr").unwrap(), SourceLanguage::Fr);
        assert_eq!(
            SourceLanguage::from_str("zh-hk").unwrap(),
            SourceLanguage::ZhHk
        );
        assert!(SourceLanguage::from_str("invalid").is_err());
    }

    #[test]
    fn test_serie_status_serialization() {
        let status = SourceSerieStatus::Ongoing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""ongoing""#);

        let deserialized: SourceSerieStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }

    #[test]
    fn test_serie_type_display() {
        assert_eq!(SourceSerieType::Manga.to_string(), "manga");
        assert_eq!(SourceSerieType::Manhwa.to_string(), "manhwa");
    }

    #[test]
    fn test_filter_order_serialization() {
        let order = FilterOrder::Ascending;
        let json = serde_json::to_string(&order).unwrap();
        assert_eq!(json, r#""ASC""#);

        let order = FilterOrder::Descending;
        let json = serde_json::to_string(&order).unwrap();
        assert_eq!(json, r#""DESC""#);
    }

    #[test]
    fn test_genre_serialization() {
        let genre = SourceSerieGenre::Action;
        let json = serde_json::to_string(&genre).unwrap();
        assert_eq!(json, r#""Action""#);

        let genre = SourceSerieGenre::SliceOfLife;
        let json = serde_json::to_string(&genre).unwrap();
        assert_eq!(json, r#""Slice of Life""#);
    }
}
