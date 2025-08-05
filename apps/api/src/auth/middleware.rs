use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use super::validate_jwt;
use crate::config::AppConfig;

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
    config: AppConfig,
) -> Result<Response, StatusCode> {
    // Skip auth for health check and auth callback
    let path = req.uri().path();
    if path == "/health" || path == "/auth/callback" {
        return Ok(next.run(req).await);
    }

    // Skip auth if not enabled
    if !config.auth.enabled {
        return Ok(next.run(req).await);
    }

    // Extract token from Authorization header
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_string())
            } else {
                None
            }
        });

    match token {
        Some(token) => {
            // Validate token
            match validate_jwt(&token, &config.auth.jwt_secret) {
                Ok(claims) => {
                    // Insert claims into request extensions
                    req.extensions_mut().insert(claims);
                    Ok(next.run(req).await)
                }
                Err(e) => {
                    tracing::warn!("Invalid JWT: {}", e);
                    Err(StatusCode::UNAUTHORIZED)
                }
            }
        }
        None => {
            // No token provided
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}