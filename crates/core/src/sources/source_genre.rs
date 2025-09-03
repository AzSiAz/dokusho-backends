use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::SourceError;

fn source_serie_genre_parse_not_found(s: &str) -> SourceError {
    SourceError::InvalidGenre(s.to_string())
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    EnumString,
    EnumIter,
    EnumCount,
    Display,
    PartialEq,
    Eq,
    Hash,
)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_genre_parse_not_found,
)]
pub enum SourceSerieGenre {
    #[strum(serialize = "Unknown")]
    #[serde(rename = "Unknown")]
    Unknown,
    #[strum(serialize = "Other")]
    #[serde(rename = "Other")]
    Other,
    #[strum(serialize = "4-Koma")]
    #[serde(rename = "4-Koma")]
    FourKoma,
    #[strum(serialize = "Action")]
    #[serde(rename = "Action")]
    Action,
    #[strum(serialize = "Adaptation")]
    #[serde(rename = "Adaptation")]
    Adaptation,
    #[strum(serialize = "Adult")]
    #[serde(rename = "Adult")]
    Adult,
    #[strum(serialize = "Adventure")]
    #[serde(rename = "Adventure")]
    Adventure,
    #[strum(serialize = "Aliens")]
    #[serde(rename = "Aliens")]
    Aliens,
    #[strum(serialize = "Animals")]
    #[serde(rename = "Animals")]
    Animals,
    #[strum(serialize = "Anthology")]
    #[serde(rename = "Anthology")]
    Anthology,
    #[strum(serialize = "Award Winning")]
    #[serde(rename = "Award Winning")]
    AwardWinning,
    #[strum(serialize = "Boy's Love")]
    #[serde(rename = "Boy's Love")]
    BoysLove,
    #[strum(serialize = "Comedy")]
    #[serde(rename = "Comedy")]
    Comedy,
    #[strum(serialize = "Cooking")]
    #[serde(rename = "Cooking")]
    Cooking,
    #[strum(serialize = "Crime")]
    #[serde(rename = "Crime")]
    Crime,
    #[strum(serialize = "Crossdressing")]
    #[serde(rename = "Crossdressing")]
    Crossdressing,
    #[strum(serialize = "Delinquents")]
    #[serde(rename = "Delinquents")]
    Delinquents,
    #[strum(serialize = "Demons")]
    #[serde(rename = "Demons")]
    Demons,
    #[strum(serialize = "Doujinshi")]
    #[serde(rename = "Doujinshi")]
    Doujinshi,
    #[strum(serialize = "Drama")]
    #[serde(rename = "Drama")]
    Drama,
    #[strum(serialize = "Ecchi")]
    #[serde(rename = "Ecchi")]
    Ecchi,
    #[strum(serialize = "Fan Colored")]
    #[serde(rename = "Fan Colored")]
    FanColored,
    #[strum(serialize = "Fantasy")]
    #[serde(rename = "Fantasy")]
    Fantasy,
    #[strum(serialize = "Full Color")]
    #[serde(rename = "Full Color")]
    FullColor,
    #[strum(serialize = "Gender Bender")]
    #[serde(rename = "Gender Bender")]
    GenderBender,
    #[strum(serialize = "Genderswap")]
    #[serde(rename = "Genderswap")]
    GenderSwap,
    #[strum(serialize = "Ghost")]
    #[serde(rename = "Ghost")]
    Ghost,
    #[strum(serialize = "Girl's Love")]
    #[serde(rename = "Girl's Love")]
    GirlsLove,
    #[strum(serialize = "Gore")]
    #[serde(rename = "Gore")]
    Gore,
    #[strum(serialize = "Gyaru")]
    #[serde(rename = "Gyaru")]
    Gyaru,
    #[strum(serialize = "Harem")]
    #[serde(rename = "Harem")]
    Harem,
    #[strum(serialize = "Hentai")]
    #[serde(rename = "Hentai")]
    Hentai,
    #[strum(serialize = "Historical")]
    #[serde(rename = "Historical")]
    Historical,
    #[strum(serialize = "Horror")]
    #[serde(rename = "Horror")]
    Horror,
    #[strum(serialize = "Incest")]
    #[serde(rename = "Incest")]
    Incest,
    #[strum(serialize = "Isekai")]
    #[serde(rename = "Isekai")]
    Isekai,
    #[strum(serialize = "Josei")]
    #[serde(rename = "Josei")]
    Josei,
    #[strum(serialize = "Kids")]
    #[serde(rename = "Kids")]
    Kids,
    #[strum(serialize = "Lolicon")]
    #[serde(rename = "Lolicon")]
    Lolicon,
    #[strum(serialize = "Long Strip")]
    #[serde(rename = "Long Strip")]
    LongStrip,
    #[strum(serialize = "Mafia")]
    #[serde(rename = "Mafia")]
    Mafia,
    #[strum(serialize = "Magic")]
    #[serde(rename = "Magic")]
    Magic,
    #[strum(serialize = "Magical Girls")]
    #[serde(rename = "Magical Girls")]
    MagicalGirls,
    #[strum(serialize = "Martial Arts")]
    #[serde(rename = "Martial Arts")]
    MartialArts,
    #[strum(serialize = "Mature")]
    #[serde(rename = "Mature")]
    Mature,
    #[strum(serialize = "Mecha")]
    #[serde(rename = "Mecha")]
    Mecha,
    #[strum(serialize = "Medical")]
    #[serde(rename = "Medical")]
    Medical,
    #[strum(serialize = "Military")]
    #[serde(rename = "Military")]
    Military,
    #[strum(serialize = "Monster Girls")]
    #[serde(rename = "Monster Girls")]
    MonsterGirls,
    #[strum(serialize = "Monsters")]
    #[serde(rename = "Monsters")]
    Monsters,
    #[strum(serialize = "Music")]
    #[serde(rename = "Music")]
    Music,
    #[strum(serialize = "Mystery")]
    #[serde(rename = "Mystery")]
    Mystery,
    #[strum(serialize = "Ninja")]
    #[serde(rename = "Ninja")]
    Ninja,
    #[strum(serialize = "Office Workers")]
    #[serde(rename = "Office Workers")]
    OfficeWorkers,
    #[strum(serialize = "Official Colored")]
    #[serde(rename = "Official Colored")]
    OfficialColored,
    #[strum(serialize = "One Shot")]
    #[serde(rename = "One Shot")]
    OneShot,
    #[strum(serialize = "Philosophical")]
    #[serde(rename = "Philosophical")]
    Philosophical,
    #[strum(serialize = "Police")]
    #[serde(rename = "Police")]
    Police,
    #[strum(serialize = "Post-Apocalyptic")]
    #[serde(rename = "Post-Apocalyptic")]
    PostApocalyptic,
    #[strum(serialize = "Psychological")]
    #[serde(rename = "Psychological")]
    Psychological,
    #[strum(serialize = "Psychological Romance")]
    #[serde(rename = "Psychological Romance")]
    PsychologicalRomance,
    #[strum(serialize = "Reincarnation")]
    #[serde(rename = "Reincarnation")]
    Reincarnation,
    #[strum(serialize = "Reverse Harem")]
    #[serde(rename = "Reverse Harem")]
    ReverseHarem,
    #[strum(serialize = "Romance")]
    #[serde(rename = "Romance")]
    Romance,
    #[strum(serialize = "Samurai")]
    #[serde(rename = "Samurai")]
    Samurai,
    #[strum(serialize = "School Life")]
    #[serde(rename = "School Life")]
    SchoolLife,
    #[strum(serialize = "Sci-Fi")]
    #[serde(rename = "Sci-Fi")]
    SciFi,
    #[strum(serialize = "Seinen")]
    #[serde(rename = "Seinen")]
    Seinen,
    #[strum(serialize = "Self Published")]
    #[serde(rename = "Self Published")]
    SelfPublished,
    #[strum(serialize = "Sexual Violence")]
    #[serde(rename = "Sexual Violence")]
    SexualViolence,
    #[strum(serialize = "Shotacon")]
    #[serde(rename = "Shotacon")]
    Shotacon,
    #[strum(serialize = "Shoujo")]
    #[serde(rename = "Shoujo")]
    Shoujo,
    #[strum(serialize = "Shoujo Ai")]
    #[serde(rename = "Shoujo Ai")]
    ShoujoAi,
    #[strum(serialize = "Shounen")]
    #[serde(rename = "Shounen")]
    Shounen,
    #[strum(serialize = "Shounen Ai")]
    #[serde(rename = "Shounen Ai")]
    ShounenAi,
    #[strum(serialize = "Slice of Life")]
    #[serde(rename = "Slice of Life")]
    SliceOfLife,
    #[strum(serialize = "Smut")]
    #[serde(rename = "Smut")]
    Smut,
    #[strum(serialize = "Space")]
    #[serde(rename = "Space")]
    Space,
    #[strum(serialize = "Sports")]
    #[serde(rename = "Sports")]
    Sports,
    #[strum(serialize = "Super Hero")]
    #[serde(rename = "Super Hero")]
    SuperHero,
    #[strum(serialize = "Supernatural")]
    #[serde(rename = "Supernatural")]
    Supernatural,
    #[strum(serialize = "Survival")]
    #[serde(rename = "Survival")]
    Survival,
    #[strum(serialize = "Suspense")]
    #[serde(rename = "Suspense")]
    Suspense,
    #[strum(serialize = "Thriller")]
    #[serde(rename = "Thriller")]
    Thriller,
    #[strum(serialize = "Time Travel")]
    #[serde(rename = "Time Travel")]
    TimeTravel,
    #[strum(serialize = "Toomics")]
    #[serde(rename = "Toomics")]
    Toomics,
    #[strum(serialize = "Traditional Games")]
    #[serde(rename = "Traditional Games")]
    TraditionalGames,
    #[strum(serialize = "Tragedy")]
    #[serde(rename = "Tragedy")]
    Tragedy,
    #[strum(serialize = "Vampires")]
    #[serde(rename = "Vampires")]
    Vampires,
    #[strum(serialize = "Video Games")]
    #[serde(rename = "Video Games")]
    VideoGames,
    #[strum(serialize = "Villainess")]
    #[serde(rename = "Villainess")]
    Villainess,
    #[strum(serialize = "Virtual Reality")]
    #[serde(rename = "Virtual Reality")]
    VirtualReality,
    #[strum(serialize = "Web Comic")]
    #[serde(rename = "Web Comic")]
    WebComic,
    #[strum(serialize = "Wuxia")]
    #[serde(rename = "Wuxia")]
    Wuxia,
    #[strum(serialize = "Yaoi")]
    #[serde(rename = "Yaoi")]
    Yaoi,
    #[strum(serialize = "Yuri")]
    #[serde(rename = "Yuri")]
    Yuri,
    #[strum(serialize = "Zombies")]
    #[serde(rename = "Zombies")]
    Zombies,
}
