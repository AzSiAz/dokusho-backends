use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    InternalServerError(String),
    ServiceUnavailable(String),
}

impl ApiError {
    pub fn not_found<T: Into<String>>(msg: T) -> Self {
        ApiError::NotFound(msg.into())
    }

    #[allow(dead_code)]
    pub fn bad_request<T: Into<String>>(msg: T) -> Self {
        ApiError::BadRequest(msg.into())
    }

    pub fn unauthorized<T: Into<String>>(msg: T) -> Self {
        ApiError::Unauthorized(msg.into())
    }

    pub fn forbidden<T: Into<String>>(msg: T) -> Self {
        ApiError::Forbidden(msg.into())
    }

    pub fn internal_server_error<T: Into<String>>(msg: T) -> Self {
        ApiError::InternalServerError(msg.into())
    }

    #[allow(dead_code)]
    pub fn service_unavailable<T: Into<String>>(msg: T) -> Self {
        ApiError::ServiceUnavailable(msg.into())
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            ApiError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            ApiError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ApiError::BadRequest(_) => (StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            ApiError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            ApiError::Forbidden(_) => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            ApiError::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR")
            }
            ApiError::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE")
            }
        };

        let message = match &self {
            ApiError::NotFound(msg) => msg.clone(),
            ApiError::BadRequest(msg) => msg.clone(),
            ApiError::Unauthorized(msg) => msg.clone(),
            ApiError::Forbidden(msg) => msg.clone(),
            ApiError::InternalServerError(msg) => msg.clone(),
            ApiError::ServiceUnavailable(msg) => msg.clone(),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
        });

        (status, body).into_response()
    }
}

// Conversion from auth errors
impl From<dokusho_auth::AuthError> for ApiError {
    fn from(err: dokusho_auth::AuthError) -> Self {
        match err {
            dokusho_auth::AuthError::InvalidToken(_) => ApiError::unauthorized("Invalid token"),
            dokusho_auth::AuthError::TokenExpired => ApiError::unauthorized("Token expired"),
            dokusho_auth::AuthError::Unauthorized => ApiError::unauthorized("Unauthorized"),
            dokusho_auth::AuthError::UserNotFound => ApiError::not_found("User not found"),
            _ => {
                tracing::error!("Auth error: {}", err);
                ApiError::internal_server_error("Authentication failed")
            }
        }
    }
}

// Note: SourceError is not publicly exported from the sources crate
// Source operations that fail will be converted to string errors instead

pub type ApiResult<T> = Result<T, ApiError>;

// Database error conversion
impl From<dokusho_database::DatabaseError> for ApiError {
    fn from(err: dokusho_database::DatabaseError) -> Self {
        tracing::error!("Database error: {}", err);
        ApiError::internal_server_error("Database operation failed")
    }
}

// Generic error conversion
impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Error: {}", err);
        ApiError::internal_server_error("Operation failed")
    }
}
