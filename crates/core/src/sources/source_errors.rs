use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("Invalid search query: {0}")]
    InvalidSearchQuery(String),

    #[error("Invalid search order: {0}")]
    InvalidSearchOrder(String),

    #[error("Invalid search sort: {0}")]
    InvalidSearchSort(String),

    #[error("Invalid search types: {0:?}")]
    InvalidSearchTypes(Vec<String>),

    #[error("Invalid search genres: {0:?}")]
    InvalidSearchGenres(Vec<String>),

    #[error("Invalid search status: {0:?}")]
    InvalidSearchStatus(Vec<String>),

    #[error("Invalid languages: {0:?}")]
    InvalidLanguages(Vec<String>),

    #[error("Invalid genre: {0}")]
    InvalidGenre(String),

    #[error("Invalid serie id: {0}")]
    InvalidSerieID(String),

    #[error("Invalid source serie status: {0}")]
    InvalidSourceSerieStatus(String),

    #[error("Invalid cover: {0}")]
    InvalidCover(String),

    #[error("Invalid source serie type: {0}")]
    InvalidSourceSerieType(String),

    #[error("error building request")]
    BuildingRequest(String),

    #[error("error building url")]
    BuildingURL(String),

    #[error("http request failed")]
    HTTPRequestFailed(String),

    #[error("http request failed")]
    CloudflareProtection,

    #[error("http request failed")]
    Network(String),

    #[error("error parsing html")]
    ParsingHTML(String),

    #[error("error parsing json")]
    ParsingJSON(String),

    #[error("error parsing url")]
    ParsingURL(String),

    #[error("error extracting data")]
    ExtractingData(String),

    #[error("timeout")]
    Timeout,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
