use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumCount, EnumIter, EnumString};
use utoipa::ToSchema;

use crate::SourceError;

fn source_serie_filter_order_parse_error(s: &str) -> SourceError {
    SourceError::InvalidSearchOrder(s.to_string())
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
    ToSchema,
)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_filter_order_parse_error,
)]
pub enum FetchSearchSerieFilterOrder {
    #[strum(serialize = "ASC")]
    #[serde(rename = "ASC")]
    ASC,
    #[strum(serialize = "DESC")]
    #[serde(rename = "DESC")]
    DESC,
}

fn source_serie_filter_sort_parse_error(s: &str) -> SourceError {
    SourceError::InvalidSearchSort(s.to_string())
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
    ToSchema,
)]
#[strum(
	parse_err_ty=SourceError,
	parse_err_fn=source_serie_filter_sort_parse_error,
)]
pub enum FetchSearchSerieFilterSort {
    #[strum(serialize = "Latest")]
    #[serde(rename = "Latest")]
    Latest,
    #[strum(serialize = "Popularity")]
    #[serde(rename = "Popularity")]
    Popularity,
    #[strum(serialize = "Relevance")]
    #[serde(rename = "Relevance")]
    Relevance,
    #[strum(serialize = "Alphabetic")]
    #[serde(rename = "Alphabetic")]
    Alphabetic,
}
