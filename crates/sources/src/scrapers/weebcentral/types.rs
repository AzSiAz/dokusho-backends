use dokusho_core::{
    FetchSearchSerieFilterOrder, FetchSearchSerieFilterSort, SourceError, SourceSerieGenre,
    SourceSerieStatus, SourceSerieType,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::EnumString;
use strum_macros::{Display, EnumIter};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WeebCentralGenre {
    #[strum(serialize = "Action")]
    Action,
    #[strum(serialize = "Adult")]
    Adult,
    #[strum(serialize = "Adventure")]
    Adventure,
    #[strum(serialize = "Comedy")]
    Comedy,
    #[strum(serialize = "Doujinshi")]
    Doujinshi,
    #[strum(serialize = "Drama")]
    Drama,
    #[strum(serialize = "Ecchi")]
    Ecchi,
    #[strum(serialize = "Fantasy")]
    Fantasy,
    #[strum(serialize = "GenderBender")]
    GenderBender,
    #[strum(serialize = "Harem")]
    Harem,
    #[strum(serialize = "Hentai")]
    Hentai,
    #[strum(serialize = "Historical")]
    Historical,
    #[strum(serialize = "Horror")]
    Horror,
    #[strum(serialize = "Isekai")]
    Isekai,
    #[strum(serialize = "Josei")]
    Josei,
    #[strum(serialize = "Lolicon")]
    Lolicon,
    #[strum(serialize = "MartialArts")]
    MartialArts,
    #[strum(serialize = "Mature")]
    Mature,
    #[strum(serialize = "Mecha")]
    Mecha,
    #[strum(serialize = "Mystery")]
    Mystery,
    #[strum(serialize = "Psychological")]
    Psychological,
    #[strum(serialize = "Romance")]
    Romance,
    #[strum(serialize = "SchoolLife")]
    SchoolLife,
    #[strum(serialize = "SciFi")]
    SciFi,
    #[strum(serialize = "Seinen")]
    Seinen,
    #[strum(serialize = "Shotacon")]
    Shotacon,
    #[strum(serialize = "Shoujo")]
    Shoujo,
    #[strum(serialize = "ShoujoAi")]
    ShoujoAi,
    #[strum(serialize = "Shounen")]
    Shounen,
    #[strum(serialize = "ShounenAi")]
    ShounenAi,
    #[strum(serialize = "SliceOfLife")]
    SliceOfLife,
    #[strum(serialize = "Smut")]
    Smut,
    #[strum(serialize = "Sports")]
    Sports,
    #[strum(serialize = "Supernatural")]
    Supernatural,
    #[strum(serialize = "Tragedy")]
    Tragedy,
    #[strum(serialize = "Yaoi")]
    Yaoi,
    #[strum(serialize = "Yuri")]
    Yuri,

    #[strum(serialize = "Other")]
    Other,
}

impl From<WeebCentralGenre> for SourceSerieGenre {
    fn from(value: WeebCentralGenre) -> Self {
        match value {
            WeebCentralGenre::Action => Self::Action,
            WeebCentralGenre::Adult => Self::Adult,
            WeebCentralGenre::Adventure => Self::Adventure,
            WeebCentralGenre::Comedy => Self::Comedy,
            WeebCentralGenre::Doujinshi => Self::Doujinshi,
            WeebCentralGenre::Drama => Self::Drama,
            WeebCentralGenre::Ecchi => Self::Ecchi,
            WeebCentralGenre::Fantasy => Self::Fantasy,
            WeebCentralGenre::GenderBender => Self::GenderBender,
            WeebCentralGenre::Harem => Self::Harem,
            WeebCentralGenre::Hentai => Self::Hentai,
            WeebCentralGenre::Historical => Self::Historical,
            WeebCentralGenre::Horror => Self::Horror,
            WeebCentralGenre::Isekai => Self::Isekai,
            WeebCentralGenre::Josei => Self::Josei,
            WeebCentralGenre::Lolicon => Self::Lolicon,
            WeebCentralGenre::MartialArts => Self::MartialArts,
            WeebCentralGenre::Mature => Self::Mature,
            WeebCentralGenre::Mecha => Self::Mecha,
            WeebCentralGenre::Mystery => Self::Mystery,
            WeebCentralGenre::Psychological => Self::Psychological,
            WeebCentralGenre::Romance => Self::Romance,
            WeebCentralGenre::Seinen => Self::Seinen,
            WeebCentralGenre::SchoolLife => Self::SchoolLife,
            WeebCentralGenre::Shotacon => Self::Shotacon,
            WeebCentralGenre::ShoujoAi => Self::ShoujoAi,
            WeebCentralGenre::Shoujo => Self::Shoujo,
            WeebCentralGenre::Shounen => Self::Shounen,
            WeebCentralGenre::ShounenAi => Self::ShounenAi,
            WeebCentralGenre::SciFi => Self::SciFi,
            WeebCentralGenre::SliceOfLife => Self::SliceOfLife,
            WeebCentralGenre::Smut => Self::Smut,
            WeebCentralGenre::Sports => Self::Sports,
            WeebCentralGenre::Supernatural => Self::Supernatural,
            WeebCentralGenre::Tragedy => Self::Tragedy,
            WeebCentralGenre::Yaoi => Self::Yaoi,
            WeebCentralGenre::Yuri => Self::Yuri,
            WeebCentralGenre::Other => Self::Other,
        }
    }
}

impl TryFrom<SourceSerieGenre> for WeebCentralGenre {
    type Error = SourceError;

    fn try_from(value: SourceSerieGenre) -> Result<Self, Self::Error> {
        match value {
            SourceSerieGenre::Action => Ok(WeebCentralGenre::Action),
            SourceSerieGenre::Adult => Ok(WeebCentralGenre::Adult),
            SourceSerieGenre::Adventure => Ok(WeebCentralGenre::Adventure),
            SourceSerieGenre::Comedy => Ok(WeebCentralGenre::Comedy),
            SourceSerieGenre::Doujinshi => Ok(WeebCentralGenre::Doujinshi),
            SourceSerieGenre::Drama => Ok(WeebCentralGenre::Drama),
            SourceSerieGenre::Ecchi => Ok(WeebCentralGenre::Ecchi),
            SourceSerieGenre::Fantasy => Ok(WeebCentralGenre::Fantasy),
            SourceSerieGenre::GenderBender => Ok(WeebCentralGenre::GenderBender),
            SourceSerieGenre::Harem => Ok(WeebCentralGenre::Harem),
            SourceSerieGenre::Hentai => Ok(WeebCentralGenre::Hentai),
            SourceSerieGenre::Historical => Ok(WeebCentralGenre::Historical),
            SourceSerieGenre::Horror => Ok(WeebCentralGenre::Horror),
            SourceSerieGenre::Isekai => Ok(WeebCentralGenre::Isekai),
            SourceSerieGenre::Josei => Ok(WeebCentralGenre::Josei),
            SourceSerieGenre::Lolicon => Ok(WeebCentralGenre::Lolicon),
            SourceSerieGenre::MartialArts => Ok(WeebCentralGenre::MartialArts),
            SourceSerieGenre::Mature => Ok(WeebCentralGenre::Mature),
            SourceSerieGenre::Mecha => Ok(WeebCentralGenre::Mecha),
            SourceSerieGenre::Mystery => Ok(WeebCentralGenre::Mystery),
            SourceSerieGenre::Psychological => Ok(WeebCentralGenre::Psychological),
            SourceSerieGenre::Romance => Ok(WeebCentralGenre::Romance),
            SourceSerieGenre::Seinen => Ok(WeebCentralGenre::Seinen),
            SourceSerieGenre::SchoolLife => Ok(WeebCentralGenre::SchoolLife),
            SourceSerieGenre::Shotacon => Ok(WeebCentralGenre::Shotacon),
            SourceSerieGenre::ShoujoAi => Ok(WeebCentralGenre::ShoujoAi),
            SourceSerieGenre::Shoujo => Ok(WeebCentralGenre::Shoujo),
            SourceSerieGenre::Shounen => Ok(WeebCentralGenre::Shounen),
            SourceSerieGenre::ShounenAi => Ok(WeebCentralGenre::ShounenAi),
            SourceSerieGenre::SciFi => Ok(WeebCentralGenre::SciFi),
            SourceSerieGenre::SliceOfLife => Ok(WeebCentralGenre::SliceOfLife),
            SourceSerieGenre::Smut => Ok(WeebCentralGenre::Smut),
            SourceSerieGenre::Sports => Ok(WeebCentralGenre::Sports),
            SourceSerieGenre::Supernatural => Ok(WeebCentralGenre::Supernatural),
            SourceSerieGenre::Tragedy => Ok(WeebCentralGenre::Tragedy),
            SourceSerieGenre::Yaoi => Ok(WeebCentralGenre::Yaoi),
            SourceSerieGenre::Yuri => Ok(WeebCentralGenre::Yuri),
            SourceSerieGenre::Other => Ok(WeebCentralGenre::Other),
            _ => Err(SourceError::InvalidGenre(value.to_string())),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WeebCentralType {
    #[strum(serialize = "Manga")]
    Manga,
    #[strum(serialize = "Manhwa")]
    Manhwa,
    #[strum(serialize = "Manhua")]
    Manhua,
    #[strum(serialize = "Comic")]
    Comic,
    #[strum(serialize = "Webtoon")]
    Webtoon,
    #[strum(serialize = "Novel")]
    Novel,
}

impl From<WeebCentralType> for SourceSerieType {
    fn from(value: WeebCentralType) -> Self {
        match value {
            WeebCentralType::Manga => Self::Manga,
            WeebCentralType::Manhwa => Self::Manhwa,
            WeebCentralType::Manhua => Self::Manhua,
            WeebCentralType::Comic => Self::Comic,
            WeebCentralType::Webtoon => Self::Webtoon,
            WeebCentralType::Novel => Self::Novel,
        }
    }
}

impl TryFrom<SourceSerieType> for WeebCentralType {
    type Error = SourceError;

    fn try_from(value: SourceSerieType) -> Result<Self, Self::Error> {
        match value {
            SourceSerieType::Manga => Ok(WeebCentralType::Manga),
            SourceSerieType::Manhwa => Ok(WeebCentralType::Manhwa),
            SourceSerieType::Manhua => Ok(WeebCentralType::Manhua),
            SourceSerieType::Comic => Ok(WeebCentralType::Comic),
            SourceSerieType::Webtoon => Ok(WeebCentralType::Webtoon),
            SourceSerieType::Novel => Ok(WeebCentralType::Novel),
            _ => Err(SourceError::InvalidSourceSerieType(value.to_string())),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WeebCentralStatus {
    #[strum(serialize = "Ongoing")]
    Ongoing,
    #[strum(serialize = "Complete")]
    Completed,
    #[strum(serialize = "Hiatus")]
    Hiatus,
    #[strum(serialize = "Canceled")]
    Canceled,
}

impl From<WeebCentralStatus> for SourceSerieStatus {
    fn from(value: WeebCentralStatus) -> Self {
        match value {
            WeebCentralStatus::Ongoing => Self::Ongoing,
            WeebCentralStatus::Completed => Self::Completed,
            WeebCentralStatus::Hiatus => Self::Hiatus,
            WeebCentralStatus::Canceled => Self::Canceled,
        }
    }
}

impl TryFrom<SourceSerieStatus> for WeebCentralStatus {
    type Error = SourceError;

    fn try_from(value: SourceSerieStatus) -> Result<Self, Self::Error> {
        match value {
            SourceSerieStatus::Ongoing => Ok(Self::Ongoing),
            SourceSerieStatus::Completed => Ok(Self::Completed),
            SourceSerieStatus::Hiatus => Ok(Self::Hiatus),
            SourceSerieStatus::Canceled => Ok(Self::Canceled),
            _ => Err(SourceError::InvalidSourceSerieStatus(value.to_string())),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WeebCentralSort {
    #[strum(serialize = "Best Match")]
    BestMatch,
    #[strum(serialize = "Popularity")]
    Popularity,
    #[strum(serialize = "Latest Updates")]
    LatestUpdates,
    #[strum(serialize = "Alphabet")]
    Alphabet,
}

impl From<WeebCentralSort> for FetchSearchSerieFilterSort {
    fn from(value: WeebCentralSort) -> Self {
        match value {
            WeebCentralSort::Popularity => Self::Popularity,
            WeebCentralSort::LatestUpdates => Self::Latest,
            WeebCentralSort::BestMatch => Self::Relevance,
            WeebCentralSort::Alphabet => Self::Alphabetic,
        }
    }
}

impl From<FetchSearchSerieFilterSort> for WeebCentralSort {
    fn from(value: FetchSearchSerieFilterSort) -> Self {
        match value {
            FetchSearchSerieFilterSort::Popularity => WeebCentralSort::Popularity,
            FetchSearchSerieFilterSort::Latest => WeebCentralSort::LatestUpdates,
            FetchSearchSerieFilterSort::Relevance => WeebCentralSort::BestMatch,
            FetchSearchSerieFilterSort::Alphabetic => WeebCentralSort::Alphabet,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, EnumIter, Display, EnumString,
)]
pub enum WeebCentralOrder {
    #[strum(serialize = "Ascending")]
    Ascending,
    #[strum(serialize = "Descending")]
    Descending,
}

impl From<WeebCentralOrder> for FetchSearchSerieFilterOrder {
    fn from(value: WeebCentralOrder) -> Self {
        match value {
            WeebCentralOrder::Ascending => Self::ASC,
            WeebCentralOrder::Descending => Self::DESC,
        }
    }
}

impl From<FetchSearchSerieFilterOrder> for WeebCentralOrder {
    fn from(value: FetchSearchSerieFilterOrder) -> Self {
        match value {
            FetchSearchSerieFilterOrder::ASC => Self::Ascending,
            FetchSearchSerieFilterOrder::DESC => Self::Descending,
        }
    }
}
