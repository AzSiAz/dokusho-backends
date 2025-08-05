use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::config::AppConfig;

pub async fn api_key_middleware(
    req: Request,
    next: Next,
    config: AppConfig,
) -> Result<Response, StatusCode> {
    // Skip API key check for health endpoint
    if req.uri().path() == "/health" {
        return Ok(next.run(req).await);
    }

    // Skip if API key auth is disabled
    if !config.sources.api_key_enabled {
        return Ok(next.run(req).await);
    }

    let expected_key = config.sources.api_key
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check X-API-Key header
    let api_key = req
        .headers()
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok());

    // Also check Authorization header with Bearer token
    let bearer_key = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                Some(&auth[7..])
            } else {
                None
            }
        });

    match api_key.or(bearer_key) {
        Some(key) if key == expected_key => Ok(next.run(req).await),
        Some(_) => {
            tracing::warn!("Invalid API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
        None => {
            tracing::warn!("No API key provided");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}