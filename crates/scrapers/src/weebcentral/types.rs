use std::convert::TryFrom;
use dokusho_core::{FilterOrder, FilterSort, SourceSerieGenre, SourceSerieStatus, SourceSerieType};

// WeebCentral specific types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeebCentralStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
}

impl WeebCentralStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ongoing => "Ongoing",
            Self::Completed => "Completed",
            Self::Hiatus => "Hiatus",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl TryFrom<&str> for WeebCentralStatus {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Ongoing" | "ongoing" => Ok(Self::Ongoing),
            "Completed" | "completed" | "Complete" => Ok(Self::Completed),
            "Hiatus" | "hiatus" | "On Hiatus" => Ok(Self::Hiatus),
            "Cancelled" | "cancelled" | "Canceled" => Ok(Self::Cancelled),
            _ => anyhow::bail!("Unknown WeebCentral status: {}", value),
        }
    }
}

impl TryFrom<WeebCentralStatus> for SourceSerieStatus {
    type Error = anyhow::Error;

    fn try_from(status: WeebCentralStatus) -> Result<Self, Self::Error> {
        Ok(match status {
            WeebCentralStatus::Ongoing => SourceSerieStatus::Ongoing,
            WeebCentralStatus::Completed => SourceSerieStatus::Completed,
            WeebCentralStatus::Hiatus => SourceSerieStatus::Hiatus,
            WeebCentralStatus::Cancelled => SourceSerieStatus::Canceled,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeebCentralType {
    Manga,
    Manhwa,
    Manhua,
    Comic,
    Webtoon,
    Novel,
}

impl WeebCentralType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manga => "Manga",
            Self::Manhwa => "Manhwa",
            Self::Manhua => "Manhua",
            Self::Comic => "Comic",
            Self::Webtoon => "Webtoon",
            Self::Novel => "Novel",
        }
    }
}

impl TryFrom<&str> for WeebCentralType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "manga" => Ok(Self::Manga),
            "manhwa" => Ok(Self::Manhwa),
            "manhua" => Ok(Self::Manhua),
            "comic" => Ok(Self::Comic),
            "webtoon" => Ok(Self::Webtoon),
            "novel" | "light novel" => Ok(Self::Novel),
            _ => anyhow::bail!("Unknown WeebCentral type: {}", value),
        }
    }
}

impl TryFrom<WeebCentralType> for SourceSerieType {
    type Error = anyhow::Error;

    fn try_from(wc_type: WeebCentralType) -> Result<Self, Self::Error> {
        Ok(match wc_type {
            WeebCentralType::Manga => SourceSerieType::Manga,
            WeebCentralType::Manhwa => SourceSerieType::Manhwa,
            WeebCentralType::Manhua => SourceSerieType::Manhua,
            WeebCentralType::Comic => SourceSerieType::Comic,
            WeebCentralType::Webtoon => SourceSerieType::Webtoon,
            WeebCentralType::Novel => SourceSerieType::Novel,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeebCentralSort {
    Title,
    Views,
    Latest,
    Rating,
    Trending,
}

impl WeebCentralSort {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Views => "views",
            Self::Latest => "latest",
            Self::Rating => "rating",
            Self::Trending => "trending",
        }
    }
}

impl TryFrom<FilterSort> for WeebCentralSort {
    type Error = anyhow::Error;

    fn try_from(sort: FilterSort) -> Result<Self, Self::Error> {
        match sort {
            FilterSort::Title => Ok(Self::Title),
            FilterSort::Popularity => Ok(Self::Views),
            FilterSort::UpdatedAt => Ok(Self::Latest),
            FilterSort::Rating => Ok(Self::Rating),
            _ => anyhow::bail!("Unsupported sort type for WeebCentral"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeebCentralOrder {
    Asc,
    Desc,
}

impl WeebCentralOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

impl From<FilterOrder> for WeebCentralOrder {
    fn from(order: FilterOrder) -> Self {
        match order {
            FilterOrder::Ascending => Self::Asc,
            FilterOrder::Descending => Self::Desc,
        }
    }
}

// Genre mappings
pub fn parse_weebcentral_genre(name: &str) -> Option<SourceSerieGenre> {
    use SourceSerieGenre::*;
    match name.trim().to_lowercase().as_str() {
        "action" => Some(Action),
        "adventure" => Some(Adventure),
        "comedy" => Some(Comedy),
        "drama" => Some(Drama),
        "fantasy" => Some(Fantasy),
        "romance" => Some(Romance),
        "horror" => Some(Horror),
        "mystery" => Some(Mystery),
        "psychological" => Some(Psychological),
        "sci-fi" | "science fiction" => Some(SciFi),
        "slice of life" => Some(SliceOfLife),
        "sports" => Some(Sports),
        "supernatural" => Some(Supernatural),
        "thriller" => Some(Thriller),
        "historical" => Some(Historical),
        "martial arts" => Some(MartialArts),
        "mecha" => Some(Mecha),
        "school life" | "school" => Some(SchoolLife),
        "shounen" => Some(Shounen),
        "shoujo" => Some(Shoujo),
        "seinen" => Some(Seinen),
        "josei" => Some(Josei),
        "isekai" => Some(Isekai),
        "harem" => Some(Harem),
        "reverse harem" => Some(ReverseHarem),
        "ecchi" => Some(Ecchi),
        "yaoi" | "boys love" | "bl" => Some(Yaoi),
        "yuri" | "girls love" | "gl" => Some(Yuri),
        "magic" => Some(Magic),
        "military" => Some(Military),
        "music" => Some(Music),
        // "parody" => Some(Parody), // Not available in core
        "police" => Some(Police),
        "post-apocalyptic" => Some(PostApocalyptic),
        "shounen ai" => Some(ShounenAi),
        "shoujo ai" => Some(ShoujoAi),
        "space" => Some(Space),
        // "super power" => Some(SuperPower), // Not available in core
        "vampire" | "vampires" => Some(Vampires),
        "demons" => Some(Demons),
        "game" | "video games" => Some(VideoGames),
        "doujinshi" => Some(Doujinshi),
        "gender bender" | "genderswap" => Some(GenderBender),
        "tragedy" => Some(Tragedy),
        "medical" => Some(Medical),
        "zombies" => Some(Zombies),
        "crime" => Some(Crime),
        "gore" => Some(Gore),
        _ => None,
    }
}

pub fn convert_genre_to_weebcentral(genre: &SourceSerieGenre) -> &'static str {
    use SourceSerieGenre::*;
    match genre {
        Action => "action",
        Adventure => "adventure",
        Comedy => "comedy",
        Drama => "drama",
        Fantasy => "fantasy",
        Romance => "romance",
        Horror => "horror",
        Mystery => "mystery",
        Psychological => "psychological",
        SciFi => "sci-fi",
        SliceOfLife => "slice-of-life",
        Sports => "sports",
        Supernatural => "supernatural",
        Thriller => "thriller",
        Historical => "historical",
        MartialArts => "martial-arts",
        Mecha => "mecha",
        SchoolLife => "school-life",
        Shounen => "shounen",
        Shoujo => "shoujo",
        Seinen => "seinen",
        Josei => "josei",
        Isekai => "isekai",
        Harem => "harem",
        ReverseHarem => "reverse-harem",
        Ecchi => "ecchi",
        Yaoi => "yaoi",
        Yuri => "yuri",
        Magic => "magic",
        Military => "military",
        Music => "music",
        // Parody => "parody", // Not available in core
        Police => "police",
        PostApocalyptic => "post-apocalyptic",
        ShounenAi => "shounen-ai",
        ShoujoAi => "shoujo-ai",
        Space => "space",
        // SuperPower => "super-power", // Not available in core
        Vampires => "vampire",
        Demons => "demons",
        VideoGames => "game",
        Doujinshi => "doujinshi",
        GenderBender => "gender-bender",
        Tragedy => "tragedy",
        Medical => "medical",
        Zombies => "zombies",
        Crime => "crime",
        Gore => "gore",
        _ => "unknown",
    }
}

// Helper functions for getting searchable filters
pub fn get_searchable_genres() -> Vec<SourceSerieGenre> {
    vec![
        SourceSerieGenre::Action,
        SourceSerieGenre::Adventure,
        SourceSerieGenre::Comedy,
        SourceSerieGenre::Drama,
        SourceSerieGenre::Fantasy,
        SourceSerieGenre::Romance,
        SourceSerieGenre::Horror,
        SourceSerieGenre::Mystery,
        SourceSerieGenre::Psychological,
        SourceSerieGenre::SciFi,
        SourceSerieGenre::SliceOfLife,
        SourceSerieGenre::Sports,
        SourceSerieGenre::Supernatural,
        SourceSerieGenre::Thriller,
        SourceSerieGenre::Historical,
        SourceSerieGenre::MartialArts,
        SourceSerieGenre::Mecha,
        SourceSerieGenre::SchoolLife,
        SourceSerieGenre::Isekai,
        SourceSerieGenre::Harem,
        SourceSerieGenre::Magic,
    ]
}

pub fn get_searchable_status() -> Vec<SourceSerieStatus> {
    vec![
        SourceSerieStatus::Ongoing,
        SourceSerieStatus::Completed,
        SourceSerieStatus::Hiatus,
        SourceSerieStatus::Canceled,
    ]
}

pub fn get_searchable_types() -> Vec<SourceSerieType> {
    vec![
        SourceSerieType::Manga,
        SourceSerieType::Manhwa,
        SourceSerieType::Manhua,
        SourceSerieType::Comic,
        SourceSerieType::Webtoon,
    ]
}