use dokusho_database::DatabaseError;
use openidconnect::{
    DiscoveryError, HttpClientError, RequestTokenError, StandardErrorResponse,
    core::CoreErrorResponseType,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("User not found")]
    UserNotFound,

    #[error("OpenID Connect error: {0}")]
    OpenIDConnect(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Updated for openidconnect 4.0 - HttpClientError now needs reqwest::Error as its generic parameter
impl From<DiscoveryError<HttpClientError<reqwest::Error>>> for AuthError {
    fn from(err: DiscoveryError<HttpClientError<reqwest::Error>>) -> Self {
        Self::OpenIDConnect(err.to_string())
    }
}

impl From<DatabaseError> for AuthError {
    fn from(err: DatabaseError) -> Self {
        Self::Database(err.to_string())
    }
}

impl
    From<
        RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    > for AuthError
{
    fn from(
        err: RequestTokenError<
            HttpClientError<reqwest::Error>,
            StandardErrorResponse<CoreErrorResponseType>,
        >,
    ) -> Self {
        Self::OpenIDConnect(err.to_string())
    }
}
