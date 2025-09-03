use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::SourceError;

fn source_serie_source_serie_type_parse_not_found(s: &str) -> SourceError {
    SourceError::InvalidSourceSerieType(s.to_string())
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
)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_source_serie_type_parse_not_found,
)]
pub enum SourceSerieType {
    #[strum(serialize = "Manga")]
    #[serde(rename = "Manga")]
    Manga,
    #[strum(serialize = "Manhwa")]
    #[serde(rename = "Manhwa")]
    Manhwa,
    #[strum(serialize = "Manhua")]
    #[serde(rename = "Manhua")]
    Manhua,
    #[strum(serialize = "Webtoon")]
    #[serde(rename = "Webtoon")]
    Webtoon,
    #[strum(serialize = "Light Novel")]
    #[serde(rename = "Light Novel")]
    Lightnovel,
    #[strum(serialize = "Novel")]
    #[serde(rename = "Novel")]
    Novel,
    #[strum(serialize = "Doujinshi")]
    #[serde(rename = "Doujinshi")]
    Doujinshi,
    #[strum(serialize = "Comic")]
    #[serde(rename = "Comic")]
    Comic,
    #[strum(serialize = "Oel")]
    #[serde(rename = "Oel")]
    Oel,
    #[strum(serialize = "Unknown")]
    #[serde(rename = "Unknown")]
    Unknown,
}
