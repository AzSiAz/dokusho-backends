use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};

use crate::SourceError;

fn source_serie_source_serie_status_parse_not_found(s: &str) -> SourceError {
    SourceError::InvalidSourceSerieStatus(s.to_string())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumString, EnumIter, EnumCount, Display)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_source_serie_status_parse_not_found,
)]
pub enum SourceSerieStatus {
    #[strum(serialize = "Ongoing")]
    #[serde(rename = "Ongoing")]
    Ongoing,
    #[strum(serialize = "Completed")]
    #[serde(rename = "Completed")]
    Completed,
    #[strum(serialize = "Hiatus")]
    #[serde(rename = "Hiatus")]
    Hiatus,
    #[strum(serialize = "Canceled")]
    #[serde(rename = "Canceled")]
    Canceled,
    #[strum(serialize = "Publishing")]
    #[serde(rename = "Publishing")]
    Publishing,
    #[strum(serialize = "Published")]
    #[serde(rename = "Published")]
    Published,
    #[strum(serialize = "Scanlating")]
    #[serde(rename = "Scanlating")]
    Scanlating,
    #[strum(serialize = "Scanlated")]
    #[serde(rename = "Scanlated")]
    Scanlated,
    #[strum(serialize = "Unknown")]
    #[serde(rename = "Unknown")]
    Unknown,
}
