use dokusho_database::DatabaseError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid state parameter")]
    InvalidState,

    #[error("State expired")]
    StateExpired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("User not found")]
    UserNotFound,

    #[error("OpenID Connect error: {0}")]
    OpenIDConnect(String),

    #[error("JWT error: {0}")]
    JWT(#[from] jsonwebtoken::errors::Error),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<openidconnect::DiscoveryError<openidconnect::reqwest::Error<reqwest::Error>>>
    for AuthError
{
    fn from(
        err: openidconnect::DiscoveryError<openidconnect::reqwest::Error<reqwest::Error>>,
    ) -> Self {
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
        openidconnect::RequestTokenError<
            openidconnect::reqwest::Error<reqwest::Error>,
            openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
        >,
    > for AuthError
{
    fn from(
        err: openidconnect::RequestTokenError<
            openidconnect::reqwest::Error<reqwest::Error>,
            openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>,
        >,
    ) -> Self {
        Self::OpenIDConnect(err.to_string())
    }
}
